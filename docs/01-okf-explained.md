# Understanding the Open Knowledge Format (OKF)

> A comprehensive walkthrough of OKF v0.1: what it is, the problem it solves, every
> component, why each exists, its constraints, and exactly where it is rigid vs flexible —
> plus how the community has received and used it. Built from the verbatim spec and a survey
> of official + independent + open-source material. Detailed source notes:
> [../research/okf-spec-verbatim.md](../research/okf-spec-verbatim.md),
> [../research/okf-discussion.md](../research/okf-discussion.md),
> [../research/okf-oss-usage.md](../research/okf-oss-usage.md).

## 1. What OKF is

The **Open Knowledge Format** (v0.1, Draft; Apache-2.0) comes from
`GoogleCloudPlatform/knowledge-catalog` (the Knowledge Catalog / former Dataplex repo). It
is explicitly **not an official Google product** — a vendor-neutral, community-style spec.
Announced 2026-06-12 on the Google Cloud Blog (Sam McVeety & Amir Hormati), it is pitched as
the fix for the **"fragmented context problem"**: the knowledge an organization needs to
operate — the metadata, business context, and curated insight *around* data and systems — is
scattered across data catalogs, wikis, code comments, Slack, and senior engineers' heads,
in incompatible, SDK-bound, service-owned formats.

OKF's thesis is deliberately humble and durable:

- **It standardizes the substrate, not the meaning.** Knowledge is a directory tree of
  UTF-8 **Markdown files with YAML frontmatter**, shippable as a Git repo (recommended),
  tarball, zip, or subdirectory.
- **"If you can `cat` a file, you can read it; if you can `git clone`, you can ship it."**
- It **references** domain schemas (Avro, Protobuf, OpenAPI) rather than trying to subsume
  them. It represents the knowledge *about* data, not the data itself.

Three stated principles: **minimally opinionated**, **producer/consumer independence**
(anyone can write a bundle, anyone can read it, with no shared SDK), and **"a format, not a
platform."** Google's own Knowledge Catalog can ingest bundles natively, and a reference
enrichment agent, an HTML visualizer, sample bundles, and a linter shipped alongside.

OKF is the formalization of Andrej Karpathy's **LLM Wiki** pattern (see
[02-llm-wiki-explained.md](02-llm-wiki-explained.md)): an agent incrementally builds and
maintains a structured, interlinked Markdown knowledge base that sits between you and the
raw sources — as a *persistent, compounding artifact*, in contrast to RAG, which
re-derives connections at query time.

## 2. The problem it solves

| Without OKF | With OKF |
|---|---|
| Context is fragmented across tools, each with its own schema and SDK | One vendor-neutral, plain-text, Git-versioned representation |
| Agent knowledge is locked into a service or framework | "Producer/consumer independence" — any tool can read/write it |
| RAG re-discovers the same connections on every query | Knowledge is curated once and **compounds**; cross-references persist |
| Knowledge is opaque/binary | Human- *and* agent-readable Markdown; diffable in Git |

The core value is **portability + persistence** of organizational/agent knowledge.

## 3. Components — what they are and why they exist

### 3.1 Bundle
The unit of distribution: a directory tree of Markdown files. Exists so knowledge can be
**shipped and versioned as a whole** with ordinary tools (Git, tar, zip). No database, no
service.

### 3.2 Concept (concept document)
One Markdown file = one concept. Anatomy: **YAML frontmatter** (the small, queryable,
structured layer) + a **Markdown body** (the rich, human-readable layer). Exists to make
each unit of knowledge **self-describing** and individually addressable.

### 3.3 Concept ID
A concept's identity = its **file path minus `.md`** (e.g. `data/orders.md` →
`data/orders`). Exists to give every concept a **stable handle** that cross-links can
target. (The PoC code further constrains path segments to `[A-Za-z0-9_][A-Za-z0-9_.\-]*`.)

### 3.4 Frontmatter fields
- **`type`** — the **only required** field; a non-empty string naming the concept's
  category (e.g. `BigQuery Table`, `Metric`, `Playbook`, `Reference`). Exists so a consumer
  can route/interpret a concept without parsing its body.
- **Recommended (not required):** `title`, `description` (one-sentence summary), `resource`
  (a URI for the underlying asset), `tags` (list), `timestamp` (ISO-8601 last-modified).
- **Producers may add any custom keys**; **consumers MUST preserve unknown keys**.

### 3.5 Body conventions
Standard Markdown. Conventional (never required) headings carry meaning: **`# Schema`**
(structured description of the underlying asset), **`# Examples`** (usage), **`# Citations`**
(external sources). Exist to give common knowledge shapes a predictable place without
mandating structure.

### 3.6 Reserved files
- **`index.md`** — a progressive-disclosure directory listing/catalog of the bundle (or a
  subtree). Carries no frontmatter except an optional root-level `okf_version`. Exists as
  the **entry point** an agent reads first to navigate at moderate scale.
- **`log.md`** — a newest-first change history using ISO `## YYYY-MM-DD` headings. Exists as
  an **append-only audit trail** of ingests/updates.
These two filenames are reserved and **MUST NOT** be used as concept names.

### 3.7 Cross-links
Plain Markdown links between concepts, in two forms: **absolute / bundle-relative**
(`/path/to/concept.md`, spec-recommended for stability under moves) and **relative**
(`./other.md`). A link asserts *some* relationship; **its meaning lives in the surrounding
prose**. Exist to turn the bundle into a navigable graph — but an *untyped* one.

