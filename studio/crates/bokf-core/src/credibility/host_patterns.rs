//! URL host-pattern classification: preprint hosts, gray-literature hosts, generic web.
//!
//! STUB: implemented in Task B3. Keep the signature stable; the waterfall (B8) depends on it.

use super::{CredibilityTier, SourceType};

/// Classify a URL by its host. Returns `(source_type, tier, confidence)`, or `None` when the
/// input is not an http(s) URL.
pub fn classify_url(_url: &str) -> Option<(SourceType, CredibilityTier, f32)> {
    None
}
