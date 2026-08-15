#!/usr/bin/env bash
# Guard against dead UI controls in the Studio frontend.
#
# The frontend is no-bundler vanilla JS: index.html declares the controls and
# app.js wires them up by id. Nothing links the two, so a control can ship
# looking perfectly normal and do nothing when clicked — which is exactly how
# the sidebar's "Open folder" item shipped inert. These checks are static (no
# browser needed) and run in the same spirit as app/hooks/tests/test-hooks.sh.
set -u
STUDIO="$(cd "$(dirname "$0")/.." && pwd)"

[ -f "$STUDIO/dist/index.html" ] || { echo "FAIL: missing dist/index.html"; exit 1; }
[ -f "$STUDIO/dist/app.js" ] || { echo "FAIL: missing dist/app.js"; exit 1; }

python3 - "$STUDIO" <<'PY'
import re
import sys
import pathlib

studio = pathlib.Path(sys.argv[1])
html = (studio / "dist" / "index.html").read_text()
js = (studio / "dist" / "app.js").read_text()
main_rs = (studio / "src-tauri" / "src" / "main.rs").read_text()

failed = False

# Every interactive element with an id must be referenced by app.js. A control
# nothing reads is a control that does nothing when clicked.
ids = re.findall(r"<(?:button|a|input|select|textarea)\b[^>]*\bid=\"([^\"]+)\"", html)
dead = [i for i in ids if i not in js]
if dead:
    failed = True
    print("FAIL: declared in index.html but never referenced in app.js:")
    for i in dead:
        print("  " + i)

# Every Tauri command app.js invokes must be registered in main.rs, or the call
# fails at runtime with no compile-time signal.
block = re.search(r"invoke_handler\(tauri::generate_handler!\[(.*?)\]\)", main_rs, re.S)
registered = set(re.findall(r"[a-z_][a-z0-9_]*", block.group(1))) if block else set()
quote = "[\"\x27]"
called = set(re.findall(r"tauriInvoke\(\s*" + quote + r"([a-z_][a-z0-9_]*)" + quote, js))
unregistered = sorted(called - registered)
if unregistered:
    failed = True
    print("FAIL: app.js invokes Tauri commands not registered in main.rs:")
    for c in unregistered:
        print("  " + c)

if failed:
    sys.exit(1)
print(f"wiring OK ({len(ids)} controls, {len(called)} commands)")
PY
