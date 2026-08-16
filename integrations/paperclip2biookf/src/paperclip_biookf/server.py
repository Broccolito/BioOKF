"""Dependency-free local web UI for paperclip2bioOKF."""

from __future__ import annotations

import json
import threading
import traceback
import uuid
import webbrowser
from http import HTTPStatus
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any, Dict
from urllib.parse import urlparse

from .agents import model_catalog
from .biookf import BioOKFBuilder, slugify
from .cli import doctor
from .pipeline import HarnessPipeline


ALLOWED_SOURCES = {
    "pmc", "biorxiv", "medrxiv", "arxiv", "abstracts",
    "fda/us", "fda/eu", "fda/jp",
    "trials/us", "trials/eu", "trials/jp", "trials/cn", "trials",
}


class JobStore:
    def __init__(self) -> None:
        self.jobs: Dict[str, Dict[str, Any]] = {}
        self.lock = threading.Lock()

    def create(self, kind: str) -> str:
        job_id = uuid.uuid4().hex[:12]
        with self.lock:
            self.jobs[job_id] = {
                "id": job_id, "kind": kind, "status": "queued", "phase": "queued",
                "message": "Queued", "events": [], "result": None, "error": None,
            }
        return job_id

    def update(self, job_id: str, **values: Any) -> None:
        with self.lock:
            self.jobs[job_id].update(values)

    def event(self, job_id: str, phase: str, message: str, data: Any = None) -> None:
        with self.lock:
            job = self.jobs[job_id]
            job.update({"status": "running", "phase": phase, "message": message})
            job["events"].append({"phase": phase, "message": message, "data": data})
            job["events"] = job["events"][-80:]

    def get(self, job_id: str) -> Dict[str, Any]:
        with self.lock:
            if job_id not in self.jobs:
                raise KeyError(job_id)
            return json.loads(json.dumps(self.jobs[job_id]))


