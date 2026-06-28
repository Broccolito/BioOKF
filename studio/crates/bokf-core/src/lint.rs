//! Lint a BioOKF bundle against the v0.5 conformance rules. Pure + deterministic:
//! it returns a typed report; it never mutates the bundle.

use crate::bundle::Bundle;
use crate::graph::Graph;
use crate::model::*;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warn,
    Info,
}

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub severity: Severity,
    pub rule: String,
    /// The node identifier (or "<bundle>") the finding concerns.
    pub subject: String,
    pub message: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct LintReport {
    pub findings: Vec<Finding>,
}

impl LintReport {
    pub fn errors(&self) -> usize {
        self.findings.iter().filter(|f| f.severity == Severity::Error).count()
    }
    pub fn warnings(&self) -> usize {
        self.findings.iter().filter(|f| f.severity == Severity::Warn).count()
    }
    pub fn infos(&self) -> usize {
        self.findings.iter().filter(|f| f.severity == Severity::Info).count()
    }
    pub fn is_clean(&self) -> bool {
        self.errors() == 0
    }
    fn push(&mut self, severity: Severity, rule: &str, subject: &str, message: String, path: Option<String>) {
        self.findings.push(Finding {
            severity,
            rule: rule.to_string(),
            subject: subject.to_string(),
            message,
            path,
        });
    }
}

fn looks_like_bare_curie(id: &str) -> bool {
    // e.g. "HGNC:6018", "MONDO:0100096", "infores:hgnc": a colon, no spaces.
    id.contains(':') && !id.contains(' ') && !id.contains('(')
}

