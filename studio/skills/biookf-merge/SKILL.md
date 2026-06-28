---
name: biookf-merge
description: Use to merge a Secondary KB (SKB) onto a canonical Main KB (MKB): candidate matching, true-match collapse, carry-over, source-node union, raw/ relocation, subtype harmonization, integrity check.
---

# Skill: biookf-merge

Merge an SKB **onto** the MKB. The **MKB is canonical** (it is the active KB): its identifiers, file
paths, `raw/` locations, and subtype names **win on every collision**; the SKB side is the one
renamed/moved/collapsed. **Lose no information, break no links.** (Follow `Merge_KBs_WF.md` +
`schema.md` + `docs/04`.)

## Loop
1. `bokf set-active <root> <mkb-id>`; pass the SKB path explicitly (this encodes MKB-is-canonical).
2. **Identifier resolution.** Compare MKB `index.md` vs SKB `index.md`; surface exact AND
   semantically-similar candidates **across the two KBs only** (never within one). When the index
   isn't enough, open the node `.md` files and reason over `synonyms`/`xref`/body. A `Disease` facet
   and its `Phenotype` facet are **distinct**; do not collapse.
3. **Collapse true matches only.** Keep the MKB identifier; concatenate the SKB node's `edges:` into
   it; combine bodies/frontmatter, discarding nothing (on direct contradiction, **keep both**, each
   tagged with its source). Then **rewrite every reference** to the vanished SKB identifier (edge
   `object`, `primary_source`, `reported_in` object) so nothing dangles.
4. **Carry over the rest.** Any SKB node with no match / judged different: append to MKB `index.md`
   (on identifier collision **rename the SKB** entity, then rewrite all refs to it); move its `.md`
   into `knowledge/<type>/`.
5. **Source nodes** merge the same way; on collapse, **union** their `raw_source` + `xref`.
6. **`raw/`.** Relocate SKB `raw/` files into the MKB `raw/` (rename on filename collision; drop a
   true duplicate by content), and update `raw_source` paths. **Do not dedupe edges** here; that is
   a future lint pass.
7. **Subtype resolution.** Harmonize equivalent-but-differently-named subtypes to the **MKB** name;
   find-and-replace the token on **both nodes and edges**; update the subtypes-in-use list.
8. **Log + gate.** `bokf log-sync <mkb> --kind merge --summary "merged <skb>" --counts`, then
   `bokf verify --workflow merge`. Final integrity: no duplicate identifiers; no dangling
   `object`/`primary_source`/`reported_in`; every `raw_source` resolves; subtypes harmonized; the
   MKB's own identifiers/paths/`raw/` unchanged except where a genuine merge required it.
