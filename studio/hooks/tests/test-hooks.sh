#!/usr/bin/env bash
# Deterministic tests for the BioOKF guardrail hooks: pipe synthetic tool_input JSON
# (the shape Claude Code passes on stdin) and assert the block/allow exit codes.
set -u
H="$(cd "$(dirname "$0")/.." && pwd)"
ROOT="$(cd "$H/.." && pwd)"
fail() { echo "FAIL: $1"; exit 1; }

run() { printf '%s' "$1" | "$H/$2"; echo $?; }

# guard-raw: editing an immutable raw original is BLOCKED (exit 2)
rc=$(run '{"tool_name":"Write","tool_input":{"file_path":"/kb/raw/abc-123/original.pdf","content":"x"}}' guard-raw.sh)
[ "$rc" = "2" ] || fail "guard-raw should block raw original (got $rc)"

# guard-raw: rewriting raw/<id>/source.md (the LLM-fallback rendering) is ALLOWED (exit 0)
rc=$(run '{"tool_name":"Write","tool_input":{"file_path":"/kb/raw/abc-123/source.md","content":"x"}}' guard-raw.sh)
[ "$rc" = "0" ] || fail "guard-raw should allow raw source.md (got $rc)"

# guard-raw: a knowledge concept doc is ALLOWED
rc=$(run '{"tool_name":"Write","tool_input":{"file_path":"/kb/knowledge/gene/il6.md","content":"x"}}' guard-raw.sh)
[ "$rc" = "0" ] || fail "guard-raw should allow knowledge write (got $rc)"

# post-write-lint: NEVER blocks (advisory), even on irrelevant paths
rc=$(run '{"tool_name":"Write","tool_input":{"file_path":"/tmp/x.txt"}}' post-write-lint.sh)
[ "$rc" = "0" ] || fail "post-write-lint must be non-blocking (got $rc)"

# stop-verify: with no active KB it is a no-op (exit 0) — it must never wedge a normal session
tmp=$(mktemp -d)
rc=$(cd "$tmp" && BOKF_BIN="$ROOT/target/debug/bokf" "$H/stop-verify.sh" </dev/null; echo $?)
rm -rf "$tmp"
[ "$rc" = "0" ] || fail "stop-verify must allow stop without an active KB (got $rc)"

echo "hooks OK"
