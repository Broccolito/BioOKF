# paperclip2bioOKF

`paperclip2bioOKF` is a local, provenance-first harness that turns evidence discovered with
Paperclip into a new BioOKF v0.5 staging knowledge base that can be registered and viewed in
BioOKF Studio.

The scientific extraction is performed by **Codex CLI or Claude Code using the user's existing
subscription login**. The harness does not request, store, or pass an OpenAI or Anthropic API key.

```text
Paperclip databases
        │ search + immutable full-text snapshot
        ▼
per-source evidence packet
        │ read-only structured curation
        ├──────── Codex subscription
        └──────── Claude subscription
        ▼
strict candidate JSON
        │ deterministic validation + materialization
        ▼
new BioOKF staging bundle ──────► BioOKF Studio
```

## Safety and scientific boundaries

- Every generation writes to a **new staging bundle**. It refuses to overwrite a non-empty KB.
- The subscription agent receives one local evidence packet at a time and is told not to use
  external sources.
- Codex runs with a read-only sandbox; Claude runs without tools in safe mode.
- Agent output must conform to a strict JSON Schema before any BioOKF Markdown is written.
- Every claim carries the BioOKF provenance triplet plus Paperclip line references and a direct
  citation URL.
- The output is marked `requires_judgment_review`; deterministic validation is not a substitute
  for scientific review.
- If the official `bokf` CLI is installed, its `verify --workflow ingest` gate runs after the
  internal checks.

## Requirements

- Python 3.9+
- Authenticated Paperclip CLI
- At least one authenticated subscription CLI:
  - `codex`, or
  - `claude`
- BioOKF Studio / `bokf` is optional for generation and required for automatic Studio
  registration and official verification.

## Install locally

```bash
cd paperclip-biookf-harness
python3 -m venv .venv
source .venv/bin/activate
python setup.py develop
pc-biookf doctor
```

No runtime Python packages or network package downloads are required. On newer Python tooling,
`python -m pip install -e .` is equivalent.

## Start the GUI

```bash
pc-biookf ui
```

The local GUI opens at `http://127.0.0.1:8765` and uses the visual language of BioOKF Studio.
It supports:

- Paperclip database toggles for PMC, preprints, arXiv, regulatory documents, and trial
  registries;
- a normal Paperclip search query;
- an optional knowledge-curation prompt for Codex or Claude;
- publication year range or a relative `since` window;
- per-database result limits;
- subscription-agent and optional model selection;
- evidence preview before generation;
- live phase status;
- optional registration/opening in BioOKF Studio.

Searches are executed separately per selected database and deduplicated by Paperclip document ID.
This avoids assuming that heterogeneous Paperclip backends can all participate in one combined
query.

## CLI examples

Preview evidence across literature and trials:

```bash
pc-biookf search \
  --query "EGFR acquired resistance in NSCLC" \
  --source pmc \
  --source trials/us \
  --year-min 2018 \
  --year-max 2025 \
  --limit 5
```

Generate with Codex subscription:

```bash
pc-biookf run \
  --query "EGFR acquired resistance in NSCLC" \
  --source pmc \
  --limit 3 \
  --year-min 2018 \
  --year-max 2025 \
  --kb-name "EGFR resistance landscape" \
  --agent codex \
  --prompt "Prioritize causal mechanisms, negative findings, and quantitative outcomes."
```

Generate with Claude subscription and register the result in Studio:

```bash
pc-biookf run \
  --query "BRAF inhibitor resistance melanoma" \
  --source pmc \
  --source biorxiv \
  --limit 3 \
  --kb-name "BRAF resistance landscape" \
  --agent claude \
  --register \
  --open-studio
```

The `auto` provider selects Codex first when both CLIs are available:

```bash
pc-biookf run ... --agent auto
```

## Generated layout

```text
paperclip2biookf-output/
├── runs/
│   └── <timestamp-query>/
│       ├── search.json
│       ├── biookf-extraction.schema.json
│       ├── extraction-<document-id>.json
│       ├── extractions.json
│       ├── result.json
│       └── sources/<document-id>/
│           ├── original.meta.json
│           ├── content.lines
│           ├── source.md
│           └── meta.yaml
└── knowledge-bases/
    └── <name-timestamp>/
        ├── raw/
        ├── knowledge/<type>/<slug>.md
        ├── operations/
        ├── index.md
        ├── log.md
        └── SCHEMA.md
```

Source nodes retain `publication_year`, `publication_date`, and `paperclip_document_id` as
frontmatter. This makes annual snapshots and temporal filters derivable without changing the
identity of scientific claims.

## Studio integration

When `bokf` is installed:

```bash
pc-biookf studio ./paperclip2biookf-output/knowledge-bases/<bundle> \
  --name "My KB" \
  --open
```

Without `bokf`, open BioOKF Studio, choose **+ New base**, and select the generated bundle.

## Development and tests

```bash
PYTHONPATH=src python3 -m unittest discover -s tests -v
```

The test suite uses local fixtures and does not call Paperclip, Codex, Claude, or BioOKF Studio.

## Deliberate v0.1 limitations

- The harness creates new staging KBs; canonical merge/reconciliation remains a separate BioOKF
  workflow.
- Figure vision is not yet orchestrated. Paperclip full-text and line-addressable content are
  preserved, but a later phase should route figures through a vision-capable agent.
- Abstract-only results may produce sparse KBs because they do not contain full article bodies.
- The GUI is local and single-user; it is not hardened for public network exposure.
