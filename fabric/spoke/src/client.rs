//! SPOKE query layer — native Neo4j Bolt (neo4rs), reusing one pooled connection for the whole run.
//! Port of spoke_client.py. Same correctness constraints: name RANGE index only on 21 labels (others
//! full-scan, so guarded to fulltext); fulltext top-score != exact (callers re-rank); type-aware
//! literals (Gene/Organism identifiers are NUMERIC); read-only-guard keyword splitting.

use anyhow::{Context, Result};
use futures::stream::{self, StreamExt};
use neo4rs::Graph;
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::normalize::{lucene_sanitize, norm_key};

/// Max in-flight Bolt queries when a phase has many independent queries (edge checks, id
/// verification). Kept moderate to be a good citizen on the shared dev server; the connection pool
/// (config::connect) is sized just above this.
pub const CONCURRENCY: usize = 32;

// spoke-cli's read-only guard rejects these write/DDL keywords at a word boundary even inside a
// string literal. Real entity names hit them ("set shifting impairment", "...gene deletion..."). We
// inline values but split these words with Cypher string concatenation: runtime value unchanged, the
// literal keyword never appears whole in the source. Kept for parity + Bolt safety.
static GUARD_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\b(create|merge|delete|remove|set|drop)\b").unwrap());

// (F1) Read-only enforcement: the fabric is a MAPPING tool and must never mutate the KG. Every query
// is checked for write/DDL clauses before execution, so even on a write-capable (superuser)
// connection the fabric — and the user-facing `query` subcommand — cannot modify SPOKE. Internal
// queries can't false-positive: value literals that contain these words are split by cypher_str
// ('se'+'t'), so the keyword never appears whole in the query text.
static WRITE_CLAUSE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(create|merge|delete|detach|set|remove|drop|foreach|load\s+csv)\b").unwrap()
});

fn assert_read_only(cypher: &str) -> Result<()> {
    if let Some(m) = WRITE_CLAUSE_RE.find(cypher) {
        anyhow::bail!(
            "refusing to run a non-read-only query (contains write/DDL clause '{}'); the spoke \
             fabric is read-only",
            m.as_str()
        );
    }
    Ok(())
}

static FALLBACK_NAME_INDEXED: &[&str] = &[
    "Anatomy", "BiologicalProcess", "CellLine", "CellType", "CellularComponent", "Complex",
    "Compound", "Disease", "Food", "Gene", "MolecularFunction", "Organism", "Pathway",
    "PharmacologicClass", "Protein", "ProteinDomain", "ProteinFamily", "PwGroup", "Reaction",
    "SideEffect", "Symptom",
];
static FALLBACK_FULLTEXT: &[&str] = &[
    "Anatomy", "BiologicalProcess", "Blend", "CellLine", "CellType", "CellularComponent",
    "Chromosome", "ClinicalLab", "Complex", "Compound", "Cytoband", "DietarySupplement", "Disease",
    "EC", "Environment", "Food", "Gene", "Haplotype", "Location", "MiRNA", "MolecularFunction",
    "Organism", "Pathway", "PharmacologicClass", "ProteinDomain", "ProteinFamily", "Protein",
    "PwGroup", "Reaction", "SDoH", "SideEffect", "Symptom", "Variant",
];

// ---- cypher literal helpers (port of cypher_str / cypher_value / cypher_list / cypher_pairs) ----

pub fn cypher_str(s: &str) -> String {
    let esc = s.replace('\\', "\\\\").replace('\'', "\\'");
    let broken = GUARD_RE.replace_all(&esc, |c: &regex::Captures| {
        let w = &c[0];
        format!("{}'+'{}", &w[..1], &w[1..])
    });
    format!("'{}'", broken)
}

/// Type-aware Cypher literal. CRITICAL: SPOKE Gene/Organism identifiers are NUMERIC; quoting them
/// ('672') never matches the integer 672, so numeric identifiers must stay bare.
pub fn cypher_value(v: &Value) -> String {
    match v {
        Value::Bool(b) => if *b { "true".into() } else { "false".into() },
        Value::Number(n) => n.to_string(),
        Value::String(s) => cypher_str(s),
        Value::Null => "null".into(),
        other => cypher_str(&other.to_string()),
    }
}

