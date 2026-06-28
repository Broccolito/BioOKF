//! Open a BioOKF bundle directory: locate concept documents, parse them into
//! `Node`s, and index them by `identifier`.

use crate::model::Node;
use crate::parse::parse_node;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Bundle {
    pub root: PathBuf,
    pub nodes: Vec<Node>,
    /// identifier -> index into `nodes` (first wins; duplicates recorded separately).
    pub by_identifier: HashMap<String, usize>,
    /// (identifier, path) pairs that duplicate an existing identifier.
    pub duplicate_identifiers: Vec<(String, PathBuf)>,
    /// (relative path, parse error message) for files that failed to parse.
    pub parse_errors: Vec<(PathBuf, String)>,
    pub has_index_md: bool,
    pub has_log_md: bool,
    pub has_schema_md: bool,
}

const RESERVED: [&str; 4] = ["index.md", "log.md", "SCHEMA.md", "README.md"];

fn is_reserved(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| RESERVED.contains(&n))
        .unwrap_or(false)
}

/// Recursively collect `*.md` concept-document files under `dir`, skipping any
/// `raw/` or `citations/` subtree and the reserved root files.
fn collect_md(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if path.is_dir() {
            if name == "raw" || name == "citations" || name.starts_with('.') {
                continue;
            }
            collect_md(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") && !is_reserved(&path) {
            out.push(path);
        }
    }
}

impl Bundle {
    /// Open a bundle rooted at `root`. If a `knowledge/` subdirectory exists it is
    /// the source of concept docs; otherwise the whole tree is scanned (minus
    /// reserved files and `raw/`).
    pub fn open(root: impl AsRef<Path>) -> std::io::Result<Bundle> {
        let root = root.as_ref().to_path_buf();
        let knowledge = root.join("knowledge");
        let scan_root = if knowledge.is_dir() { knowledge.clone() } else { root.clone() };

        let mut files = Vec::new();
        collect_md(&scan_root, &mut files);
        files.sort();

        let mut nodes = Vec::new();
        let mut by_identifier: HashMap<String, usize> = HashMap::new();
        let mut duplicate_identifiers = Vec::new();
        let mut parse_errors = Vec::new();

        for file in files {
            let rel = file.strip_prefix(&root).unwrap_or(&file).to_path_buf();
            let content = match std::fs::read_to_string(&file) {
                Ok(c) => c,
                Err(e) => {
                    parse_errors.push((rel, e.to_string()));
                    continue;
                }
            };
            match parse_node(&content, &rel) {
                Ok(node) => {
                    let id = node.identifier.clone();
                    if by_identifier.contains_key(&id) {
                        duplicate_identifiers.push((id, node.path.clone()));
                    } else {
                        by_identifier.insert(id, nodes.len());
                    }
                    nodes.push(node);
                }
                Err(e) => parse_errors.push((rel, e.to_string())),
            }
        }

        Ok(Bundle {
            has_index_md: root.join("index.md").is_file(),
            has_log_md: root.join("log.md").is_file(),
            has_schema_md: root.join("SCHEMA.md").is_file(),
            root,
            nodes,
            by_identifier,
            duplicate_identifiers,
            parse_errors,
        })
    }

    pub fn get(&self, identifier: &str) -> Option<&Node> {
        self.by_identifier.get(identifier).map(|&i| &self.nodes[i])
    }

    pub fn contains(&self, identifier: &str) -> bool {
        self.by_identifier.contains_key(identifier)
    }
}
