"""Paperclip CLI adapter with JSON contracts and auditable command manifests."""

from __future__ import annotations

import hashlib
import json
import re
import shutil
import subprocess
import uuid
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, List, Optional
from urllib.parse import quote

from .constants import EXTRACTION_PROMPT, EXTRACTION_SCHEMA


class PaperclipError(RuntimeError):
    pass


DOCUMENT_ID = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._:-]{0,255}$")


def validate_document_id(value: Any) -> str:
    """Accept only one path-free Paperclip VFS identifier segment."""
    if not isinstance(value, str) or not DOCUMENT_ID.fullmatch(value):
        raise PaperclipError("Paperclip returned an unsafe document_id")
    return value


def document_storage_id(value: Any) -> str:
    """Return a collision-resistant local filename for a validated document id."""
    document_id = validate_document_id(value)
    stem = re.sub(r"[^A-Za-z0-9._-]+", "-", document_id).strip("-.")[:80] or "document"
    digest = hashlib.sha256(document_id.encode("utf-8")).hexdigest()[:12]
    return f"{stem}-{digest}"


@dataclass
class CommandResult:
    argv: List[str]
    stdout: str
    stderr: str


class PaperclipClient:
    def __init__(self, binary: Optional[str] = None) -> None:
        self.binary = binary or shutil.which("paperclip") or str(Path.home() / ".local/bin/paperclip")

    def available(self) -> bool:
        return Path(self.binary).is_file() or shutil.which(self.binary) is not None

    def _run(self, args: List[str]) -> CommandResult:
        if not self.available():
            raise PaperclipError(f"paperclip not found: {self.binary}")
        argv = [self.binary] + args
        completed = subprocess.run(argv, text=True, capture_output=True)
        if completed.returncode != 0:
            message = completed.stderr.strip() or completed.stdout.strip()
            raise PaperclipError(f"paperclip command failed ({completed.returncode}): {message}")
        return CommandResult(argv=argv, stdout=completed.stdout, stderr=completed.stderr)

    def doctor(self) -> Dict[str, Any]:
        if not self.available():
            return {"ok": False, "binary": self.binary, "error": "not found"}
        result = self._run(["config"])
        return {"ok": True, "binary": self.binary, "output": result.stdout.strip()}

    def search(
        self,
        query: str,
        source: str,
        limit: int,
        year_min: Optional[int] = None,
        year_max: Optional[int] = None,
        since: Optional[str] = None,
    ) -> Dict[str, Any]:
        alias = "p2b_" + uuid.uuid4().hex[:12]
        args = ["search", "-s", source, query, "-n", str(limit), "--quiet", "--save-as", alias]
        if year_min is not None:
            args += ["--year-min", str(year_min)]
        if year_max is not None:
            args += ["--year-max", str(year_max)]
        if since:
            args += ["--since", since]
        result = self._run(args)
        payload = _last_json_object(result.stdout)
        payload["command"] = result.argv
        return payload

    def extract(self, results_id: str, run_dir: Path, limit: Optional[int] = None) -> Dict[str, Any]:
        schema = json.dumps(EXTRACTION_SCHEMA, separators=(",", ":"))
        args = ["map", "--from", results_id, "--output-schema", schema]
        if limit is not None:
            args += ["-n", str(limit)]
        args.append(EXTRACTION_PROMPT)
        result = self._run(args)
        match = re.search(r"Results ID:\s*(m_[A-Za-z0-9]+)", result.stdout)
        if not match:
            raise PaperclipError("Paperclip map completed without a parseable Results ID")
        map_id = match.group(1)
        export_path = run_dir / "paperclip-map.txt"
        self._run(["results", map_id, "--save", str(export_path)])
        records = parse_map_export(export_path.read_text(encoding="utf-8"))
        return {
            "map_id": map_id,
            "command": result.argv,
            "records": records,
        }

    def snapshot_document(self, document: Dict[str, Any], destination: Path) -> Dict[str, str]:
        doc_id = validate_document_id(document.get("document_id"))
        source = document.get("source", "pmc")
        vfs_root = _vfs_root(source, doc_id)
        destination.mkdir(parents=True, exist_ok=True)
        meta = self._run(["cat", f"{vfs_root}/meta.json"]).stdout
        content = self._run(["cat", "--full", f"{vfs_root}/content.lines"]).stdout
        (destination / "original.meta.json").write_text(meta, encoding="utf-8")
        (destination / "content.lines").write_text(content, encoding="utf-8")
        source_md = _content_lines_to_markdown(document, content)
        (destination / "source.md").write_text(source_md, encoding="utf-8")
        meta_yaml = _paperclip_meta_yaml(document, vfs_root)
        (destination / "meta.yaml").write_text(meta_yaml, encoding="utf-8")
        return {
            "vfs_root": vfs_root,
            "raw_source": str(destination / "source.md"),
        }


