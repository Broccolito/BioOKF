# Karpathy's "LLM Wiki" — Research Notes

> Compiled 2026-06-25. Primary source: Andrej Karpathy's gist
> "LLM Wiki" (https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f),
> published April 2026, 5,000+ stars within days. Posted as an "idea file"
> (see his X thread, https://x.com/karpathy/status/2040470801506541998).
>
> **Reproduction note:** the raw gist could not be copied verbatim (the fetch
> tool's copyright guardrail refused full reproduction). Everything below is a
> faithful, cited paraphrase reconstructed from the gist's structural summary
> plus multiple detailed community guides that quote it. Direct quotes are
> marked and attributed.

---

## 1. What the LLM Wiki is

The LLM Wiki is a **pattern, not a product**. Instead of doing
retrieval-augmented generation (RAG) against raw documents on every query, an
LLM agent **incrementally compiles** your sources once into a persistent,
interlinked collection of markdown files (the "wiki") and then keeps that wiki
current as new sources arrive. Knowledge is **compiled once and maintained**,
not re-derived from scratch each time.

Karpathy's framing of *why this works now*:

- > "The tedious part of maintaining a knowledge base is not the reading or the
  > thinking — it's the bookkeeping." Humans abandon wikis because the
  > maintenance burden grows faster than the value. "LLMs do not get bored. They
  > do not forget to update a cross-reference."
- The wiki is "a persistent, **compounding** artifact." Good query answers get
  filed back into the wiki, so explorations become permanent knowledge.
- IDE metaphor he uses: **"Obsidian is the IDE. The LLM is the programmer. The
  wiki is the codebase."** His actual setup: Claude Code on one side, Obsidian
  on the other, agent editing markdown in real time.
- Historical lineage: he positions this as the spiritual successor to Vannevar
  Bush's 1945 **Memex** (personal, curated knowledge with "associative trails"
  between documents) — but with the maintenance problem finally solved by
  agents.
- The "idea file" rationale: > "in this era of LLM agents, there is less of a
  point/need of sharing the specific code/app, you just share the idea, then
  the other person's agent customizes & builds it for your specific needs."
  The gist is deliberately abstract — a spec to hand to your agent, not a
  codebase.

Works with any coding agent: **Claude Code, OpenAI Codex, Cursor, OpenCode/Pi**,
etc. Designed to be copy-pasted as the agent's instruction/schema file.

---

## 2. The exact ingest / query / lint workflow

Three operations drive the whole system.

### Ingest (when you add a source)
1. **Read** the new source document in `raw/`.
2. **Discuss** key takeaways with the user.
3. **Write a source-summary page** in `wiki/sources/<source-name>.md`.
4. **Cascade updates** — update or create the relevant concept/entity pages.
   A single source typically touches **10–15 related pages**, revising
   summaries and **noting where new data contradicts old claims**.
5. **Create new** concept/entity pages where the source introduces something
   not yet covered.
6. **Update `wiki/index.md`** with the new/changed entries.
7. **Append to `wiki/log.md`** recording the operation, the pages affected, and
   notable findings.

### Query (when you ask a question)
1. **Read `wiki/index.md`** first to identify relevant pages.
2. **Read those pages** and synthesize an answer.
3. **Cite** sources with `[[wikilinks]]` (and trace claims back to `raw/`).
4. **Optionally file** a valuable answer back into the wiki as a new permanent
   page — this is the "compounding" step.

### Lint (periodic health check; e.g. every ~20 new pages)
1. **Scan all pages for contradictions** between claims.
2. **Identify orphan pages** (no incoming links).
3. **Flag missing concepts** (referenced via `[[…]]` but no page exists yet).
4. **Find stale claims** superseded by newer sources.
5. **Suggest** new questions/sources worth investigating.
6. **Save** the report (e.g. `outputs/lint-YYYY-MM-DD.md`).

---

## 3. Universal vs. domain-specific

**Universal (the load-bearing core — true for any field):**
- The three-layer model (`raw/` → `wiki/` → schema).
- The three operations: **ingest → query → lint**.
- Markdown pages + YAML frontmatter + `[[wikilinks]]` cross-linking.
- `index.md` (progressive-disclosure catalog) and `log.md` (append-only
  history) as reserved files.
- The ingest discipline: read → summarize → cascade-update related pages →
  flag contradictions → update index → append log.
- "Compile once, maintain forever; compound answers back in."

**Domain-specific (what each user/field customizes — lives in the schema):**
- **Entity *types*** that matter for the domain (e.g. companies/people for
  competitive analysis; genes/diseases/drugs for biomedicine).
- **Relationship/edge vocabulary** between entities.
- **Page templates** and required frontmatter fields per type.
- **Citation/provenance rules** and confidence conventions.
- **Ingest emphasis** — which facts to extract from a given source format.

Karpathy's gist gives **no domain examples** (no medicine/law/finance recipe).
Customization is intentionally pushed entirely into the **schema file**
(`CLAUDE.md` / `AGENTS.md`) — described as "turning a generic chatbot into a
disciplined wiki maintainer." This is exactly where a vertical like
biomedicine plugs in (see §6 and the closing note).

---

## 4. The raw / wiki / schema three-layer model

```
my-research/
├── raw/                 # LAYER 1 — immutable sources (read, never modified)
│   ├── articles/  papers/  repos/  data/  images/  assets/
├── wiki/                # LAYER 2 — LLM-owned compiled markdown
│   ├── index.md         #   content catalog (updated every ingest)
│   ├── log.md           #   append-only operation log
│   ├── overview.md
│   ├── concepts/        #   concept pages   (attention-mechanism.md)
│   ├── entities/        #   entity pages    (openai.md)
│   ├── sources/         #   source summaries
│   └── comparisons/     #   comparison pages
├── outputs/             # lint reports, exported answers
├── CLAUDE.md            # LAYER 3 — schema / config / conventions
└── .gitignore
```

- **Layer 1 — Raw sources:** the curated, immutable ground truth (articles,
  papers, repos, data, images). The LLM reads but **never edits** them.
- **Layer 2 — The wiki:** markdown the **LLM owns completely** — entity pages,
  concept pages, source summaries, comparisons, plus `index.md` and `log.md`.
- **Layer 3 — The schema:** `CLAUDE.md` (Claude Code) or `AGENTS.md` (Codex).
  Defines page types, naming (kebab-case), frontmatter template,
  cross-reference standard, and the ingest/query/lint procedures. The "config
  that turns a generic agent into a disciplined wiki maintainer." Rohit's
  *LLM Wiki v2* calls the schema **"the real product."**

Git provides version history / provenance over the whole tree.

---

## 5. Entities, concept pages, cross-links, index.md, log.md

**Page types**
- **Entity pages** — one actor/object: a company, person, tool, organization
  (`entities/openai.md`).
- **Concept pages** — one idea/framework/definition
  (`concepts/attention-mechanism.md`).
- **Source summaries** — one ingested document, distilled.
- **Comparisons** — head-to-head pages synthesized across entities/concepts.

Each page is "a structured, Wikipedia-style entry for one thing," with **YAML
frontmatter**:
```yaml
---
title: Page Title
type: concept | entity | source-summary | comparison
sources: [ raw/papers/filename.md ]   # provenance back to raw/
related: [ "[[related-concept]]" ]
created: YYYY-MM-DD
updated: YYYY-MM-DD
confidence: high | medium | low
---
```

**Cross-links** — `[[wikilinks]]` for every internal reference; every claim
links back to its `raw/` file path. This is what makes the corpus a navigable
**graph** (e.g. Obsidian graph view) rather than a flat pile, and what `lint`
audits for orphans/missing targets.

**index.md** — the content-oriented **catalog / table of contents**: every page
listed by category with a one-line summary. The LLM **reads it first on every
query** and **rewrites it on every ingest**. Stays human-scannable up to
~300 pages before you'd need vector search (the single-index approach is what
*v2* says breaks around 200–500 docs).

