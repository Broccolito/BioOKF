//! OpenAlex resolver (Crossref fallback): map an OpenAlex work to a verdict, and fetch one by DOI.
//!
//! The waterfall (B8) depends on these signatures; keep them stable.

use super::{CredibilityTier, SourceType};
use serde_json::Value;

/// Map an OpenAlex work object to `(source_type, tier, venue, publisher, retracted)`.
///
/// Returns `None` for work types we don't classify.
pub fn map_work(v: &Value) -> Option<(SourceType, CredibilityTier, Option<String>, Option<String>, bool)> {
    let work_type = v.get("type").and_then(Value::as_str)?;

    let (source_type, tier) = match work_type {
        "journal-article" | "article" => (SourceType::JournalArticle, CredibilityTier::PeerReviewed),
        "review" | "review-article" => (SourceType::Review, CredibilityTier::PeerReviewed),
        "posted-content" | "preprint" => (SourceType::Preprint, CredibilityTier::Preprint),
        "book" | "book-chapter" | "monograph" => (SourceType::Book, CredibilityTier::PeerReviewed),
        "dataset" => (SourceType::Dataset, CredibilityTier::Archive),
        _ => return None,
    };

    let venue = v
        .get("host_venue")
        .and_then(|hv| hv.get("display_name"))
        .and_then(Value::as_str)
        .map(str::to_string);

    let publisher = v
        .get("host_venue")
        .and_then(|hv| hv.get("publisher"))
        .and_then(Value::as_str)
        .map(str::to_string);

    let retracted = v.get("is_retracted").and_then(Value::as_bool).unwrap_or(false);

    Some((source_type, tier, venue, publisher, retracted))
}

/// Fetch `https://api.openalex.org/works/doi:{doi}` and return the work object. Network call;
/// fails soft to `None`. OpenAlex returns the work directly (no `message` wrapper).
pub fn fetch(doi: &str) -> Option<Value> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("BioOKF/0.1 (mailto:wanjun.gu@ucsf.edu)")
        .build()
        .ok()?;

    let url = format!("https://api.openalex.org/works/doi:{doi}");
    let resp = client.get(&url).send().ok()?;
    let work: Value = resp.json().ok()?;
    Some(work)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_openalex_posted_content() {
        let v: serde_json::Value = serde_json::from_str(r#"{
          "type":"posted-content","is_retracted":false,
          "host_venue":{"display_name":"bioRxiv","publisher":"Cold Spring Harbor Laboratory"}
        }"#).unwrap();
        let (st, tier, venue, _pubr, retracted) = map_work(&v).unwrap();
        assert!(matches!(st, SourceType::Preprint));
        assert!(matches!(tier, CredibilityTier::Preprint));
        assert_eq!(venue.as_deref(), Some("bioRxiv"));
        assert!(!retracted);
    }

    #[test]
    #[ignore]
    fn fetch_openalex_live() { assert!(fetch("10.1038/s41591-020-0968-3").is_some()); }
}
