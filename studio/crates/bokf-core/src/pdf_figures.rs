//! Lossless figure extraction from a PDF.
//!
//! Pulls EMBEDDED raster images out of a PDF the same way `convert.rs` pulls `word/media/*` out of
//! a docx: we walk the PDF's object table with `lopdf`, find every stream whose dict declares
//! `/Subtype /Image`, and emit the underlying image bytes verbatim where the stream filter already
//! IS a standalone image container (JPEG/JPEG2000). This is byte-for-byte lossless: we never
//! re-encode.
//!
//! Filters handled vs skipped:
//! - `/DCTDecode`  (JPEG)      -> emitted as `fig-NNN.jpg`. The raw stream bytes are a complete
//!                                JPEG file, so we copy them out.
//! - `/JPXDecode`  (JPEG2000)  -> emitted as `fig-NNN.jp2`. The raw stream bytes are a complete
//!                                JPEG2000 codestream.
//! - everything else (`/FlateDecode` raw bitmaps, `/CCITTFaxDecode`, `/LZWDecode`,
//!   `/RunLengthDecode`, `/ASCII85Decode`, no filter, ...) is SKIPPED. Those streams hold
//!   raw/packed samples, not a self-describing image file; turning them into a valid PNG/TIFF
//!   would require re-assembling the bitmap from `/Width`, `/Height`, `/ColorSpace`,
//!   `/BitsPerComponent`, `/Decode`, palette and `/SMask` data. Emitting the raw bytes with an
//!   image extension would produce a corrupt file, so we deliberately skip rather than emit
//!   garbage. A downstream LLM-fallback path can still rasterize those.
//!
//! The entry point never panics on malformed input: any parse error (or a panic from inside
//! `lopdf`) is mapped to an empty `Vec`.

use lopdf::{Document, Object};
use std::panic::{catch_unwind, AssertUnwindSafe};

/// Extract every embedded raster image XObject from a PDF.
///
/// Returns `(suggested_name, image_bytes)` for each image we can emit losslessly, numbered
/// `fig-001`, `fig-002`, ... in object order. Filters that do not map to a self-contained image
/// file are skipped (see the module docs). Returns an empty `Vec` on any malformed input and never
/// panics.
pub fn pdf_figures(bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    // `lopdf` parsing walks attacker-controlled bytes; guard against any panic.
    catch_unwind(AssertUnwindSafe(|| extract(bytes))).unwrap_or_default()
}

fn extract(bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let doc = match Document::load_mem(bytes) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    let mut out: Vec<(String, Vec<u8>)> = Vec::new();

    // `objects` is a BTreeMap keyed by ObjectId, so iteration is in stable (object number) order,
    // which gives us deterministic fig-NNN numbering.
    for obj in doc.objects.values() {
        let stream = match obj {
            Object::Stream(s) => s,
            _ => continue,
        };

        // Only image XObjects. `/Subtype` may be a direct name or a reference; resolve through the
        // document, then fall back to the direct lookup.
        let subtype = stream
            .dict
            .get_deref(b"Subtype", &doc)
            .or_else(|_| stream.dict.get(b"Subtype"))
            .ok()
            .and_then(|o| o.as_name().ok());
        if subtype != Some(b"Image".as_slice()) {
            continue;
        }

        // `filters()` returns the filter name(s); `/Filter` may be a single name or an array (a
        // filter chain). For a self-contained image container the image codec filter is the last
        // (outermost-decoded) one.
        let filters = match stream.filters() {
            Ok(f) => f,
            Err(_) => continue, // no /Filter -> raw samples -> skip
        };
        let codec: &[u8] = match filters.last() {
            Some(name) => *name,
            None => continue,
        };

        let ext = match codec {
            b"DCTDecode" => "jpg", // JPEG: raw stream bytes ARE a JPEG file
            b"JPXDecode" => "jp2", // JPEG2000: raw stream bytes ARE a JP2 codestream
            _ => continue,         // FlateDecode/CCITTFax/LZW/... -> not a standalone file -> skip
        };

        if stream.content.is_empty() {
            continue;
        }

        let name = format!("fig-{:03}.{}", out.len() + 1, ext);
        out.push((name, stream.content.clone()));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use printpdf::{
        Mm, Op, PdfDocument, PdfPage, PdfSaveOptions, RawImage, RawImageData, RawImageFormat,
        XObjectTransform,
    };
    use std::io::Read;

    /// Build a PDF in memory that embeds a tiny RGB JPEG image and return its bytes.
    fn make_pdf_with_jpeg() -> Vec<u8> {
        // 2x2 image, 3 bytes/pixel (RGB): red, green, blue, white.
        let pixels = vec![
            255, 0, 0, // red
            0, 255, 0, // green
            0, 0, 255, // blue
            255, 255, 255, // white
        ];
        let image = RawImage {
            pixels: RawImageData::U8(pixels),
            width: 2,
            height: 2,
            data_format: RawImageFormat::RGB8,
            tag: Vec::new(),
        };

        let mut doc = PdfDocument::new("pdf_figures_test");
        let image_id = doc.add_image(&image);
        let page = PdfPage::new(
            Mm(50.0),
            Mm(50.0),
            vec![Op::UseXobject {
                id: image_id,
                transform: XObjectTransform::default(),
            }],
        );
        doc.with_pages(vec![page]);

        let mut warnings = Vec::new();
        doc.save(&PdfSaveOptions::default(), &mut warnings)
    }

    #[test]
    fn extracts_embedded_jpeg() {
        let pdf_bytes = make_pdf_with_jpeg();

        // Round-trip through a tempfile to mirror real on-disk ingestion.
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut tmp, &pdf_bytes).unwrap();
        let mut read_back = Vec::new();
        std::fs::File::open(tmp.path())
            .unwrap()
            .read_to_end(&mut read_back)
            .unwrap();

        let figs = pdf_figures(&read_back);
        assert!(!figs.is_empty(), "expected at least one embedded image, got none");

        let (name, data) = &figs[0];
        assert!(!data.is_empty(), "image bytes must be non-empty");
        assert_eq!(name, "fig-001.jpg", "first JPEG figure name");
        // DCTDecode payload must be a real JPEG: starts with the SOI magic 0xFF 0xD8.
        assert_eq!(&data[..2], &[0xFF, 0xD8], "DCTDecode stream must begin with the JPEG magic bytes");
    }

    #[test]
    fn not_a_pdf_returns_empty_without_panicking() {
        let figs = pdf_figures(b"not a pdf");
        assert!(figs.is_empty());
    }

    #[test]
    fn empty_input_returns_empty() {
        assert!(pdf_figures(b"").is_empty());
        assert!(pdf_figures(&[0xFF, 0xD8, 0xFF, 0x00, 0x01]).is_empty());
    }
}
