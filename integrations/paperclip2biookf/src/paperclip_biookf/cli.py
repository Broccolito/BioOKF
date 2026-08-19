"""Command-line entry point for paperclip2bioOKF."""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from pathlib import Path

from .agents import SubscriptionAgent
from .biookf import BioOKFBuilder
from .paperclip import PaperclipClient
from .pipeline import HarnessPipeline


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(prog="pc-biookf", description="Paperclip → subscription LLM → BioOKF Studio")
    root.add_argument("--workspace", type=Path, default=Path("./paperclip2biookf-output"))
    sub = root.add_subparsers(dest="command", required=True)

    sub.add_parser("doctor", help="Check Paperclip, Codex/Claude, bokf, and Studio")

    run = sub.add_parser("run", help="Discover evidence and build a new staging BioOKF bundle")
    run.add_argument("--query", required=True)
    run.add_argument("--source", action="append", dest="sources", required=True, help="Repeat for multiple Paperclip databases")
    run.add_argument("--limit", type=int, default=3, help="Maximum papers per selected source")
    run.add_argument("--year-min", type=int)
    run.add_argument("--year-max", type=int)
    run.add_argument("--since")
    run.add_argument("--kb-name", required=True)
    run.add_argument("--prompt", default="", help="Additional curation instructions")
    run.add_argument("--agent", choices=["auto", "codex", "claude"], default="auto")
    run.add_argument("--model")
    run.add_argument("--register", action="store_true")
    run.add_argument("--open-studio", action="store_true")

    discover = sub.add_parser("search", help="Preview a multi-database Paperclip search")
    discover.add_argument("--query", required=True)
    discover.add_argument("--source", action="append", dest="sources", required=True)
    discover.add_argument("--limit", type=int, default=5)
    discover.add_argument("--year-min", type=int)
    discover.add_argument("--year-max", type=int)
    discover.add_argument("--since")

    verify = sub.add_parser("verify", help="Verify an existing generated bundle")
    verify.add_argument("bundle", type=Path)

    studio = sub.add_parser("studio", help="Register a generated bundle in BioOKF Studio")
    studio.add_argument("bundle", type=Path)
    studio.add_argument("--name", required=True)
    studio.add_argument("--id")
    studio.add_argument("--open", action="store_true")

    ui = sub.add_parser("ui", help="Launch the local paperclip2bioOKF GUI")
    ui.add_argument("--host", default="127.0.0.1")
    ui.add_argument("--port", type=int, default=8765)
    ui.add_argument("--no-browser", action="store_true")
    return root


def main(argv=None) -> None:
    args = parser().parse_args(argv)
    try:
        if args.command == "doctor":
            result = doctor()
        elif args.command == "search":
            pipeline = HarnessPipeline(args.workspace)
            result = pipeline.discover(args.query, args.sources, args.limit, args.year_min, args.year_max, args.since, _print_progress)
        elif args.command == "run":
            pipeline = HarnessPipeline(args.workspace)
            result = pipeline.run(
                query=args.query, sources=args.sources, limit=args.limit, kb_name=args.kb_name,
                agent_provider=args.agent, model=args.model, custom_prompt=args.prompt,
                year_min=args.year_min, year_max=args.year_max, since=args.since,
                register=args.register or args.open_studio, open_studio=args.open_studio,
                progress=_print_progress,
            )
        elif args.command == "verify":
            result = BioOKFBuilder(args.bundle, args.bundle.name).verify()
        elif args.command == "studio":
            result = BioOKFBuilder(args.bundle, args.name).register_for_studio(args.id, args.open)
        elif args.command == "ui":
            from .server import serve
            serve(args.workspace, args.host, args.port, not args.no_browser)
            return
        else:
            raise RuntimeError("unreachable")
    except Exception as exc:
        print(json.dumps({"ok": False, "error": str(exc)}, indent=2), file=sys.stderr)
        raise SystemExit(1)
    print(json.dumps(result, indent=2))


def doctor():
    paperclip = PaperclipClient()
    agents = {}
    for provider in ("codex", "claude"):
        try:
            agents[provider] = {"ok": True, **SubscriptionAgent(provider).describe(), **_subscription_status(provider)}
        except Exception as exc:
            agents[provider] = {"ok": False, "error": str(exc)}
    studio = Path("/Applications/BioOKF Studio.app")
    return {
        "paperclip": paperclip.doctor(), "agents": agents,
        "bokf": {"ok": bool(shutil.which("bokf")), "binary": shutil.which("bokf")},
        "studio": {"ok": studio.exists(), "path": str(studio)},
    }


def _subscription_status(provider):
    argv = ["codex", "login", "status"] if provider == "codex" else ["claude", "auth", "status"]
    completed = subprocess.run(argv, text=True, capture_output=True, timeout=20)
    output = (completed.stdout + "\n" + completed.stderr).strip()
    if provider == "claude" and completed.returncode == 0:
        try:
            value = json.loads(completed.stdout)
            return {
                "authenticated": bool(value.get("loggedIn")),
                "auth_method": value.get("authMethod"),
                "subscription_type": value.get("subscriptionType"),
            }
        except json.JSONDecodeError:
            pass
    return {
        "authenticated": completed.returncode == 0 and "logged in" in output.lower(),
        "auth_method": "ChatGPT subscription" if provider == "codex" and "ChatGPT" in output else None,
    }


def _print_progress(phase, message, data):
    print(f"[{phase}] {message}", file=sys.stderr, flush=True)


if __name__ == "__main__":
    main()
