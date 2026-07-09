# SPOKE lookup mechanics — empirical findings

These are established by direct query against live SPOKE (`spokedev.cgl.ucsf.edu`, db `spoke`),
and they define the confident tiers of the mapping waterfall. Every claim here was verified with
a real query.

## Index inventory (relevant to matching)

- **RANGE index on `identifier`** for (almost) every label → O(log n) exact identifier lookup.
- **RANGE index on `name`** for most biomedical labels (Gene, Disease, Compound, Anatomy,
  Protein, Organism, Pathway, MolecularFunction, BiologicalProcess, CellType, Food,
  PharmacologicClass, Symptom, SideEffect, …) → O(log n) exact **case-sensitive** name lookup.
- **FULLTEXT index per label**, named `<Label>NamesAndIds`, over
  `[name, description, identifier, synonyms, (pref_name|entryName)]` → Lucene search incl. synonyms.
- **One combined FULLTEXT** `anyNamesAndIds` over 25 biomedical labels. Noisy across types; prefer
  the per-label index when the target label is known.

## The waterfall tiers (in confidence order)

1. **Exact identifier** — `MATCH (n:`Label` {identifier:$id})`. Only usable when the BioOKF node
   carries an `xref` in SPOKE's own scheme (NCBIGene int, DOID, inchikey:, UniProt, rsID, UBERON,
   NCBITaxon). **In the BioRED benchmark biomedical entities carry no xrefs**, so this tier rarely
   fires there — but the harness must still support it (other bundles will have xrefs).
2. **Exact name (case-sensitive)** — `MATCH (n:`Label` {name:$name})`. Index-backed, fast.
3. **Exact name/synonym (case-insensitive)** — generate candidates from the fulltext index, then
   filter in-app: `toLower(node.name)=toLower($q)` OR `toLower($q) ∈ toLower(synonyms)`. Fast and
   robust. This is the workhorse tier.
4. **Fuzzy candidate + adjudication** — fulltext top-k with score; hand the top candidates to a
   heuristic (normalized-token equality, edit distance) and, when still ambiguous, to the LLM.
5. **Not found** — mark `not_found_in_spoke`.

## Pitfalls that shape the design (each verified)

- **Fulltext top score ≠ exact match.** Query `atorvastatin` on `CompoundNamesAndIds` ranks
  *"Atorvastatin lactone"* (15.5) **above** the true *"Atorvastatin"* (10.4). ⇒ NEVER trust the
  top fulltext hit; always prefer app-side exact/case-insensitive equality (tier 3) before fuzzy.
- **Lucene special chars break naive queries.** `15-F(2t)-isoprostane` sent raw returns unrelated
  hits (`+ - ( ) : > < * ?` etc. are Lucene operators); phrase-quoting it returns empty. ⇒ the
  query string must be **sanitized** (special chars → spaces, collapse ws) before hitting fulltext,
  and names that sanitize to noise are expected `not_found` / defer-to-LLM cases.
- **The combined `anyNamesAndIds` is noisy.** `PTPN22` there ranks Reactome reactions/complexes
  above the actual `Gene` (26191). ⇒ query the **per-label** index once the candidate label(s) are
  chosen from the type crosswalk.
- **Some labels have no `name`/`synonyms` to match on.** `Variant` exposes only `identifier`
  (dbSNP rsID). A BioOKF Variant named `105-bp deletion` or `+2740 A>G` cannot be name-matched;
  it is mappable only via an rsID/HGVS xref (absent in the benchmark) ⇒ expected `not_found`.
- **Near-miss names need semantics.** BioOKF `type 1 diabetes` vs SPOKE `type 1 diabetes mellitus`
  (DOID:9744) is not string-equal; tier 3 misses, tier 4 (fuzzy/LLM) must catch it.

## Query cookbook (CLI serializes scalars only)

```bash
# exact by name
spoke-cli query "MATCH (n:`Gene` {name:'PTPN22'}) RETURN n.identifier AS id, n.name AS name, n.synonyms AS syn" --stdout
# fulltext candidates (sanitize $q first!)
spoke-cli query "CALL db.index.fulltext.queryNodes('DiseaseNamesAndIds','type 1 diabetes') YIELD node,score RETURN node.identifier AS id, node.name AS name, score ORDER BY score DESC LIMIT 25" --stdout
# edge existence between two identifiers
spoke-cli query "MATCH (a:`Gene` {identifier:X})-[r]-(b:`Disease` {identifier:Y}) RETURN type(r) AS rt, keys(r) AS keys LIMIT 10" --stdout
```
Return individual scalar properties (bare nodes/maps serialize to `null`).
