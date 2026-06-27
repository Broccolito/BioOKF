# BioOKF Curation Harness — Foundation Slice (SP0 + SP1 + SP2) — Design

**Date:** 2026-06-27
**Status:** Design (awaiting user review → implementation plan)
**Scope:** This spec covers only the **foundation slice** of a larger 7-subproject curation
harness. The full catalog was synthesized separately; this document details the three
foundational sub-projects and treats SP3–SP6 as out of scope (each gets its own spec).

---

## 1. Context — the full harness, and where this slice sits

The goal is to operationalize `Ingestion_WF.md` and `Merge_KBs_WF.md` as a Claude Code plugin
(plugin root = `studio/`) made of three accountable layers: **CLI/MCP tools** (deterministic
enforcement), **skills** (procedures + judgment), and **hooks** (guardrails). The work decomposes
into seven sub-projects:

| # | Sub-project | In this spec? |
|---|---|---|
| **SP0** | predicate-reconcile (23 → 24, `used_to_study`) | ✅ yes |
| **SP1** | version-tracking (git-backed, `log-sync`, ChangeKind) | ✅ yes |
| **SP2** | active-kb (registry + set/get-active) | ✅ yes |
| SP3 | convert-pipeline (docs → raw Markdown, naming, tests) | ❌ later spec |
| SP4 | deterministic lint-extensions + `okf verify` | ❌ later spec |
| SP5 | skills (ingest/merge/convert/verify/version) | ❌ later spec |
| SP6 | hooks + `studio/` plugin packaging | ❌ later spec |

**Approved decisions driving this slice:** (1) build foundation first; (2) conversion will be
Rust-native in-process *(SP3, not here)*; (3) the eventual Stop gate blocks on Errors + verify
items only *(SP6)*; (4) `log.md` keeps the current `## YYYY-MM-DD` newest-first heading.

**Why foundation first:** SP3–SP6 all depend on a correct predicate vocabulary (SP0), a way to
commit/track changes (SP1), and a notion of which bundle is active (SP2). Building these three
first unblocks everything and delivers the version-tracking capability the user explicitly asked
for.

---

## 2. SP0 — Predicate reconciliation (23 → 24)

### Problem
`schema.md` (authoritative per both workflow headers) defines **24** forward-only predicates: the
v0.1–v0.4 core of 23 **plus `used_to_study`**. Current code encodes **23** — verified:
`okf-core/src/model.rs` `PREDICATES: [&str; 23]`, `okf-core/src/lib.rs` asserts length 23, and no
`UsedToStudy`/`used_to_study` token exists anywhere. Consequence today: a `used_to_study` edge
parses to `Predicate::Unknown` and lints as `predicate.invalid` (Error) — directly contradicting
`schema.md` and the workflows. `SPEC.md` §6 also still says 23 ("unchanged"); `schema.md` wins.

### Changes
1. **`okf-core/src/model.rs`** — add `Predicate::UsedToStudy`; grow `PREDICATES` to `[&str; 24]`
   inserting `"used_to_study"` in canonical order (between `associated_with` and `reported_in`);
   add the `parse()` and `as_str()` arms. `used_to_study` is **not** symmetric.
2. **`okf-core/src/lib.rs`** — update the compile/test assertion from `== 23` to `== 24`.
3. **Domain/range** (`lint.rs` domain/range checker) — add `used_to_study` from `schema.md`:
   - **Domain:** `MethodOrProcedure`, `Study`, `Dataset`, `Device`, `Organism`, `CellType`,
     `MaterialSample`.
   - **Range:** `Disease`, `Phenotype`, `BiologicalPathway`, `BiologicalFunction`, `Gene`,
     `Variant`, `Molecule`.
   - Direction: resource → entity-under-study (forward-only). Violations are **Warn** (consistent
     with existing `edge.range` severity).
4. **`okf-mcp/src/instructions.rs`** — sync the predicate line (currently lists 23) to 24.
5. **New tool `okf predicates [--json]`** — print the active controlled vocabulary (28 types, the
   24 predicates, `knowledge_level`/`agent_type` enums) straight from the single
   `okf_core::PREDICATES`/`NODE_TYPES` constants. This is the **single source of truth**: skills
   (authoring side) and the validator (checking side) both read it, so they can never disagree on
   the count.
6. **Changelog** — record in `schema.md`/`SPEC.md` that the predicate set is reconciled to 24
   (note `SPEC.md` §6 was the stale side).

### Acceptance
- `okf_core::PREDICATES.len() == 24`; `used_to_study` round-trips parse↔as_str.
- A concept doc with a `used_to_study` edge validates and lints **clean** (no `predicate.invalid`);
  a domain/range violation surfaces as `edge.range` Warn.
- `okf predicates --json` lists exactly 24 predicates incl. `used_to_study`.

---

## 3. SP1 — Git-backed version tracking

### Problem
No version tracking exists (verified: no `git.rs`, zero git references in `okf-core/src`). The user
wants every curation step committed with comprehensive messages, and commit history kept in sync
with `log.md`, for ingestion / linting / merging.