pub fn lint(bundle: &Bundle) -> LintReport {
    let mut r = LintReport::default();

    // --- file-level parse errors ---
    for (path, err) in &bundle.parse_errors {
        r.push(
            Severity::Error,
            "parse",
            "<file>",
            format!("failed to parse: {err}"),
            Some(path.to_string_lossy().to_string()),
        );
    }

    // --- duplicate identifiers ---
    for (id, path) in &bundle.duplicate_identifiers {
        r.push(
            Severity::Error,
            "identifier.duplicate",
            id,
            format!("identifier `{id}` is duplicated across the bundle"),
            Some(path.to_string_lossy().to_string()),
        );
    }

    for n in &bundle.nodes {
        let path = Some(n.path.to_string_lossy().to_string());

        // type must be one of 28
        if !n.node_type.is_valid() {
            r.push(
                Severity::Error,
                "type.invalid",
                &n.identifier,
                format!("`{}` is not one of the 28 controlled node types", n.raw_type),
                path.clone(),
            );
        }

        // identifier human-readability
        if looks_like_bare_curie(&n.identifier) {
            r.push(
                Severity::Warn,
                "identifier.opaque",
                &n.identifier,
                "identifier looks like a bare CURIE; it should be human-readable (move the CURIE to `xref`)".to_string(),
                path.clone(),
            );
        }

        // subtype expected
        if n.subtype.is_none() {
            r.push(
                Severity::Info,
                "subtype.missing",
                &n.identifier,
                "no `subtype` (agent-coined, expected but not validated)".to_string(),
                path.clone(),
            );
        }

        // source nodes should be anchored (raw_source OR an external xref)
        if n.node_type.is_provenance() && n.raw_source.is_empty() && n.xref.is_empty() {
            r.push(
                Severity::Warn,
                "source.unanchored",
                &n.identifier,
                "provenance node has neither a `raw_source` path nor an external `xref`".to_string(),
                path.clone(),
            );
        }

        // edges
        for e in &n.edges {
            lint_edge(&mut r, bundle, n, e, &path);
        }

        // misclassification: node filed under a type directory that disagrees with its type
        if let Some(dir) = n.path.parent().and_then(|p| p.file_name()).and_then(|s| s.to_str()) {
            let dir_l = dir.to_lowercase();
            let is_type_dir = NODE_TYPES.iter().any(|t| t.eq_ignore_ascii_case(&dir_l));
            if is_type_dir && n.node_type.is_valid() && !n.node_type.as_str().eq_ignore_ascii_case(&dir_l) {
                r.push(
                    Severity::Warn,
                    "type.path_mismatch",
                    &n.identifier,
                    format!("node typed `{}` is filed under `{dir_l}/`; type and directory disagree (possible misclassification)", n.node_type.as_str()),
                    path.clone(),
                );
            }
        }

        // duplicate edges: identical predicate + object + primary_source on the same node
        // (a different primary_source is a legitimate parallel provenance edge, not a dup)
        let mut seen_edges = std::collections::HashSet::new();
        for e in &n.edges {
            let key = (e.predicate.as_str().to_string(), e.object.clone(), e.primary_source.clone().unwrap_or_default());
            if !seen_edges.insert(key) {
                r.push(
                    Severity::Warn,
                    "edge.duplicate",
                    &n.identifier,
                    format!("duplicate edge `{} -> {}` from the same source (merge sources or remove the redundant one)", e.predicate.as_str(), e.object),
                    path.clone(),
                );
            }
        }

        // value-as-identifier: a bare measurement value is edge data, never a node
        if looks_like_measurement_value(&n.identifier) {
            r.push(Severity::Warn, "value.as_identifier", &n.identifier,
                "identifier looks like a measurement value, not a standalone entity (values live on edges, never as nodes)".to_string(), path.clone());
        }
        // `Other` requires a `note:` explaining why no controlled type fits
        if matches!(n.node_type, NodeType::Other) && n.note.is_none() {
            r.push(Severity::Warn, "other.missing_note", &n.identifier,
                "type `Other` requires a `note:` explaining why no controlled type fits".to_string(), path.clone());
        }
        // provenance link: a non-source node should carry a `reported_in` edge
        if !n.node_type.is_provenance()
            && !n.edges.iter().any(|e| matches!(e.predicate.base(), Predicate::ReportedIn))
        {
            r.push(Severity::Warn, "node.no_reported_in", &n.identifier,
                "node has no `reported_in` edge linking it to a source".to_string(), path.clone());
        }
        // raw_source paths must resolve to a real file under the bundle
        for rs in &n.raw_source {
            if !bundle.root.join(rs).exists() {
                r.push(Severity::Warn, "source.raw_missing_file", &n.identifier,
                    format!("raw_source `{rs}` does not exist under the bundle"), path.clone());
            }
        }
    }

    // --- orphans ---
    let graph = Graph::from_bundle(bundle);
    for id in graph.orphans() {
        r.push(Severity::Warn, "node.orphan", &id, "node has no edges (orphan)".to_string(), None);
    }

    // --- contradictions (same subject/predicate/object, opposite negation) ---
    lint_contradictions(&mut r, bundle);

    // --- near-duplicate subtypes within a type (merge candidates) ---
    lint_similar_subtypes(&mut r, bundle);

    // --- raw sources still awaiting faithful LLM conversion to Markdown ---
    lint_raw_conversion(&mut r, bundle);

    // --- figures still provisional or referenced without a description ---
    lint_figures(&mut r, bundle);

    // --- index.md currency ---
    if bundle.has_index_md {
        for id in crate::index::missing_from_index(bundle) {
            r.push(Severity::Warn, "index.stale", &id, "identifier is not registered in index.md; run `bokf index`".to_string(), None);
        }
    }

    r
}