**log.md** — **append-only chronological** record of every ingest/query/lint,
with parseable prefixes, e.g. `## [YYYY-MM-DD] operation | description`, listing
affected pages and findings. It's the audit trail of how the wiki evolved.

---

## 6. Relationship to OKF (Open Knowledge Format)

Days after the gist went viral, **Google Cloud published OKF v0.1 on June 12,
2026** (announced by Sam McVeety and Amir Hormati), a **vendor-neutral,
agent- and human-friendly specification** for storing knowledge as **markdown +
YAML frontmatter** directories — effectively the **standardized formalization**
of Karpathy's emergent LLM-Wiki pattern.

- OKF requires exactly **one mandatory frontmatter field: `type`** (every doc
  declares its category, e.g. "BigQuery Table"); optional fields include
  `title`, `description`, `resource`, `tags`, `timestamp`.
- **Markdown cross-links form a typed knowledge graph** from the directory
  hierarchy.
- **Reserved filenames `index.md` (progressive disclosure) and `log.md`
  (change history)** carry standardized meaning — directly mirroring the gist.
- Where Karpathy described a loosely-structured, personal wiki, OKF adds the
  **interoperability layer**: a shared schema so one team's/agent's wiki can be
  consumed by another **without bespoke parsers** or translation layers.

So: **Karpathy = the pattern; OKF = the portable standard** that pins it down
(required `type`, frontmatter conventions, reserved files, cross-link graph).

