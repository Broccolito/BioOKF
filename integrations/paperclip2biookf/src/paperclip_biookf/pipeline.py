"""End-to-end orchestration shared by the CLI and local GUI."""

from __future__ import annotations

import datetime as dt
import hashlib
import json
from pathlib import Path
from typing import Any, Callable, Dict, List, Optional

from .agents import SubscriptionAgent
from .biookf import BioOKFBuilder, slugify, validate_extraction
from .constants import EXTRACTION_SCHEMA
from .paperclip import PaperclipClient, document_storage_id, validate_document_id


Progress = Callable[[str, str, Optional[Dict[str, Any]]], None]


class HarnessPipeline:
    def __init__(
        self,
        workspace: Path,
        paperclip_binary: Optional[str] = None,
        bokf_binary: Optional[str] = None,
    ) -> None:
        self.workspace = workspace.resolve()
        self.workspace.mkdir(parents=True, exist_ok=True)
        self.paperclip = PaperclipClient(paperclip_binary)
        self.bokf_binary = bokf_binary

    def discover(
        self,
        query: str,
        sources: List[str],
        limit: int,
        year_min: Optional[int] = None,
        year_max: Optional[int] = None,
        since: Optional[str] = None,
        progress: Optional[Progress] = None,
    ) -> Dict[str, Any]:
        if not query.strip():
            raise ValueError("query cannot be empty")
        if not sources:
            raise ValueError("select at least one Paperclip source")
        papers: Dict[str, Dict[str, Any]] = {}
        searches = []
        for index, source in enumerate(sources, start=1):
            _progress(progress, "search", f"Searching {source} ({index}/{len(sources)})", {"source": source})
            result = self.paperclip.search(query, source, limit, year_min, year_max, since)
            searches.append({
                "source": source,
                "results_id": result.get("results_id"),
                "count": result.get("count", 0),
                "command": result.get("command"),
            })
            for paper in result.get("papers", []):
                normalized = dict(paper)
                normalized["document_id"] = validate_document_id(paper.get("document_id"))
                normalized["paperclip_source_label"] = paper.get("source")
                normalized["source"] = source
                papers.setdefault(normalized["document_id"], normalized)
        ordered = sorted(
            papers.values(),
            key=lambda item: (item.get("score") is None, -(item.get("score") or 0), -(item.get("pub_year") or 0)),
        )
        return {
            "format": "paperclip2biookf-search/v1",
            "query": query,
            "sources": sources,
            "year_min": year_min,
            "year_max": year_max,
            "since": since,
            "limit_per_source": limit,
            "searches": searches,
            "count": len(ordered),
            "papers": ordered,
        }

    def run(
        self,
        query: str,
        sources: List[str],
        limit: int,
        kb_name: str,
        agent_provider: str = "auto",
        model: Optional[str] = None,
        custom_prompt: str = "",
        year_min: Optional[int] = None,
        year_max: Optional[int] = None,
        since: Optional[str] = None,
        register: bool = False,
        open_studio: bool = False,
        progress: Optional[Progress] = None,
    ) -> Dict[str, Any]:
        stamp = dt.datetime.now().strftime("%Y%m%d-%H%M%S")
        run_id = f"{stamp}-{slugify(query)[:42]}"
        run_dir = self.workspace / "runs" / run_id
        run_dir.mkdir(parents=True, exist_ok=False)
        kb_path = self.workspace / "knowledge-bases" / f"{slugify(kb_name)}-{stamp}"
        agent = SubscriptionAgent(agent_provider, model)

        _progress(progress, "search", "Discovering evidence", {"run_id": run_id})
        search = self.discover(query, sources, limit, year_min, year_max, since, progress)
        search["curation"] = {
            "agent": agent.describe(), "custom_prompt": custom_prompt,
            "schema": "biookf-extraction/v1",
        }
        (run_dir / "search.json").write_text(json.dumps(search, indent=2) + "\n", encoding="utf-8")
        if not search["papers"]:
            raise RuntimeError("Paperclip returned no documents for the selected filters")

        schema_path = run_dir / "biookf-extraction.schema.json"
        schema_path.write_text(json.dumps(EXTRACTION_SCHEMA, indent=2) + "\n", encoding="utf-8")
        source_root = run_dir / "sources"
        source_root.mkdir()
        _progress(progress, "snapshot", f"Snapshotting {len(search['papers'])} sources", None)
        for index, document in enumerate(search["papers"], start=1):
            _progress(progress, "snapshot", f"Fetching {index}/{len(search['papers'])}: {document.get('title', document['document_id'])}", {"document_id": document["document_id"]})
            self.paperclip.snapshot_document(
                document, source_root / document_storage_id(document["document_id"])
            )

        _progress(progress, "extract", f"Curating with {agent.provider}", agent.describe())
        records = []
        for index, document in enumerate(search["papers"], start=1):
            doc_id = document["document_id"]
            stored_id = document_storage_id(doc_id)
            _progress(progress, "extract", f"Curating {index}/{len(search['papers'])}: {document.get('title', doc_id)}", {"document_id": doc_id})
            output = run_dir / f"extraction-{stored_id}.json"
            try:
                payload = agent.extract(source_root / stored_id, output, schema_path, custom_prompt)
                errors = validate_extraction(payload)
                if errors:
                    raise RuntimeError("; ".join(errors))
                source_bytes = (source_root / stored_id / "source.md").read_bytes()
                extraction_bytes = json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
                records.append({
                    "status": "success", "title": document.get("title", doc_id),
                    "document_id": doc_id, "extraction": payload,
                    "agent": agent.describe(),
                    "source_sha256": hashlib.sha256(source_bytes).hexdigest(),
                    "extraction_sha256": hashlib.sha256(extraction_bytes).hexdigest(),
                })
            except Exception as exc:
                records.append({
                    "status": "failed", "title": document.get("title", doc_id),
                    "document_id": doc_id, "error": str(exc), "agent": agent.describe(),
                })
        (run_dir / "extractions.json").write_text(json.dumps(records, indent=2) + "\n", encoding="utf-8")
        if not any(item["status"] == "success" for item in records):
            errors = []
            for item in records:
                error = str(item.get("error", "unknown extraction error"))
                if error not in errors:
                    errors.append(error)
            detail = "; ".join(errors[:3])
            raise RuntimeError(
                f"all subscription-agent extractions failed: {detail}. "
                f"Full diagnostics: {run_dir / 'extractions.json'}"
            )

        _progress(progress, "build", "Materializing BioOKF candidate bundle", {"bundle": str(kb_path)})
        builder = BioOKFBuilder(kb_path, kb_name, self.bokf_binary)
        manifest = builder.build(run_dir, search, records)
        studio = None
        if register:
            _progress(progress, "studio", "Registering bundle in BioOKF Studio", None)
            studio = builder.register_for_studio(open_studio=open_studio)
        result = {
            "run_id": run_id, "run_dir": str(run_dir), "bundle": str(kb_path),
            "search": {"count": search["count"], "sources": sources},
            "agent": agent.describe(), "manifest": manifest, "studio": studio,
            "failed_documents": [item for item in records if item["status"] != "success"],
        }
        (run_dir / "result.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
        _progress(progress, "done", "Knowledge base ready", result)
        return result


def _progress(callback: Optional[Progress], phase: str, message: str, data: Optional[Dict[str, Any]]) -> None:
    if callback:
        callback(phase, message, data)