/// (F3) Bind a JSON identifier Value as a typed Bolt parameter, preserving int (Gene/
/// DietarySupplement) vs string so a parameterized `{identifier:$id}` matches the same node the
/// inlined literal did.
fn bind_id_param(q: neo4rs::Query, key: &str, v: &Value) -> neo4rs::Query {
    match v {
        Value::Number(n) if n.is_i64() => q.param(key, n.as_i64().unwrap()),
        Value::Number(n) if n.is_u64() => q.param(key, n.as_u64().unwrap() as i64),
        Value::Number(n) => q.param(key, n.as_f64().unwrap_or(0.0)),
        Value::Bool(b) => q.param(key, *b),
        Value::String(s) => q.param(key, s.clone()),
        _ => q.param(key, String::new()),
    }
}

pub fn cypher_list(items: &[String]) -> String {
    let inner: Vec<String> = items.iter().map(|x| cypher_str(x)).collect();
    format!("[{}]", inner.join(", "))
}

pub fn cypher_pairs(pairs: &[(String, String)]) -> String {
    let inner: Vec<String> = pairs
        .iter()
        .map(|(a, b)| format!("[{}, {}]", cypher_str(a), cypher_str(b)))
        .collect();
    format!("[{}]", inner.join(", "))
}

fn dedup_nonempty(items: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = vec![];
    for i in items {
        if i.is_empty() {
            continue;
        }
        if seen.insert(i.clone()) {
            out.push(i.clone());
        }
    }
    out
}

// row field readers
fn r_id(row: &Value) -> Value {
    row.get("id").cloned().unwrap_or(Value::Null)
}
fn r_str(row: &Value, k: &str) -> Option<String> {
    row.get(k).and_then(|v| v.as_str().map(|s| s.to_string()))
}
fn r_vec(row: &Value, k: &str) -> Option<Vec<String>> {
    row.get(k).and_then(|v| v.as_array()).map(|a| {
        a.iter()
            .filter_map(|x| x.as_str().map(|s| s.to_string()))
            .collect()
    })
}
fn r_f64(row: &Value, k: &str) -> f64 {
    row.get(k).and_then(|v| v.as_f64()).unwrap_or(0.0)
}

#[derive(Clone, Debug)]
pub struct Hit {
    pub id: Value,
    pub name: Option<String>,
    pub syn: Option<Vec<String>>,
    pub descr: Option<String>,
}

#[derive(Clone, Debug)]
pub struct FtHit {
    pub id: Value,
    pub name: Option<String>,
    pub syn: Option<Vec<String>>,
    pub score: f64,
}

