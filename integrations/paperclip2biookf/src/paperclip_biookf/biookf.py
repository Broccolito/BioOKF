"""Materialize validated agent output as a BioOKF v0.5 Studio bundle."""

from __future__ import annotations

import datetime as dt
import hashlib
import json
import re
import shutil
import subprocess
import unicodedata
from collections import defaultdict
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional, Tuple
from urllib.parse import quote

from .constants import KNOWLEDGE_LEVELS, NODE_TYPES, PREDICATES
from .paperclip import document_storage_id


class BioOKFError(RuntimeError):
    pass


def validate_extraction(payload: Dict[str, Any]) -> List[str]:
    errors: List[str] = []
    if not isinstance(payload, dict):
        return ["extraction must be a JSON object"]
    nodes = payload.get("nodes")
    edges = payload.get("edges")
    if not isinstance(nodes, list):
        errors.append("nodes must be an array")
        nodes = []
    if not isinstance(edges, list):
        errors.append("edges must be an array")
        edges = []
    identifiers = set()
    for index, node in enumerate(nodes):
        prefix = f"nodes[{index}]"
        if not isinstance(node, dict):
            errors.append(f"{prefix} must be an object")
            continue
        identifier = node.get("identifier")
        if not isinstance(identifier, str) or not identifier.strip():
            errors.append(f"{prefix}.identifier must be a non-empty string")
        else:
            identifiers.add(identifier)
        if node.get("type") not in NODE_TYPES:
            errors.append(f"{prefix}.type is not a BioOKF v0.5 type")
        if not isinstance(node.get("subtype"), str) or not node.get("subtype", "").strip():
            errors.append(f"{prefix}.subtype must be a non-empty string")
        for field in ("description",):
            if not isinstance(node.get(field), str):
                errors.append(f"{prefix}.{field} must be a string")
        for field in ("synonyms", "xref"):
            if not isinstance(node.get(field), list) or not all(isinstance(v, str) for v in node.get(field, [])):
                errors.append(f"{prefix}.{field} must be an array of strings")
    line_pattern = re.compile(r"^L[0-9]+(?:-L?[0-9]+)?$")
    for index, edge in enumerate(edges):
        prefix = f"edges[{index}]"
        if not isinstance(edge, dict):
            errors.append(f"{prefix} must be an object")
            continue
        if edge.get("subject") not in identifiers:
            errors.append(f"{prefix}.subject does not resolve in this extraction")
        if edge.get("object") not in identifiers:
            errors.append(f"{prefix}.object does not resolve in this extraction")
        if edge.get("predicate") not in PREDICATES:
            errors.append(f"{prefix}.predicate is not a BioOKF v0.5 predicate")
        if edge.get("knowledge_level") not in KNOWLEDGE_LEVELS:
            errors.append(f"{prefix}.knowledge_level is invalid")
        evidence = edge.get("evidence_lines")
        if not isinstance(evidence, list) or not evidence or not all(isinstance(v, str) and line_pattern.match(v) for v in evidence):
            errors.append(f"{prefix}.evidence_lines must contain Paperclip L<n> references")
        if not isinstance(edge.get("statement"), str) or not edge.get("statement", "").strip():
            errors.append(f"{prefix}.statement must be a non-empty string")
    return errors


