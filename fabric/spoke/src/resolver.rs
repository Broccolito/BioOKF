//! The waterfall resolver: BioOKF nodes -> SPOKE nodes, and BioOKF edges -> SPOKE agreement.
//! Faithful port of resolver.py. Node tiers (highest confidence first):
//!   1 exact_identifier  2 exact_name  2b exact_name_ci(variants)  3 exact_name_ci(fulltext)
//!   4 exact_synonym (guarded)  5 fuzzy_accept  else ambiguous / not_found

use anyhow::Result;
use futures::stream::{self, StreamExt};
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::{json, Value};
use std::collections::{BTreeSet, HashMap, HashSet};

use crate::client::{Client, FoundRel, FtHit, Hit};
use crate::crosswalk::{PredicateCrosswalk, TypeCrosswalk};
use crate::normalize::{jaccard, norm_key, norm_tokens};

// Internal tier -> reported match_method. Case-sensitive/insensitive name both report exact_name.
pub fn match_method(tier: &str) -> Option<&'static str> {
    match tier {
        "exact_identifier" => Some("exact_identifier"),
        "exact_name" => Some("exact_name"),
        "exact_name_ci" => Some("exact_name"),
        "exact_synonym" => Some("synonym"),
        "protein_desc" => Some("exact_name"), // matched the protein's recommended name (in description)
        "fuzzy_accept" => Some("fuzzy"),
        "llm_accept" => Some("reviewed"),
        _ => None,
    }
}

pub const CONFIDENT_TIERS: &[&str] =
    &["exact_identifier", "exact_name", "exact_name_ci", "exact_synonym", "protein_desc"];

// Names that denote a MODIFIER OF an entity, not the entity itself. A fuzzy hit that adds one of
// these words (absent from the query) is a derivative (e.g. "NF-kappaB inhibitor", "VDR agonist")
// and must not be accepted as the entity. (fix: fuzzy derivative guard)
const DERIVATIVE_WORDS: &[&str] = &[
    "inhibitor", "inhibitors", "agonist", "agonists", "antagonist", "antagonists", "blocker",
    "blockers", "activator", "activators", "modulator", "modulators", "inducer", "inducers",
];

fn is_derivative(query: &str, cand_name: Option<&str>) -> bool {
    let cand = match cand_name {
        Some(c) if !c.is_empty() => c,
        _ => return false,
    };
    let qt = norm_tokens(query);
    let ct = norm_tokens(cand);
    DERIVATIVE_WORDS.iter().any(|w| ct.contains(*w) && !qt.contains(*w))
}

/// The SPOKE description's recommended name — the text before the first " (" qualifier.
fn primary_name(descr: Option<&str>) -> String {
    match descr {
        Some(d) => d.split(" (").next().unwrap_or(d).trim().to_string(),
        None => String::new(),
    }
}

/// Name-plausibility guard for the identifier tier: an xref/identifier hit is only trustworthy if the
/// SPOKE node's lexical identity (name + synonyms + description recommended-name) shares at least one
/// normalized token with the BioOKF node's (name + synonyms). Catches wrong source xrefs (e.g. "Axon"
/// tagged UBERON:0001020 = "nervous system commissure") while allowing mnemonic-named hits (Protein).
fn name_plausible(bio_name: &str, bio_syns: &[String], hit: &Hit) -> bool {
    let mut bio: HashSet<String> = norm_tokens(bio_name);
    for s in bio_syns {
        bio.extend(norm_tokens(s));
    }
    let mut sp: HashSet<String> = norm_tokens(hit.name.as_deref().unwrap_or(""));
    if let Some(syns) = &hit.syn {
        for s in syns {
            sp.extend(norm_tokens(s));
        }
    }
    let pn = primary_name(hit.descr.as_deref());
    sp.extend(norm_tokens(&pn));
    if bio.is_empty() || sp.is_empty() {
        return true; // nothing to judge on (e.g. rsID-named Variant with null name) -> allow
    }
    bio.intersection(&sp).next().is_some()
}

static RE_RS: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^rs\d+$").unwrap());
static RE_STAR: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[A-Za-z0-9]{2,}\*\w+$").unwrap());
static RE_CURIE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[A-Za-z][A-Za-z0-9.]{1,20}:\S+$").unwrap());

fn id_like(s: &str) -> bool {
    let s = s.trim();
    RE_RS.is_match(s) || RE_STAR.is_match(s) || RE_CURIE.is_match(s)
}

/// Multi-word, long (>=6 chars), or identifier-shaped => specific enough to auto-accept a synonym.
fn specific(term: &str) -> bool {
    let t = term.trim();
    t.contains(' ') || t.chars().count() >= 6 || id_like(t)
}

