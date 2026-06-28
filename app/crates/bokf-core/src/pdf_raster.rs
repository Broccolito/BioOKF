//! PDF page rasterizer: renders each page of a PDF to a PNG image so the LLM can *read* the page
//! with vision.
//!
//! This is deliberately **not** text or figure extraction: it does no OCR, no layout parsing, and
//! no semantic interpretation. It simply turns each page into a faithful raster image; the LLM
//! downstream is what actually reads and interprets the content. Think of it as "screenshot every
//! page" so a vision-capable model can look at the original typesetting, tables, equations, and
//! figures exactly as a human would.
//!
//! Rendering depends on the native PDFium library being present at runtime. That library is large
//! and platform-specific, so it is **not** bundled with the crate. When it is absent (the common
//! case in CI and in a fresh checkout), this module degrades gracefully: every entry point returns
//! an empty `Vec` and never panics. Callers treat an empty result as "rasterization unavailable",
//! and the agent reads `original.pdf` directly instead.
//!
//! ## Enabling rasterization at runtime
//!
//! Provide a PDFium dynamic library via one of (tried in this order):
//!   1. `libpdfium.<ext>` in the process working directory (`./`).
//!   2. The `PDFIUM_DYNAMIC_LIB_PATH` environment variable (the library file, or a directory
//!      containing it).
//!   3. The system library search path (e.g. `/usr/local/lib`).
//!
//! Prebuilt binaries: <https://github.com/bblanchon/pdfium-binaries>.

use pdfium_render::prelude::*;
use std::path::PathBuf;
use std::time::Duration;

/// Target render width in pixels. Legible for a vision model while keeping each JPEG small.
const TARGET_WIDTH_PX: i32 = 1200;

/// JPEG quality for page images. Pages are rendered raster (not the committed truth), so a lossy
/// encode at this quality keeps them readable for vision at roughly a tenth the size of PNG.
const JPEG_QUALITY: u8 = 85;

/// Rasterize up to `max_pages` pages of a PDF into PNG images, one `(name, png_bytes)` per page
/// (`page-001.png`, `page-002.png`, ...). Never panics and never errors: on any failure (PDFium
/// library not found, load error, render error) it returns an empty `Vec`.
pub fn pdf_rasterize_pages(bytes: &[u8], max_pages: usize) -> Vec<(String, Vec<u8>)> {
    // FFI into a C library: guard against any unexpected panic so a malformed PDF or a misbehaving
    // PDFium build can never take down the host process.
    std::panic::catch_unwind(|| rasterize_inner(bytes, max_pages)).unwrap_or_default()
}

/// The fallible core. Returns an empty `Vec` for every failure mode rather than an `Err`, so the
/// public wrapper stays trivially total.
fn rasterize_inner(bytes: &[u8], max_pages: usize) -> Vec<(String, Vec<u8>)> {
    if max_pages == 0 || bytes.is_empty() {
        return Vec::new();
    }

    let bindings = match bind_pdfium() {
        Some(b) => b,
        None => return Vec::new(),
    };
    let pdfium = Pdfium::new(bindings);

    let document = match pdfium.load_pdf_from_byte_slice(bytes, None) {
        Ok(doc) => doc,
        Err(_) => return Vec::new(),
    };

    let render_config = PdfRenderConfig::new().set_target_width(TARGET_WIDTH_PX);

    let mut out: Vec<(String, Vec<u8>)> = Vec::new();
    for (index, page) in document.pages().iter().enumerate() {
        if index >= max_pages {
            break;
        }

        // The `DynamicImage` here is the `image` 0.25 type re-exported by pdfium-render, so it
        // lines up with our own `image` dependency for PNG encoding below.
        let dynamic_image = match page
            .render_with_config(&render_config)
            .and_then(|bitmap| bitmap.as_image())
        {
            Ok(img) => img,
            // Skip a page that fails to render rather than aborting the whole document.
            Err(_) => continue,
        };

        let mut jpg_bytes: Vec<u8> = Vec::new();
        let encoder =
            image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpg_bytes, JPEG_QUALITY);
        if dynamic_image.write_with_encoder(encoder).is_err() {
            continue;
        }

        out.push((format!("page-{:03}.jpg", index + 1), jpg_bytes));
    }

    out
}