---

## 7. Community reaction, critiques, and implementations

**Discussion (HN / X):**
- Original gist hit HN front page (~158 points). Recurring critiques:
  - **"This is just RAG with extra steps"** — rebutted: unlike RAG/DB retrieval,
    the LLM does the heavy lifting (contradiction resolution, graph building)
    **at ingest time**, shifting cognitive load off query time.
  - **"LLMs are stateless — isn't this just markdown in the prompt?"** — yes,
    but the discipline + persistence is the point.
  - **Source/copyright/size concerns** for ingesting whole books.
  - **Model-collapse risk:** repeatedly LLM-rewritten pages may degrade into
    "the average of the average" (Nature 2024 cited).
  - **"Vibe thinking":** outsourcing the cognition you were supposed to do.
- A Zettelkasten reviewer noted the structure mirrors LYT/PKM skeletons (raw →
  wiki → schema → index), but credited three genuine innovations: **contradiction
  detection on ingest, single-source cascade updates (10–15 pages), and lint for
  orphans/gaps.** Counter-position: atomic notes "sidestep classification
  entirely," so the wiki's "which page does this merge into?" problem persists.

**Notable open-source implementations** (all cite the pattern):
- **Astro-Han/karpathy-llm-wiki** — Agent-Skills-compatible (Claude Code /
  Cursor / Codex) ingest/query/lint skill; `npx add-skill Astro-Han/karpathy-llm-wiki`.
- **lucasastorian/llmwiki** — upload docs (PDF/Word/Excel/PPT/Obsidian) →
  indexed markdown; connects Claude via **MCP** to maintain the wiki (llmwiki.app).
- **nashsu/llm_wiki** — cross-platform desktop app + local HTTP API + MCP server
  + agent skill.
- **Pratiyush/llm-wiki** — builds the wiki from Claude Code / Codex / Copilot /
  Cursor / Gemini sessions.
- **ar9av/obsidian-wiki** & **Programming-With-Maury/Karpathy-LLM-Wiki**
  (`AGENTS.md`) — Obsidian-vault framings.
- **rohitg00 — "LLM Wiki v2"** (gist) — extends the pattern with: confidence
  scoring + supersession + Ebbinghaus-style retention/decay; a typed knowledge
  graph (people/projects/libs/concepts with `uses`/`depends-on`/`contradicts`
  edges + graph traversal); **hybrid retrieval (BM25 + vector + graph, fused via
  reciprocal rank fusion)** for >200–500 docs where a single `index.md` breaks;
  event-driven hooks (auto-ingest, auto-lint, session context injection,
  auto-filing); memory tiers (working → episodic → semantic → procedural).

---

## Sources
- Gist: https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f
- Karpathy "idea file" X thread: https://x.com/karpathy/status/2040470801506541998
- LLM Wiki v2 (rohitg00): https://gist.github.com/rohitg00/2067ab416f7bbe447c1977edaaa681e2
- Starmorph guide (workflows + dir structure + schema): https://blog.starmorph.com/blog/karpathy-llm-wiki-knowledge-base-guide
- AI Builder Club: https://www.aibuilderclub.com/blog/karpathy-llm-wiki
- Data Science Dojo tutorial: https://datasciencedojo.com/blog/llm-wiki-tutorial/
- Level Up Coding (Plaban Nayak): https://levelup.gitconnected.com/beyond-rag-how-andrej-karpathys-llm-wiki-pattern-builds-knowledge-that-actually-compounds-31a08528665e
- OKF write-up (explainx.ai): https://explainx.ai/blog/google-open-knowledge-format-okf-ai-agents-2026
- OKF (themenonlab): https://themenonlab.blog/blog/google-okf-open-knowledge-format-karpathy-llm-wiki-standard
- Zettelkasten review (WenHao Yu): https://yu-wenhao.com/en/blog/karpathy-zettelkasten-comparison/
- HN — open-source impl: https://news.ycombinator.com/item?id=47656181
- HN — agents maintain (Markdown+Git): https://news.ycombinator.com/item?id=47899844
- HN — cognitive governance: https://news.ycombinator.com/item?id=47750193
- GitHub — Astro-Han/karpathy-llm-wiki: https://github.com/Astro-Han/karpathy-llm-wiki
- GitHub — lucasastorian/llmwiki: https://github.com/lucasastorian/llmwiki
- GitHub — nashsu/llm_wiki: https://github.com/nashsu/llm_wiki
- GitHub — Pratiyush/llm-wiki: https://github.com/Pratiyush/llm-wiki
- GitHub — ar9av/obsidian-wiki: https://github.com/ar9av/obsidian-wiki