/// A shared synonym KEY is a trustworthy anchor (specific enough to accept a match on) if it is
/// multi-word, long, identifier-shaped, OR contains a digit — gene/receptor symbols like HTR6, SOD1,
/// CHRNA4 are short but specific. Bare short all-letter acronyms ("UVB") are NOT anchors.
fn is_anchor_key(k: &str) -> bool {
    k.contains(' ') || k.chars().count() >= 6 || id_like(k) || k.chars().any(|c| c.is_ascii_digit())
}

/// True when the candidate's name is a strict generalization of the term (proper subset of tokens).
fn is_broad_parent(term: &str, cand_name: Option<&str>) -> bool {
    let cand_name = match cand_name {
        Some(c) if !c.is_empty() => c,
        _ => return false,
    };
    let tt: HashSet<String> = norm_key(term).split(' ').filter(|s| !s.is_empty()).map(|s| s.to_string()).collect();
    let ct: HashSet<String> = norm_key(cand_name).split(' ').filter(|s| !s.is_empty()).map(|s| s.to_string()).collect();
    if ct.is_empty() || tt.is_empty() {
        return false;
    }
    ct.is_subset(&tt) && ct != tt
}

fn synset(name: &str, synonyms: &[String]) -> HashSet<String> {
    let mut keys = HashSet::new();
    let k = norm_key(name);
    if !k.is_empty() {
        keys.insert(k);
    }
    for s in synonyms {
        let k = norm_key(s);
        if !k.is_empty() {
            keys.insert(k);
        }
    }
    keys
}

#[derive(Clone, Debug)]
pub struct BNode {
    pub identifier: String,
    pub node_type: Option<String>,
    pub subtype: Option<String>,
    pub synonyms: Vec<String>,
    pub xref: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Candidate {
    pub label: String,
    pub identifier: Value,
    pub name: Option<String>,
    pub score: Option<f64>,
    pub reason: String,
}

#[derive(Clone, Debug)]
pub struct NodeMatch {
    pub status: String,
    pub spoke_label: Option<String>,
    pub spoke_identifier: Option<Value>,
    pub spoke_name: Option<String>,
    pub match_tier: Option<String>,
    pub candidate_labels: Vec<String>,
    pub candidates: Vec<Candidate>,
    pub notes: String,
}

impl NodeMatch {
    fn new(status: &str) -> Self {
        NodeMatch {
            status: status.to_string(),
            spoke_label: None,
            spoke_identifier: None,
            spoke_name: None,
            match_tier: None,
            candidate_labels: vec![],
            candidates: vec![],
            notes: String::new(),
        }
    }

    pub fn to_json(&self, knowledge_graph: &str) -> Value {
        let mapped = self.status == "mapped";
        let method = self.match_tier.as_deref().and_then(match_method);
        let cands: Vec<Value> = self
            .candidates
            .iter()
            .map(|c| {
                json!({
                    "target_type": c.label,
                    "target_identifier": c.identifier,
                    "target_name": c.name,
                    "score": c.score,
                    "source": c.reason,
                })
            })
            .collect();
        json!({
            "knowledge_graph": knowledge_graph,
            "mapping_status": self.status,
            "candidate_types": self.candidate_labels,
            "target_type": if mapped { json!(self.spoke_label) } else { Value::Null },
            "target_identifier": if mapped { self.spoke_identifier.clone().unwrap_or(Value::Null) } else { Value::Null },
            "target_name": if mapped { json!(self.spoke_name) } else { Value::Null },
            "match_method": if mapped { json!(method) } else { Value::Null },
            "candidates": cands,
            "notes": self.notes,
        })
    }
}

struct Mappable {
    ident: String,
    labels: Vec<String>,
    synonyms: Vec<String>,
    xref: Vec<String>,
}

pub struct NodeResolver<'a> {
    c: &'a Client,
    tx: &'a TypeCrosswalk,
    fuzzy_min_score: f64,
    fuzzy_min_jaccard: f64,
    fuzzy_dominance: f64,
}

impl<'a> NodeResolver<'a> {
    pub fn new(c: &'a Client, tx: &'a TypeCrosswalk) -> Self {
        NodeResolver { c, tx, fuzzy_min_score: 3.0, fuzzy_min_jaccard: 0.6, fuzzy_dominance: 1.3 }
    }