/// Flag any `raw/<id>/source.md` that still carries the needs-conversion marker, i.e. an
/// unknown/binary source the agent has not yet rendered to faithful Markdown.
fn lint_raw_conversion(r: &mut LintReport, bundle: &Bundle) {
    let raw = bundle.root.join("raw");
    let entries = match std::fs::read_dir(&raw) {
        Ok(e) => e,
        Err(_) => return,
    };
    for e in entries.flatten() {
        let src = e.path().join("source.md");
        if let Ok(txt) = std::fs::read_to_string(&src) {
            if txt.contains(crate::convert::NEEDS_CONVERSION_MARKER) {
                let id = e.file_name().to_string_lossy().to_string();
                r.push(
                    Severity::Warn,
                    "source.needs_conversion",
                    &id,
                    "raw source still needs faithful Markdown conversion (read original.*, render ALL content, remove the marker)".to_string(),
                    Some(format!("raw/{id}/source.md")),
                );
            }
        }
    }
}

/// Walk `raw/*/meta.yaml` and flag figures that are still provisional (`source.figure_unnamed`)
/// or referenced in `source.md` without a non-empty description (`source.figure_undescribed`).
fn lint_figures(r: &mut LintReport, bundle: &Bundle) {
    let raw = bundle.root.join("raw");
    let entries = match std::fs::read_dir(&raw) {
        Ok(e) => e,
        Err(_) => return,
    };
    for e in entries.flatten() {
        let dir = e.path();
        let meta_path = dir.join("meta.yaml");
        let txt = match std::fs::read_to_string(&meta_path) {
            Ok(t) => t,
            Err(_) => continue,
        };
        let meta: crate::convert::SourceMeta = match serde_yaml::from_str(&txt) {
            Ok(m) => m,
            Err(_) => continue,
        };
        if meta.figures.is_empty() {
            continue;
        }
        let id = e.file_name().to_string_lossy().to_string();
        let src_md = std::fs::read_to_string(dir.join("source.md")).unwrap_or_default();
        for f in &meta.figures {
            if f.provisional {
                r.push(
                    Severity::Warn,
                    "source.figure_unnamed",
                    &id,
                    format!("figure `{}` still has a provisional name; run `bokf name-figure` to give it a content name", f.file),
                    Some(format!("raw/{id}/{}", f.file)),
                );
            }
            // Described when source.md carries `[<non-empty>](<file>)` for this figure.
            if !figure_is_described(&src_md, &f.file) {
                r.push(
                    Severity::Warn,
                    "source.figure_undescribed",
                    &id,
                    format!("figure `{}` is referenced without a description; write a faithful description beside its reference in source.md", f.file),
                    Some(format!("raw/{id}/source.md")),
                );
            }
        }
    }
}

/// True when `source.md` references `file` with a non-empty alt/description, i.e.
/// `[<non-empty>](<file>)` (the leading `!` of an image reference is optional).
fn figure_is_described(md: &str, file: &str) -> bool {
    let needle = format!("]({file})");
    let mut from = 0;
    while let Some(rel) = md[from..].find(&needle) {
        let close = from + rel; // index of the `]` before `(`
        // Walk back to the matching `[` and read the alt text between them.
        if let Some(open_rel) = md[..close].rfind('[') {
            let alt = md[open_rel + 1..close].trim();
            if !alt.is_empty() {
                return true;
            }
        }
        from = close + needle.len();
    }
    false
}

/// Within each node type, flag distinct `subtype` tokens that normalize to the same
/// form (e.g. `protein_coding` vs `protein-coding` vs `ProteinCoding`), merge candidates.
fn lint_similar_subtypes(r: &mut LintReport, bundle: &Bundle) {
    use std::collections::{BTreeMap, BTreeSet};
    let norm = |s: &str| -> String { s.chars().filter(|c| c.is_ascii_alphanumeric()).map(|c| c.to_ascii_lowercase()).collect() };
    let mut by_type: BTreeMap<String, BTreeMap<String, BTreeSet<String>>> = BTreeMap::new();
    for n in &bundle.nodes {
        if let Some(st) = &n.subtype {
            by_type
                .entry(n.node_type.as_str().to_string())
                .or_default()
                .entry(norm(st))
                .or_default()
                .insert(st.clone());
        }
    }
    for (ty, groups) in by_type {
        for (_norm, raws) in groups {
            if raws.len() > 1 {
                let list: Vec<String> = raws.into_iter().collect();
                r.push(
                    Severity::Info,
                    "subtype.similar",
                    &ty,
                    format!("type `{ty}` uses near-duplicate subtypes {list:?}; consider merging to one canonical token"),
                    None,
                );
            }
        }
    }
}

