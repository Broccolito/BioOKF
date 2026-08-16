---
name: analyze-biookf-network
description: Calculate and interpret topology metrics, node centralities, Leiden communities, and source-year evidence growth for a local BioOKF knowledge base. Use when asked to characterize network structure, find hubs or bridges, inspect fragmentation or modularity, compare KBs, export metric JSON, or analyze how the evidence base grows over time.
---

# Analyze a BioOKF network

Use the deterministic `bokf network-metrics` command. Do not ask an LLM to estimate graph metrics.

1. Resolve the exact bundle path and run `bokf verify BUNDLE` first. Report verification errors before interpreting topology.
2. Run `bokf network-metrics BUNDLE --out REPORT.json`. By default this analyzes a simple undirected biological projection: external nodes and synthesized edges are removed, parallel authored edges are collapsed, self-loops are removed, and Publication, Study, Dataset, and Agent nodes are excluded.
3. Add `--include-provenance` only when the question concerns the evidence layer itself. Never compare a provenance-inclusive report with a provenance-excluding report without saying so.
4. Report density with mean degree, transitivity, giant-component average shortest path, global efficiency, Leiden modularity Q, giant-component fraction, degree assortativity, and algebraic connectivity. Treat zero algebraic connectivity as expected for a disconnected graph.
5. Use degree CCDF, betweenness, C(k), k-core, participation coefficient, and Leiden membership for node-level interpretation. Do not claim a power law from a visual log-log line.
6. For evidence growth, use `source_years`, `sources_with_year`, and `sources_without_year`. Years come only from explicit Publication, Study, or Dataset metadata; do not infer missing dates from titles or prose.
7. Use BioOKF Studio's **Metrics** window when the user wants plots. It provides annual and cumulative source growth, topology plots, the ranked node table, and JSON export.

When comparing KBs, keep projection settings identical and discuss size/density effects explicitly.
