# biookf: Claude Code and Codex plugin

Curate, visualize, and reason over **BioOKF** (Biomedical Open Knowledge Format)
knowledge bases directly from your coding agent. This plugin bundles three things:

- **`bokf` MCP server**: 33 tools for curation, analysis, and **live control of the
  BioOKF Studio desktop app** (the `bokf_studio_*` family).
- **`bokf` CLI**: the same engine as a command-line tool.
- **BioOKF Studio**: a desktop GUI that visualizes a knowledge base as an
  interactive graph, provides grounded Chat and evidence-backed Doctor revision,
  computes network metrics and source-year evidence timelines, and exports a
  shareable, self-contained HTML graph view.
  Shipped **prebuilt**, so you never compile it.

## Install: Claude Code

In Claude Code:

```
/plugin marketplace add Broccolito/BioOKF
/plugin install biookf@biookf
```

Restart Claude Code. The first time a tool runs, the plugin downloads the prebuilt
binaries for your platform from the project's GitHub Release and caches them under
`~/.local/share/biookf`, with no build step.

## Install: Codex

The same plugin root also contains a Codex manifest at `.codex-plugin/plugin.json`
and a Codex skill at `skills/biookf/SKILL.md`. Add this plugin through the Codex
plugin manager from this repository or from a marketplace entry that points at
`plugins/biookf`:

```
codex plugin add biookf@<marketplace-name>
```

Both Claude Code and Codex use the same MCP launcher, `scripts/bokf-mcp`, so
release downloads, local overrides, and Studio control stay identical.

## Paperclip and local-subscription workflows

The CLI and Studio expose `generate-from-paperclip`, `create-local`, `chat`,
`merge-agent`, `doctor`, and `network-metrics`. Install the dependency-free bridge
from `integrations/paperclip2biookf` when using Paperclip generation:

```bash
python3 -m pip install -e ./integrations/paperclip2biookf
pc-biookf doctor
```

Codex and Claude Code authenticate through their native subscription CLIs. The
workflows remove API-key and cloud-provider variables before launching either
agent; BioOKF does not read or store subscription tokens.

## How it works

The Claude Code and Codex manifests each register one MCP server whose command is
`scripts/bokf-mcp`. That launcher resolves `bokf-mcp` in this order:

1. `BIOOKF_MCP_BIN`, if set — the escape hatch for a local `cargo build`.
2. An installed `BioOKF Studio.app` (`/Applications`, then `~/Applications`). The DMG already
   ships signed, notarized `bokf` and `bokf-mcp` inside the bundle, so these are used directly:
   no download, and `bokf_studio_open` launches the app you actually installed. A bundle newer
   than the pinned version is used as-is; an older one is skipped with a note on stderr.
3. Otherwise, on first run, it detects your OS/arch, downloads `biookf-<platform>.tar.gz` from
   `Broccolito/BioOKF` Releases into `~/.local/share/biookf`, and de-quarantines the unsigned
   binaries (macOS) so they launch cleanly.

In every case it execs `bokf-mcp` with `BIOOKF_STUDIO_BIN` pointed at the matching Studio app, so
`bokf_studio_open` can launch the GUI.

### Overrides (env)

| Variable | Purpose |
| --- | --- |
| `BIOOKF_VERSION` | Release tag to install (default: the plugin's pinned version). |
| `BIOOKF_HOME` | Cache root (default: `~/.local/share/biookf`). |
| `BIOOKF_REPO` | `owner/repo` of the release (default: `Broccolito/BioOKF`). |
| `BIOOKF_MCP_BIN` | Path to an existing `bokf-mcp` binary, which skips everything else (use a local `cargo build` for development). |
| `BIOOKF_STUDIO_APP` | Path to an installed Studio bundle, replacing the `/Applications` default in step 2. |

### Tests

`scripts/tests/test-launcher.sh` covers the resolution order with fake `.app` bundles and a
`curl` shim, so it never touches the network or the real cache:

```bash
./plugins/biookf/scripts/tests/test-launcher.sh   # prints "launcher OK"
```

See the [project README](https://github.com/Broccolito/BioOKF#readme) for the full
tool reference and the knowledge-base format.