fn lint_edge(r: &mut LintReport, bundle: &Bundle, n: &Node, e: &Edge, path: &Option<String>) {
    let subj = &n.identifier;

    // predicate must be one of 23
    if !e.predicate.is_valid() {
        r.push(
            Severity::Error,
            "predicate.invalid",
            subj,
            format!("`{}` is not one of the {} controlled predicates", e.raw_predicate, PREDICATES.len()),
            path.clone(),
        );
    }
    // negation is only allowed on the curated negatable predicates
    if e.negated && !e.predicate.is_negative() && e.predicate.is_valid() {
        r.push(
            Severity::Error,
            "edge.not_negatable",
            subj,
            format!("`{}` is not a negatable predicate: only effect predicates (treats, causes, binds, associated_with, expressed_in, regulates, has_phenotype, …) may be negated", e.predicate.as_str()),
            path.clone(),
        );
    }
    if e.reversed {
        r.push(
            Severity::Info,
            "predicate.inverse",
            subj,
            format!("`{}` is a deprecated inverse alias; author the forward `{}` on the other node", e.raw_predicate, e.predicate.as_str()),
            path.clone(),
        );
    }

    // provenance triplet (mandatory)
    match &e.knowledge_level {
        None => r.push(Severity::Error, "edge.missing_knowledge_level", subj, format!("edge `{} -> {}` missing knowledge_level", e.predicate.as_str(), e.object), path.clone()),
        Some(v) if !KNOWLEDGE_LEVELS.contains(&v.as_str()) => r.push(Severity::Error, "edge.invalid_knowledge_level", subj, format!("invalid knowledge_level `{v}`"), path.clone()),
        _ => {}
    }
    match &e.agent_type {
        None => r.push(Severity::Error, "edge.missing_agent_type", subj, format!("edge `{} -> {}` missing agent_type", e.predicate.as_str(), e.object), path.clone()),
        Some(v) if !AGENT_TYPES.contains(&v.as_str()) => r.push(Severity::Error, "edge.invalid_agent_type", subj, format!("invalid agent_type `{v}`"), path.clone()),
        _ => {}
    }
    match &e.primary_source {
        None => r.push(Severity::Error, "edge.missing_primary_source", subj, format!("edge `{} -> {}` missing primary_source", e.predicate.as_str(), e.object), path.clone()),
        Some(ps) if ps == "not_provided" => r.push(Severity::Warn, "edge.primary_source_not_provided", subj, "primary_source is `not_provided` (allowed only as a rare exception)".to_string(), path.clone()),
        Some(ps) => {
            match bundle.get(ps) {
                None => r.push(Severity::Warn, "edge.primary_source_unresolved", subj, format!("primary_source `{ps}` does not resolve to a source node"), path.clone()),
                Some(src) if !src.node_type.is_provenance() => r.push(Severity::Warn, "edge.primary_source_not_source", subj, format!("primary_source `{ps}` resolves to a {} node, not a Publication/Study/Dataset/Agent", src.node_type.as_str()), path.clone()),
                _ => {}
            }
        }
    }

    // object resolution (skip the always-external case is impossible to know; warn)
    if !bundle.contains(&e.object) {
        let sev = if looks_like_bare_curie(&e.object) { Severity::Warn } else { Severity::Warn };
        r.push(sev, "edge.object_unresolved", subj, format!("edge object `{}` does not resolve to any node's identifier", e.object), path.clone());
    } else {
        // domain/range checks (only when object resolves to a known node)
        lint_domain_range(r, bundle, n, e, path);
    }

    // `regulates`/`expressed_in` require a `direction`
    if matches!(e.predicate, Predicate::Regulates | Predicate::ExpressedIn) && e.direction.is_none() {
        r.push(Severity::Info, "edge.missing_direction", subj, format!("`{}` should carry a `direction` (increased/decreased)", e.predicate.as_str()), path.clone());
    }

    // §7.3 quantitative sanity
    if let (Some(lo), Some(hi)) = (stat_num(e, "ci_lower"), stat_num(e, "ci_upper")) {
        if lo > hi {
            r.push(Severity::Warn, "edge.stat_ci", subj, format!("ci_lower ({lo}) > ci_upper ({hi})"), path.clone());
        }
    }
    if let Some(p) = stat_num(e, "p_value") {
        if !(0.0..=1.0).contains(&p) {
            r.push(Severity::Warn, "edge.stat_pvalue", subj, format!("p_value {p} is outside [0, 1]"), path.clone());
        }
    }
}