    /// label -> indices into `mappable` for nodes not yet matched, in first-seen order.
    fn remaining_by_label(
        mappable: &[Mappable],
        matches: &HashMap<String, NodeMatch>,
    ) -> IndexMap<String, Vec<usize>> {
        let mut by: IndexMap<String, Vec<usize>> = IndexMap::new();
        for (i, n) in mappable.iter().enumerate() {
            if matches.contains_key(&n.ident) {
                continue;
            }
            for label in &n.labels {
                by.entry(label.clone()).or_default().push(i);
            }
        }
        by
    }

    pub async fn resolve_all(&self, bnodes: &[BNode]) -> Result<HashMap<String, NodeMatch>> {
        let timing = std::env::var("SPOKE_FABRIC_TIMING").is_ok();
        let t0 = std::time::Instant::now();
        let mut matches: HashMap<String, NodeMatch> = HashMap::new();
        let mut mappable: Vec<Mappable> = vec![];
        for n in bnodes {
            let labels = self
                .tx
                .candidate_labels(n.node_type.as_deref(), n.subtype.as_deref());
            if labels.is_empty() {
                let mut m = NodeMatch::new("not_mappable");
                m.candidate_labels = vec![];
                m.notes = format!(
                    "BioOKF type '{}' has no SPOKE analogue (provenance/context)",
                    n.node_type.as_deref().unwrap_or("")
                );
                matches.insert(n.identifier.clone(), m);
                continue;
            }
            mappable.push(Mappable {
                ident: n.identifier.clone(),
                labels,
                synonyms: n.synonyms.clone(),
                xref: n.xref.clone(),
            });
        }

        // ---- Tier 1: identifier match (name + id-like synonyms + xref CURIEs) ----
        let mut id_by_label: IndexMap<String, IndexMap<String, Vec<String>>> = IndexMap::new();
        for n in &mappable {
            let id_likes: Vec<String> =
                n.synonyms.iter().filter(|s| id_like(s)).cloned().collect();
            let mut cand_ids = vec![n.ident.clone()];
            cand_ids.extend(id_likes.clone());
            for label in &n.labels {
                let mut xr_input = n.xref.clone();
                xr_input.extend(n.synonyms.clone());
                let xr = self.tx.xref_identifiers(label, &xr_input);
                let mut all = cand_ids.clone();
                all.extend(xr);
                id_by_label
                    .entry(label.clone())
                    .or_default()
                    .insert(n.ident.clone(), all);
            }
        }
        for (label, node_ids) in &id_by_label {
            let mut allset: BTreeSet<String> = BTreeSet::new();
            for lst in node_ids.values() {
                for i in lst {
                    allset.insert(i.clone());
                }
            }
            let allids: Vec<String> = allset.into_iter().collect();
            let hits = self.c.batch_exact_by_identifier(label, &allids).await?;
            for n in &mappable {
                if matches.contains_key(&n.ident) || !n.labels.contains(label) {
                    continue;
                }
                if let Some(cids) = node_ids.get(&n.ident) {
                    for cid in cids {
                        if let Some(got) = hits.get(cid) {
                            if let Some(r) = got.first() {
                                // guard: reject an xref/identifier hit whose SPOKE lexical identity
                                // shares no token with the BioOKF name (catches wrong source xrefs)
                                if !name_plausible(&n.ident, &n.synonyms, r) {
                                    continue;
                                }
                                let mut m = NodeMatch::new("mapped");
                                m.spoke_label = Some(label.clone());
                                m.spoke_identifier = Some(r.id.clone());
                                m.spoke_name = r.name.clone();
                                m.match_tier = Some("exact_identifier".into());
                                m.candidate_labels = n.labels.clone();
                                m.notes = format!("identifier match on '{}'", cid);
                                matches.insert(n.ident.clone(), m);
                                break;
                            }
                        }
                    }
                }
            }
        }

        if timing { eprintln!("  [tier1 identifier] {:.1}s cum", t0.elapsed().as_secs_f64()); }

        // ---- Tier 2: exact name (case-sensitive), per name-indexed candidate label ----
        for (label, idxs) in Self::remaining_by_label(&mappable, &matches) {
            let names: Vec<String> = idxs.iter().map(|&i| mappable[i].ident.clone()).collect();
            let pref = self.tx.label_preference(&label);
            if let Some(pref) = &pref {
                let phits = self.c.batch_exact_by_name(&label, &names, Some(pref)).await?;
                for &i in &idxs {
                    let ident = &mappable[i].ident;
                    if matches.contains_key(ident) {
                        continue;
                    }
                    if let Some(got) = phits.get(ident) {
                        if got.len() == 1 {
                            let r = &got[0];
                            let mut m = NodeMatch::new("mapped");
                            m.spoke_label = Some(label.clone());
                            m.spoke_identifier = Some(r.id.clone());
                            m.spoke_name = r.name.clone();
                            m.match_tier = Some("exact_name".into());
                            m.candidate_labels = mappable[i].labels.clone();
                            m.notes = format!("exact name + preference {}", py_dict_repr(pref));
                            matches.insert(ident.clone(), m);
                        }
                    }
                }
            }
            // name-alias pass
            let mut alias_of: Vec<(String, String)> = vec![];
            for &i in &idxs {
                let ident = &mappable[i].ident;
                if matches.contains_key(ident) {
                    continue;
                }
                if let Some(a) = self.tx.name_alias(&label, ident) {
                    alias_of.push((ident.clone(), a));
                }
            }
            if !alias_of.is_empty() {
                let uniq: Vec<String> = {
                    let mut seen = HashSet::new();
                    alias_of
                        .iter()
                        .filter_map(|(_, a)| if seen.insert(a.clone()) { Some(a.clone()) } else { None })
                        .collect()
                };
                let ahits = self.c.batch_exact_by_name(&label, &uniq, pref.as_ref()).await?;
                // Fallback: an alias the preference query didn't uniquely resolve (e.g. viruses whose
                // taxon is level='no rank', not 'species') gets a plain exact-name match.
                let need_plain: Vec<String> = uniq
                    .iter()
                    .filter(|a| ahits.get(*a).map(|g| g.len() != 1).unwrap_or(true))
                    .cloned()
                    .collect();
                let ahits_plain = if pref.is_some() && !need_plain.is_empty() {
                    self.c.batch_exact_by_name(&label, &need_plain, None).await?
                } else {
                    HashMap::new()
                };
                for (ident, ali) in &alias_of {
                    if matches.contains_key(ident) {
                        continue;
                    }
                    let got = match ahits.get(ali) {
                        Some(g) if g.len() == 1 => Some(g),
                        _ => ahits_plain.get(ali).filter(|g| g.len() == 1),
                    };
                    if let Some(g) = got {
                        let r = &g[0];
                        let mut m = NodeMatch::new("mapped");
                        m.spoke_label = Some(label.clone());
                        m.spoke_identifier = Some(r.id.clone());
                        m.spoke_name = r.name.clone();
                        m.match_tier = Some("exact_name".into());
                        m.candidate_labels =
                            mappable[idxs.iter().copied().find(|&i| &mappable[i].ident == ident).unwrap()].labels.clone();
                        m.notes = format!("name alias '{}' -> '{}'", ident, ali);
                        matches.insert(ident.clone(), m);
                    }
                }
            }
            // normal exact-name pass
            let hits = self.c.batch_exact_by_name(&label, &names, None).await?;
            for &i in &idxs {
                let ident = &mappable[i].ident;
                if matches.contains_key(ident) {
                    continue;
                }
                if let Some(got) = hits.get(ident) {
                    if got.len() == 1 {
                        let r = &got[0];
                        let mut m = NodeMatch::new("mapped");
                        m.spoke_label = Some(label.clone());
                        m.spoke_identifier = Some(r.id.clone());
                        m.spoke_name = r.name.clone();
                        m.match_tier = Some("exact_name".into());
                        m.candidate_labels = mappable[i].labels.clone();
                        matches.insert(ident.clone(), m);
                    } else if got.len() > 1 {
                        let mut m = NodeMatch::new("ambiguous");
                        m.spoke_label = Some(label.clone());
                        m.candidate_labels = mappable[i].labels.clone();
                        m.candidates = got
                            .iter()
                            .take(6)
                            .map(|g| Candidate {
                                label: label.clone(),
                                identifier: g.id.clone(),
                                name: g.name.clone(),
                                score: None,
                                reason: "exact_name".into(),
                            })
                            .collect();
                        m.notes = format!("{} exact-name hits in {}", got.len(), label);
                        matches.insert(ident.clone(), m);
                    }
                }
            }
        }

        if timing { eprintln!("  [tier2 exact-name] {:.1}s cum", t0.elapsed().as_secs_f64()); }

        // ---- Tier 2b: case-insensitive exact name via case variants ----
        for (label, idxs) in Self::remaining_by_label(&mappable, &matches) {
            let names: Vec<String> = idxs.iter().map(|&i| mappable[i].ident.clone()).collect();
            let hits = self.c.batch_exact_by_name_ci(&label, &names).await?;
            for &i in &idxs {
                let ident = &mappable[i].ident;
                if matches.contains_key(ident) {
                    continue;
                }
                if let Some(got) = hits.get(ident) {
                    if got.len() == 1 {
                        let r = &got[0];
                        let mut m = NodeMatch::new("mapped");
                        m.spoke_label = Some(label.clone());
                        m.spoke_identifier = Some(r.id.clone());
                        m.spoke_name = r.name.clone();
                        m.match_tier = Some("exact_name_ci".into());
                        m.candidate_labels = mappable[i].labels.clone();
                        m.notes = "case-variant exact name".into();
                        matches.insert(ident.clone(), m);
                    }
                }
            }
        }

        if timing { eprintln!("  [tier2b ci-variant] {:.1}s cum", t0.elapsed().as_secs_f64()); }

        // ---- Tiers 3-5: batched fulltext -> ci-exact / synonym / fuzzy ----
        let mut pool: HashMap<String, Vec<(FtHit, String)>> = mappable
            .iter()
            .filter(|n| !matches.contains_key(&n.ident))
            .map(|n| (n.ident.clone(), vec![]))
            .collect();
        // Fetch every label's fulltext candidates concurrently, then fill `pool` strictly in
        // remaining_by_label (label) order — so per-node candidate ordering is identical to the
        // sequential version (ci_hit picks the first name-key match; syn_hits[0] drives the guard).
        let rbl = Self::remaining_by_label(&mappable, &matches);
        let label_inputs: Vec<(String, Vec<String>)> = rbl
            .iter()
            .map(|(l, idxs)| (l.clone(), idxs.iter().map(|&i| mappable[i].ident.clone()).collect()))
            .collect();
        let fetched: Vec<Result<(String, HashMap<String, Vec<FtHit>>)>> =
            stream::iter(label_inputs.iter())
                .map(|(l, names)| async move {
                    self.c.batch_fulltext(l, names, 25).await.map(|ft| (l.clone(), ft))
                })
                .buffer_unordered(crate::client::CONCURRENCY)
                .collect()
                .await;
        let mut ft_by_label: HashMap<String, HashMap<String, Vec<FtHit>>> = HashMap::new();
        for res in fetched {
            let (l, ft) = res?;
            ft_by_label.insert(l, ft);
        }
        for (label, idxs) in &rbl {
            let ft = match ft_by_label.get(label) {
                Some(f) => f,
                None => continue,
            };
            for &i in idxs {
                let ident = &mappable[i].ident;
                if !pool.contains_key(ident) {
                    continue;
                }
                if let Some(cs) = ft.get(ident) {
                    for cand in cs {
                        pool.get_mut(ident).unwrap().push((cand.clone(), label.clone()));
                    }
                }
            }
        }

        if timing { eprintln!("  [fulltext gather] {:.1}s cum", t0.elapsed().as_secs_f64()); }

        for n in &mappable {
            let ident = &n.ident;
            if matches.contains_key(ident) {
                continue;
            }
            let empty = vec![];
            let cands = pool.get(ident).unwrap_or(&empty);
            let b_keys = synset(ident, &n.synonyms);
            let b_name_key = norm_key(ident);
            let mut ci_hit: Option<&(FtHit, String)> = None;
            let mut syn_hits: Vec<&(FtHit, String)> = vec![];
            for cand in cands {
                let cand_name_key = norm_key(cand.0.name.as_deref().unwrap_or(""));
                if ci_hit.is_none() && !cand_name_key.is_empty() && cand_name_key == b_name_key {
                    ci_hit = Some(cand);
                    continue;
                }
                let c_keys = synset(
                    cand.0.name.as_deref().unwrap_or(""),
                    cand.0.syn.as_deref().unwrap_or(&[]),
                );
                if b_keys.intersection(&c_keys).next().is_some() {
                    syn_hits.push(cand);
                }
            }
            if let Some(ci) = ci_hit {
                let mut m = NodeMatch::new("mapped");
                m.spoke_label = Some(ci.1.clone());
                m.spoke_identifier = Some(ci.0.id.clone());
                m.spoke_name = ci.0.name.clone();
                m.match_tier = Some("exact_name_ci".into());
                m.candidate_labels = n.labels.clone();
                m.notes = "case-insensitive exact name".into();
                matches.insert(ident.clone(), m);
                continue;
            }
            // disambiguation guard
            let distinct: HashSet<(String, String)> = syn_hits
                .iter()
                .map(|c| (c.1.clone(), c.0.id.to_string()))
                .collect();
            let syn_hit = syn_hits.first().copied();
            let mut guarded = false;
            if let Some(sh) = syn_hit {
                if distinct.len() > 1 {
                    guarded = true;
                } else if !specific(ident) {
                    guarded = true;
                } else if is_broad_parent(ident, sh.0.name.as_deref()) {
                    guarded = true;
                } else {
                    // token-specificity guard: the shared key(s) that produced the match must include
                    // a SPECIFIC term. A specific node (e.g. "UVB radiation exposure") that overlaps a
                    // candidate only on a bare short acronym ("UVB") is an alias collision -> route to
                    // review rather than accept.
                    let sh_c_keys = synset(
                        sh.0.name.as_deref().unwrap_or(""),
                        sh.0.syn.as_deref().unwrap_or(&[]),
                    );
                    if !b_keys.intersection(&sh_c_keys).any(|k| is_anchor_key(k)) {
                        guarded = true;
                    }
                }
            }
            if let Some(sh) = syn_hit {
                if !guarded {
                    let mut m = NodeMatch::new("mapped");
                    m.spoke_label = Some(sh.1.clone());
                    m.spoke_identifier = Some(sh.0.id.clone());
                    m.spoke_name = sh.0.name.clone();
                    m.match_tier = Some("exact_synonym".into());
                    m.candidate_labels = n.labels.clone();
                    m.notes = "normalized name/synonym match".into();
                    matches.insert(ident.clone(), m);
                    continue;
                }
            }
            if guarded {
                let mut m = NodeMatch::new("ambiguous");
                m.candidate_labels = n.labels.clone();
                m.candidates = syn_hits
                    .iter()
                    .take(6)
                    .map(|c| Candidate {
                        label: c.1.clone(),
                        identifier: c.0.id.clone(),
                        name: c.0.name.clone(),
                        score: None,
                        reason: "synonym".into(),
                    })
                    .collect();
                m.notes = "synonym match failed disambiguation guard; routed to LLM/review".into();
                matches.insert(ident.clone(), m);
                continue;
            }
            // fuzzy
            let mut cands_sorted: Vec<&(FtHit, String)> = cands.iter().collect();
            cands_sorted.sort_by(|a, b| b.0.score.partial_cmp(&a.0.score).unwrap_or(std::cmp::Ordering::Equal));
            if let Some(top) = cands_sorted.first() {
                let runner = cands_sorted.get(1).map(|c| c.0.score).unwrap_or(0.0);
                let jac = jaccard(ident, top.0.name.as_deref().unwrap_or(""));
                let denom = if runner != 0.0 { runner } else { 0.001 };
                let dominates = top.0.score >= self.fuzzy_dominance * denom;
                // guard: never accept a derivative (inhibitor/agonist/...) as the entity itself
                let derivative = is_derivative(ident, top.0.name.as_deref());
                if top.0.score >= self.fuzzy_min_score && jac >= self.fuzzy_min_jaccard && dominates && !derivative {
                    let mut m = NodeMatch::new("mapped");
                    m.spoke_label = Some(top.1.clone());
                    m.spoke_identifier = Some(top.0.id.clone());
                    m.spoke_name = top.0.name.clone();
                    m.match_tier = Some("fuzzy_accept".into());
                    m.candidate_labels = n.labels.clone();
                    m.notes = format!("fuzzy score={:.1} jaccard={:.2}", top.0.score, jac);
                    matches.insert(ident.clone(), m);
                } else {
                    let mut m = NodeMatch::new("ambiguous");
                    m.candidate_labels = n.labels.clone();
                    m.candidates = cands_sorted
                        .iter()
                        .take(6)
                        .map(|p| Candidate {
                            label: p.1.clone(),
                            identifier: p.0.id.clone(),
                            name: p.0.name.clone(),
                            score: Some(round2(p.0.score)),
                            reason: "fulltext".into(),
                        })
                        .collect();
                    m.notes = "no confident match; review-queue candidates for LLM adjudication".into();
                    matches.insert(ident.clone(), m);
                }
            } else {
                let mut m = NodeMatch::new("not_found");
                m.candidate_labels = n.labels.clone();
                m.notes = "no SPOKE candidate by identifier/name/synonym/fulltext".into();
                matches.insert(ident.clone(), m);
            }
        }

        // ---- Tier 6: last-resort Protein recovery by recommended name (in `description`) ----
        // Runs AFTER the compound/fulltext tiers so it never preempts a good Compound match (e.g.
        // L-arginine -> the amino-acid compound). Overrides an ambiguous/not_found Protein-candidate
        // node only when exactly ONE reviewed human protein carries that recommended name / gene.
        {
            let prot_idxs: Vec<usize> = mappable
                .iter()
                .enumerate()
                .filter(|(_, n)| {
                    n.labels.iter().any(|l| l == "Protein")
                        && matches.get(&n.ident).map(|m| m.status != "mapped").unwrap_or(false)
                })
                .map(|(i, _)| i)
                .collect();
            if !prot_idxs.is_empty() {
                let names: Vec<String> = prot_idxs.iter().map(|&i| mappable[i].ident.clone()).collect();
                let hits = self.c.batch_protein_by_description(&names).await?;
                for &i in &prot_idxs {
                    let ident = &mappable[i].ident;
                    if matches.get(ident).map(|m| m.status == "mapped").unwrap_or(false) {
                        continue;
                    }
                    if let Some(got) = hits.get(ident) {
                        let mut seen = HashSet::new();
                        let distinct: Vec<&Hit> =
                            got.iter().filter(|h| seen.insert(h.id.to_string())).collect();
                        if distinct.len() == 1 {
                            let r = distinct[0];
                            let mut m = NodeMatch::new("mapped");
                            m.spoke_label = Some("Protein".into());
                            m.spoke_identifier = Some(r.id.clone());
                            m.spoke_name = r.name.clone();
                            m.match_tier = Some("protein_desc".into());
                            m.candidate_labels = mappable[i].labels.clone();
                            m.notes = "protein recommended-name (description) match".into();
                            matches.insert(ident.clone(), m);
                        }
                    }
                }
            }
        }
        if timing { eprintln!("  [tier6 protein-desc] {:.1}s cum", t0.elapsed().as_secs_f64()); }

        Ok(matches)
    }
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

/// Render a preference map the way Python str(dict) does — `{'k': 'v'}` — so the human-readable
/// `notes` string matches the reference byte-for-byte (prefs are single-key in practice).
fn py_dict_repr(m: &HashMap<String, String>) -> String {
    let inner: Vec<String> = m.iter().map(|(k, v)| format!("'{}': '{}'", k, v)).collect();
    format!("{{{}}}", inner.join(", "))
}

/// Glob match supporting `*` and `?` (fnmatch semantics for our reltype patterns), case-sensitive.
fn glob_match(text: &str, pat: &str) -> bool {
    let t: Vec<char> = text.chars().collect();
    let p: Vec<char> = pat.chars().collect();
    let (mut ti, mut pi) = (0usize, 0usize);
    let (mut star_p, mut star_t): (Option<usize>, usize) = (None, 0);
    while ti < t.len() {
        if pi < p.len() && (p[pi] == '?' || p[pi] == t[ti]) {
            ti += 1;
            pi += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star_p = Some(pi);
            star_t = ti;
            pi += 1;
        } else if let Some(sp) = star_p {
            pi = sp + 1;
            star_t += 1;
            ti = star_t;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }
    pi == p.len()
}

fn reltype_matches(rt: &str, patterns: &[String]) -> bool {
    patterns.iter().any(|p| p == rt || glob_match(rt, p))
}

pub struct EdgeChecker<'a> {
    c: &'a Client,
    px: &'a PredicateCrosswalk,
}

impl<'a> EdgeChecker<'a> {
    pub fn new(c: &'a Client, px: &'a PredicateCrosswalk) -> Self {
        EdgeChecker { c, px }
    }