class BioOKFBuilder:
    def __init__(self, bundle: Path, name: str, bokf_binary: Optional[str] = None) -> None:
        self.bundle = bundle.resolve()
        self.name = name
        self.bokf = bokf_binary or shutil.which("bokf")

    def build(self, run_dir: Path, search: Dict[str, Any], records: List[Dict[str, Any]]) -> Dict[str, Any]:
        self._scaffold()
        documents = {item["document_id"]: item for item in search.get("papers", [])}
        successful = [item for item in records if item.get("status") == "success"]
        if not successful:
            raise BioOKFError("no successful agent extractions to materialize")
        all_errors: List[str] = []
        for item in successful:
            errors = validate_extraction(item.get("extraction", {}))
            all_errors.extend(f"{item.get('document_id')}: {error}" for error in errors)
        if all_errors:
            raise BioOKFError("agent output failed validation:\n" + "\n".join(all_errors))

        source_names = self._unique_source_names(successful, documents)
        aliases = self._resolve_identifier_collisions(successful)
        nodes: Dict[str, Dict[str, Any]] = {}
        evidence: Dict[str, List[Dict[str, str]]] = defaultdict(list)

        for item in successful:
            doc_id = item["document_id"]
            document = documents.get(doc_id, {"document_id": doc_id, "title": item.get("title", doc_id)})
            source_identifier = source_names[doc_id]
            raw_rel = self._copy_raw_snapshot(run_dir, doc_id)
            source_node = self._source_node(document, source_identifier, raw_rel)
            nodes[source_identifier] = source_node
            local_alias = aliases[doc_id]
            for candidate in item["extraction"]["nodes"]:
                canonical = local_alias[(candidate["identifier"], candidate["type"])]
                node = nodes.setdefault(canonical, self._concept_node(candidate, canonical))
                node["synonyms"] = sorted(set(node.get("synonyms", [])) | set(candidate.get("synonyms", [])))
                node["xref"] = sorted(set(node.get("xref", [])) | set(candidate.get("xref", [])))
                if candidate.get("description") and not node.get("description"):
                    node["description"] = candidate["description"]
            for candidate in item["extraction"]["edges"]:
                subject_type = self._node_type_for(item, candidate["subject"])
                object_type = self._node_type_for(item, candidate["object"])
                subject = local_alias[(candidate["subject"], subject_type)]
                object_ = local_alias[(candidate["object"], object_type)]
                edge = self._claim_edge(candidate, object_, source_identifier, document)
                self._append_unique_edge(nodes[subject]["edges"], edge)
                evidence[subject].append({
                    "statement": candidate["statement"],
                    "source": source_identifier,
                    "url": edge["evidence_url"],
                })
            for candidate in item["extraction"]["nodes"]:
                canonical = local_alias[(candidate["identifier"], candidate["type"])]
                self._append_unique_edge(nodes[canonical]["edges"], self._reported_in(source_identifier))

        output_paths = self._node_output_paths(nodes)
        for identifier, node in nodes.items():
            self._write_node(node, evidence.get(identifier, []), output_paths[identifier])
        self._write_index(nodes.values())
        self._write_log(len(nodes), successful)
        manifest = self._write_manifest(run_dir, search, successful, nodes)
        verification = self.verify()
        manifest["verification"] = verification
        manifest_path = self.bundle / "operations" / "latest-run.json"
        manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
        return manifest

    def verify(self) -> Dict[str, Any]:
        internal = self._internal_verify()
        result: Dict[str, Any] = {"internal": internal, "bokf": None}
        if self.bokf:
            completed = subprocess.run(
                [self.bokf, "verify", str(self.bundle), "--workflow", "ingest", "--json"],
                text=True, capture_output=True,
            )
            try:
                report = json.loads(completed.stdout) if completed.stdout.strip() else None
            except json.JSONDecodeError:
                report = None
            result["bokf"] = {
                "ok": completed.returncode == 0,
                "returncode": completed.returncode,
                "report": report,
                "stdout": completed.stdout.strip() if report is None else "",
                "stderr": completed.stderr.strip(),
            }
            if completed.returncode != 0:
                raise BioOKFError("bokf verify failed:\n" + (completed.stdout or completed.stderr))
        if not internal["ok"]:
            raise BioOKFError("internal verification failed:\n" + "\n".join(internal["errors"]))
        return result

    def register_for_studio(self, kb_id: Optional[str] = None, open_studio: bool = False) -> Dict[str, Any]:
        kb_id = kb_id or slugify(self.name)
        if not self.bokf:
            return {
                "registered": False,
                "reason": "bokf not installed",
                "manual": f"Open BioOKF Studio, click '+ New base', and select {self.bundle}",
            }
        registration = subprocess.run(
            [self.bokf, "register", kb_id, str(self.bundle)],
            text=True, capture_output=True,
        )
        if registration.returncode != 0:
            registered_path = self._registered_path(kb_id)
            if registered_path is None or registered_path.resolve() != self.bundle.resolve():
                raise BioOKFError(
                    "command failed: " + " ".join([self.bokf, "register", kb_id, str(self.bundle)])
                    + "\n" + (registration.stdout or registration.stderr)
                )
        self._checked([self.bokf, "set-active", kb_id])
        opened = False
        if open_studio:
            app = Path("/Applications/BioOKF Studio.app")
            if app.exists() and shutil.which("open"):
                self._checked(["open", "-a", "BioOKF Studio"])
                opened = True
        return {"registered": True, "kb_id": kb_id, "path": str(self.bundle), "opened": opened}

    def _registered_path(self, kb_id: str) -> Optional[Path]:
        completed = subprocess.run(
            [self.bokf, "register", "--list"], text=True, capture_output=True,
        )
        if completed.returncode != 0:
            return None
        for line in completed.stdout.splitlines():
            parts = line.strip().split(None, 1)
            if len(parts) == 2 and parts[0] == kb_id:
                return Path(parts[1])
        return None

    def _scaffold(self) -> None:
        if self.bundle.exists() and any(self.bundle.iterdir()):
            knowledge = self.bundle / "knowledge"
            if knowledge.exists() and any(knowledge.rglob("*.md")):
                raise BioOKFError(
                    f"refusing to write into non-empty KB: {self.bundle}; use a new staging path"
                )
        self.bundle.mkdir(parents=True, exist_ok=True)
        (self.bundle / "raw").mkdir(exist_ok=True)
        (self.bundle / "knowledge").mkdir(exist_ok=True)
        (self.bundle / "operations").mkdir(exist_ok=True)
        schema = self.bundle / "SCHEMA.md"
        if not schema.exists():
            schema.write_text(_minimal_schema(), encoding="utf-8")
        index = self.bundle / "index.md"
        if not index.exists():
            index.write_text(f"# {self.name}\n\n", encoding="utf-8")
        log = self.bundle / "log.md"
        if not log.exists():
            log.write_text("# Change log\n\n", encoding="utf-8")

    def _copy_raw_snapshot(self, run_dir: Path, doc_id: str) -> str:
        stored_id = document_storage_id(doc_id)
        source = run_dir / "sources" / stored_id
        if not source.is_dir():
            raise BioOKFError(f"missing Paperclip snapshot for {doc_id}: {source}")
        raw_id = "paperclip-" + stored_id
        destination = self.bundle / "raw" / raw_id
        if destination.exists():
            raise BioOKFError(f"raw destination already exists: {destination}")
        shutil.copytree(source, destination)
        return f"raw/{raw_id}/source.md"

    @staticmethod
    def _unique_source_names(records: List[Dict[str, Any]], documents: Dict[str, Dict[str, Any]]) -> Dict[str, str]:
        by_title: Dict[str, List[str]] = defaultdict(list)
        for item in records:
            doc = documents.get(item["document_id"], {})
            by_title[doc.get("title") or item.get("title") or item["document_id"]].append(item["document_id"])
        result: Dict[str, str] = {}
        for title, ids in by_title.items():
            for doc_id in ids:
                result[doc_id] = title if len(ids) == 1 else f"{title} ({doc_id})"
        return result

    @staticmethod
    def _resolve_identifier_collisions(records: List[Dict[str, Any]]) -> Dict[str, Dict[Tuple[str, str], str]]:
        types_by_name: Dict[str, set] = defaultdict(set)
        spellings_by_key: Dict[Tuple[str, str], set] = defaultdict(set)
        for item in records:
            for node in item["extraction"]["nodes"]:
                folded = node["identifier"].casefold()
                node_type = node["type"]
                types_by_name[folded].add(node_type)
                spellings_by_key[(folded, node_type)].add(node["identifier"])

        # A concept frequently arrives as both "Multiple sclerosis" and
        # "multiple sclerosis" when independent documents are extracted.  The
        # two spellings slug to the same file, so retaining both identifiers
        # causes the later file to overwrite the first and leaves unresolved
        # edge targets.  Pick one deterministic display spelling per
        # case-insensitive identifier + type before materializing any node.
        canonical_spelling: Dict[Tuple[str, str], str] = {
            key: min(values, key=lambda value: (value == value.lower(), value.casefold(), value))
            for key, values in spellings_by_key.items()
        }
        result: Dict[str, Dict[Tuple[str, str], str]] = {}
        for item in records:
            local: Dict[Tuple[str, str], str] = {}
            for node in item["extraction"]["nodes"]:
                identifier, node_type = node["identifier"], node["type"]
                folded = identifier.casefold()
                canonical = canonical_spelling[(folded, node_type)]
                local[(identifier, node_type)] = (
                    f"{canonical} ({node_type.lower()})"
                    if len(types_by_name[folded]) > 1 else canonical
                )
            result[item["document_id"]] = local
        return result

    @staticmethod
    def _node_type_for(record: Dict[str, Any], identifier: str) -> str:
        matches = [n["type"] for n in record["extraction"]["nodes"] if n["identifier"] == identifier]
        if len(matches) != 1:
            raise BioOKFError(f"identifier {identifier!r} is missing or ambiguous within one extraction")
        return matches[0]

    @staticmethod
    def _source_node(document: Dict[str, Any], identifier: str, raw_rel: str) -> Dict[str, Any]:
        source = document.get("source", "pmc")
        if str(source).startswith("trial"):
            node_type, subtype = "Study", "clinical_trial"
        elif str(source).startswith("fda"):
            node_type, subtype = "Publication", "regulatory_document"
        else:
            node_type = "Publication"
            subtype = _subtype(document.get("article_type") or ("preprint" if source in {"biorxiv", "medrxiv"} else "article"))
        xref = []
        for prefix, key in (("DOI", "doi"), ("PMID", "pmid"), ("PMC", "pmc_id")):
            if document.get(key):
                raw = str(document[key])
                if prefix == "PMC" and raw.upper().startswith("PMC"):
                    raw = raw[3:]
                xref.append(f"{prefix}:{raw}")
        node = {
            "type": node_type,
            "identifier": identifier,
            "subtype": subtype,
            "description": document.get("abstract_snippet") or document.get("tldr") or "Paperclip source document.",
            "synonyms": [], "xref": xref, "raw_source": [raw_rel],
            "publication_year": document.get("pub_year"),
            "publication_date": document.get("pub_date"),
            "paperclip_document_id": document.get("document_id"),
            "edges": [],
        }
        node["edges"].append(BioOKFBuilder._reported_in(identifier))
        return node

    @staticmethod
    def _concept_node(candidate: Dict[str, Any], identifier: str) -> Dict[str, Any]:
        return {
            "type": candidate["type"], "identifier": identifier,
            "subtype": _subtype(candidate["subtype"]),
            "description": candidate.get("description", ""),
            "synonyms": candidate.get("synonyms", []), "xref": candidate.get("xref", []),
            "edges": [],
        }

    @staticmethod
    def _claim_edge(candidate: Dict[str, Any], object_: str, source: str, document: Dict[str, Any]) -> Dict[str, Any]:
        lines = candidate["evidence_lines"]
        anchor = ",".join(_normalize_line(value) for value in lines)
        source_kind = str(document.get("source", ""))
        if source_kind == "local":
            url = f"raw/paperclip-{document_storage_id(document['document_id'])}/source.md#{anchor}"
        else:
            family = "trials" if source_kind.startswith("trial") else "fda" if source_kind.startswith("fda") else "papers"
            encoded_id = quote(str(document["document_id"]), safe="")
            url = f"https://paperclip.gxl.ai/citations/{family}/{encoded_id}#{anchor}"
        edge: Dict[str, Any] = {
            "predicate": candidate["predicate"], "object": object_,
            "knowledge_level": candidate["knowledge_level"],
            "agent_type": "text_mining_agent", "primary_source": source,
            "publications": [source], "evidence_lines": lines,
            "evidence_url": url, "note": candidate["statement"],
        }
        for field in ("effect_metric", "effect_size", "ci_lower", "ci_upper", "p_value", "sample_size", "direction"):
            if candidate.get(field) is not None:
                edge[field] = candidate[field]
        return edge

    @staticmethod
    def _reported_in(source: str) -> Dict[str, Any]:
        return {
            "predicate": "reported_in", "object": source,
            "knowledge_level": "knowledge_assertion", "agent_type": "automated_agent",
            "primary_source": source,
        }

    @staticmethod
    def _append_unique_edge(edges: List[Dict[str, Any]], edge: Dict[str, Any]) -> None:
        identity = (edge["predicate"], edge["object"], edge["primary_source"])
        for existing in edges:
            if (existing["predicate"], existing["object"], existing["primary_source"]) == identity:
                if "evidence_lines" in edge:
                    existing["evidence_lines"] = sorted(set(existing.get("evidence_lines", [])) | set(edge["evidence_lines"]))
                return
        edges.append(edge)

    @staticmethod
    def _node_output_paths(nodes: Dict[str, Dict[str, Any]]) -> Dict[str, Path]:
        groups: Dict[Tuple[str, str], List[str]] = defaultdict(list)
        for identifier, node in nodes.items():
            groups[(node["type"].lower(), slugify(identifier))].append(identifier)
        paths: Dict[str, Path] = {}
        for (node_type, stem), identifiers in groups.items():
            for identifier in sorted(identifiers, key=str.casefold):
                suffix = ""
                if len(identifiers) > 1:
                    suffix = "-" + hashlib.sha256(identifier.encode("utf-8")).hexdigest()[:10]
                paths[identifier] = Path("knowledge") / node_type / f"{stem}{suffix}.md"
        return paths

    def _write_node(
        self, node: Dict[str, Any], evidence: List[Dict[str, str]], relative_path: Path
    ) -> None:
        path = self.bundle / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        frontmatter = _node_yaml(node)
        body = [f"# {node['identifier']}", ""]
        if node.get("description"):
            body += [node["description"].strip(), ""]
        if evidence:
            body += ["## Evidence", ""]
            seen = set()
            for item in evidence:
                key = (item["statement"], item["url"])
                if key in seen:
                    continue
                seen.add(key)
                body.append(f"- {item['statement']} ([source]({item['url']}))")
            body.append("")
        path.write_text("---\n" + frontmatter + "---\n\n" + "\n".join(body), encoding="utf-8")

    def _write_index(self, nodes: Iterable[Dict[str, Any]]) -> None:
        if self.bokf:
            self._checked([self.bokf, "index", str(self.bundle)])
            return
        grouped: Dict[str, List[str]] = defaultdict(list)
        subtypes = set()
        for node in nodes:
            grouped[node["type"]].append(node["identifier"])
            subtypes.add(node["subtype"])
        lines = [f"# {self.name}", "", "## Identifier registry", ""]
        for node_type in sorted(grouped):
            lines += [f"### {node_type}", ""] + [f"- {value}" for value in sorted(grouped[node_type], key=str.casefold)] + [""]
        lines += ["## Subtypes in use", ""] + [f"- {value}" for value in sorted(subtypes)] + [""]
        (self.bundle / "index.md").write_text("\n".join(lines), encoding="utf-8")

    def _write_log(self, node_count: int, records: List[Dict[str, Any]]) -> None:
        path = self.bundle / "log.md"
        existing = path.read_text(encoding="utf-8") if path.exists() else "# Change log\n\n"
        entry = (
            f"## {dt.date.today().isoformat()}\n\n"
            f"- Generated candidate KB from {len(records)} Paperclip sources.\n"
            f"- Materialized {node_count} BioOKF nodes with line-addressable provenance.\n"
            f"- Requires scientific judgment review before canonical merge.\n\n"
        )
        path.write_text(existing.rstrip() + "\n\n" + entry, encoding="utf-8")

    def _write_manifest(self, run_dir: Path, search: Dict[str, Any], records: List[Dict[str, Any]], nodes: Dict[str, Dict[str, Any]]) -> Dict[str, Any]:
        edge_count = sum(len(node["edges"]) for node in nodes.values())
        manifest = {
            "format": "paperclip-biookf-run/v1", "created_at": dt.datetime.now(dt.timezone.utc).isoformat(),
            "query": search.get("query"), "paperclip_searches": search.get("searches", []),
            "curation": search.get("curation"),
            "source_count": len(records), "node_count": len(nodes), "edge_count": edge_count,
            "sources": [
                {
                    "document_id": item["document_id"], "title": item.get("title"),
                    "agent": item.get("agent"), "source_sha256": item.get("source_sha256"),
                    "extraction_sha256": item.get("extraction_sha256"),
                }
                for item in records
            ],
            "name": self.name, "bundle": str(self.bundle),
            "candidate_status": "requires_judgment_review",
        }
        operations = self.bundle / "operations"
        run_name = run_dir.name or "run"
        (operations / f"{slugify(run_name)}.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
        return manifest

    def _internal_verify(self) -> Dict[str, Any]:
        errors: List[str] = []
        identifiers: Dict[str, Path] = {}
        parsed: List[Tuple[Path, Dict[str, Any]]] = []
        for path in sorted((self.bundle / "knowledge").rglob("*.md")):
            data = _parse_generated_frontmatter(path.read_text(encoding="utf-8"))
            parsed.append((path, data))
            identifier = data.get("identifier")
            if not identifier:
                errors.append(f"{path}: missing identifier")
            elif identifier in identifiers:
                errors.append(f"{path}: duplicate identifier {identifier!r}")
            else:
                identifiers[identifier] = path
            if data.get("type") not in NODE_TYPES:
                errors.append(f"{path}: invalid type {data.get('type')!r}")
            expected = str(data.get("type", "")).lower()
            if path.parent.name != expected:
                errors.append(f"{path}: path/type mismatch")
        for path, data in parsed:
            for edge in data.get("edges", []):
                if edge.get("predicate") not in PREDICATES:
                    errors.append(f"{path}: invalid predicate {edge.get('predicate')!r}")
                for field in ("object", "primary_source"):
                    if edge.get(field) not in identifiers:
                        errors.append(f"{path}: unresolved {field} {edge.get(field)!r}")
                if edge.get("knowledge_level") not in KNOWLEDGE_LEVELS:
                    errors.append(f"{path}: invalid knowledge_level")
                if not edge.get("agent_type"):
                    errors.append(f"{path}: missing agent_type")
        return {"ok": not errors, "errors": errors, "node_count": len(identifiers)}

    @staticmethod
    def _checked(argv: List[str]) -> None:
        completed = subprocess.run(argv, text=True, capture_output=True)
        if completed.returncode != 0:
            raise BioOKFError(f"command failed: {' '.join(argv)}\n{completed.stderr or completed.stdout}")


def _node_yaml(node: Dict[str, Any]) -> str:
    lines = [
        f"type: {_yaml_scalar(node['type'])}",
        f"identifier: {_yaml_scalar(node['identifier'])}",
        f"subtype: {_yaml_scalar(node['subtype'])}",
    ]
    for field in ("description", "synonyms", "xref", "raw_source", "publication_year", "publication_date", "paperclip_document_id"):
        value = node.get(field)
        if value not in (None, "", []):
            lines.append(f"{field}: {_yaml_value(value)}")
    lines.append("edges:")
    for edge in node.get("edges", []):
        first = True
        for key, value in edge.items():
            prefix = "  - " if first else "    "
            lines.append(f"{prefix}{key}: {_yaml_value(value)}")
            first = False
    return "\n".join(lines) + "\n"


def _yaml_value(value: Any) -> str:
    if isinstance(value, list):
        return json.dumps(value, ensure_ascii=False)
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, (int, float)):
        return json.dumps(value)
    return _yaml_scalar(str(value))


