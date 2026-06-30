# biookf: Claude Code and Codex plugin

Curate, visualize, and reason over **BioOKF** (Biomedical Open Knowledge Format)
knowledge bases directly from your coding agent. This plugin bundles three things:

- **`bokf` MCP server**: 33 tools for curation, analysis, and **live control of the
  BioOKF Studio desktop app** (the `bokf_studio_*` family).
- **`bokf` CLI**: the same engine as a command-line tool.
- **BioOKF Studio**: a desktop GUI that visualizes a knowledge base as an
  interactive graph. Shipped **prebuilt**, so you never compile it.

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

## How it works

The Claude Code and Codex manifests each register one MCP server whose command is
`scripts/bokf-mcp`. That launcher:

1. Detects your OS/arch and, on first run, downloads
   `biookf-<platform>.tar.gz` from `Broccolito/BioOKF` Releases.
2. De-quarantines the unsigned binaries (macOS) so they launch cleanly.
3. Execs `bokf-mcp`, with `BIOOKF_STUDIO_BIN` pointed at the bundled Studio app so that
   `bokf_studio_open` can launch the GUI.

### Overrides (env)

| Variable | Purpose |
| --- | --- |
| `BIOOKF_VERSION` | Release tag to install (default: the plugin's pinned version). |
| `BIOOKF_HOME` | Cache root (default: `~/.local/share/biookf`). |
| `BIOOKF_REPO` | `owner/repo` of the release (default: `Broccolito/BioOKF`). |
| `BIOOKF_MCP_BIN` | Path to an existing `bokf-mcp` binary, which skips the download entirely (use a local `cargo build` for development). |

See the [project README](https://github.com/Broccolito/BioOKF#readme) for the full
tool reference and the knowledge-base format.