    pub async fn check(
        &self,
        edge: &Value,
        matches: &HashMap<String, NodeMatch>,
        knowledge_graph: &str,
    ) -> Result<Value> {
        let pred = edge.get("predicate").and_then(|v| v.as_str()).unwrap_or("");
        let negated = pred.starts_with("not_");
        let base = if negated { &pred[4..] } else { pred };
        let src = edge
            .get("source")
            .and_then(|v| v.as_str())
            .or_else(|| edge.get("subject").and_then(|v| v.as_str()));
        let tgt = edge
            .get("target")
            .and_then(|v| v.as_str())
            .or_else(|| edge.get("object").and_then(|v| v.as_str()));
        let sm = src.and_then(|s| matches.get(s));
        let tm = tgt.and_then(|s| matches.get(s));

        // Endpoint mapping status is ALWAYS reported, so a not_evaluated edge is still self-describing
        // (a reader sees the fabric ran and which endpoints resolved to the KG).
        let src_mapped = sm.map(|m| m.status == "mapped").unwrap_or(false);
        let tgt_mapped = tm.map(|m| m.status == "mapped").unwrap_or(false);
        let src_id = if src_mapped {
            sm.and_then(|m| m.spoke_identifier.clone()).unwrap_or(Value::Null)
        } else {
            Value::Null
        };
        let tgt_id = if tgt_mapped {
            tm.and_then(|m| m.spoke_identifier.clone()).unwrap_or(Value::Null)
        } else {
            Value::Null
        };

        let mut ann = json!({
            "knowledge_graph": knowledge_graph,
            "support_status": "not_evaluated",
            "evaluated": false,
            "endpoints": {
                "source_mapped": src_mapped,
                "source_identifier": src_id,
                "target_mapped": tgt_mapped,
                "target_identifier": tgt_id,
            },
            "expected_relation_types": [],
            "found_relation_types": [],
            "evidence": Value::Null,
            "notes": "",
        });

        if self.px.is_provenance_only(base) {
            ann["notes"] = json!(format!(
                "predicate '{}' is provenance-only (no {} relationship to check)",
                base, knowledge_graph
            ));
            return Ok(ann);
        }
        let (sm, tm) = match (sm, tm) {
            (Some(sm), Some(tm)) if sm.status == "mapped" && tm.status == "mapped" => (sm, tm),
            _ => {
                let why = match (src_mapped, tgt_mapped) {
                    (false, false) => "neither endpoint is mapped",
                    (false, true) => "source endpoint is not mapped",
                    (true, false) => "target endpoint is not mapped",
                    (true, true) => "endpoints not both mapped",
                };
                ann["notes"] = json!(format!("{} to {}", why, knowledge_graph));
                return Ok(ann);
            }
        };

        let expected = self.px.expected_families(base);
        let related = self.px.related_families(base);
        ann["evaluated"] = json!(true);
        ann["endpoints"] = json!({
            "source_mapped": true,
            "source_identifier": sm.spoke_identifier.clone().unwrap_or(Value::Null),
            "target_mapped": true,
            "target_identifier": tm.spoke_identifier.clone().unwrap_or(Value::Null),
        });
        ann["expected_relation_types"] = json!(expected);

        let found = self
            .c
            .edges_between(
                sm.spoke_label.as_deref().unwrap_or(""),
                sm.spoke_identifier.as_ref().unwrap_or(&Value::Null),
                tm.spoke_label.as_deref().unwrap_or(""),
                tm.spoke_identifier.as_ref().unwrap_or(&Value::Null),
            )
            .await?;
        let found_types: BTreeSet<String> = found.iter().map(|f| f.rt.clone()).collect();
        ann["found_relation_types"] = json!(found_types.iter().cloned().collect::<Vec<_>>());

        let matched: Vec<&FoundRel> =
            found.iter().filter(|f| reltype_matches(&f.rt, &expected)).collect();
        let rel_matched: Vec<&FoundRel> = if !related.is_empty() {
            found.iter().filter(|f| reltype_matches(&f.rt, &related)).collect()
        } else {
            vec![]
        };
        if !matched.is_empty() {
            ann["evidence"] = edge_evidence(&matched);
            ann["support_status"] = json!(if negated { "contradicted" } else { "supported" });
        } else if !rel_matched.is_empty() && !negated {
            ann["evidence"] = edge_evidence(&rel_matched);
            ann["support_status"] = json!("related");
        } else if !found_types.is_empty() {
            ann["support_status"] = json!("different_relation");
            let all: Vec<&FoundRel> = found.iter().collect();
            ann["evidence"] = edge_evidence(&all);
        } else {
            ann["support_status"] = json!("unsupported");
        }
        Ok(ann)
    }
}

fn edge_evidence(rels: &[&FoundRel]) -> Value {
    let mut keys: BTreeSet<String> = BTreeSet::new();
    let mut srcs: BTreeSet<String> = BTreeSet::new();
    let mut detail: Vec<Value> = vec![];
    let mut rtset: BTreeSet<String> = BTreeSet::new();
    for f in rels {
        for k in &f.keys {
            keys.insert(k.clone());
        }
        match &f.sources {
            Value::String(s) => {
                srcs.insert(s.clone());
            }
            Value::Array(a) => {
                for x in a {
                    if let Some(s) = x.as_str() {
                        srcs.insert(s.to_string());
                    }
                }
            }
            _ => {}
        }
        rtset.insert(f.rt.clone());
        if detail.len() < 12 {
            detail.push(json!({
                "relation_type": f.rt,
                "sources": f.sources,
                "attribute_keys": f.keys,
            }));
        }
    }
    json!({
        "matched_relation_types": rtset.iter().cloned().collect::<Vec<_>>(),
        "relation_count": rels.len(),
        "attribute_keys": keys.iter().cloned().collect::<Vec<_>>(),
        "sources": srcs.iter().filter(|x| !x.is_empty()).cloned().collect::<Vec<_>>(),
        "relations": detail,
    })
}
