//! Deterministic coverage guardrail for vision-rendered PDFs.
//!
//! After a PDF is rendered to Markdown by LLM vision, this re-extracts the PDF's deterministic text
//! layer (reliable for prose, even though it garbles formulas) purely to check that the rendering
//! did not silently drop large sections. It is never used as the conversion output: the LLM vision
//! rendering is authoritative for text, formulas, and figures. This only flags likely omissions.

use std::collections::HashSet;

/// Fraction (0.0..=1.0) of distinctive content words from the deterministic text that also appear
/// in the rendered Markdown. Returns `None` when there is too little deterministic text to compare
/// against (a scanned PDF, or a very short one), so the caller does not warn in that case.
pub fn coverage_of(deterministic_text: &str, rendered_md: &str) -> Option<f64> {
    // Distinctive content words: ASCII-alphabetic, length >= 5. This skips short stopwords and the
    // garbled-formula tokens (which carry non-ASCII glyphs), so a math-heavy paper is not penalized
    // for formulas the deterministic extractor itself mangled.
    let det_words: HashSet<String> = deterministic_text
        .split(|c: char| !c.is_ascii_alphabetic())
        .filter(|w| w.len() >= 5)
        .map(|w| w.to_ascii_lowercase())
        .collect();
    if det_words.len() < 50 {
        return None;
    }
    let md = rendered_md.to_ascii_lowercase();
    let present = det_words.iter().filter(|w| md.contains(w.as_str())).count();
    Some(present as f64 / det_words.len() as f64)
}

/// Re-extract a PDF's text layer and report how much of it the rendered Markdown covers. Never
/// panics; returns `None` when extraction fails or there is no usable text layer.
pub fn pdf_coverage(pdf_bytes: &[u8], rendered_md: &str) -> Option<f64> {
    let det = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        pdf_extract::extract_text_from_mem(pdf_bytes)
    }))
    .ok()?
    .ok()?;
    coverage_of(&det, rendered_md)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coverage_none_without_text_layer() {
        assert!(pdf_coverage(b"not a pdf", "anything").is_none());
        // too few distinctive words -> no signal
        assert!(coverage_of("short text here", "short text here").is_none());
    }

    #[test]
    fn coverage_detects_dropped_content() {
        // 80 distinct all-alphabetic content words.
        let words: Vec<String> = (0..80u32)
            .map(|i| {
                let a = (b'a' + (i / 26 % 26) as u8) as char;
                let b = (b'a' + (i % 26) as u8) as char;
                format!("word{a}{b}x")
            })
            .collect();
        let det = words.join(" ");
        // A faithful rendering contains all of them.
        assert!(coverage_of(&det, &det).unwrap() > 0.95);
        // A rendering that dropped most sections covers little.
        let partial = words[..20].join(" ");
        let c = coverage_of(&det, &partial).unwrap();
        assert!(c < 0.3, "coverage was {c}");
    }
}