/// True when an identifier is a bare measurement value (e.g. `183 cm`, `2.9`, `45%`): edge
/// data masquerading as a node. Conservative: requires a leading number and at most a 1 to 3 letter
/// (or `%`) unit, so real entities like `2-AG`, `5-HT`, `TP53` are not flagged.
fn looks_like_measurement_value(id: &str) -> bool {
    let s = id.trim();
    let mut chars = s.chars().peekable();
    match chars.peek() {
        Some(c) if c.is_ascii_digit() => {}
        _ => return false,
    }
    let mut seen_dot = false;
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            chars.next();
        } else if c == '.' && !seen_dot {
            seen_dot = true;
            chars.next();
        } else {
            break;
        }
    }
    let rest: String = chars.collect();
    let rest = rest.trim();
    rest.is_empty() || rest == "%" || (rest.len() <= 3 && rest.chars().all(|c| c.is_ascii_alphabetic()))
}

fn stat_num(e: &Edge, key: &str) -> Option<f64> {
    e.stats.get(key).and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|i| i as f64)))
}

fn lint_domain_range(r: &mut LintReport, bundle: &Bundle, n: &Node, e: &Edge, path: &Option<String>) {
    let obj = match bundle.get(&e.object) {
        Some(o) => o,
        None => return,
    };
    let subj = &n.identifier;
    use NodeType::*;
    let warn = |r: &mut LintReport, msg: String| {
        r.push(Severity::Warn, "edge.range", subj, msg, path.clone());
    };
    match e.predicate.base() {
        Predicate::Treats | Predicate::Prevents => {
            if !matches!(obj.node_type, Disease | Phenotype) {
                warn(r, format!("`{}` should target a Disease/Phenotype, but `{}` is a {}", e.predicate.as_str(), e.object, obj.node_type.as_str()));
            }
        }
        Predicate::HasPhenotype => {
            if !matches!(obj.node_type, Phenotype) {
                warn(r, format!("`{}` should target a Phenotype, but `{}` is a {}", e.predicate.as_str(), e.object, obj.node_type.as_str()));
            }
        }
        Predicate::Encodes => {
            if !matches!(obj.node_type, Molecule) {
                warn(r, format!("`encodes` should target a Molecule, but `{}` is a {}", e.object, obj.node_type.as_str()));
            }
        }
        Predicate::ReportedIn => {
            if !obj.node_type.is_provenance() {
                warn(r, format!("`reported_in` should target a Publication/Study/Dataset/Agent, but `{}` is a {}", e.object, obj.node_type.as_str()));
            }
        }
        Predicate::UsedToStudy => {
            if !matches!(obj.node_type, Disease | Phenotype | BiologicalPathway | BiologicalFunction | Gene | Variant | Molecule) {
                warn(r, format!("`used_to_study` should target a studied entity (Disease/Phenotype/BiologicalPathway/BiologicalFunction/Gene/Variant/Molecule), but `{}` is a {}", e.object, obj.node_type.as_str()));
            }
        }
        _ => {}
    }
}

