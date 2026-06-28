//! Crossref resolver: map a Crossref work object to a verdict, and fetch one by DOI.
//!
//! Keep the signatures stable; the waterfall (B8) depends on them.

use super::{CredibilityTier, SourceType};
use serde_json::Value;

/// Map a Crossref `message` object to `(source_type, tier, venue, publisher, retracted)`.
pub fn map_work(v: &Value) -> Option<(SourceType, CredibilityTier, Option<String>, Option<String>, bool)> {
    let work_type = v["type"].as_str()?;

    let (source_type, tier) = match work_type {
        "journal-article" | "proceedings-article" => {
            (SourceType::JournalArticle, CredibilityTier::PeerReviewed)
        }
        "review-article" => (SourceType::Review, CredibilityTier::PeerReviewed),
        "posted-content" => (SourceType::Preprint, CredibilityTier::Preprint),
        "book" | "book-chapter" | "monograph" | "edited-book" => {
            (SourceType::Book, CredibilityTier::PeerReviewed)
        }
        "dataset" => (SourceType::Dataset, CredibilityTier::Archive),
        _ => return None,
    };

    let venue = v["container-title"][0].as_str().map(|s| s.to_string());
    let publisher = v["publisher"].as_str().map(|s| s.to_string());

    let retracted = v["update-to"]
        .as_array()
        .map(|updates| {
            updates
                .iter()
                .any(|u| u["type"].as_str() == Some("retraction"))
        })
        .unwrap_or(false);

    Some((source_type, tier, venue, publisher, retracted))
}

/// Fetch `https://api.crossref.org/works/{doi}` and return its `message` object. Network call;
/// fails soft to `None`.
pub fn fetch(doi: &str) -> Option<Value> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("BioOKF/0.1 (mailto:wanjun.gu@ucsf.edu)")
        .build()
        .ok()?;

    let url = format!("https://api.crossref.org/works/{doi}");
    let resp = client.get(url).send().ok()?;
    let json: Value = resp.json().ok()?;

    json.get("message").map(|m| m.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_crossref_journal_article() {
        let v: serde_json::Value = serde_json::from_str(r#"{
          "type":"journal-article","publisher":"Springer Nature",
          "container-title":["Nature Medicine"],
          "update-to":[{"type":"retraction"}]
        }"#).unwrap();
        let (st, tier, venue, pubr, retracted) = map_work(&v).unwrap();
        assert!(matches!(st, SourceType::JournalArticle));
        assert!(matches!(tier, CredibilityTier::PeerReviewed));
        assert_eq!(venue.as_deref(), Some("Nature Medicine"));
        assert_eq!(pubr.as_deref(), Some("Springer Nature"));
        assert!(retracted);
    }

    #[test]
    #[ignore]
    fn fetch_crossref_live() { assert!(fetch("10.1038/s41591-020-0968-3").is_some()); }
}
