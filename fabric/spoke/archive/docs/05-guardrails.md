# Guardrails — a fail-closed pipeline that doesn't depend on the operator remembering

The mapping's deterministic core is code, but the *sequence* (resolve → apply reviewed decisions →
**verify every id exists** → canonicalize → check edges → validate) must run in order, every time.
An LLM operator running steps by hand across turns can, with low context, skip a step — most
dangerously the id-verification of LLM-supplied mappings. [pipeline.py](../linker/pipeline.py)
removes that risk: it runs every step itself, each wrapped in a gate that raises on violation, writes
a manifest proving each step ran, and refuses to declare success unless every gate passed. An
**independent audit** then re-checks everything from the output alone.

## One command runs everything, gated

```bash
bokf-spoke-link pipeline export.json --out o.json --report r.json [--curation verified.json]
bokf-spoke-link audit --out o.json --manifest o.manifest.json
```

## The steps and their gates (each raises → pipeline stops fail-closed, no output written)

| step | precondition | postcondition (GATE) |
|------|--------------|----------------------|
| **preflight** | — | SPOKE reachable; type_crosswalk covers all **28** BioOKF types; predicate_crosswalk covers all **24** base predicates |
| **load_export** | file exists & parses | has `graph.nodes/edges` + `pages`; ≥1 node; records input SHA-256 |
| **resolve_nodes** | — | `len(matches)==len(nodes)`; **no mapped node has a null id** |
| **apply_curation** | if given, file exists & parses | records curation SHA-256 + decision count (or `skipped`) |
| **verify_node_ids** ⚠ | — | **UNCONDITIONAL: every mapped (label,id) must EXIST in live SPOKE** — catches hallucinated/stale ids from any tier incl. the LLM curation layer |
| **canonicalize** | — | any remapped compound id still exists in SPOKE |
| **check_edges** | — | every node & edge has a `spoke.status` |
| **validate_invariants** | — | contract invariants (mapped⇒id, contradicts⇒negative, agrees/contradicts⇒both mapped); **shape identity** (node/edge counts == input, order preserved) |

Any gate failure ⇒ the step is recorded `fail`, downstream steps are **NOT RUN**, the manifest
records `result: FAIL`, and **no annotated output is written**. Proven with a poison test: injecting
a fake `DOID:99999999` for a review-queue node makes the pipeline stop at `verify_node_ids` with
`GateError: 1 mapped nodes point at a SPOKE id that does not exist`.

## The manifest — proof of execution

Every run writes `<out>.manifest.json`: the ordered list of steps with `pass`/`skipped`/`fail`, key
metrics per step (mapped counts, tier histogram, `broken=0`, agree/related/contradict, `shape_identical`),
input/output/curation SHA-256s, and the overall `result`. It is the checklist that the LLM (or a
human, or CI) reads to confirm nothing was skipped — printed at the end of every run as:

```
PIPELINE CHECKLIST  ->  PASS
  [✓] preflight            pass  types=28 predicates=23 ...
  [✓] resolve_nodes        pass  mapped=1251 ...
  [✓] verify_node_ids      pass  checked=1464 broken=0
  [✓] canonicalize         pass  remapped=18
  [✓] check_edges          pass  agree=209 related=97 contradict=14
  [✓] validate_invariants  pass  nodes=2550 edges=5360 shape_identical=True
```

## The audit — independent, does NOT trust the manifest

`audit` re-derives every check from the **output file alone** and re-queries **live SPOKE**, so it
catches a corrupted output even if the manifest claims PASS:
- manifest recorded PASS and all required steps ran without failure;
- contract invariants hold on the actual output;
- every node & edge annotated; every mapped node has an id;
- **a sample of mapped ids re-verified to exist in SPOKE** (independent broken-id check);
- contradictions really are `not_*` predicates.

Proven: nulling one mapped id + dropping one edge annotation (while leaving a PASS manifest) makes
`audit` report `FAIL` with the exact failing checks and exit code 2.

## Why this answers "the LLM might forget a step"

- The **order and existence** of steps is code, not operator memory — `pipeline` runs them all.
- The **most safety-critical step (id verification) is unconditional** and cannot be skipped or
  bypassed, even if a prior LLM tier hallucinated an identifier.
- **Fail-closed**: on any violation the run aborts and writes no output — a partial/incorrect graph
  is never produced silently.
- A **manifest** makes execution auditable, and an **independent audit** provides a final
  yes/no that re-verifies against ground truth rather than trusting the run's own claims.
