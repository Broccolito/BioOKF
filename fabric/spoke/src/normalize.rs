//! Name normalization + Lucene sanitization for BioOKF<->SPOKE matching.
//! Faithful port of normalize.py — normalization builds *comparison keys* and *candidate queries*,
//! never mutates stored names. Two names are an "insensitive exact" match iff norm_key() are equal.

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;
use unicode_normalization::char::is_combining_mark;
use unicode_normalization::UnicodeNormalization;

// Greek letter names <-> symbols (biomedical text mixes "alpha" and "α" freely).
const GREEK: &[(char, &str)] = &[
    ('α', "alpha"), ('Α', "alpha"), ('β', "beta"), ('Β', "beta"), ('γ', "gamma"), ('Γ', "gamma"),
    ('δ', "delta"), ('Δ', "delta"), ('ε', "epsilon"), ('ζ', "zeta"), ('η', "eta"), ('θ', "theta"),
    ('κ', "kappa"), ('λ', "lambda"), ('μ', "mu"), ('µ', "mu"), ('ν', "nu"), ('ξ', "xi"),
    ('π', "pi"), ('ρ', "rho"), ('σ', "sigma"), ('τ', "tau"), ('φ', "phi"), ('Φ', "phi"),
    ('χ', "chi"), ('ψ', "psi"), ('ω', "omega"), ('Ω', "omega"),
];

// Lucene query-syntax special characters that must be neutralized before a fulltext query.
const LUCENE_SPECIAL: &str = "+-&|!(){}[]^\"~*?:\\/<>=";

static RE_QUALIFIER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"\s*\((?:finding|disorder|observable entity|procedure|situation|morphologic abnormality|substance|qualifier value|body structure|product|regime/therapy|environment)\)\s*$",
    )
    .unwrap()
});
static RE_SEP: Lazy<Regex> = Lazy::new(|| Regex::new(r"[\-_/,()\[\]{}]").unwrap());
static RE_KEEP: Lazy<Regex> = Lazy::new(|| Regex::new(r"[^a-z0-9 %+.]").unwrap());
static RE_WS: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());

fn strip_accents(s: &str) -> String {
    s.nfkd().filter(|c| !is_combining_mark(*c)).collect()
}

pub fn expand_greek(s: &str) -> String {
    let mut out = s.to_string();
    for (k, v) in GREEK {
        if out.contains(*k) {
            out = out.replace(*k, v);
        }
    }
    out
}

/// Canonical comparison key: lowercase, greek expanded, accents stripped, punctuation collapsed to
/// single spaces, whitespace collapsed. Order of operations matches normalize.py exactly.
pub fn norm_key(s: &str) -> String {
    let s = expand_greek(s);
    let s = strip_accents(&s);
    let s = s.to_lowercase();
    let s = RE_QUALIFIER.replace(&s, "");
    let s = RE_SEP.replace_all(&s, " ");
    let s = RE_KEEP.replace_all(&s, " ");
    RE_WS.replace_all(&s, " ").trim().to_string()
}

/// Make a string safe as a Lucene fulltext query. Returns "" if nothing usable remains.
pub fn lucene_sanitize(s: &str) -> String {
    let s = expand_greek(s);
    let cleaned: String = s
        .chars()
        .map(|ch| if LUCENE_SPECIAL.contains(ch) { ' ' } else { ch })
        .collect();
    RE_WS.replace_all(&cleaned, " ").trim().to_string()
}

pub fn norm_tokens(s: &str) -> HashSet<String> {
    norm_key(s)
        .split(' ')
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .collect()
}

pub fn jaccard(a: &str, b: &str) -> f64 {
    let ta = norm_tokens(a);
    let tb = norm_tokens(b);
    if ta.is_empty() || tb.is_empty() {
        return 0.0;
    }
    let inter = ta.intersection(&tb).count();
    let uni = ta.union(&tb).count();
    inter as f64 / uni as f64
}