## 4. Conformance — the constraints

A bundle **conforms** iff (exactly three MUSTs):
1. Every non-reserved `.md` file has a **parseable YAML frontmatter mapping**.
2. Every frontmatter has a **non-empty `type`**.
3. `index.md` / `log.md`, when present, **follow their prescribed structures**.

Consumers **MUST NOT** reject a bundle for: missing optional frontmatter fields, **unknown
`type` values**, unknown extra keys, **broken cross-links**, or a missing `index.md`.
Consumers **MUST** tolerate broken links and preserve unknown keys.

This is an intentionally **low bar** — "self-describing Markdown with a category on every
file." Everything else is convention.

## 5. Where OKF is rigid vs flexible

| Rigid (fixed by the spec) | Flexible (left open — the deliberate freedom) |
|---|---|
| Substrate = Markdown + YAML frontmatter | **`type` is required but uncontrolled** — no registry, no taxonomy; producers coin any string, consumers must degrade unknowns gracefully |
| Every concept has a non-empty `type` | **Cross-links are untyped** — the *kind* of relationship (joins-with, depends-on, parent/child) lives only in prose; a graph consumer sees a directed, untyped edge |
| `index.md`/`log.md` structure when present | Directory layout, extra frontmatter keys, body sections, and which recommended fields to use are all producer-defined |
| Concept ID = path − `.md` | Whether to use absolute or relative links |

**The two deliberate open points** are the crux of OKF — and exactly what BioOKF closes:

1. **Open `type` vocabulary.** Maximizes generality (any domain works out of the box) but
   means two conformant bundles can share *no vocabulary*. The sharpest published critique
   (Marc Bara, *"A Standard, or Just a Folder?"*) is precisely this: OKF **standardizes
   structure but not meaning** — "a shared way to store context, not yet a shared way to
   make sense of it."
2. **Untyped links.** A relationship exists, but its semantics are not machine-readable.
   Graph reasoning is therefore shallow.

These are *features* for a general-purpose catalog and *gaps* for any domain that wants
real interoperability or graph queries — which is the opening BioOKF exploits.

## 6. Spec-vs-implementation discrepancies (worth knowing)

The shipped reference code diverges from the prose spec in instructive ways:
- The PoC enrichment agent **requires four keys** (`type`, `title`, `description`,
  `timestamp`), not the spec's one.
- The agent **forbids absolute links** ("Never start a link with `/` — it breaks GitHub
  rendering") — the **opposite** of the spec's recommendation; shipped bundles use relative
  links.
- Real bundles write citations as bare-URL bullet lists, not the spec's numbered
  `[n] [Title](url)` form.
- **No `log.md` exists anywhere** in the repo.

Takeaway for any consumer (including BioOKF tooling): support **both** link forms and treat
the §4/§8 formatting conventions as **soft**.

## 7. How the community received and uses OKF

**Reception.** Trade press (Search Engine Journal, MarkTechPost) was descriptive and
promotional. Developers on Hacker News were more critical — split between "elegant
simplicity" and deflation ("Markdown with YAML front-matter… applause"), with semantic-web
déjà vu ("we revisit RDF/OWL every 10 years"). Semantic-web voices (Kurt Cagle) were
constructive: OKF as a grassroots **adoption vector** for formal ontologies, since RDF/OWL's
failure "was never the design; it was adoption." PKM developers (Obsidian/Git crowd) were
enthusiastic — OKF validates existing Markdown+frontmatter+Git practice and makes vaults
agent-consumable. A common framing places OKF in a stack: **`llms.txt` = map, OKF = payload,
MCP = pipe, AGENTS.md = instructions.**

**Open-source usage (within days, the GitHub `okf` topic).** Five patterns emerged:
1. **Agent skills/plugins** (most common): `okf-skills`, `okf-skill`,
   `open-knowledge-format-starter` teach Claude Code / Cursor / Codex to author, validate,
   and visualize bundles. They vendor the spec + a Python conformance checker and **do not
   invent types** — they leave `type` to the user.
2. **Mechanical producers/connectors:** Go tools that extract DB schemas, FKs, file trees,
   and Git history into OKF with no embedded LLM; doc-site crawlers that emit typed bundles
   + an MCP server.
3. **Converters:** zero-dependency `feishu/obsidian/notion/github/html → okf` tools.
4. **Validators/linters.**
5. **MCP servers** exposing a bundle to agents.

The pattern that recurs: because OKF **leaves `type` open**, every implementation either
punts the vocabulary to the user or invents an ad-hoc one — confirming the gap BioOKF fills
for biomedicine.

## 8. Lessons BioOKF takes from OKF

- **Keep the substrate.** Markdown + YAML + Git is the right portable, diffable, human- and
  agent-readable base. BioOKF changes nothing here.
- **Keep the minimalism of the *required* set.** OKF requires one field; BioOKF requires
  three on nodes and five on edges — still tiny. Don't over-burden the curator.
- **Close the two open points — but only because biomedicine *can*.** A controlled `type`
  set and typed edges are liabilities in a general catalog (no universal vocabulary exists)
  but assets in biomedicine (UMLS/Biolink/SPOKE already provide one).
- **Stay a strict profile.** Every BioOKF bundle remains a conformant OKF bundle, so OKF
  tooling (viewers, Git workflows, the Knowledge Catalog) still works on it.
