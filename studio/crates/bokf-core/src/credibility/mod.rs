//! Source origin and credibility classification.
//!
//! A deterministic-first waterfall (identifiers, then Crossref/OpenAlex resolution, then host
//! patterns, then a conservative text heuristic) decides where a source comes from and how
//! credible it is. `source_type` (origin) and `credibility` (trust) are kept separate.
//! The network resolvers live in `crossref`/`openalex`; everything else is pure and offline.

use serde::{Deserialize, Serialize};

pub mod allowlist;
pub mod crossref;
pub mod host_patterns;
pub mod identifiers;
pub mod openalex;
pub mod text_signal;

/// The kind/origin of a source, distinct from how much it is trusted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    JournalArticle,
    Preprint,
    Review,
    Book,
    Dataset,
    Database,
    ClinicalGuideline,
    GovReport,
    WebPage,
    Personal,
    Unknown,
}

impl Default for SourceType {
    fn default() -> Self {
        SourceType::Unknown
    }
}

impl SourceType {
    /// True when the origin is unknown (used to omit the field from a clean meta.yaml).
    pub fn is_unknown(&self) -> bool {
        matches!(self, SourceType::Unknown)
    }
}

/// Trust tier, an ordered ramp from peer-reviewed down to an unverified web page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredibilityTier {
    PeerReviewed,
    Preprint,
    Archive,
    GrayLit,
    Web,
    Unknown,
}

impl Default for CredibilityTier {
    fn default() -> Self {
        CredibilityTier::Unknown
    }
}

/// A classification verdict for one source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Credibility {
    pub tier: CredibilityTier,
    pub confidence: f32,
    pub retracted: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub venue: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
    pub reasoning: String,
    /// 0 = unset, 1 = deterministic, 2 = agentic.
    pub classifier_version: u32,
}

impl Default for Credibility {
    fn default() -> Self {
        Credibility {
            tier: CredibilityTier::Unknown,
            confidence: 0.3,
            retracted: false,
            venue: None,
            publisher: None,
            reasoning: String::new(),
            classifier_version: 0,
        }
    }
}

impl Credibility {
    /// True when no classifier has run yet (used to omit the field from a clean meta.yaml).
    pub fn is_unset(&self) -> bool {
        self.classifier_version == 0 && matches!(self.tier, CredibilityTier::Unknown)
    }
}

/// Bibliographic identifiers extracted from a source.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SourceIds {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pmid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pmcid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arxiv: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub isbn: Option<String>,
}

impl SourceIds {
    /// True when no identifier was found (used to omit the field from a clean meta.yaml).
    pub fn is_empty(&self) -> bool {
        self.doi.is_none()
            && self.pmid.is_none()
            && self.pmcid.is_none()
            && self.arxiv.is_none()
            && self.isbn.is_none()
    }
}

/// Input to the classification waterfall. `online` gates the Crossref/OpenAlex network calls.
pub struct ClassifyInput<'a> {
    pub url: Option<&'a str>,
    pub filename: Option<&'a str>,
    pub body: &'a str,
    pub online: bool,
}

/// Classify a source's origin and credibility with a deterministic-first waterfall:
/// identifier extraction, then (when `online`) Crossref/OpenAlex DOI resolution, then URL host
/// patterns, then a conservative text heuristic, then a default. Every branch here is
/// deterministic and sets `classifier_version = 1`; an agentic fallback (version 2) is layered on
/// by the calling skill when nothing deterministic fires.
pub fn classify(input: &ClassifyInput) -> (SourceType, Credibility, SourceIds) {
    let probe = format!(
        "{} {} {}",
        input.url.unwrap_or(""),
        input.filename.unwrap_or(""),
        input.body
    );
    let ids = identifiers::extract(&probe);

    // 1. DOI resolution against the registries (online only). Crossref first, OpenAlex fallback.
    if input.online {
        if let Some(doi) = ids.doi.as_deref() {
            let resolved = crossref::fetch(doi)
                .and_then(|m| crossref::map_work(&m))
                .or_else(|| openalex::fetch(doi).and_then(|m| openalex::map_work(&m)));
            if let Some((st, tier, venue, publisher, retracted)) = resolved {
                let allowlisted = publisher.as_deref().map(allowlist::is_allowlisted).unwrap_or(false);
                let cred = Credibility {
                    tier,
                    confidence: if allowlisted { 0.95 } else { 0.85 },
                    retracted,
                    venue,
                    publisher,
                    reasoning: format!("registry resolved DOI {doi}"),
                    classifier_version: 1,
                };
                return (st, cred, ids);
            }
        }
    }

    // 2. URL host patterns. A generic-web verdict can be upgraded by a DOI plus scholarly text.
    if let Some(url) = input.url {
        if let Some((st, tier, conf)) = host_patterns::classify_url(url) {
            if matches!(tier, CredibilityTier::Web) {
                if let Some((st2, tier2, conf2)) = text_signal::scholarly_text_signal(input.body, &ids) {
                    let cred = Credibility {
                        tier: tier2,
                        confidence: conf2,
                        retracted: false,
                        venue: None,
                        publisher: None,
                        reasoning: "scholarly text over a generic web host".into(),
                        classifier_version: 1,
                    };
                    return (st2, cred, ids);
                }
            }
            let cred = Credibility {
                tier,
                confidence: conf,
                retracted: false,
                venue: None,
                publisher: None,
                reasoning: format!("host pattern for {url}"),
                classifier_version: 1,
            };
            return (st, cred, ids);
        }
    }

    // 3. Conservative scholarly text heuristic (files / pasted text with no resolvable DOI).
    if let Some((st, tier, conf)) = text_signal::scholarly_text_signal(input.body, &ids) {
        let cred = Credibility {
            tier,
            confidence: conf,
            retracted: false,
            venue: None,
            publisher: None,
            reasoning: "scholarly text signal".into(),
            classifier_version: 1,
        };
        return (st, cred, ids);
    }

    // 4. Default: a web page when there is a URL/file, otherwise unattributed personal text.
    let (st, tier, confidence, reasoning) = if input.url.is_some() || input.filename.is_some() {
        (SourceType::WebPage, CredibilityTier::Web, 0.5, "no scholarly signal")
    } else {
        (SourceType::Personal, CredibilityTier::Unknown, 0.3, "no provenance")
    };
    let cred = Credibility {
        tier,
        confidence,
        retracted: false,
        venue: None,
        publisher: None,
        reasoning: reasoning.into(),
        classifier_version: 1,
    };
    (st, cred, ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn types_roundtrip_snake_case() {
        let c = Credibility {
            tier: CredibilityTier::PeerReviewed,
            confidence: 0.9,
            retracted: false,
            venue: Some("Nature".into()),
            publisher: Some("Springer Nature".into()),
            reasoning: "crossref journal-article".into(),
            classifier_version: 1,
        };
        let y = serde_yaml::to_string(&c).unwrap();
        assert!(y.contains("tier: peer_reviewed"));
        let back: Credibility = serde_yaml::from_str(&y).unwrap();
        assert_eq!(c, back);
    }

    #[test]
    fn classify_offline_uses_host_then_text() {
        let inp = ClassifyInput {
            url: Some("https://www.medrxiv.org/content/10.1101/2021.01.01.21249000v1"),
            filename: None,
            body: "",
            online: false,
        };
        let (st, cred, ids) = classify(&inp);
        assert!(matches!(st, SourceType::Preprint));
        assert!(matches!(cred.tier, CredibilityTier::Preprint));
        assert!(ids.doi.is_some());
        assert_eq!(cred.classifier_version, 1);
    }
}
