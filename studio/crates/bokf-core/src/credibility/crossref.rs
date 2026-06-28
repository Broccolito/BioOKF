//! Crossref resolver: map a Crossref work object to a verdict, and fetch one by DOI.
//!
//! STUB: implemented in Task B5. Keep the signatures stable; the waterfall (B8) depends on them.

use super::{CredibilityTier, SourceType};
use serde_json::Value;

/// Map a Crossref `message` object to `(source_type, tier, venue, publisher, retracted)`.
pub fn map_work(_v: &Value) -> Option<(SourceType, CredibilityTier, Option<String>, Option<String>, bool)> {
    None
}

/// Fetch `https://api.crossref.org/works/{doi}` and return its `message` object. Network call;
/// fails soft to `None`.
pub fn fetch(_doi: &str) -> Option<Value> {
    None
}