/// Try the three documented ways of locating the PDFium dynamic library, in order, returning the
/// first set of bindings that loads successfully.
fn bind_pdfium() -> Option<Box<dyn PdfiumLibraryBindings>> {
    // 1. Working-directory `./libpdfium.<ext>`.
    if let Ok(bindings) = Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./")) {
        return Some(bindings);
    }

    // 2. Explicit override via `PDFIUM_DYNAMIC_LIB_PATH`.
    if let Ok(raw) = std::env::var("PDFIUM_DYNAMIC_LIB_PATH") {
        if !raw.trim().is_empty() {
            let path = PathBuf::from(&raw);
            if let Ok(bindings) = Pdfium::bind_to_library(&path) {
                return Some(bindings);
            }
            if path.is_dir() {
                if let Ok(bindings) =
                    Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(&path))
                {
                    return Some(bindings);
                }
            }
        }
    }

    // 3. The BioOKF auto-install directory (`bokf install-pdfium` puts the library here).
    if let Some(dir) = default_pdfium_dir() {
        if let Ok(bindings) =
            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(&dir))
        {
            return Some(bindings);
        }
    }

    // 4. System library search path.
    Pdfium::bind_to_system_library().ok()
}

/// True when a PDFium library can be loaded, i.e. page rasterization is enabled.
pub fn is_available() -> bool {
    std::panic::catch_unwind(|| bind_pdfium().is_some()).unwrap_or(false)
}

/// Where `bokf install-pdfium` puts the library, and where [`bind_pdfium`] looks for it:
/// `$BIOOKF_PDFIUM_DIR` if set, otherwise `~/.biookf` (`%USERPROFILE%\.biookf` on Windows).
pub fn default_pdfium_dir() -> Option<PathBuf> {
    if let Ok(d) = std::env::var("BIOOKF_PDFIUM_DIR") {
        if !d.trim().is_empty() {
            return Some(PathBuf::from(d));
        }
    }
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).ok()?;
    Some(PathBuf::from(home).join(".biookf"))
}

/// (release asset, member path inside the archive, output library filename) for this platform,
/// or `None` when there is no prebuilt PDFium for it.
fn pdfium_asset() -> Option<(&'static str, &'static str, &'static str)> {
    use std::env::consts::{ARCH, OS};
    Some(match (OS, ARCH) {
        ("macos", "aarch64") => ("pdfium-mac-arm64.tgz", "lib/libpdfium.dylib", "libpdfium.dylib"),
        ("macos", "x86_64") => ("pdfium-mac-x64.tgz", "lib/libpdfium.dylib", "libpdfium.dylib"),
        ("linux", "x86_64") => ("pdfium-linux-x64.tgz", "lib/libpdfium.so", "libpdfium.so"),
        ("linux", "aarch64") => ("pdfium-linux-arm64.tgz", "lib/libpdfium.so", "libpdfium.so"),
        ("windows", "x86_64") => ("pdfium-win-x64.tgz", "bin/pdfium.dll", "pdfium.dll"),
        _ => return None,
    })
}

