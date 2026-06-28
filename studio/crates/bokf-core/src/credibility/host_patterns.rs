//! URL host-pattern classification: preprint hosts, gray-literature hosts, generic web.
//!
//! A host-based fallback for the classification waterfall (B8): when no identifier or registry
//! resolution applies, the URL host alone often pins down origin and credibility.

use super::{CredibilityTier, SourceType};

/// Preprint servers. A host matches when it equals an entry or ends with `.<entry>`.
const PREPRINT_HOSTS: &[&str] = &[
    "arxiv.org",
    "biorxiv.org",
    "medrxiv.org",
    "chemrxiv.org",
    "ssrn.com",
    "preprints.org",
    "researchsquare.com",
    "osf.io",
    "psyarxiv.com",
];

/// Named gray-literature hosts (in addition to the `.gov`/`.edu` TLD rules).
const GRAY_LIT_HOSTS: &[&str] = &[
    "who.int",
    "clinicaltrials.gov",
    "europa.eu",
    "nih.gov",
    "cdc.gov",
    "fda.gov",
];

/// Extract the lowercased host from an http(s) URL, or `None` when the input is not an
/// http(s) URL. Strips the scheme and returns everything up to the first `/`.
fn host_of(url: &str) -> Option<String> {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    let host = rest.split('/').next().unwrap_or(rest);
    Some(host.to_ascii_lowercase())
}

/// True when `host` equals `entry` or ends with `.<entry>`.
fn host_matches(host: &str, entry: &str) -> bool {
    host == entry || host.ends_with(&format!(".{entry}"))
}

/// Classify a URL by its host. Returns `(source_type, tier, confidence)`, or `None` when the
/// input is not an http(s) URL.
pub fn classify_url(url: &str) -> Option<(SourceType, CredibilityTier, f32)> {
    let host = host_of(url)?;

    if PREPRINT_HOSTS.iter().any(|e| host_matches(&host, e)) {
        return Some((SourceType::Preprint, CredibilityTier::Preprint, 0.9));
    }

    let is_gray = host.ends_with(".gov")
        || host.ends_with(".edu")
        || GRAY_LIT_HOSTS.iter().any(|e| host_matches(&host, e));
    if is_gray {
        return Some((SourceType::GovReport, CredibilityTier::GrayLit, 0.8));
    }

    Some((SourceType::WebPage, CredibilityTier::Web, 0.6))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_hosts() {
        assert!(matches!(
            classify_url("https://www.biorxiv.org/content/x"),
            Some((SourceType::Preprint, CredibilityTier::Preprint, _))
        ));
        assert!(matches!(
            classify_url("https://www.cdc.gov/x"),
            Some((_, CredibilityTier::GrayLit, _))
        ));
        assert!(matches!(
            classify_url("https://example.com/x"),
            Some((SourceType::WebPage, CredibilityTier::Web, _))
        ));
        assert!(classify_url("not a url").is_none());
    }
}
