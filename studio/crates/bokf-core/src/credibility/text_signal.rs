//! Conservative scholarly text heuristic, used when no DOI resolves to a registry record.
//!
//! STUB: implemented in Task B7. Keep the signature stable; the waterfall (B8) depends on it.

use super::{CredibilityTier, SourceIds, SourceType};

/// Infer a verdict from scholarly markers in the body text. Returns `(source_type, tier,
/// confidence)`, or `None` when the signal is too weak to claim anything.
pub fn scholarly_text_signal(_text: &str, _ids: &SourceIds) -> Option<(SourceType, CredibilityTier, f32)> {
    None
}