def _yaml_scalar(value: str) -> str:
    return json.dumps(value, ensure_ascii=False)


def _parse_generated_frontmatter(text: str) -> Dict[str, Any]:
    """Small parser for the deterministic YAML subset emitted above."""
    if not text.startswith("---\n") or "\n---\n" not in text[4:]:
        return {}
    raw = text[4:].split("\n---\n", 1)[0].splitlines()
    data: Dict[str, Any] = {}
    edges: List[Dict[str, Any]] = []
    current: Optional[Dict[str, Any]] = None
    for line in raw:
        if line == "edges:":
            data["edges"] = edges
            continue
        if line.startswith("  - "):
            current = {}
            edges.append(current)
            key, value = line[4:].split(": ", 1)
            current[key] = _parse_yaml_value(value)
        elif line.startswith("    ") and current is not None:
            key, value = line[4:].split(": ", 1)
            current[key] = _parse_yaml_value(value)
        elif ": " in line:
            key, value = line.split(": ", 1)
            data[key] = _parse_yaml_value(value)
    return data


def _parse_yaml_value(value: str) -> Any:
    try:
        return json.loads(value)
    except json.JSONDecodeError:
        return value


def slugify(value: str) -> str:
    normalized = unicodedata.normalize("NFKD", value).encode("ascii", "ignore").decode("ascii")
    slug = re.sub(r"[^a-zA-Z0-9]+", "-", normalized).strip("-").lower()
    return slug[:100] or "node"


def _subtype(value: str) -> str:
    return slugify(value).replace("-", "_")


def _normalize_line(value: str) -> str:
    if "-" not in value:
        return value
    start, end = value.split("-", 1)
    return start + "-" + (end if end.startswith("L") else "L" + end)


def _minimal_schema() -> str:
    return f"""# BioOKF operating schema (v0.5)

Generated candidate bundle. Normative specification:
https://github.com/Broccolito/BioOKF/blob/main/SPEC.md

## Controlled node types

{', '.join(NODE_TYPES)}

## Controlled predicates

{', '.join(PREDICATES)}

Every edge requires `predicate`, `object`, `knowledge_level`, `agent_type`, and
`primary_source`. The primary source names a source node in this bundle.
"""
