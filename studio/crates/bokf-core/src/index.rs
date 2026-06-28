//! `index.md` generation: an identifier registry + by-type catalog + subtypes-in-use,
//! derived from the parsed bundle. Regenerate with `bokf index`. The generated block is
//! delimited by markers so a human-authored header/intro above it is preserved.

use crate::bundle::Bundle;
use std::collections::{BTreeMap, BTreeSet};

pub const START: &str = "<!-- bokf:index:start -->";
pub const END: &str = "<!-- bokf:index:end -->";

/// Render the generated index block (registry + catalog + subtypes-in-use).
pub fn generate(bundle: &Bundle) -> String {
    let mut nodes: Vec<&crate::model::Node> = bundle.nodes.iter().collect();
    nodes.sort_by(|a, b| a.identifier.cmp(&b.identifier));

    // identifier registry
    let mut registry = String::from("## Identifier registry\n\n| identifier | type | subtype | description |\n|---|---|---|---|\n");
    for n in &nodes {
        let desc = n
            .description
            .clone()
            .or_else(|| n.note.clone())
            .or_else(|| n.body.lines().map(str::trim).find(|l| !l.is_empty() && !l.starts_with('#')).map(|s| s.to_string()))
            .unwrap_or_default();
        let desc: String = desc.replace(['\n', '|'], " ").chars().take(80).collect();
        registry.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            n.identifier.replace('|', "\\|"),
            n.node_type.as_str(),
            n.subtype.clone().unwrap_or_default().replace('|', "\\|"),
            desc.trim()
        ));
    }

    // by-type catalog
    let mut by_type: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for n in &nodes {
        by_type.entry(n.node_type.as_str()).or_default().push(&n.identifier);
    }
    let mut catalog = String::from("\n## By type\n\n");
    for (t, ids) in &by_type {
        catalog.push_str(&format!("- **{}** ({}): {}\n", t, ids.len(), ids.join(", ")));
    }

    // subtypes in use (node subtypes per type + edge subtypes)
    let mut node_sub: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    let mut edge_sub: BTreeSet<String> = BTreeSet::new();
    for n in &nodes {
        if let Some(st) = &n.subtype {
            node_sub.entry(n.node_type.as_str()).or_default().insert(st);
        }
        for e in &n.edges {
            if let Some(v) = e.stats.get("subtype").and_then(|v| v.as_str()) {
                edge_sub.insert(v.to_string());
            }
        }
    }
    let mut subs = String::from("\n## Subtypes in use\n\n");
    for (t, set) in &node_sub {
        subs.push_str(&format!("- **{}**: {}\n", t, set.iter().copied().collect::<Vec<_>>().join(", ")));
    }
    if !edge_sub.is_empty() {
        subs.push_str(&format!("- **edges**: {}\n", edge_sub.iter().cloned().collect::<Vec<_>>().join(", ")));
    }

    format!("{START}\n{registry}{catalog}{subs}{END}\n")
}

/// Regenerate `index.md`, preserving any human header above the generated block.
pub fn write_index(bundle: &Bundle, name: &str) -> std::io::Result<()> {
    let path = bundle.root.join("index.md");
    let generated = generate(bundle);
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    let new = match (existing.find(START), existing.find(END)) {
        (Some(s), Some(e)) if e >= s => format!("{}{generated}{}", &existing[..s], &existing[e + END.len()..]),
        _ if existing.trim().is_empty() => format!(
            "# {name}\n\nokf_version: 0.5\nbiookf_version: 0.5\n\n> Catalog of concept pages. Regenerate with `bokf index`.\n\n{generated}"
        ),
        _ => format!("{}\n\n{generated}", existing.trim_end()),
    };
    std::fs::write(&path, new)
}

/// Identifiers present in the bundle but not mentioned anywhere in `index.md` (stale index).
pub fn missing_from_index(bundle: &Bundle) -> Vec<String> {
    let idx = std::fs::read_to_string(bundle.root.join("index.md")).unwrap_or_default();
    bundle
        .nodes
        .iter()
        .filter(|n| !idx.contains(&n.identifier))
        .map(|n| n.identifier.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundle::Bundle;

    #[test]
    fn generate_writes_registry_catalog_subtypes_and_check() {
        let dir = tempfile::tempdir().unwrap();
        for (rel, body) in [
            ("gene/braf.md", "---\ntype: Gene\nidentifier: BRAF\nsubtype: protein_coding\n---\n# BRAF\nA kinase gene."),
            ("disease/melanoma.md", "---\ntype: Disease\nidentifier: Melanoma\nsubtype: neoplasm\n---\n# Melanoma\n"),
        ] {
            let p = dir.path().join("knowledge").join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(&p, body).unwrap();
        }
        let b = Bundle::open(dir.path()).unwrap();
        assert_eq!(missing_from_index(&b).len(), 2); // no index.md yet
        write_index(&b, "Demo").unwrap();
        let idx = std::fs::read_to_string(dir.path().join("index.md")).unwrap();
        assert!(idx.contains("## Identifier registry"));
        assert!(idx.contains("| BRAF | Gene | protein_coding |"));
        assert!(idx.contains("**Gene** (1): BRAF"));
        assert!(idx.contains("protein_coding"));
        let b2 = Bundle::open(dir.path()).unwrap();
        assert!(missing_from_index(&b2).is_empty(), "index should be current after write");
    }
}
