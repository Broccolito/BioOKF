#!/usr/bin/env python3
"""Machine-local subscription workflows invoked by BioOKF Studio.

The Tauri process owns validation, connection policy, progress events and
registration.  This helper reuses the strict paperclip2bioOKF extraction
contract for local papers and gives Codex/Claude a constrained filesystem for
LLM-assisted KB reconciliation.
"""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

from paperclip_biookf.agents import SubscriptionAgent
from paperclip_biookf.biookf import BioOKFBuilder, slugify, validate_extraction
from paperclip_biookf.constants import EXTRACTION_SCHEMA


SUPPORTED = {".pdf", ".md", ".txt", ".rst", ".docx"}


def progress(phase: str, message: str) -> None:
    print(f"[{phase}] {message}", file=sys.stderr, flush=True)


def checked(argv: list[str], **kwargs) -> subprocess.CompletedProcess[str]:
    env = dict(os.environ)
    for name in (
        "OPENAI_API_KEY", "CODEX_API_KEY", "ANTHROPIC_API_KEY",
        "CLAUDE_CODE_USE_BEDROCK", "CLAUDE_CODE_USE_VERTEX",
        "CLAUDE_CODE_USE_FOUNDRY", "AWS_BEARER_TOKEN_BEDROCK",
        "ANTHROPIC_VERTEX_PROJECT_ID",
    ):
        env.pop(name, None)
    kwargs.setdefault("env", env)
    completed = subprocess.run(argv, text=True, capture_output=True, **kwargs)
    if completed.returncode != 0:
        detail = completed.stderr.strip() or completed.stdout.strip()
        raise RuntimeError(f"command failed ({completed.returncode}): {' '.join(argv)}\n{detail[-4000:]}")
    return completed


def read_local_document(path: Path) -> str:
    suffix = path.suffix.lower()
    if suffix == ".pdf":
        return checked([shutil.which("pdftotext") or "pdftotext", "-layout", str(path), "-"]).stdout
    if suffix == ".docx":
        return checked(["/usr/bin/textutil", "-convert", "txt", "-stdout", str(path)]).stdout
    return path.read_text(encoding="utf-8", errors="replace")


def source_packet(path: Path, destination: Path, document_id: str) -> dict:
    text = read_local_document(path).replace("\x00", "")
    if not text.strip():
        raise RuntimeError(f"no readable text extracted from {path.name}")
    # Keep a single source within a subscription model's practical context.
    text = text[:750_000]
    lines = text.splitlines()
    numbered = "\n".join(f"L{i}: {line}" for i, line in enumerate(lines, 1))
    title = path.stem.replace("_", " ").strip() or path.name
    destination.mkdir(parents=True)
    source_md = (
        f"# {title}\n\nLocal source imported from `{path}`.\n\n"
        f"```text\n{numbered}\n```\n"
    )
    meta = {
        "document_id": document_id,
        "title": title,
        "source": "local",
        "local_path": str(path),
        "filename": path.name,
    }
    (destination / "source.md").write_text(source_md, encoding="utf-8")
    (destination / "content.lines").write_text(numbered + "\n", encoding="utf-8")
    (destination / "original.meta.json").write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")
    (destination / "meta.yaml").write_text(
        "source_type: local_file\n" + f"original_path: {json.dumps(str(path))}\n",
        encoding="utf-8",
    )
    return meta


def register(builder: BioOKFBuilder, name: str) -> dict:
    base = slugify(name)
    used = set()
    bokf = shutil.which("bokf")
    if bokf:
        listed = subprocess.run([bokf, "register", "--list"], text=True, capture_output=True)
        if listed.returncode == 0:
            used = {line.split(None, 1)[0] for line in listed.stdout.splitlines() if line.strip()}
    candidate, suffix = base, 2
    while candidate in used:
        candidate = f"{base}-{suffix}"
        suffix += 1
    return builder.register_for_studio(kb_id=candidate, open_studio=False)