def parse_map_export(text: str) -> List[Dict[str, Any]]:
    records: List[Dict[str, Any]] = []
    header = re.compile(r"^--- \[(\d+)\] \[([^]]+)\] (.+) ---$")
    lines = text.splitlines()
    index = 0
    while index < len(lines):
        match = header.match(lines[index])
        if not match:
            index += 1
            continue
        status, title = match.group(2), match.group(3)
        index += 1
        doc_id = ""
        if index < len(lines) and lines[index].startswith("doc_id:"):
            doc_id = lines[index].split(":", 1)[1].strip()
            index += 1
        payload_lines: List[str] = []
        while index < len(lines) and not header.match(lines[index]):
            if lines[index].strip():
                payload_lines.append(lines[index])
            index += 1
        record: Dict[str, Any] = {"status": status, "title": title, "document_id": doc_id}
        if status == "success":
            try:
                record["extraction"] = json.loads("\n".join(payload_lines))
            except json.JSONDecodeError as exc:
                raise PaperclipError(f"Invalid JSON extraction for {doc_id}: {exc}") from exc
        else:
            record["error"] = "\n".join(payload_lines)
        records.append(record)
    if not records:
        raise PaperclipError("No per-document records found in Paperclip map export")
    return records


def _last_json_object(text: str) -> Dict[str, Any]:
    for line in reversed(text.splitlines()):
        line = line.strip()
        if line.startswith("{") and line.endswith("}"):
            try:
                value = json.loads(line)
            except json.JSONDecodeError:
                continue
            if isinstance(value, dict):
                return value
    raise PaperclipError("No JSON object found in Paperclip output")


def _vfs_root(source: str, doc_id: str) -> str:
    if source in {"pmc", "biorxiv", "medrxiv", "arxiv", "papers"}:
        return f"/papers/{doc_id}"
    if source.startswith("trial"):
        return f"/trials/{doc_id}"
    if source.startswith("fda"):
        return f"/fda/{doc_id}"
    return f"/papers/{doc_id}"


def _content_lines_to_markdown(document: Dict[str, Any], content: str) -> str:
    title = document.get("title") or document["document_id"]
    doc_id = quote(validate_document_id(document.get("document_id")), safe="")
    source_url = f"https://paperclip.gxl.ai/citations/papers/{doc_id}"
    return (
        f"# {title}\n\n"
        f"Paperclip full-text snapshot. Canonical line-addressable source: {source_url}\n\n"
        "```text\n"
        f"{content.rstrip()}\n"
        "```\n"
    )


def _paperclip_meta_yaml(document: Dict[str, Any], vfs_root: str) -> str:
    fields = {
        "source_type": "paperclip_vfs_snapshot",
        "paperclip_path": vfs_root,
        "document_id": document.get("document_id"),
        "title": document.get("title"),
        "doi": document.get("doi"),
        "pmid": document.get("pmid"),
        "publication_year": document.get("pub_year"),
        "publication_date": document.get("pub_date"),
    }
    return "\n".join(f"{key}: {json.dumps(value)}" for key, value in fields.items() if value is not None) + "\n"