fn lint_contradictions(r: &mut LintReport, bundle: &Bundle) {
    use std::collections::HashMap;
    // key: (subject, BASE predicate, object) -> (positive_seen, negative_seen)
    let mut seen: HashMap<(String, String, String), (bool, bool)> = HashMap::new();
    for n in &bundle.nodes {
        for e in &n.edges {
            let key = (n.identifier.clone(), e.predicate.base().as_str().to_string(), e.object.clone());
            let entry = seen.entry(key).or_insert((false, false));
            if e.predicate.is_negative() || e.negated {
                entry.1 = true;
            } else {
                entry.0 = true;
            }
        }
    }
    for ((subj, pred, obj), (pos, neg)) in seen {
        if pos && neg {
            r.push(
                Severity::Warn,
                "edge.contradiction",
                &subj,
                format!("contradictory edges: both `{pred}` and `not_{pred}` are asserted for `{obj}`"),
                None,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::bundle::Bundle;

    fn lint_docs(docs: &[(&str, &str)]) -> crate::lint::LintReport {
        let dir = tempfile::tempdir().unwrap();
        for (rel, body) in docs {
            let p = dir.path().join("knowledge").join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(&p, body).unwrap();
        }
        crate::lint::lint(&Bundle::open(dir.path()).unwrap())
    }

    #[test]
    fn lints_flag_unnamed_and_undescribed_figures() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join("raw/x-1/figures")).unwrap();
        std::fs::create_dir_all(root.join("knowledge")).unwrap();
        std::fs::write(root.join("raw/x-1/figures/fig-001.png"), b"i").unwrap();
        std::fs::write(root.join("raw/x-1/source.md"), "![](figures/fig-001.png)").unwrap();
        let meta = crate::convert::SourceMeta { id:"x-1".into(), title:"X".into(), sha256:"d".into(), format:"image".into(), original_filename:None, ingested_at:"2026-06-27".into(), needs_llm_fallback:true, figures: vec![crate::convert::FigureMeta{ file:"figures/fig-001.png".into(), provisional:true, described:false, origin:"data-uri".into() }] };
        std::fs::write(root.join("raw/x-1/meta.yaml"), serde_yaml::to_string(&meta).unwrap()).unwrap();
        let rep = crate::lint::lint(&crate::bundle::Bundle::open(root).unwrap());
        assert!(rep.findings.iter().any(|f| f.rule == "source.figure_unnamed"));
        assert!(rep.findings.iter().any(|f| f.rule == "source.figure_undescribed"));
    }

    #[test]
    fn used_to_study_range_violation_warns() {
        // used_to_study -> Disease is in range; used_to_study -> Publication is out of range.
        let study = "---\ntype: Study\nidentifier: T2D GWAS\nsubtype: gwas\nraw_source: [raw/x]\nedges:\n  - predicate: used_to_study\n    object: Type 2 Diabetes\n    knowledge_level: knowledge_assertion\n    agent_type: data_analysis_pipeline\n    primary_source: T2D GWAS\n  - predicate: used_to_study\n    object: Some Paper\n    knowledge_level: knowledge_assertion\n    agent_type: data_analysis_pipeline\n    primary_source: T2D GWAS\n---\n# T2D GWAS\n";
        let disease = "---\ntype: Disease\nidentifier: Type 2 Diabetes\nsubtype: metabolic\n---\n# T2D\n";
        let paper = "---\ntype: Publication\nidentifier: Some Paper\nsubtype: article\nraw_source: [raw/p]\n---\n# p\n";
        let r = lint_docs(&[
            ("study/gwas.md", study),
            ("disease/t2d.md", disease),
            ("publication/paper.md", paper),
        ]);
        assert!(
            r.findings.iter().any(|f| f.rule == "edge.range" && f.message.contains("used_to_study")),
            "expected an edge.range warning for used_to_study -> Publication; got {:?}",
            r.findings
        );
        assert!(!r.findings.iter().any(|f| f.rule == "predicate.invalid"));
    }

    #[test]
    fn flags_duplicate_edge_same_source() {
        let gene = "---\ntype: Gene\nidentifier: BRAF\nsubtype: protein_coding\nedges:\n  - predicate: associated_with\n    object: Melanoma\n    knowledge_level: statistical_association\n    agent_type: text_mining_agent\n    primary_source: Demo\n  - predicate: associated_with\n    object: Melanoma\n    knowledge_level: statistical_association\n    agent_type: text_mining_agent\n    primary_source: Demo\n---\n# BRAF\n";
        let disease = "---\ntype: Disease\nidentifier: Melanoma\nsubtype: neoplasm\n---\n# M\n";
        let src = "---\ntype: Publication\nidentifier: Demo\nsubtype: article\nraw_source: [raw/d]\n---\n# d\n";
        let r = lint_docs(&[("gene/braf.md", gene), ("disease/melanoma.md", disease), ("publication/demo.md", src)]);
        assert!(r.findings.iter().any(|f| f.rule == "edge.duplicate"), "{:?}", r.findings);
    }

    #[test]
    fn flags_type_path_mismatch() {
        // a Gene filed under knowledge/disease/
        let gene = "---\ntype: Gene\nidentifier: BRAF\nsubtype: protein_coding\n---\n# BRAF\n";
        let r = lint_docs(&[("disease/braf.md", gene)]);
        assert!(r.findings.iter().any(|f| f.rule == "type.path_mismatch"), "{:?}", r.findings);
    }

    #[test]
    fn flags_similar_subtypes() {
        let g1 = "---\ntype: Gene\nidentifier: BRAF\nsubtype: protein_coding\n---\n# BRAF\n";
        let g2 = "---\ntype: Gene\nidentifier: KRAS\nsubtype: protein-coding\n---\n# KRAS\n";
        let r = lint_docs(&[("gene/braf.md", g1), ("gene/kras.md", g2)]);
        assert!(r.findings.iter().any(|f| f.rule == "subtype.similar" && f.message.contains("protein")), "{:?}", r.findings);
    }

    #[test]
    fn not_predicate_is_valid_and_range_checked() {
        // not_treats targeting a Gene (out of range, same as treats) -> edge.range, NOT predicate.invalid
        let drug = "---\ntype: Molecule\nidentifier: DrugX\nsubtype: drug\nedges:\n  - predicate: not_treats\n    object: BRAF\n    knowledge_level: statistical_association\n    agent_type: data_analysis_pipeline\n    primary_source: Trial\n---\n# DrugX\n";
        let gene = "---\ntype: Gene\nidentifier: BRAF\nsubtype: protein_coding\n---\n# BRAF\n";
        let src = "---\ntype: Study\nidentifier: Trial\nsubtype: rct\nraw_source: [raw/t]\n---\n# t\n";
        let r = lint_docs(&[("molecule/drugx.md", drug), ("gene/braf.md", gene), ("study/trial.md", src)]);
        assert!(!r.findings.iter().any(|f| f.rule == "predicate.invalid"), "{:?}", r.findings);
        assert!(r.findings.iter().any(|f| f.rule == "edge.range" && f.message.contains("not_treats")), "{:?}", r.findings);
    }

    #[test]
    fn negated_on_nonnegatable_predicate_errors() {
        // `is_a` + negated:true is not a negatable predicate
        let g = "---\ntype: Gene\nidentifier: BRAF\nsubtype: protein_coding\nedges:\n  - predicate: is_a\n    object: Oncogene\n    negated: true\n    knowledge_level: knowledge_assertion\n    agent_type: manual_agent\n    primary_source: Src\n---\n# BRAF\n";
        let oncogene = "---\ntype: MolecularClass\nidentifier: Oncogene\nsubtype: gene_set\n---\n# o\n";
        let src = "---\ntype: Publication\nidentifier: Src\nsubtype: article\nraw_source: [raw/s]\n---\n# s\n";
        let r = lint_docs(&[("gene/braf.md", g), ("molecularclass/oncogene.md", oncogene), ("publication/src.md", src)]);
        assert!(r.findings.iter().any(|f| f.rule == "edge.not_negatable"), "{:?}", r.findings);
    }

    #[test]
    fn positive_and_negative_edge_contradict() {
        // treats and not_treats for the same subject -> object
        let drug = "---\ntype: Molecule\nidentifier: DrugX\nsubtype: drug\nedges:\n  - predicate: treats\n    object: Asthma\n    knowledge_level: statistical_association\n    agent_type: data_analysis_pipeline\n    primary_source: TrialA\n  - predicate: not_treats\n    object: Asthma\n    knowledge_level: statistical_association\n    agent_type: data_analysis_pipeline\n    primary_source: TrialB\n---\n# DrugX\n";
        let dis = "---\ntype: Disease\nidentifier: Asthma\nsubtype: respiratory\n---\n# a\n";
        let a = "---\ntype: Study\nidentifier: TrialA\nsubtype: rct\nraw_source: [raw/a]\n---\n# a\n";
        let b = "---\ntype: Study\nidentifier: TrialB\nsubtype: rct\nraw_source: [raw/b]\n---\n# b\n";
        let r = lint_docs(&[("molecule/drugx.md", drug), ("disease/asthma.md", dis), ("study/a.md", a), ("study/b.md", b)]);
        assert!(r.findings.iter().any(|f| f.rule == "edge.contradiction"), "{:?}", r.findings);
    }

    #[test]
    fn value_as_identifier_flagged_but_not_real_entities() {
        let val = "---\ntype: BiomedicalMeasure\nidentifier: 183 cm\nsubtype: anthropometric\n---\n# h\n";
        let mol = "---\ntype: Molecule\nidentifier: 2-AG\nsubtype: lipid\n---\n# m\n";
        let r = lint_docs(&[("biomedicalmeasure/h.md", val), ("molecule/2ag.md", mol)]);
        assert!(r.findings.iter().any(|f| f.rule == "value.as_identifier" && f.subject == "183 cm"), "{:?}", r.findings);
        assert!(!r.findings.iter().any(|f| f.rule == "value.as_identifier" && f.subject == "2-AG"), "{:?}", r.findings);
    }

    #[test]
    fn other_requires_note() {
        let other = "---\ntype: Other\nidentifier: Some thing\nsubtype: misc\n---\n# x\n";
        let r = lint_docs(&[("other/x.md", other)]);
        assert!(r.findings.iter().any(|f| f.rule == "other.missing_note"), "{:?}", r.findings);
    }

    #[test]
    fn ci_ordering_flagged() {
        let g = "---\ntype: Molecule\nidentifier: DrugZ\nsubtype: drug\nedges:\n  - predicate: treats\n    object: Flu\n    knowledge_level: statistical_association\n    agent_type: data_analysis_pipeline\n    primary_source: T1\n    ci_lower: 1.2\n    ci_upper: 0.8\n  - predicate: reported_in\n    object: T1\n    knowledge_level: knowledge_assertion\n    agent_type: manual_agent\n    primary_source: T1\n---\n# DrugZ\n";
        let dis = "---\ntype: Disease\nidentifier: Flu\nsubtype: infection\n---\n# f\n";
        let t1 = "---\ntype: Study\nidentifier: T1\nsubtype: rct\nraw_source: [raw/t1]\n---\n# t\n";
        let r = lint_docs(&[("molecule/drugz.md", g), ("disease/flu.md", dis), ("study/t1.md", t1)]);
        assert!(r.findings.iter().any(|f| f.rule == "edge.stat_ci"), "{:?}", r.findings);
    }
}
