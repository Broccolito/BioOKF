//! Identifier extraction: DOI, arXiv, PMID, PMCID, ISBN, plus the bioRxiv/medRxiv DOI prefix.
//!
//! Keep the signatures stable; the waterfall (B8) depends on them.

use super::SourceIds;
use regex::Regex;

/// Extract bibliographic identifiers from arbitrary text (URL, filename, or converted body).
pub fn extract(text: &str) -> SourceIds {
    let mut ids = SourceIds::default();

    // DOI: take the first match, lowercase it, trim a single over-captured trailing `.`.
    let doi_re = Regex::new(r"(?i)\b10\.\d{4,9}/[-._;()/:a-z0-9]+\b").unwrap();
    if let Some(m) = doi_re.find(text) {
        let mut doi = m.as_str().to_lowercase();
        if doi.ends_with('.') {
            doi.pop();
        }
        ids.doi = Some(doi);
    }

    // arXiv: store the captured id (whichever group matched).
    let arxiv_re =
        Regex::new(r"(?i)arxiv:(\d{4}\.\d{4,5})|arxiv\.org/(?:abs|pdf)/(\d{4}\.\d{4,5})").unwrap();
    if let Some(caps) = arxiv_re.captures(text) {
        if let Some(g) = caps.get(1).or_else(|| caps.get(2)) {
            ids.arxiv = Some(g.as_str().to_string());
        }
    }

    // PMID: store captured digits (whichever group matched).
    let pmid_re =
        Regex::new(r"(?i)\bpmid[:\s]\s*(\d{6,9})\b|pubmed\.ncbi\.nlm\.nih\.gov/(\d{6,9})").unwrap();
    if let Some(caps) = pmid_re.captures(text) {
        if let Some(g) = caps.get(1).or_else(|| caps.get(2)) {
            ids.pmid = Some(g.as_str().to_string());
        }
    }

    // PMCID: store as `PMC<digits>`.
    let pmcid_re = Regex::new(r"(?i)\bPMC(\d{5,9})\b").unwrap();
    if let Some(caps) = pmcid_re.captures(text) {
        if let Some(g) = caps.get(1) {
            ids.pmcid = Some(format!("PMC{}", g.as_str()));
        }
    }

    // ISBN: store the raw captured match.
    let isbn_re = Regex::new(
        r"(?i)\bISBN(?:-1[03])?:?\s*(97[89][- ]?(?:\d[- ]?){9}\d|(?:\d[- ]?){9}[\dX])\b",
    )
    .unwrap();
    if let Some(caps) = isbn_re.captures(text) {
        if let Some(g) = caps.get(1) {
            ids.isbn = Some(g.as_str().to_string());
        }
    }

    ids
}

/// True when a DOI is registered under the bioRxiv/medRxiv prefix `10.1101/`.
pub fn is_biorxiv_doi(doi: &str) -> bool {
    doi.starts_with("10.1101/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_identifiers_and_biorxiv() {
        let t = "See https://doi.org/10.1101/2020.01.02.123456 and PMID: 31234567 and arXiv:2003.01234";
        let ids = extract(t);
        assert_eq!(ids.doi.as_deref(), Some("10.1101/2020.01.02.123456"));
        assert_eq!(ids.pmid.as_deref(), Some("31234567"));
        assert_eq!(ids.arxiv.as_deref(), Some("2003.01234"));
        assert!(is_biorxiv_doi(ids.doi.as_deref().unwrap()));
    }
}