### Design decision: shell out to system `git`
`git2` is **not** in `studio/Cargo.lock` (verified count 0; only `sha2` present). To avoid taking a
new libgit2/C dependency, `GitRepo` shells out to the system `git` binary via
`std::process::Command`. A preflight runs `git --version` and errors clearly if git is absent.

### Components

**`okf-core/src/git.rs` — `GitRepo`** (thin wrapper over `std::process::Command`):
- `ensure_repo(bundle)` — **init-on-first-use**: `git init` + bot identity config if no `.git`
  exists. Lives inside the committer path (not only in scaffold) so pre-existing un-versioned
  bundles (e.g. `studio/test-kb/*`) get a repo before their first commit instead of failing.
- Bot identity per-invocation: `-c user.name='BioOKF Curation'
  -c user.email=curation@biookf.local -c commit.gpgsign=false`.
- `commit_all(kind, summary, delta)` — `git add -A` then
  `git commit -m '[<kind>] <summary>' -m 'delta: <delta>'`.
- `log(limit)` — `git log --format=…`, parsed into `HistoryEntry` newest-first.
- `restore_to(sha, summary)` — **forward-only**: commit the old tree as a new `[restore]` commit on
  top of HEAD (never rewinds history).
- Transactions: `begin_txn(label)` branches `txn/<slug>-<uuid>` off HEAD; steps commit on the
  branch; `commit_txn` **squash-merges** onto the main branch as one commit (so a 10–15-page ingest
  becomes a single history entry); `abort_txn` discards the branch.
- **`.gitignore`**: `raw/**/original.*` (immutable source bytes are not version-controlled);
  derived `raw/**/source.md` + `meta.yaml`, all of `knowledge/`, `index.md`, `log.md`, `schema.md`
  **are** committed.

**`okf-core/src/types` — `ChangeKind`** (serde `snake_case`, exactly 8 variants):
`Ingest`, `Convert`, `Link`, `Merge`, `Lint`, `Index`, `Restore`, `Manual`. **No inference** — the
kind is always passed explicitly by the caller. Documented kind→step mapping: convert→`Convert`,
ingest squash→`Ingest`, merge squash→`Merge`, `okf index`→`Index`, lint fixes→`Lint`,
restore→`Restore`, scaffold/manual edits→`Manual`, link-only edits→`Link`.