#[derive(Clone, Debug)]
pub struct FoundRel {
    pub rt: String,
    #[allow(dead_code)]
    pub dir: String,
    pub keys: Vec<String>,
    pub sources: Value,
    #[allow(dead_code)]
    pub source: Value,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Stats {
    pub queries: u64,
    pub cache_hits: u64,
}

pub struct Client {
    graph: Graph,
    cache: RefCell<HashMap<String, Vec<Value>>>,
    name_indexed: RefCell<Option<HashSet<String>>>,
    fulltext_indexed: RefCell<Option<HashSet<String>>>,
    stats: RefCell<Stats>,
    verbose: bool,
    disk_cache: Option<PathBuf>,
    cache_ttl_secs: u64,
}

impl Client {
    pub fn new(graph: Graph, verbose: bool) -> Self {
        // Transparent on-disk query cache (like the Python spoke_client): first run is fully live,
        // re-runs read cached rows. Keyed by sha256(cypher). Disable with SPOKE_FABRIC_NO_CACHE.
        let disk_cache = if std::env::var("SPOKE_FABRIC_NO_CACHE").is_ok() {
            None
        } else {
            let dir = crate::config::root().join("runs").join(".spoke_cache");
            let _ = std::fs::create_dir_all(&dir);
            Some(dir)
        };
        // (F6) Disk-cache entries expire after this many seconds (default 7 days) so a re-run can't
        // silently serve stale SPOKE data indefinitely. 0 = never expire (opt out).
        let cache_ttl_secs = std::env::var("SPOKE_FABRIC_CACHE_TTL_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(7 * 24 * 3600);
        Client {
            graph,
            cache: RefCell::new(HashMap::new()),
            name_indexed: RefCell::new(None),
            fulltext_indexed: RefCell::new(None),
            stats: RefCell::new(Stats::default()),
            verbose,
            disk_cache,
            cache_ttl_secs,
        }
    }

    /// (F6) True if a disk-cache file is still within the TTL window (or TTL disabled).
    fn disk_fresh(&self, path: &std::path::Path) -> bool {
        if self.cache_ttl_secs == 0 {
            return true;
        }
        match std::fs::metadata(path).and_then(|m| m.modified()) {
            Ok(mtime) => mtime
                .elapsed()
                .map(|age| age.as_secs() <= self.cache_ttl_secs)
                .unwrap_or(false),
            Err(_) => false,
        }
    }

    fn disk_path(&self, cypher: &str) -> Option<PathBuf> {
        self.disk_cache.as_ref().map(|dir| {
            let mut h = Sha256::new();
            h.update(cypher.as_bytes());
            dir.join(format!("{:x}.json", h.finalize()))
        })
    }

    pub fn stats(&self) -> Stats {
        *self.stats.borrow()
    }

    /// Read-only query by raw text -> rows as JSON objects. The user-facing entry (and static
    /// internal queries). Value-bearing internal queries use `run_cached` with Bolt parameters.
    pub async fn query(&self, cypher: &str) -> Result<Vec<Value>> {
        assert_read_only(cypher)?; // (F1) fabric is read-only; never mutate the KG
        self.run_cached(cypher, neo4rs::query(cypher)).await
    }

    /// (F3) Execute a (possibly parameterized) neo4rs query, caching by `cache_key` — the fully
    /// rendered query string. Values travel as Bolt PARAMETERS (`.param(...)`) rather than being
    /// inlined, so an escaping bug can't inject; caching key == rendered string keeps the on-disk
    /// cache and the emitted results byte-identical to the previous inlined form.
    async fn run_cached(&self, cache_key: &str, q: neo4rs::Query) -> Result<Vec<Value>> {
        if let Some(rows) = self.cache.borrow().get(cache_key) {
            self.stats.borrow_mut().cache_hits += 1;
            return Ok(rows.clone());
        }
        if let Some(path) = self.disk_path(cache_key) {
            if self.disk_fresh(&path) {
                if let Ok(text) = std::fs::read_to_string(&path) {
                    if let Ok(rows) = serde_json::from_str::<Vec<Value>>(&text) {
                        self.cache.borrow_mut().insert(cache_key.to_string(), rows.clone());
                        self.stats.borrow_mut().cache_hits += 1;
                        return Ok(rows);
                    }
                }
            }
        }
        self.stats.borrow_mut().queries += 1;
        if self.verbose {
            let head: String = cache_key.chars().take(100).collect();
            eprintln!("  [spoke] {}", head);
        }
        let mut stream = self
            .graph
            .execute(q)
            .await
            .with_context(|| format!("cypher failed: {}", &cache_key[..cache_key.len().min(160)]))?;
        let mut rows = vec![];
        while let Some(row) = stream.next().await? {
            rows.push(row.to::<Value>()?);
        }
        if let Some(path) = self.disk_path(cache_key) {
            if let Ok(text) = serde_json::to_string(&rows) {
                let _ = std::fs::write(&path, text);
            }
        }
        self.cache.borrow_mut().insert(cache_key.to_string(), rows.clone());
        Ok(rows)
    }

    // ---- index introspection ----
    pub async fn name_indexed_labels(&self) -> HashSet<String> {
        if self.name_indexed.borrow().is_some() {
            return self.name_indexed.borrow().clone().unwrap();
        }
        let set = match self
            .query(
                "SHOW INDEXES YIELD labelsOrTypes, properties, type \
                 WHERE type='RANGE' AND properties=['name'] RETURN labelsOrTypes AS labels",
            )
            .await
        {
            Ok(rows) => {
                let s: HashSet<String> = rows
                    .iter()
                    .filter_map(|r| r_vec(r, "labels").and_then(|v| v.into_iter().next()))
                    .collect();
                if s.is_empty() {
                    FALLBACK_NAME_INDEXED.iter().map(|s| s.to_string()).collect()
                } else {
                    s
                }
            }
            Err(_) => FALLBACK_NAME_INDEXED.iter().map(|s| s.to_string()).collect(),
        };
        *self.name_indexed.borrow_mut() = Some(set.clone());
        set
    }

    pub async fn fulltext_indexed_labels(&self) -> HashSet<String> {
        if self.fulltext_indexed.borrow().is_some() {
            return self.fulltext_indexed.borrow().clone().unwrap();
        }
        let set = match self
            .query(
                "SHOW INDEXES YIELD name, labelsOrTypes, type \
                 WHERE type='FULLTEXT' AND size(labelsOrTypes)=1 RETURN labelsOrTypes AS labels",
            )
            .await
        {
            Ok(rows) => {
                let s: HashSet<String> = rows
                    .iter()
                    .filter_map(|r| r_vec(r, "labels").and_then(|v| v.into_iter().next()))
                    .collect();
                if s.is_empty() {
                    FALLBACK_FULLTEXT.iter().map(|s| s.to_string()).collect()
                } else {
                    s
                }
            }
            Err(_) => FALLBACK_FULLTEXT.iter().map(|s| s.to_string()).collect(),
        };
        *self.fulltext_indexed.borrow_mut() = Some(set.clone());
        set
    }

    // ---- exact lookups ----
    pub async fn batch_exact_by_identifier(
        &self,
        label: &str,
        ids: &[String],
    ) -> Result<HashMap<String, Vec<Hit>>> {
        let ids = dedup_nonempty(ids);
        let mut result: HashMap<String, Vec<Hit>> =
            ids.iter().map(|i| (i.clone(), vec![])).collect();
        if ids.is_empty() {
            return Ok(result);
        }
        for chunk in ids.chunks(300) {
            let cols = "RETURN q AS q, n.identifier AS id, n.name AS name, n.synonyms AS syn, n.description AS descr";
            let cache_key = format!("UNWIND {} AS q MATCH (n:`{}` {{identifier:q}}) {}", cypher_list(chunk), label, cols);
            let cy = format!("UNWIND $qs AS q MATCH (n:`{}` {{identifier:q}}) {}", label, cols);
            let qq = neo4rs::query(&cy).param("qs", chunk.to_vec());
            for r in self.run_cached(&cache_key, qq).await? {
                if let Some(q) = r_str(&r, "q") {
                    result.entry(q).or_default().push(Hit {
                        id: r_id(&r),
                        name: r_str(&r, "name"),
                        syn: r_vec(&r, "syn"),
                        descr: r_str(&r, "descr"),
                    });
                }
            }
        }
        Ok(result)
    }

    pub async fn batch_exact_by_name(
        &self,
        label: &str,
        names: &[String],
        extra_props: Option<&HashMap<String, String>>,
    ) -> Result<HashMap<String, Vec<Hit>>> {
        let names = dedup_nonempty(names);
        let mut result: HashMap<String, Vec<Hit>> =
            names.iter().map(|n| (n.clone(), vec![])).collect();
        if names.is_empty() || !self.name_indexed_labels().await.contains(label) {
            return Ok(result);
        }
        let extra = match extra_props {
            Some(m) if !m.is_empty() => {
                let parts: Vec<String> = m
                    .iter()
                    .map(|(k, v)| format!("{}:{}", k, cypher_value(&Value::String(v.clone()))))
                    .collect();
                format!(", {}", parts.join(", "))
            }
            _ => String::new(),
        };
        for chunk in names.chunks(300) {
            // `extra` (crosswalk-derived tie-breaker props) is trusted and stays inlined; the
            // KB-controlled names travel as the $qs parameter.
            let cols = "RETURN q AS q, n.identifier AS id, n.name AS name, n.synonyms AS syn";
            let cache_key = format!("UNWIND {} AS q MATCH (n:`{}` {{name:q{}}}) {}", cypher_list(chunk), label, extra, cols);
            let cy = format!("UNWIND $qs AS q MATCH (n:`{}` {{name:q{}}}) {}", label, extra, cols);
            let qq = neo4rs::query(&cy).param("qs", chunk.to_vec());
            for r in self.run_cached(&cache_key, qq).await? {
                if let Some(q) = r_str(&r, "q") {
                    result.entry(q).or_default().push(Hit {
                        id: r_id(&r),
                        name: r_str(&r, "name"),
                        syn: r_vec(&r, "syn"),
                        descr: None,
                    });
                }
            }
        }
        Ok(result)
    }

    pub async fn batch_exact_by_name_ci(
        &self,
        label: &str,
        names: &[String],
    ) -> Result<HashMap<String, Vec<Hit>>> {
        let names = dedup_nonempty(names);
        let mut variants: HashMap<String, Vec<String>> = HashMap::new();
        let mut allv: HashSet<String> = HashSet::new();
        for n in &names {
            let vs = name_case_variants(n);
            for v in &vs {
                allv.insert(v.clone());
            }
            variants.insert(n.clone(), vs);
        }
        let mut allv: Vec<String> = allv.into_iter().collect();
        allv.sort();
        let hit = self.batch_exact_by_name(label, &allv, None).await?;
        let mut result: HashMap<String, Vec<Hit>> = HashMap::new();
        for (n, vs) in &variants {
            let mut nodes = vec![];
            let mut seen: HashSet<String> = HashSet::new();
            for v in vs {
                if let Some(hs) = hit.get(v) {
                    for h in hs {
                        let key = h.id.to_string();
                        if seen.insert(key) {
                            nodes.push(h.clone());
                        }
                    }
                }
            }
            result.insert(n.clone(), nodes);
        }
        Ok(result)
    }

    pub async fn batch_fulltext(
        &self,
        label: &str,
        names: &[String],
        per: usize,
    ) -> Result<HashMap<String, Vec<FtHit>>> {
        let names = dedup_nonempty(names);
        let mut result: HashMap<String, Vec<FtHit>> =
            names.iter().map(|n| (n.clone(), vec![])).collect();
        if names.is_empty() || !self.fulltext_indexed_labels().await.contains(label) {
            return Ok(result);
        }
        let idx = format!("{}NamesAndIds", label);
        let pairs: Vec<(String, String)> = names
            .iter()
            .map(|n| (n.clone(), lucene_sanitize(n)))
            .filter(|(_, q)| !q.is_empty())
            .collect();

        // run one chunk -> (rows, limit); errors degrade to empty (parity with the Python try/except).
        async fn run(
            this: &Client,
            idx: &str,
            chunk: &[(String, String)],
            per: usize,
        ) -> (Vec<Value>, usize) {
            let limit = chunk.len() * (per + 15);
            let tail = format!("RETURN pair[0] AS q, node.identifier AS id, node.name AS name, node.synonyms AS syn, score ORDER BY q, score DESC LIMIT {}", limit);
            let cache_key = format!("UNWIND {} AS pair CALL db.index.fulltext.queryNodes('{}', pair[1]) YIELD node, score {}", cypher_pairs(chunk), idx, tail);
            let cy = format!("UNWIND $pairs AS pair CALL db.index.fulltext.queryNodes('{}', pair[1]) YIELD node, score {}", idx, tail);
            let pairs: Vec<Vec<String>> = chunk.iter().map(|(a, b)| vec![a.clone(), b.clone()]).collect();
            let qq = neo4rs::query(&cy).param("pairs", pairs);
            match this.run_cached(&cache_key, qq).await {
                Ok(rows) => (rows, limit),
                Err(_) => (vec![], limit),
            }
        }

        let absorb = |result: &mut HashMap<String, Vec<FtHit>>, rows: &[Value]| {
            for r in rows {
                if let Some(q) = r_str(r, "q") {
                    let lst = result.entry(q).or_default();
                    if lst.len() < per {
                        lst.push(FtHit {
                            id: r_id(r),
                            name: r_str(r, "name"),
                            syn: r_vec(r, "syn"),
                            score: r_f64(r, "score"),
                        });
                    }
                }
            }
        };

        // Run the chunk queries concurrently. Names are partitioned across chunks (each name in
        // exactly one chunk), so a name's candidate ordering is fixed by its own chunk — concurrency
        // does not change per-name results. Absorb in chunk order for full determinism.
        let idx_ref: &str = &idx;
        let chunks: Vec<Vec<(String, String)>> = pairs.chunks(40).map(|c| c.to_vec()).collect();
        let mut init: Vec<(usize, Vec<Value>, usize)> = stream::iter(chunks.iter().enumerate())
            .map(|(ci, chunk)| async move {
                let (rows, limit) = run(self, idx_ref, chunk, per).await;
                (ci, rows, limit)
            })
            .buffer_unordered(CONCURRENCY)
            .collect()
            .await;
        init.sort_by_key(|(ci, _, _)| *ci);
        for (_, rows, _) in &init {
            absorb(&mut result, rows);
        }
        // A name that got zero candidates FROM A SATURATED CHUNK may be starved, not truly empty —
        // re-query it individually (concurrently).
        let mut starved: Vec<(String, String)> = vec![];
        for (ci, rows, limit) in &init {
            if rows.len() >= *limit {
                for (n, q) in &chunks[*ci] {
                    if result.get(n).map(|v| v.is_empty()).unwrap_or(true) {
                        starved.push((n.clone(), q.clone()));
                    }
                }
            }
        }
        let re: Vec<Vec<Value>> = stream::iter(starved.iter())
            .map(|(n, q)| {
                let pair = vec![(n.clone(), q.clone())];
                async move { run(self, idx_ref, &pair, per).await.0 }
            })
            .buffer_unordered(CONCURRENCY)
            .collect()
            .await;
        for rows in &re {
            absorb(&mut result, rows);
        }
        Ok(result)
    }

    // ---- edges ----
    pub async fn edges_between(
        &self,
        label_a: &str,
        id_a: &Value,
        label_b: &str,
        id_b: &Value,
    ) -> Result<Vec<FoundRel>> {
        let mut out = vec![];
        for (la, ia, lb, ib, dir) in [
            (label_a, id_a, label_b, id_b, "a_to_b"),
            (label_b, id_b, label_a, id_a, "b_to_a"),
        ] {
            let tail = "RETURN type(r) AS rt, keys(r) AS keys, r.sources AS sources, r.source AS source LIMIT 25";
            let cache_key = format!(
                "MATCH (a:`{}` {{identifier:{}}})-[r]->(b:`{}` {{identifier:{}}}) {}",
                la, cypher_value(ia), lb, cypher_value(ib), tail
            );
            let cy = format!(
                "MATCH (a:`{}` {{identifier:$ia}})-[r]->(b:`{}` {{identifier:$ib}}) {}",
                la, lb, tail
            );
            let qq = bind_id_param(bind_id_param(neo4rs::query(&cy), "ia", ia), "ib", ib);
            for r in self.run_cached(&cache_key, qq).await? {
                out.push(FoundRel {
                    rt: r_str(&r, "rt").unwrap_or_default(),
                    dir: dir.to_string(),
                    keys: r_vec(&r, "keys").unwrap_or_default(),
                    sources: r.get("sources").cloned().unwrap_or(Value::Null),
                    source: r.get("source").cloned().unwrap_or(Value::Null),
                });
            }
        }
        Ok(out)
    }

    /// Protein recovery by recommended name. SPOKE `Protein.name` is a UniProt mnemonic (AQP1_HUMAN);
    /// the human-readable recommended name lives in `description` ("Aquaporin-1 (AQP-1) ..."), which
    /// fulltext ranks poorly (buried under unreviewed/ortholog hits). This surfaces the ONE reviewed
    /// human protein whose description primary-name (before the first " (") equals the query name.
    /// Uses a per-row CALL subquery (native Bolt supports it) so each name is bounded independently.
    pub async fn batch_protein_by_description(
        &self,
        names: &[String],
    ) -> Result<HashMap<String, Vec<Hit>>> {
        let names = dedup_nonempty(names);
        let mut result: HashMap<String, Vec<Hit>> =
            names.iter().map(|n| (n.clone(), vec![])).collect();
        if names.is_empty() || !self.fulltext_indexed_labels().await.contains("Protein") {
            return Ok(result);
        }
        // AND semantics via +required terms: OR (the default) floods on common tokens like "1" and
        // buries the reviewed-human protein; requiring every term returns a small, precise set.
        let and_pairs: Vec<(String, String)> = names
            .iter()
            .filter_map(|n| {
                let toks: Vec<String> =
                    lucene_sanitize(n).split_whitespace().map(|t| format!("+{}", t)).collect();
                if toks.is_empty() { None } else { Some((n.clone(), toks.join(" "))) }
            })
            .collect();
        let chunks: Vec<Vec<(String, String)>> = and_pairs.chunks(30).map(|c| c.to_vec()).collect();
        // static subquery body (constant, trusted); only the pair values are parameterized.
        let body = "CALL { WITH pair \
                       CALL db.index.fulltext.queryNodes('ProteinNamesAndIds', pair[1]) YIELD node, score \
                       WITH node, score WHERE node.reviewed STARTS WITH 'Reviewed' \
                         AND node.org_name STARTS WITH 'Homo sapiens' \
                       RETURN node LIMIT 15 } \
                     RETURN pair[0] AS q, node.identifier AS id, node.name AS name, \
                            node.description AS descr, node.gene AS gene";
        let rowsets: Vec<Result<Vec<Value>>> = stream::iter(chunks.iter())
            .map(|chunk| {
                let cache_key = format!("UNWIND {} AS pair {}", cypher_pairs(chunk), body);
                let cy = format!("UNWIND $pairs AS pair {}", body);
                let pairs: Vec<Vec<String>> = chunk.iter().map(|(a, b)| vec![a.clone(), b.clone()]).collect();
                let qq = neo4rs::query(&cy).param("pairs", pairs);
                async move { self.run_cached(&cache_key, qq).await }
            })
            .buffer_unordered(CONCURRENCY)
            .collect()
            .await;
        for rs in rowsets {
            for r in rs? {
                let q = match r_str(&r, "q") {
                    Some(q) => q,
                    None => continue,
                };
                let descr = r_str(&r, "descr");
                let gene = r.get("gene").cloned().unwrap_or(Value::Null);
                // Match app-side: the BioOKF name equals a description name-segment (norm_key) or the
                // protein's gene symbol — either identifies this exact protein.
                if protein_identity_matches(&q, descr.as_deref(), &gene) {
                    result.entry(q).or_default().push(Hit {
                        id: r_id(&r),
                        name: r_str(&r, "name"),
                        syn: None,
                        descr,
                    });
                }
            }
        }
        Ok(result)
    }
}

/// Does the BioOKF name identify this SPOKE protein? True when it equals (norm_key) any name segment
/// of the description ("RecName (AltName1) (AltName2) ...") or the gene symbol. High precision: every
/// description parenthetical is an alternative name of the SAME protein.
fn protein_identity_matches(q: &str, descr: Option<&str>, gene: &Value) -> bool {
    let qk = norm_key(q);
    if qk.is_empty() {
        return false;
    }
    let qt = q.trim();
    if qt.len() >= 2 {
        let gene_hit = match gene {
            Value::String(g) => g.eq_ignore_ascii_case(qt),
            Value::Array(a) => a
                .iter()
                .any(|x| x.as_str().map(|s| s.eq_ignore_ascii_case(qt)).unwrap_or(false)),
            _ => false,
        };
        if gene_hit {
            return true;
        }
    }
    if let Some(d) = descr {
        // ONLY the recommended (primary) name — the text before the first " (". UniProt lists cleaved
        // products and alt-names in later parentheticals; matching those maps a product to its
        // precursor (e.g. "Angiotensin II" -> angiotensinogen P01019), so we deliberately don't.
        let primary = d.split(" (").next().unwrap_or(d);
        if norm_key(primary) == qk {
            return true;
        }
    }
    false
}

/// Case variants of a name for case-insensitive index-backed lookup: the name itself, all-lower,
/// all-upper, title-case, capitalize-first — deduped, order preserved.
pub fn name_case_variants(n: &str) -> Vec<String> {
    let mut vs = vec![
        n.to_string(),
        n.to_lowercase(),
        n.to_uppercase(),
        py_title(n),
        py_capitalize_first(n),
    ];
    vs.retain(|v| !v.is_empty());
    let mut seen = HashSet::new();
    vs.retain(|v| seen.insert(v.clone()));
    vs
}

/// Port of Python str.title(): a letter starts a word (uppercase) if the previous char is not a
/// letter; other letters lowercase; non-letters unchanged.
fn py_title(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_is_alpha = false;
    for c in s.chars() {
        if c.is_alphabetic() {
            if prev_is_alpha {
                out.extend(c.to_lowercase());
            } else {
                out.extend(c.to_uppercase());
            }
            prev_is_alpha = true;
        } else {
            out.push(c);
            prev_is_alpha = false;
        }
    }
    out
}

/// Port of Python `n[:1].upper() + n[1:]`: uppercase first char, rest verbatim.
fn py_capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => {
            let mut out: String = c.to_uppercase().collect();
            out.push_str(chars.as_str());
            out
        }
        None => String::new(),
    }
}
