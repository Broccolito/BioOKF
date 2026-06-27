# Nature Review Articles — Downloaded & Cited-Paper Corpus

A local markdown corpus of the **first 100 review articles** from
[nature.com → Review Articles](https://www.nature.com/nature/articles?type=review-article),
each accompanied by the full-text markdown of its **open-access cited papers**.

All conversion was done with **BioRouter's own knowledge-base pipeline**
(`biorouter_mcp::knowledge::convert::convert()` — `htmd` for HTML, `pdf-inspector`/
`pdf-extract` for PDF), invoked through a small workspace binary built for this task at
`crates/biokf-convert/`. This is the same code path the in-app *Knowledge* feature uses to
ingest URLs/PDFs.

## Folder layout

```
reviews/
├── README.md                         ← this file
├── INDEX.md                          ← per-article table (review md + citation counts)
└── <abbreviated-title>/              ← one folder per review, named from its title
    ├── <abbreviated-title>.md        ← the review article itself (markdown)
    └── citations/
        ├── _index.md                 ← every reference: status, title, DOI, link to md
        ├── _references.json          ← raw Crossref reference list
        ├── _oa.json                  ← open-access resolution per reference
        └── <doi-sanitised>.md        ← one markdown file per downloaded OA cited paper
```

## How it was built

1. **Listing** — paginated `?type=review-article` and extracted the first 100 article DOIs + titles.
2. **Review articles** — converted each article page to markdown via BioRouter's converter.
3. **Citations** — pulled each review's full reference list from the **Crossref** API
   (`api.crossref.org/works/{doi}`), the same source BioRouter's credibility module uses.
4. **Open-access resolution** — resolved each cited DOI through **Unpaywall**, preferring a
   direct PDF, then a PubMed Central / **Europe PMC** render PDF, then a repository/landing page.
5. **Download + convert** — fetched every open-access cited paper and converted it to markdown
   with BioRouter's converter, run in parallel across all 100 articles.
6. **Europe PMC fallback** — for OA papers blocked by publisher bot-walls (Wiley/Elsevier
   Cloudflare), retried via their Europe PMC `?pdf=render` full text. This recovered **+1,617**
   papers on top of the main wave.

## Results

| Metric | Count |
|---|---|
| Review articles converted to markdown | **100 / 100** |
| Total references across all reviews | **15,581** |
| References with a resolvable DOI | **14,532** |
| Cited papers that are open-access | **10,219** |
| **Open-access cited papers downloaded + converted** | **6,822** (66.8% of OA) |
| Total markdown files | **7,023** |
| Corpus size | **~435 MB** |

## Scope & limitations (honest notes)

- **Paywalls.** Nature review articles and ~30% of their citations are not open access. For the
  review pages this means the markdown captures the **abstract, metadata, and full reference
  list** (the body text is behind Nature's paywall). Only **open-access** cited papers were
  downloaded, as requested.
- **Why not 100% of OA.** The remaining ~33% of open-access citations are mostly papers whose
  only OA copy sits behind a JavaScript/Cloudflare bot-wall (Wiley `pdfdirect`, ScienceDirect)
  with **no** PubMed Central version to fall back to, or are non-biomedical (physics/chemistry/
  earth-science) OA pages that don't expose a clean PDF. Each such reference is still recorded in
  the article's `citations/_index.md` with its title, DOI, and OA link, marked `⚠️ OA-link-failed`.
- Each `citations/_index.md` gives the exact per-article breakdown
  (downloaded ✅ / paywalled 🔒 / OA-link-failed ⚠️).