def serve(workspace: Path, host: str, port: int, open_browser: bool) -> None:
    pipeline = HarnessPipeline(workspace)
    store = JobStore()
    ui_path = Path(__file__).parent / "ui" / "index.html"
    html = ui_path.read_bytes()

    class Handler(BaseHTTPRequestHandler):
        server_version = "paperclip2bioOKF/0.1"

        def do_GET(self) -> None:
            path = urlparse(self.path).path
            if path == "/":
                self._bytes(HTTPStatus.OK, html, "text/html; charset=utf-8")
            elif path == "/api/doctor":
                try:
                    self._json(HTTPStatus.OK, doctor())
                except Exception as exc:
                    self._json(HTTPStatus.INTERNAL_SERVER_ERROR, {"error": str(exc)})
            elif path == "/api/models":
                self._json(HTTPStatus.OK, model_catalog())
            elif path == "/api/history":
                self._json(HTTPStatus.OK, {"runs": _run_history(pipeline.workspace)})
            elif path == "/api/bundles":
                self._json(HTTPStatus.OK, {"bundles": _generated_bundles(pipeline.workspace)})
            elif path.startswith("/api/jobs/"):
                try:
                    self._json(HTTPStatus.OK, store.get(path.rsplit("/", 1)[-1]))
                except KeyError:
                    self._json(HTTPStatus.NOT_FOUND, {"error": "job not found"})
            else:
                self._json(HTTPStatus.NOT_FOUND, {"error": "not found"})

        def do_POST(self) -> None:
            path = urlparse(self.path).path
            if path == "/api/bundles/open":
                try:
                    body = self._body()
                    bundle = _resolve_bundle(pipeline.workspace, body.get("bundle"))
                    name = str(body.get("name") or bundle.name)
                    result = BioOKFBuilder(bundle, name, pipeline.bokf_binary).register_for_studio(open_studio=True)
                    self._json(HTTPStatus.OK, result)
                except (ValueError, OSError, RuntimeError) as exc:
                    self._json(HTTPStatus.BAD_REQUEST, {"error": str(exc)})
                return
            if path not in {"/api/search", "/api/run"}:
                self._json(HTTPStatus.NOT_FOUND, {"error": "not found"})
                return
            try:
                body = self._body()
                request = _validate_request(body, build=path == "/api/run")
            except (ValueError, json.JSONDecodeError) as exc:
                self._json(HTTPStatus.BAD_REQUEST, {"error": str(exc)})
                return
            kind = "run" if path == "/api/run" else "search"
            job_id = store.create(kind)
            thread = threading.Thread(target=_worker, args=(pipeline, store, job_id, kind, request), daemon=True)
            thread.start()
            self._json(HTTPStatus.ACCEPTED, {"job_id": job_id})

        def log_message(self, format: str, *args: Any) -> None:
            return

        def _body(self) -> Dict[str, Any]:
            length = int(self.headers.get("Content-Length", "0"))
            if length > 200_000:
                raise ValueError("request too large")
            value = json.loads(self.rfile.read(length).decode("utf-8"))
            if not isinstance(value, dict):
                raise ValueError("request must be an object")
            return value

        def _json(self, status: HTTPStatus, value: Any) -> None:
            self._bytes(status, json.dumps(value).encode("utf-8"), "application/json")

        def _bytes(self, status: HTTPStatus, value: bytes, content_type: str) -> None:
            self.send_response(status)
            self.send_header("Content-Type", content_type)
            self.send_header("Content-Length", str(len(value)))
            self.send_header("Cache-Control", "no-store")
            self.send_header("X-Content-Type-Options", "nosniff")
            self.send_header("Content-Security-Policy", "default-src 'self'; style-src 'self' 'unsafe-inline'; script-src 'self' 'unsafe-inline'; connect-src 'self'")
            self.end_headers()
            self.wfile.write(value)

    server = ThreadingHTTPServer((host, port), Handler)
    url = f"http://{host}:{port}"
    print(f"paperclip2bioOKF listening on {url}")
    print(f"workspace: {pipeline.workspace}")
    if open_browser:
        threading.Timer(0.4, lambda: webbrowser.open(url)).start()
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()


def _read_json(path: Path) -> Dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
        return value if isinstance(value, dict) else {}
    except (OSError, json.JSONDecodeError):
        return {}


def _run_history(workspace: Path) -> list:
    bundles = workspace / "knowledge-bases"
    completed = {
        workspace / "runs" / operation.stem
        for operation in bundles.glob("*/operations/*.json")
        if operation.name != "latest-run.json"
    }
    items = []
    for run_dir in sorted((workspace / "runs").glob("*"), reverse=True):
        if not run_dir.is_dir():
            continue
        search = _read_json(run_dir / "search.json")
        try:
            records = json.loads((run_dir / "extractions.json").read_text(encoding="utf-8"))
            records = records if isinstance(records, list) else []
        except (OSError, json.JSONDecodeError):
            records = []
        successes = sum(item.get("status") == "success" for item in records if isinstance(item, dict))
        failures = sum(item.get("status") == "failed" for item in records if isinstance(item, dict))
        if run_dir in completed:
            status = "complete"
        elif records and successes == 0:
            status = "failed"
        elif records:
            status = "extracted"
        else:
            status = "incomplete"
        items.append({
            "id": run_dir.name,
            "query": search.get("query") or run_dir.name,
            "sources": search.get("sources", []),
            "documents": search.get("count", len(search.get("papers", []))),
            "successful": successes,
            "failed": failures,
            "status": status,
            "path": str(run_dir),
        })
    return items


