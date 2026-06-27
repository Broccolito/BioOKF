# The Open Knowledge Format (OKF): How It Is Documented and Discussed Outside Its Own Spec

Research compiled 2026-06-25. Focus: how OKF is positioned and understood by Google,
the press, the developer/HN community, the SEO/GEO and marketing communities, and
semantic-web experts. The OKF SPEC itself is referenced only for grounding; the
emphasis below is on commentary *around* it.

---

## 1. What OKF is (baseline facts)

- **Announced:** June 12, 2026, on the Google Cloud Blog, by **Sam McVeety** (Tech Lead,
  Data Analytics) and **Amir Hormati** (Tech Lead, BigQuery).
- **Version:** v0.1, explicitly described as "a starting point, not a finished standard."
- **What it is:** A vendor-neutral specification that represents a body of knowledge as a
  **directory of Markdown files with YAML frontmatter**. The frontmatter carries queryable
  fields (`type`, `title`, `description`, `resource`, `tags`, `timestamp`); the Markdown body
  carries prose, schemas, examples, and `[[wiki-style]]`/relative cross-links that form a graph.
- **Only one field is strictly required: `type`.** Everything else is producer-determined.
- **Framing:** "formalizes the LLM-wiki pattern into a portable, interoperable format" —
  derived from **Andrej Karpathy's "LLM Wiki" gist**. Positioned as "format, not platform."
- **Shipped alongside the spec:** a reference enrichment agent (walks BigQuery datasets and
  drafts OKF docs), a static HTML graph visualizer, three sample bundles (GA4 e-commerce,
  Stack Overflow, Bitcoin), and (later) a linter.
- **Home:** `GoogleCloudPlatform/knowledge-catalog` on GitHub. Google Cloud's **Knowledge
  Catalog** product was updated to ingest OKF bundles natively and serve them to Google
  Cloud agents — the obvious enterprise on-ramp.

---

## 2. Google's own positioning (primary source)

