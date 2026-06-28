//! Pure-Rust PDF text-layer extraction.
//!
//! [`pdf_text`] pulls the embedded text layer out of a born-digital PDF without any LLM. It is
//! deliberately conservative: a scanned / image-only PDF carries no text layer, so extraction
//! yields nothing and we return [`None`]. Callers use that `None` as the signal to fall back to
//! an LLM (e.g. vision OCR).
//!
//! The underlying `pdf_extract` crate is pure Rust (no native/system libraries), but it can
//! `panic!` on some malformed or exotic PDFs, so every call is wrapped in
//! [`std::panic::catch_unwind`]; any error or panic becomes `None` rather than crashing the caller.

/// Extract the text layer of a PDF.
///
/// Returns `Some(text)` when the PDF has a non-empty text layer, and `None` when:
/// - the bytes are not a valid PDF / extraction fails, or
/// - the PDF parses but contains no real text (e.g. a scanned, image-only document) i.e. the
///   extracted string is empty or only whitespace.
///
/// This never panics, even on malformed input.
pub fn pdf_text(bytes: &[u8]) -> Option<String> {
    // `pdf_extract::extract_text_from_mem` can both return `Err` and, on some inputs, panic
    // internally. Catch both. `AssertUnwindSafe` is sound here: we only read the borrowed `bytes`
    // and discard all state on a panic.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        pdf_extract::extract_text_from_mem(bytes)
    }));

    let text = match result {
        Ok(Ok(text)) => text,      // extracted cleanly
        Ok(Err(_)) => return None, // extraction returned an error
        Err(_) => return None,     // extraction panicked
    };

    // Treat empty / whitespace-only output as "no text layer" so the caller falls back to the LLM.
    // (Scanned PDFs land here.)
    if text.trim().is_empty() {
        None
    } else {
        Some(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal born-digital PDF in memory that prints the given text using a built-in
    /// (PDF base-14) font, so it carries a real text layer.
    fn make_text_pdf(text: &str) -> Vec<u8> {
        use printpdf::{
            BuiltinFont, Mm, Op, PdfDocument, PdfFontHandle, PdfPage, PdfSaveOptions, Point, Pt,
            TextItem,
        };

        let ops = vec![
            Op::StartTextSection,
            Op::SetTextCursor {
                pos: Point {
                    x: Pt(50.0),
                    y: Pt(700.0),
                },
            },
            Op::SetFont {
                font: PdfFontHandle::Builtin(BuiltinFont::Helvetica),
                size: Pt(24.0),
            },
            Op::ShowText {
                items: vec![TextItem::Text(text.to_string())],
            },
            Op::EndTextSection,
        ];

        let page = PdfPage::new(Mm(210.0), Mm(297.0), ops);

        let mut doc = PdfDocument::new("BioOKF test");
        doc.with_pages(vec![page]);

        let mut warnings = Vec::new();
        doc.save(&PdfSaveOptions::default(), &mut warnings)
    }

    #[test]
    fn extracts_text_from_born_digital_pdf() {
        let pdf_bytes = make_text_pdf("Hello BioOKF Extraction");
        let extracted = pdf_text(&pdf_bytes).expect("born-digital PDF should yield text");
        assert!(
            extracted.contains("Hello"),
            "expected extracted text to contain \"Hello\", got: {extracted:?}"
        );
        assert!(
            extracted.contains("BioOKF"),
            "expected extracted text to contain \"BioOKF\", got: {extracted:?}"
        );
    }

    #[test]
    fn garbage_input_returns_none_without_panicking() {
        assert!(pdf_text(b"not a pdf at all").is_none());
    }

    #[test]
    fn empty_input_returns_none() {
        assert!(pdf_text(b"").is_none());
    }
}