def chat_with_base(args: argparse.Namespace) -> dict:
    bundle = Path(args.bundle).expanduser().resolve()
    if not bundle.is_dir():
        raise ValueError("knowledge base must be an existing directory")
    agent = SubscriptionAgent(args.provider, args.model)
    bokf = shutil.which("bokf") or "bokf"
    hits = json.loads(checked([bokf, "search", str(bundle), args.question, "--limit", "14", "--json"]).stdout)
    with tempfile.TemporaryDirectory(prefix="biookf-chat-") as temp:
        export_path = Path(temp) / "bundle.json"
        checked([bokf, "export", str(bundle), "--out", str(export_path)])
        exported = json.loads(export_path.read_text(encoding="utf-8"))
        pages = exported.get("pages", {})
        wanted = {item.get("identifier") for item in hits if item.get("identifier")}
        for identifier in list(wanted):
            page = pages.get(identifier, {})
            for edge in page.get("edges", [])[:16]:
                if len(wanted) >= 36:
                    break
                wanted.add(edge.get("object"))
                wanted.add(edge.get("primary_source"))
        context_pages = {key: pages[key] for key in wanted if key in pages}
        context = json.dumps({"hits": hits, "nodes": context_pages}, indent=2)[:300_000]
        prompt = (
            "You are BioOKF's knowledge-base analyst. Answer only from the supplied retrieved nodes and edges. "
            "Distinguish explicit evidence, association, prediction, contradiction, and missing evidence. "
            "Cite node identifiers in square brackets and include evidence_url values when present. "
            "Do not use external knowledge or access the network.\n\n"
            f"QUESTION\n{args.question}\n\nRETRIEVED BIOOKF CONTEXT\n{context}"
        )
        if args.provider == "codex":
            output = Path(temp) / "answer.txt"
            argv = [
                "codex", "exec", "--ephemeral", "--skip-git-repo-check",
                "--ignore-user-config", "--ignore-rules", "--sandbox", "read-only",
                "--cd", str(bundle), "--output-last-message", str(output), "--color", "never",
            ]
            if args.model:
                argv += ["--model", args.model]
            argv.append(prompt)
            checked(argv, timeout=1800)
            answer = output.read_text(encoding="utf-8", errors="replace").strip()
        else:
            argv = [
                "claude", "--print", "--no-session-persistence", "--safe-mode",
                "--permission-mode", "dontAsk", "--tools", "", "--output-format", "text",
            ]
            if args.model:
                argv += ["--model", args.model]
            answer = checked(argv, cwd=bundle, input=prompt, timeout=1800).stdout.strip()
    return {
        "workflow": "chat", "bundle": str(bundle), "question": args.question,
        "answer": answer, "context_nodes": len(context_pages), "agent": agent.describe(),
    }