Source: [Google Cloud Blog — "How the Open Knowledge Format can improve data sharing"](https://cloud.google.com/blog/products/data-analytics/how-the-open-knowledge-format-can-improve-data-sharing)
and the [knowledge-catalog GitHub repo](https://github.com/GoogleCloudPlatform/knowledge-catalog/tree/main/okf).

**The problem Google names — "the Fragmented Context Problem":** internal knowledge
(table schemas, metric meanings, runbooks, join paths, deprecation notices) is scattered
across metadata catalogs with proprietary APIs, wikis, shared drives, code comments, and
"the heads of a few senior engineers." Their key line:

> "Every agent builder is solving the same context-assembly problem from scratch, every
> catalog vendor is reinventing the same data models, and the knowledge itself is locked
> behind whichever surface created it."

**Three stated design principles:**
1. **Minimally opinionated** — only `type` required; freely extensible.
2. **Producer/consumer independence** — bundles can be hand-authored, exported from a
   catalog, or synthesized by one LLM and consumed by another.
3. **Format, not platform** — "not tied to any specific cloud, database, model provider, or
   agent framework," and "will never require a proprietary account or SDK to read, write, or
   serve." Stated rationale: "the value of a knowledge format comes from how many parties
   speak it, not from who owns it."

**The LLM-wiki rationale (from Karpathy):** "LLMs don't get bored, don't forget to update a
cross-reference, and can touch 15 files in one pass." Agents read and update the shared
Markdown library; humans curate it like code. Google explicitly likens it to Obsidian vaults,
AGENTS.md/CLAUDE.md convention files, and "metadata as code."

**Roadmap framing:** v0.1 is published "in the open" from day one, designed for
backward-compatible growth; Google invites producers, consumers, issues, PRs, and proposed
extensions. "The format itself is the contribution."

---

## 3. Trade-press coverage (mostly descriptive, low skepticism)

- [Search Engine Journal](https://www.searchenginejournal.com/google-cloud-announces-the-open-knowledge-format/579253/)
  treats OKF strictly as **AI-agent infrastructure**, *not* an SEO mechanism. It frames OKF as
  standardizing what teams were already doing ad-hoc, and identifies producers (docs teams,
  engineers) and consumers (agents, LLMs, analysis systems). Notably **no SEO/ranking angle.**
- [MarkTechPost](https://www.marktechpost.com/2026/06/16/google-cloud-introduces-open-knowledge-format-okf-a-vendor-neutral-markdown-spec-for-giving-ai-agents-curated-context/)
  emphasizes the **OKF-vs-RAG contrast**: "Unlike RAG, OKF stores curated, version-controlled
  concepts that agents read and update directly," whereas "RAG re-derives knowledge at query
  time from raw chunks." This coverage is essentially promotional — **no skeptical analysis** of
  scalability, maintenance burden, or where RAG would win.
- Other outlets (TechTimes, NPowerUser, Let's Data Science, StartupHub.ai, explainx.ai,
  Noah News, MyHostNews) largely restate the Google framing: "vendor-neutral," "lingua franca,"
  "turns scattered org knowledge into agent-ready bundles."

---

## 4. Developer / Hacker News community reaction (the most critical layer)

Threads:
[HN #48517735 — "Google proposes Open Knowledge Format based on Markdown"](https://news.ycombinator.com/item?id=48517735)
and [HN #48643541 — "Linter for OKF — Google's Take on Karpathy's LLM Wiki"](https://news.ycombinator.com/item?id=48643541).

Range of opinions on HN:

- **Praise for simplicity:** "I love the simplicity of this OKF spec"; "Markdown is the de-facto
  format for LLMs and humans to interoperate."
- **"Is this even new?" deflation:** "Google has announced… Markdown with YAML front-matter…
  Please applause." (The recurring "a standard, or just a folder?" sentiment.)
- **Markdown-sufficiency skepticism:** "I'm not sure everything can be represented well in 'just
  Markdown'" (e.g., nested tables don't render); calls for whether a Markdown flavor
  (CommonMark) is even specified.
- **Semantic-web déjà vu:** "I love revisiting RDF/OWL Semantic Web formats every 10 years. One
  of these years will be the one!" — i.e., skepticism that a lightweight format succeeds where
  heavier standards failed.
- **Human-vs-AI usability tension:** worry that the format optimizes for agents and could feel
  "much worse than current authoring/viz tools" for non-developer contributors. Pushback on
  "accepting a downgrade of the human knowledge representation experience just to make it
  AI-accessible."
- **Implicit prompt-injection / quality concerns** (also raised in the AI-Driven Lab analysis):
  agent-maintained wikis risk semantic duplication and injection vectors.

---

## 5. Independent technical commentary

### The sharpest critique — "A Standard, or Just a Folder?"
[Marc Bara, Medium](https://medium.com/@marc.bara.iniesta/googles-new-format-for-agent-context-a-standard-or-just-a-folder-82fb21d92041).
Central argument: **OKF standardizes *structure* but not *meaning*.**
- "The container is standardized; the meaning is left to each producer." Two conformant bundles
  can share no vocabulary, so an agent can read another team's bundle without *understanding* it.
- OKF gives "a shared way to store context, not yet a shared way to make sense of it."
- **Spec inconsistency:** Google's reference parser expects four fields (`type`, `title`,
  `description`, `timestamp`) while the spec mandates only `type` — "even the required surface of
  the container is not fully settled at v0.1."
- **Vendor-neutrality is partly illusory:** the file format is open, but "the gravity of the
  ecosystem is still Google Cloud" via Gemini, BigQuery as the reference source, and Knowledge
  Catalog as the obvious ingestion path.
- Verdict: a pragmatic first step, incomplete for true interoperability. This
  structural-vs-semantic interoperability distinction recurs across the more careful write-ups.

### The semantic-web expert view — surprisingly constructive
[Kurt Cagle & Chloe Shannon, "The Format Convergence" (Ontologist Substack)](https://ontologist.substack.com/p/the-format-convergence).
- Rather than dismiss OKF as naïve, they see it as an **adoption vector for the semantic web.**
  Markdown is "the closest thing the contemporary web has to a universal document format," and
  its emergence "less like design and more like inevitability."
- They're explicit that **Markdown alone is insufficient** for formal knowledge work, and propose
  layering ("DataBook = an OKF document with semantic web superpowers").
- Their critique of RDF/OWL is *institutional, not technical*: "The problem was never the design;
  it was adoption. The tooling was too heavy, the learning curve too steep." OKF's grassroots
  Markdown substrate could finally carry formal ontological typing into a willing ecosystem.

### Personal-knowledge-management developers — enthusiastic
[Sébastien Dubois (dsebastien.net)](https://www.dsebastien.net/open-knowledge-format-okf/):
"This validates the approach I have used for years. Markdown notes, YAML frontmatter, Git
underneath, and an AGENTS.md / CLAUDE.md at the root." Sees the payoff as friction removal: "the
knowledge bases people already keep in Obsidian become directly consumable by any agent that
speaks OKF, with no export step." Praises OKF for "the lowest-friction substrate that already
exists … instead of inventing a new one."

### Balanced explainer with honest caveats
[AI-Driven Lab (note.com)](https://note.com/ai_driven/n/n8e2726b98180?hl=en): frames OKF as "a
common language called Markdown" for agent knowledge, standardizing the "Wiki body" layer of
Karpathy's three-layer model. Distinguishes it cleanly from AGENTS.md/CLAUDE.md (those define
behavior/config; OKF defines the knowledge structure). Honest about v0.1 risks: prompt injection,
semantic duplication, quality variance — but supports the deliberate minimalism because "past
standardization efforts failed by trying to solve everything at once."

---

## 6. The SEO / GEO / marketing reinterpretation (a parallel, contested reading)

A whole cluster of commentary repurposes OKF as a **website/AI-search ("GEO/AEO") play** — a
reading Google did *not* advance. Interpretations diverge sharply:

- **Cautious-bullish:** [Suganthan Mohanadasan](https://suganthan.com/blog/open-knowledge-format/)
  frames OKF as "a second layer of the web designed for machines," a long-term protocol bet
  ("protocol-layer work is registration, not advertising"), while conceding "a bundle will not
  move your rankings or your AI visibility this week" and "nothing crawls the web for these
  bundles yet." He flags that Google "marketed OKF within an enterprise data product rebrand,
  despite broader applicability." (He also ships a WordPress plugin / web tool.)
- **Explicitly skeptical / hype-deflating:** [SEO-Kreativ (Christian Ott), "SEO Hype or GEO
  Tool?"](https://www.seo-kreativ.de/en/blog/open-knowledge-format-okf/) is blunt: "OKF is a data
  layer for your agents; SEO remains the discovery layer for Google." "OKF points inward (your
  own agents), llms.txt points outward (external crawlers)" — and it is **not** a web standard like
  schema.org. He directly debunks the viral myth: "If you read anywhere that you now need to
  upload an OKF file to your website to get cited in AI answers, that's simply wrong." Real value
  is internal productivity, "not SEO benefit."
- **Creative extension:** [No Hacks, "OKF Could Work For Websites, Too"](https://nohacks.co/blog/okf-website-knowledge-graph)
  argues OKF gives sites "a graph of relationships" (vs. the "flat" page-by-page copies of
  existing machine formats) and ties it to a "Machine-First Architecture" discipline. Honest about
  the cost: "a second copy is a second thing to keep in sync." Found the real win was that writing
  the bundle "surfaced gaps I would not have found writing another page."
- **Marketing-ops framing:** [PPC.land](https://ppc.land/googles-okf-wants-to-be-the-lingua-franca-for-ai-agent-knowledge/)
  applies it to agencies running programmatic campaigns (knowledge split across warehouse schemas,
  campaign taxonomies, Confluence runbooks, dashboard configs). Notably raises **governance
  skepticism:** v0.1 "is published on GitHub under Google's account, which provides credibility
  but also raises the question of how governance will evolve as external contributors propose
  changes."

A common framing across this cluster places OKF in a **layered stack with sibling standards:**
"llms.txt is the map / signpost, OKF is the library / payload, MCP is the librarian / pipe, and
AGENTS.md is the in-repo instruction sheet — none replaces another."

---

## 7. Where interpretations differ (synthesis of the disagreements)

| Axis | Optimistic reading | Skeptical reading |
|---|---|---|
| **Novelty** | Formalizes a real, missing "portable context package" layer | "Markdown with YAML front-matter" — just a folder; not new |
| **Standard-ness** | Open, minimal, will grow backward-compatibly | Standardizes structure, **not semantics** → limited interoperability at v0.1 |
| **Vendor-neutrality** | No SDK, no cloud, no lock-in; "format not platform" | Ecosystem gravity (Gemini/BigQuery/Knowledge Catalog) + Google-owned governance |
| **Markdown choice** | Lowest-friction universal substrate; human + agent friendly | Can't express complex/nested structures; CommonMark flavor unspecified |
| **vs. prior art** | Lighter, adoptable successor to heavy semantic-web stacks | "Revisiting RDF/OWL every 10 years"; could become an adoption vector *for* the semantic web (Cagle) |
| **SEO/GEO relevance** | Future machine-readable web layer; structure early | Inward-facing data layer; **not** a ranking signal or web standard (debunks the upload-to-rank myth) |
| **Risks** | — | Prompt injection, semantic duplication in agent-maintained wikis, sync tax of a second copy, spec gaps (4 fields in parser vs. 1 in spec) |

---

## 8. How OKF was communicated

- Primary channel was the **Google Cloud Blog post** plus a **Google Cloud Tech tweet/X post**
  ([@GoogleCloudTech](https://x.com/GoogleCloudTech/status/2067012903337664886)) and the public
  GitHub repo. No evidence surfaced of a dedicated conference talk or podcast episode tied to the
  launch — coverage propagated through the blog, the GitHub spec/linter, and a wave of trade,
  SEO/GEO, and developer-blog write-ups within days.

---

## Source list

**Primary / Google:**
- Google Cloud Blog: https://cloud.google.com/blog/products/data-analytics/how-the-open-knowledge-format-can-improve-data-sharing
- GitHub (knowledge-catalog / okf): https://github.com/GoogleCloudPlatform/knowledge-catalog/tree/main/okf · SPEC: https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md
- Google Cloud Tech on X: https://x.com/GoogleCloudTech/status/2067012903337664886
- Karpathy "LLM Wiki" gist: https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f

**Trade press:**
- Search Engine Journal: https://www.searchenginejournal.com/google-cloud-announces-the-open-knowledge-format/579253/
- MarkTechPost: https://www.marktechpost.com/2026/06/16/google-cloud-introduces-open-knowledge-format-okf-a-vendor-neutral-markdown-spec-for-giving-ai-agents-curated-context/
- TechTimes: https://www.techtimes.com/articles/318416/20260615/google-cloud-open-knowledge-format-turns-scattered-org-knowledge-agent-ready-bundles.htm

**Developer / community:**
- Hacker News (proposal thread): https://news.ycombinator.com/item?id=48517735
- Hacker News (linter thread): https://news.ycombinator.com/item?id=48643541

**Independent technical commentary:**
- Marc Bara, "A Standard, or Just a Folder?" (Medium): https://medium.com/@marc.bara.iniesta/googles-new-format-for-agent-context-a-standard-or-just-a-folder-82fb21d92041
- Kurt Cagle & Chloe Shannon, "The Format Convergence" (Ontologist Substack): https://ontologist.substack.com/p/the-format-convergence
- Sébastien Dubois: https://www.dsebastien.net/open-knowledge-format-okf/
- AI-Driven Lab (note.com): https://note.com/ai_driven/n/n8e2726b98180?hl=en

**SEO / GEO / marketing reinterpretation:**
- Suganthan Mohanadasan: https://suganthan.com/blog/open-knowledge-format/
- SEO-Kreativ, "SEO Hype or GEO Tool?": https://www.seo-kreativ.de/en/blog/open-knowledge-format-okf/
- No Hacks, "OKF Could Work For Websites, Too": https://nohacks.co/blog/okf-website-knowledge-graph
- PPC.land, "OKF wants to be the lingua franca…": https://ppc.land/googles-okf-wants-to-be-the-lingua-franca-for-ai-agent-knowledge/