def _generated_bundles(workspace: Path) -> list:
    items = []
    for bundle in sorted((workspace / "knowledge-bases").glob("*"), reverse=True):
        if not bundle.is_dir():
            continue
        manifest = _read_json(bundle / "operations" / "latest-run.json")
        if not manifest:
            continue
        verification = manifest.get("verification") or {}
        name = manifest.get("name")
        if not name:
            try:
                first_line = (bundle / "index.md").read_text(encoding="utf-8").splitlines()[0]
                name = first_line.lstrip("# ").strip()
            except (OSError, IndexError):
                name = bundle.name
        items.append({
            "name": name,
            "bundle": str(bundle.resolve()),
            "query": manifest.get("query"),
            "created_at": manifest.get("created_at"),
            "source_count": manifest.get("source_count", 0),
            "node_count": manifest.get("node_count", 0),
            "edge_count": manifest.get("edge_count", 0),
            "verified": bool((verification.get("internal") or {}).get("ok")),
        })
    return items


def _resolve_bundle(workspace: Path, raw: Any) -> Path:
    if not raw:
        raise ValueError("bundle path is required")
    root = (workspace / "knowledge-bases").resolve()
    bundle = Path(str(raw)).resolve()
    if bundle.parent != root or not (bundle / "SCHEMA.md").is_file():
        raise ValueError("bundle is not a generated BioOKF knowledge base")
    return bundle


def _worker(pipeline: HarnessPipeline, store: JobStore, job_id: str, kind: str, request: Dict[str, Any]) -> None:
    try:
        callback = lambda phase, message, data: store.event(job_id, phase, message, data)
        if kind == "search":
            result = pipeline.discover(
                request["query"], request["sources"], request["limit"],
                request.get("year_min"), request.get("year_max"), request.get("since"), callback,
            )
        else:
            result = pipeline.run(
                query=request["query"], sources=request["sources"], limit=request["limit"],
                kb_name=request["kb_name"], agent_provider=request["agent"],
                model=request.get("model"), custom_prompt=request.get("prompt", ""),
                year_min=request.get("year_min"), year_max=request.get("year_max"),
                since=request.get("since"), register=request.get("register", False),
                open_studio=request.get("open_studio", False), progress=callback,
            )
        store.update(job_id, status="complete", phase="done", message="Complete", result=result)
    except Exception as exc:
        store.update(job_id, status="failed", phase="failed", message=str(exc), error=str(exc), trace=traceback.format_exc())


def _validate_request(value: Dict[str, Any], build: bool) -> Dict[str, Any]:
    query = str(value.get("query", "")).strip()
    if not query:
        raise ValueError("Enter a search query")
    sources = value.get("sources")
    if not isinstance(sources, list) or not sources:
        raise ValueError("Select at least one Paperclip database")
    if any(source not in ALLOWED_SOURCES for source in sources):
        raise ValueError("Unknown Paperclip database")
    limit = int(value.get("limit", 3))
    if not 1 <= limit <= 25:
        raise ValueError("Limit must be between 1 and 25 per database")
    result = {"query": query, "sources": sources, "limit": limit}
    for field in ("year_min", "year_max"):
        raw = value.get(field)
        if raw not in (None, ""):
            parsed = int(raw)
            if not 1800 <= parsed <= 2100:
                raise ValueError(f"{field} is outside the supported range")
            result[field] = parsed
    if result.get("year_min") and result.get("year_max") and result["year_min"] > result["year_max"]:
        raise ValueError("Start year must not exceed end year")
    if value.get("since"):
        result["since"] = str(value["since"]).strip()
    if build:
        kb_name = str(value.get("kb_name", "")).strip()
        if not kb_name:
            raise ValueError("Enter a knowledge-base name")
        agent = value.get("agent", "auto")
        if agent not in {"auto", "codex", "claude"}:
            raise ValueError("Unknown subscription agent")
        result.update({
            "kb_name": kb_name, "agent": agent,
            "model": str(value.get("model", "")).strip() or None,
            "prompt": str(value.get("prompt", ""))[:20_000],
            "register": bool(value.get("register", False)),
            "open_studio": bool(value.get("open_studio", False)),
        })
    return result