/// Download and install a prebuilt PDFium library so page rasterization works, with no manual
/// steps. Fetches the platform binary from bblanchon/pdfium-binaries into `dir` (default
/// [`default_pdfium_dir`]) and extracts the library there. Returns the installed library path.
pub fn install_pdfium(dir: Option<PathBuf>) -> Result<PathBuf, String> {
    let (asset, member, libname) =
        pdfium_asset().ok_or("no prebuilt PDFium is available for this platform")?;
    let target = dir
        .or_else(default_pdfium_dir)
        .ok_or("could not determine an install directory; set BIOOKF_PDFIUM_DIR")?;
    std::fs::create_dir_all(&target).map_err(|e| e.to_string())?;

    let url =
        format!("https://github.com/bblanchon/pdfium-binaries/releases/latest/download/{asset}");
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(120))
        .user_agent(concat!("BioOKF/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| e.to_string())?;
    let bytes = client
        .get(&url)
        .send()
        .and_then(|r| r.error_for_status())
        .and_then(|r| r.bytes())
        .map_err(|e| format!("download failed: {e}"))?;

    let archive = target.join(asset);
    std::fs::write(&archive, &bytes).map_err(|e| e.to_string())?;
    // Extract just the library, stripping the leading archive directory so it lands directly in
    // `target`. `tar` ships with macOS, Linux, and Windows 10+ and reads .tgz natively.
    let status = std::process::Command::new("tar")
        .arg("-xzf")
        .arg(&archive)
        .arg("-C")
        .arg(&target)
        .arg("--strip-components=1")
        .arg(member)
        .status()
        .map_err(|e| format!("could not run `tar` to extract PDFium: {e}"))?;
    let _ = std::fs::remove_file(&archive);
    if !status.success() {
        return Err("failed to extract the PDFium library (tar)".into());
    }
    let libpath = target.join(libname);
    if !libpath.exists() {
        return Err(format!("PDFium library not found after extraction at {}", libpath.display()));
    }
    Ok(libpath)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Default (non-ignored): must pass with NO PDFium library present, so the normal suite is
    /// green on a fresh checkout. A non-PDF byte slice rasterizes to nothing without panicking.
    #[test]
    fn non_pdf_input_returns_empty_and_does_not_panic() {
        let out = pdf_rasterize_pages(b"not a pdf", 5);
        assert!(out.is_empty(), "expected empty result for non-PDF input, got {} entries", out.len());
    }

    #[test]
    fn degenerate_inputs_return_empty() {
        assert!(pdf_rasterize_pages(b"", 5).is_empty());
        assert!(pdf_rasterize_pages(b"%PDF-1.7 ...", 0).is_empty());
    }

    #[test]
    fn platform_asset_and_install_dir_resolve() {
        // The common desktop/CI targets all have a prebuilt PDFium and a resolvable install dir,
        // so `bokf install-pdfium` has somewhere to fetch from and put the library.
        assert!(pdfium_asset().is_some());
        assert!(default_pdfium_dir().is_some());
    }

    /// Real rasterization. Ignored by default because it needs the native PDFium library. Run with:
    /// `PDFIUM_DYNAMIC_LIB_PATH=/path/to/libpdfium.dylib cargo test -p bokf-core pdf_raster -- --ignored`
    #[test]
    #[ignore = "requires the native PDFium dynamic library at runtime"]
    fn renders_two_page_pdf_to_pngs() {
        use printpdf::*;

        let mut doc = PdfDocument::new("rasterizer-test");
        let page1 = PdfPage::new(Mm(210.0), Mm(297.0), vec![Op::Marker { id: "page-1".to_string() }]);
        let page2 = PdfPage::new(Mm(210.0), Mm(297.0), vec![Op::Marker { id: "page-2".to_string() }]);
        let mut warnings = Vec::new();
        let pdf_bytes: Vec<u8> = doc.with_pages(vec![page1, page2]).save(&PdfSaveOptions::default(), &mut warnings);

        let pages = pdf_rasterize_pages(&pdf_bytes, 5);
        assert_eq!(pages.len(), 2, "expected 2 rasterized pages, got {} (is libpdfium reachable?)", pages.len());

        const JPEG_MAGIC: [u8; 2] = [0xFF, 0xD8];
        for (i, (name, jpg)) in pages.iter().enumerate() {
            assert_eq!(name, &format!("page-{:03}.jpg", i + 1));
            assert!(jpg.len() >= 2 && jpg[0..2] == JPEG_MAGIC, "page {name} does not start with the JPEG magic bytes");
            assert!(jpg.len() > 100, "page {name} JPEG is suspiciously small");
        }
    }
}
