//! Self-contained BM25 full-text search over a bundle's concept documents.
//! Mirrors BioRouter's `kb_search` primitive (an in-memory BM25 index built per
//! call) without pulling an external crate.

use crate::bundle::Bundle;
use serde::Serialize;
use std::collections::HashMap;

const K1: f64 = 1.5;
const B: f64 = 0.75;

fn tokenize(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for ch in text.chars() {
        if ch.is_alphanumeric() {
            cur.extend(ch.to_lowercase());
        } else if !cur.is_empty() {
            out.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchHit {
    pub identifier: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub score: f64,
    pub snippet: String,
}

struct Doc {
    identifier: String,
    node_type: String,
    len: usize,
    tf: HashMap<String, usize>,
    raw: String,
}

pub struct SearchIndex {
    docs: Vec<Doc>,
    df: HashMap<String, usize>,
    avgdl: f64,
}

impl SearchIndex {
    pub fn build(bundle: &Bundle) -> SearchIndex {
        let mut docs = Vec::new();
        let mut df: HashMap<String, usize> = HashMap::new();
        let mut total_len = 0usize;
        for n in &bundle.nodes {
            let raw = n.search_text();
            let toks = tokenize(&raw);
            total_len += toks.len();
            let mut tf: HashMap<String, usize> = HashMap::new();
            for t in &toks {
                *tf.entry(t.clone()).or_insert(0) += 1;
            }
            for t in tf.keys() {
                *df.entry(t.clone()).or_insert(0) += 1;
            }
            docs.push(Doc {
                identifier: n.identifier.clone(),
                node_type: n.node_type.as_str().to_string(),
                len: toks.len(),
                tf,
                raw,
            });
        }
        let avgdl = if docs.is_empty() { 1.0 } else { total_len as f64 / docs.len() as f64 };
        SearchIndex { docs, df, avgdl }
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchHit> {
        let q_terms = tokenize(query);
        let n = self.docs.len() as f64;
        let mut scored: Vec<(f64, &Doc)> = Vec::new();
        for doc in &self.docs {
            let mut score = 0.0;
            for term in &q_terms {
                let f = match doc.tf.get(term) {
                    Some(&f) => f as f64,
                    None => continue,
                };
                let df = *self.df.get(term).unwrap_or(&0) as f64;
                let idf = ((n - df + 0.5) / (df + 0.5) + 1.0).ln();
                let denom = f + K1 * (1.0 - B + B * (doc.len as f64 / self.avgdl));
                score += idf * (f * (K1 + 1.0)) / denom;
            }
            if score > 0.0 {
                scored.push((score, doc));
            }
        }
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored
            .into_iter()
            .take(limit)
            .map(|(score, doc)| SearchHit {
                identifier: doc.identifier.clone(),
                node_type: doc.node_type.clone(),
                score: (score * 1000.0).round() / 1000.0,
                snippet: snippet(&doc.raw, &q_terms),
            })
            .collect()
    }
}

fn snippet(text: &str, q_terms: &[String]) -> String {
    let lower = text.to_lowercase();
    let pos = q_terms
        .iter()
        .filter_map(|t| lower.find(t.as_str()))
        .min()
        .unwrap_or(0);
    let start = pos.saturating_sub(40);
    let end = (start + 160).min(text.len());
    // align to char boundaries
    let start = (0..=start).rev().find(|&i| text.is_char_boundary(i)).unwrap_or(0);
    let end = (end..=text.len()).find(|&i| text.is_char_boundary(i)).unwrap_or(text.len());
    let mut s: String = text[start..end].split_whitespace().collect::<Vec<_>>().join(" ");
    if start > 0 {
        s.insert_str(0, "…");
    }
    if end < text.len() {
        s.push('…');
    }
    s
}