**`okf-core/src/log_sync.rs` — the sole *step* committer.** `log_sync(bundle, kind, summary, delta,
counts, txn)` appends a `## YYYY-MM-DD` block to `log.md` (seeding `# Log` if absent, newest-first
per the existing okf convention) **and** commits, atomically, in one call. It is the only path that
produces a *logged curation-step* commit; the lower-level `okf commit` handles **non-logged
lifecycle** commits only (scaffold's initial commit, txn squash-merge, restore), which do not get a
`log.md` entry. The removed-and-never-added per-tool auto-commit hook is what "sole committer" rules
out — there is exactly one coupling of "a step changed the bundle" → "log.md + git both updated". `--counts` derives
node-by-type / edge / source-node counts from `okf stats` to populate the Step-4/Step-3 required
entry bodies. Because log.md and the commit happen in one call, they cannot drift between commits.

> **Why a single committer (not a per-tool auto-commit hook):** PostToolUse hooks fire after the
> mutation, can't block, don't know the open txn branch, and would race / double-commit / re-trigger
> on their own `log.md` write. So "every step commits with a comprehensive message" is realized by
> each skill step **explicitly** calling `okf log-sync` with the right `--kind` and a descriptive
> message. Final log↔commit **parity** is verified later at the Stop gate (SP6) via `okf verify`,
> which is the real guarantee since a PostToolUse hook cannot enforce it.

### CLI + MCP surface (this slice)
- CLI: `okf log-sync <bundle> --kind K --summary S [--delta D] [--counts] [--txn LABEL]`,
  `okf commit` (lower-level stage+commit, used by txn squash/restore), `okf log [--limit N]
  [--json]`, `okf restore <bundle> <sha> [--summary S]`.
- MCP (server name `biookf`): `mcp__biookf__okf_log_sync`, `okf_log`, `okf_restore`.

### Acceptance
- `okf scaffold` produces a bundle with a `.git` and an initial `[manual] create knowledge base
  <id>` commit.
- `okf log-sync … --kind ingest --summary "…"` appends a dated `log.md` block **and** creates one
  matching `[ingest]` commit; `okf log --json` reconstructs it with the right kind/summary/delta.
- A pre-existing bundle with **no** `.git` gets one on its first `log-sync` (init-on-first-use).
- `okf restore <old-sha>` adds a new `[restore]` commit; history is never rewound.
- With `git` absent from PATH, every committer path fails with a clear, actionable error.

---

## 4. SP2 — Active-KB context

### Problem
No "active KB" concept exists (verified: the Tauri app hardcodes `candidate_bundles()`). The agent
needs persistent context on which knowledge graph it is working on, and tools to set/get it.

### Components
**`okf-core/src/registry.rs`** — `<root>/registry.yaml` = `{ bases: [ { id, path } ] }`, the known-
bundle list. `register` / `--unregister` / `--list`; duplicate-id rejected; atomic temp+rename
write. Replaces the Tauri hardcoded list as the shared source of known bundles.

**`okf-core/src/active.rs`** — `<root>/.active-kb` = plaintext kb-id of the active graph.
`get_active_persisted` reads+trims (missing/empty → `None`); `set_active_persisted` writes via
temp+rename (`None` deletes), guarded by a process `Mutex` + file lock. `kb-id` validated by
`validate_kb_id` (non-empty, ≤64, `[a-z0-9-]`, no leading/trailing/double `-`).

> **Per-session scoping:** the design defaults to a **single global** `<root>/.active-kb` (fits the
> single-user studio app). An optional `<root>/.active-kb-sessions/<sha256(session)>` for parallel
> MCP sessions is **deferred** — documented as a future extension, not built in this slice.

**Scaffold integration (critical sequencing fix):** `okf scaffold` now also **registers** and
**set-actives** the newly created bundle (in addition to `git init`). Without this, the first
`okf convert` after scaffold (SP3) would be denied by the future require-active hook.

**Active-as-default:** every CLI/MCP command that takes a bundle path defaults to the active KB's
path when the path is omitted. In Merge, the **MKB is the active KB** and the SKB is passed
explicitly — encoding "MKB is canonical" at the tool layer.

### CLI + MCP surface (this slice)
- CLI: `okf set-active <root> <kb-id>`, `okf get-active <root> [--json]`,
  `okf register <root> <kb-id> <path> | --list | --unregister <kb-id>`.
- MCP: `mcp__biookf__okf_set_active`, `okf_get_active`, and `okf_list_bases` made active-aware.

> The **SessionStart hook** that injects "Active BioOKF KB: …" into agent context consumes
> `okf get-active`, but the hook itself is wired in SP6. This slice ships the tool it depends on.

### Acceptance
- `okf register` then `okf set-active` then `okf get-active --json` returns the id + resolved path;
  unset → `None`. Invalid kb-ids are rejected.
- `okf scaffold foo --name Foo` leaves `foo` registered **and** active.
- Duplicate-id registration is rejected; writes are atomic (no partial files on crash).

---

## 5. Module / interface boundaries (isolation)

Each new `okf-core` module has one responsibility and a small interface:

| Module | Responsibility | Depends on |
|---|---|---|
| `model.rs` (edit) | controlled vocab incl. the 24th predicate | — |
| `git.rs` | git operations over `std::process::Command` | system `git` |
| `log_sync.rs` | atomic log.md append + commit (sole committer) | `git.rs`, `stats` |
| `registry.rs` | known-bundle registry | fs |
| `active.rs` | active-KB pointer | fs, `registry.rs` |

CLI (`okf-cli/src/main.rs`) and MCP (`okf-mcp/src/{main,ops}.rs`) are thin front-ends over these.
No module reaches into another's internals; e.g. `log_sync` calls `git.commit_all` and the stats
function, nothing more.

## 6. Error handling
- **git absent** → preflight error with remediation ("install git / ensure it's on PATH").
- **dirty/empty commit** → `commit_all` tolerates "nothing to commit" (no-op, not an error).
- **registry/active races** → atomic temp+rename + file lock; readers treat missing/garbage as
  `None`/empty rather than crashing.
- **invalid kb-id** → typed validation error before any write.
- **restore of unknown sha** → clear error, no mutation.

## 7. Testing (TDD-ready)
- **SP0 unit:** `PREDICATES.len()==24`; parse/as_str round-trip for `used_to_study`; a fixture doc
  with a valid `used_to_study` edge lints clean; a domain/range violation → `edge.range` Warn;
  `okf predicates --json` snapshot.
- **SP1 unit/integration:** init-on-first-use on a bundle with no `.git`; `log_sync` produces one
  commit + one dated `log.md` block; `okf log` parses kind/summary/delta; forward-only `restore`
  adds (never rewinds); txn begin→commit squashes to one entry; git-absent path errors. Uses
  `tempfile` bundles.
- **SP2 unit:** register/list/unregister; duplicate-id rejected; set/get/clear active round-trip;
  `validate_kb_id` table; atomic-write (no partial file).
- **End-to-end:** `scaffold → get-active → write a node → log-sync → log → restore` round-trip on a
  temp bundle asserts the full foundation works together.

## 8. Out of scope (later specs)
SP3 conversion pipeline; SP4 lint-extensions + `okf verify`; SP5 skills; SP6 hooks + plugin
packaging (including the blocking Stop gate that enforces log↔commit parity and the
SessionStart/PreToolUse guardrails). This slice deliberately ships the **tools** those layers will
consume, but not the hooks/skills themselves.