def create_local(args: argparse.Namespace) -> dict:
    source = Path(args.source).expanduser().resolve()
    if not source.is_dir():
        raise ValueError("local paper source must be a directory")
    files = sorted(p for p in source.rglob("*") if p.is_file() and p.suffix.lower() in SUPPORTED)
    if not files:
        raise RuntimeError("the selected folder contains no PDF, DOCX, Markdown, or text papers")
    if len(files) > args.max_files:
        files = files[: args.max_files]
    stamp = dt.datetime.now().strftime("%Y%m%d-%H%M%S")
    workspace = Path(args.workspace).expanduser().resolve()
    run_dir = workspace / "runs" / f"{stamp}-local-{slugify(args.name)[:36]}"
    bundle = workspace / "knowledge-bases" / f"{slugify(args.name)}-{stamp}"
    source_root = run_dir / "sources"
    source_root.mkdir(parents=True)
    agent = SubscriptionAgent(args.provider, args.model)
    schema_path = run_dir / "biookf-extraction.schema.json"
    schema_path.write_text(json.dumps(EXTRACTION_SCHEMA, indent=2) + "\n", encoding="utf-8")

    documents = []
    progress("local", f"Reading {len(files)} local papers")
    for index, path in enumerate(files, 1):
        with path.open("rb") as handle:
            prefix = handle.read(65536)
        digest = hashlib.sha256(str(path).encode() + prefix).hexdigest()[:16]
        document_id = f"local_{digest}"
        progress("local", f"Converting {index}/{len(files)}: {path.name}")
        try:
            documents.append(source_packet(path, source_root / document_id, document_id))
        except Exception as exc:
            progress("warning", f"Skipping {path.name}: {exc}")
    if not documents:
        raise RuntimeError("none of the selected local papers could be converted to text")

    search = {
        "format": "paperclip2biookf-local/v1",
        "query": f"Local folder: {source}",
        "sources": ["local"],
        "count": len(documents),
        "papers": documents,
        "curation": {"agent": agent.describe(), "schema": "biookf-extraction/v1"},
    }
    (run_dir / "search.json").write_text(json.dumps(search, indent=2) + "\n", encoding="utf-8")
    records = []
    for index, document in enumerate(documents, 1):
        doc_id = document["document_id"]
        progress("extract", f"Curating {index}/{len(documents)}: {document['title']}")
        output = run_dir / f"extraction-{doc_id}.json"
        try:
            payload = agent.extract(source_root / doc_id, output, schema_path)
            errors = validate_extraction(payload)
            if errors:
                raise RuntimeError("; ".join(errors))
            records.append({
                "status": "success", "title": document["title"], "document_id": doc_id,
                "extraction": payload, "agent": agent.describe(),
            })
        except Exception as exc:
            records.append({
                "status": "failed", "title": document["title"], "document_id": doc_id,
                "error": str(exc), "agent": agent.describe(),
            })
            progress("warning", f"Extraction failed for {document['title']}: {exc}")
    (run_dir / "extractions.json").write_text(json.dumps(records, indent=2) + "\n", encoding="utf-8")
    if not any(item["status"] == "success" for item in records):
        raise RuntimeError(f"all local-paper extractions failed; inspect {run_dir / 'extractions.json'}")

    progress("build", "Materializing and verifying local BioOKF bundle")
    builder = BioOKFBuilder(bundle, args.name, shutil.which("bokf"))
    manifest = builder.build(run_dir, search, records)
    studio = register(builder, args.name)
    result = {
        "workflow": "local", "run_dir": str(run_dir), "bundle": str(bundle),
        "agent": agent.describe(), "manifest": manifest, "studio": studio,
        "failed_documents": [item for item in records if item["status"] != "success"],
    }
    (run_dir / "result.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    progress("done", "Local knowledge base ready")
    return result


MERGE_PROMPT = """\
You are merging BioOKF v0.5 knowledge bases on the local filesystem. Work only
inside the current run directory. `target/` is a copy of the first knowledge
base and is canonical. Every directory below `inputs/` is a secondary KB.

Merge all secondary KBs into target while preserving evidence and provenance:
1. Run `bokf merge-snapshot target` before editing.
2. Run `bokf merge-raw target <secondary>` for each secondary input.
3. Inspect all knowledge Markdown. Match identifiers case-insensitively and use
   identity/type, synonyms and xrefs to reconcile true duplicates. Preserve the
   target spelling and path for existing concepts. Do not merge homonyms with
   different biological identity or type.
4. For duplicates, union synonyms, xrefs, raw_source and distinct edges; merge
   useful prose without deleting contradictions or negative findings. Rewrite
   secondary edge objects and primary_source values to canonical identifiers.
5. Copy genuinely new nodes into the appropriate target knowledge/<type>/
   directory. Every edge object and primary_source must resolve to a target node.
6. Deduplicate only identical (subject, predicate, object, primary_source)
   claims; preserve differing sources, quantitative fields and polarity.
7. Regenerate the index with `bokf index target`, verify the primary snapshot
   with `bokf merge-snapshot --verify target`, then run `bokf verify target
   --workflow merge --json`. Fix all errors and warnings before finishing.
8. Append a dated merge entry to target/log.md describing all input KBs and the
   provider/model used. Do not alter SCHEMA.md.

Return a short plain-text summary after the verified merge. Do not access the
network and do not modify anything outside this run directory.
"""


def run_merge_agent(run_dir: Path, provider: str, model: str | None) -> str:
    output = run_dir / "agent-summary.txt"
    if provider == "codex":
        argv = [
            "codex", "exec", "--ephemeral", "--skip-git-repo-check",
            "--ignore-user-config", "--ignore-rules", "--sandbox", "workspace-write",
            "--cd", str(run_dir), "--output-last-message", str(output), "--color", "never",
        ]
        if model:
            argv += ["--model", model]
        argv.append(MERGE_PROMPT)
        checked(argv, timeout=3600)
        return output.read_text(encoding="utf-8", errors="replace") if output.exists() else ""
    argv = [
        "claude", "--print", "--no-session-persistence", "--safe-mode",
        "--permission-mode", "dontAsk", "--allowedTools",
        "Read,Glob,Grep,Write,Edit,Bash(/usr/local/bin/bokf *)",
        "--output-format", "text",
    ]
    if model:
        argv += ["--model", model]
    completed = checked(argv, cwd=run_dir, input=MERGE_PROMPT, timeout=3600)
    output.write_text(completed.stdout, encoding="utf-8")
    return completed.stdout


def merge_bases(args: argparse.Namespace) -> dict:
    # Construction is also the subscription-authentication gate.
    SubscriptionAgent(args.provider, args.model)
    inputs = [Path(value).expanduser().resolve() for value in args.inputs]
    if len(inputs) < 2 or any(not p.is_dir() for p in inputs):
        raise ValueError("select at least two existing BioOKF knowledge bases")
    stamp = dt.datetime.now().strftime("%Y%m%d-%H%M%S")
    workspace = Path(args.workspace).expanduser().resolve()
    run_dir = workspace / "runs" / f"{stamp}-merge-{slugify(args.name)[:36]}"
    input_root = run_dir / "inputs"
    target = run_dir / "target"
    input_root.mkdir(parents=True)
    progress("merge", f"Preparing {len(inputs)} knowledge bases")
    shutil.copytree(inputs[0], target)
    for index, source in enumerate(inputs[1:], 1):
        shutil.copytree(source, input_root / f"{index:02d}-{slugify(source.name)}")
    progress("merge", f"Reconciling concepts with {args.provider}")
    summary = run_merge_agent(run_dir, args.provider, args.model)
    progress("verify", "Running deterministic BioOKF merge verification")
    verify = checked([shutil.which("bokf") or "bokf", "verify", str(target), "--workflow", "merge", "--json"])
    report = json.loads(verify.stdout) if verify.stdout.strip() else {"ok": True}
    bundle = workspace / "knowledge-bases" / f"{slugify(args.name)}-{stamp}"
    shutil.move(str(target), str(bundle))
    builder = BioOKFBuilder(bundle, args.name, shutil.which("bokf"))
    studio = register(builder, args.name)
    result = {
        "workflow": "merge", "run_dir": str(run_dir), "bundle": str(bundle),
        "inputs": [str(p) for p in inputs], "agent": {"provider": args.provider, "model": args.model or "subscription default"},
        "verification": report, "summary": summary.strip(), "studio": studio,
    }
    (run_dir / "result.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    progress("done", "Merged knowledge base ready")
    return result


DOCTOR_PROMPT = """\
You are BioOKF Doctor, an evidence-backed manual curation agent. Work only in
the candidate BioOKF bundle at the current directory. The user instruction is
below.

Rules:
1. Inspect the relevant knowledge nodes and the cited raw/source material before
   changing anything. Never invent support and never use the network.
2. You may modify only knowledge/**/*.md and index.md. Never modify raw/,
   SCHEMA.md, log.md, or files outside this candidate.
3. For an edge review, trace primary_source to its Publication/Study/Dataset
   node and raw_source. Preserve exact quantitative, negative, and contradictory
   evidence. If the source is insufficient, leave the claim unchanged or mark
   the uncertainty in prose; explain why.
4. For a node merge, prove identity using type, synonyms, xrefs, and source
   context. Keep one canonical identifier, union distinct provenance and edges,
   rewrite every object and primary_source reference, and remove only the true
   duplicate. Never merge biological homonyms or different node types.
5. Every added or edited edge must have a resolving object and primary_source.
   Do not weaken evidence provenance to make validation pass.
6. Run `bokf index .` and `bokf verify . --workflow manual --json`; fix all
   errors before finishing.
7. Write `.doctor-result.json` containing JSON with: summary (string),
   rationale (string), evidence_checked (array of node/source identifiers), and
   unresolved (array of strings). Do not put Markdown fences around the JSON.

USER INSTRUCTION
{instruction}
"""


def tree_state(root: Path) -> dict[str, str]:
    state = {}
    for path in sorted(root.rglob("*")):
        if not path.is_file() or ".git" in path.parts:
            continue
        relative = path.relative_to(root).as_posix()
        state[relative] = hashlib.sha256(path.read_bytes()).hexdigest()
    return state


def clone_tree(source: Path, destination: Path) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    # APFS clone-on-write keeps Doctor practical even when raw/ contains PDFs.
    completed = subprocess.run(
        ["/bin/cp", "-cR", str(source), str(destination)],
        text=True, capture_output=True,
    )
    if completed.returncode != 0:
        shutil.copytree(source, destination)
    shutil.rmtree(destination / ".git", ignore_errors=True)


def restore_doctor_snapshot(original: Path, bundle: Path) -> None:
    for relative in ("knowledge", "index.md", "log.md"):
        target = bundle / relative
        source = original / relative
        if target.is_dir():
            shutil.rmtree(target)
        elif target.exists():
            target.unlink()
        if source.is_dir():
            shutil.copytree(source, target)
        elif source.exists():
            shutil.copy2(source, target)


def apply_doctor_candidate(candidate: Path, bundle: Path) -> None:
    knowledge = bundle / "knowledge"
    replacement = candidate / "knowledge"
    staged = bundle / ".knowledge.doctor-staged"
    if staged.exists():
        shutil.rmtree(staged)
    shutil.copytree(replacement, staged)
    if knowledge.exists():
        shutil.rmtree(knowledge)
    staged.rename(knowledge)
    shutil.copy2(candidate / "index.md", bundle / "index.md")


def run_doctor_agent(candidate: Path, provider: str, model: str | None, instruction: str) -> str:
    prompt = DOCTOR_PROMPT.format(instruction=instruction)
    output = candidate / ".doctor-agent-summary.txt"
    if provider == "codex":
        argv = [
            "codex", "exec", "--ephemeral", "--skip-git-repo-check",
            "--ignore-user-config", "--ignore-rules", "--sandbox", "workspace-write",
            "--cd", str(candidate), "--output-last-message", str(output), "--color", "never",
        ]
        if model:
            argv += ["--model", model]
        argv.append(prompt)
        checked(argv, timeout=3600)
        return output.read_text(encoding="utf-8", errors="replace") if output.exists() else ""
    bokf = shutil.which("bokf") or "bokf"
    argv = [
        "claude", "--print", "--no-session-persistence", "--safe-mode",
        "--permission-mode", "dontAsk", "--allowedTools",
        f"Read,Glob,Grep,Write,Edit,Bash(bokf *),Bash({bokf} *)", "--output-format", "text",
    ]
    if model:
        argv += ["--model", model]
    completed = checked(argv, cwd=candidate, input=prompt, timeout=3600)
    output.write_text(completed.stdout, encoding="utf-8")
    return completed.stdout


def doctor_base(args: argparse.Namespace) -> dict:
    bundle = Path(args.bundle).expanduser().resolve()
    if not bundle.is_dir():
        raise ValueError("knowledge base must be an existing directory")
    instruction = args.instruction.strip()
    if not instruction:
        raise ValueError("Doctor needs a revision instruction")
    agent = SubscriptionAgent(args.provider, args.model)
    bokf = shutil.which("bokf") or "bokf"
    progress("doctor", "Creating a reversible preflight checkpoint")
    checked([bokf, "commit", str(bundle), "--kind", "manual", "--summary", "Doctor preflight checkpoint"])

    stamp = dt.datetime.now().strftime("%Y%m%d-%H%M%S")
    workspace = Path(args.workspace).expanduser().resolve()
    run_dir = workspace / "runs" / f"{stamp}-doctor-{slugify(bundle.name)[:36]}"
    original, candidate = run_dir / "original", run_dir / "candidate"
    progress("doctor", "Cloning the selected knowledge base for isolated revision")
    clone_tree(bundle, original)
    clone_tree(original, candidate)
    baseline = tree_state(original)

    progress("doctor", f"Reviewing evidence with {args.provider}")
    agent_summary = run_doctor_agent(candidate, args.provider, args.model, instruction)
    result_file = candidate / ".doctor-result.json"
    if not result_file.is_file():
        raise RuntimeError("Doctor agent did not produce .doctor-result.json")
    try:
        report = json.loads(result_file.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise RuntimeError(f"Doctor result is not valid JSON: {exc}") from exc
    evidence_checked = report.get("evidence_checked")
    if not isinstance(evidence_checked, list) or not any(str(value).strip() for value in evidence_checked):
        raise RuntimeError("Doctor did not identify any evidence checked; no revision was applied")
    for transient in (result_file, candidate / ".doctor-agent-summary.txt"):
        transient.unlink(missing_ok=True)

    after = tree_state(candidate)
    all_paths = sorted(set(baseline) | set(after))
    changed = [path for path in all_paths if baseline.get(path) != after.get(path)]
    forbidden = [
        path for path in changed
        if path != "index.md" and not (path.startswith("knowledge/") and path.endswith(".md"))
    ]
    if forbidden:
        raise RuntimeError("Doctor attempted forbidden changes: " + ", ".join(forbidden))
    if not changed:
        raise RuntimeError("Doctor found no evidence-backed change to apply")

    progress("verify", "Verifying the isolated Doctor candidate")
    candidate_verify = json.loads(checked([
        bokf, "verify", str(candidate), "--workflow", "manual", "--json",
    ]).stdout)
    if tree_state(bundle) != baseline:
        raise RuntimeError("the active knowledge base changed during Doctor review; retry on the latest state")

    progress("apply", f"Applying {len(changed)} verified file changes")
    try:
        apply_doctor_candidate(candidate, bundle)
        final_verify = json.loads(checked([
            bokf, "verify", str(bundle), "--workflow", "manual", "--json",
        ]).stdout)
        evidence = [str(value) for value in evidence_checked]
        delta = (
            f"provider={args.provider}; model={args.model or 'subscription default'}; "
            f"files={', '.join(changed)}; evidence={', '.join(evidence) or 'none reported'}"
        )
        summary = str(report.get("summary") or instruction)[:240]
        commit = checked([
            bokf, "log-sync", str(bundle), "--kind", "manual",
            "--summary", f"Doctor: {summary}", "--delta", delta,
        ]).stderr.strip()
    except Exception:
        restore_doctor_snapshot(original, bundle)
        raise

    result = {
        "workflow": "doctor", "bundle": str(bundle), "run_dir": str(run_dir),
        "instruction": instruction, "summary": report.get("summary") or agent_summary.strip(),
        "rationale": report.get("rationale", ""),
        "evidence_checked": report.get("evidence_checked", []),
        "unresolved": report.get("unresolved", []), "changed_files": changed,
        "candidate_verification": candidate_verify, "verification": final_verify,
        "commit": commit, "agent": agent.describe(),
    }
    (run_dir / "result.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    progress("done", "Doctor revision applied, verified, and committed")
    return result


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser()
    sub = root.add_subparsers(dest="command", required=True)
    local = sub.add_parser("local")
    local.add_argument("--source", required=True)
    local.add_argument("--workspace", required=True)
    local.add_argument("--name", required=True)
    local.add_argument("--provider", choices=["codex", "claude"], required=True)
    local.add_argument("--model")
    local.add_argument("--max-files", type=int, default=25)
    merge = sub.add_parser("merge")
    merge.add_argument("--input", dest="inputs", action="append", required=True)
    merge.add_argument("--workspace", required=True)
    merge.add_argument("--name", required=True)
    merge.add_argument("--provider", choices=["codex", "claude"], required=True)
    merge.add_argument("--model")
    chat = sub.add_parser("chat")
    chat.add_argument("--bundle", required=True)
    chat.add_argument("--question", required=True)
    chat.add_argument("--provider", choices=["codex", "claude"], required=True)
    chat.add_argument("--model")
    doctor = sub.add_parser("doctor")
    doctor.add_argument("--bundle", required=True)
    doctor.add_argument("--workspace", required=True)
    doctor.add_argument("--instruction", required=True)
    doctor.add_argument("--provider", choices=["codex", "claude"], required=True)
    doctor.add_argument("--model")
    return root


def main() -> None:
    args = parser().parse_args()
    try:
        if args.command == "local":
            value = create_local(args)
        elif args.command == "merge":
            value = merge_bases(args)
        elif args.command == "chat":
            value = chat_with_base(args)
        else:
            value = doctor_base(args)
        print(json.dumps({"ok": True, **value}), flush=True)
    except Exception as exc:
        print(json.dumps({"ok": False, "error": str(exc)}), flush=True)
        raise SystemExit(1)


if __name__ == "__main__":
    main()
