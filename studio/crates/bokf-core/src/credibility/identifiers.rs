//! Identifier extraction: DOI, arXiv, PMID, PMCID, ISBN, plus the bioRxiv/medRxiv DOI prefix.
//!
//! STUB: implemented in Task B2. Keep the signatures stable; the waterfall (B8) depends on them.

use super::SourceIds;

/// Extract bibliographic identifiers from arbitrary text (URL, filename, or converted body).
pub fn extract(_text: &str) -> SourceIds {
    SourceIds::default()
}

/// True when a DOI is registered under the bioRxiv/medRxiv prefix `10.1101/`.
pub fn is_biorxiv_doi(_doi: &str) -> bool {
    false
}
