//! Conservative scholarly text heuristic, used when no DOI resolves to a registry record.
//!
//! Keep the signature stable; the waterfall (B8) depends on it.

use super::{CredibilityTier, SourceIds, SourceType};

/// Infer a verdict from scholarly markers in the body text. Returns `(source_type, tier,
/// confidence)`, or `None` when the signal is too weak to claim anything.
pub fn scholarly_text_signal(text: &str, ids: &SourceIds) -> Option<(SourceType, CredibilityTier, f32)> {
    let lower = text.to_lowercase();

    const PREPRINT_FINGERPRINTS: &[&str] =
        &["biorxiv", "medrxiv", "arxiv", "chemrxiv", "preprint server"];
    const JOURNAL_MARKERS: &[&str] = &[
        "received:",
        "accepted:",
        "peer-reviewed",
        "corresponding author",
        "doi:",
        "journal of",
        "et al.",
        "abstract",
    ];

    let has_preprint = PREPRINT_FINGERPRINTS.iter().any(|fp| lower.contains(fp));
    let marker_count = JOURNAL_MARKERS.iter().filter(|m| lower.contains(*m)).count();

    if has_preprint && (ids.doi.is_some() || marker_count >= 1) {
        Some((SourceType::Preprint, CredibilityTier::Preprint, 0.7))
    } else if ids.doi.is_some() && marker_count >= 2 {
        Some((SourceType::JournalArticle, CredibilityTier::PeerReviewed, 0.72))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_signal_detects_journal() {
        let ids = SourceIds { doi: Some("10.x/y".into()), ..Default::default() };
        let t = "Received: 1 Jan. Accepted: 2 Feb. Corresponding author: x. Journal of Things.";
        assert!(matches!(scholarly_text_signal(t, &ids), Some((_, CredibilityTier::PeerReviewed, _))));
    }
}
