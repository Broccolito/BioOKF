//! Recognized scholarly publishers. Membership only boosts confidence; it never gates the tier.
//!
//! Keep the signature stable; the waterfall (B8) depends on it.

/// Lowercase tokens for recognized scholarly publishers. A publisher name matches when any
/// token appears as a substring of its lowercased form.
const PUBLISHERS: &[&str] = &[
    "elsevier",
    "springer",
    "wiley",
    "ieee",
    "plos",
    "mdpi",
    "oxford university press",
    "cambridge university press",
    "nature",
    "the lancet",
    "lancet",
    "cell press",
    "american chemical society",
    "frontiers",
    "bmj",
    "american medical association",
    "massachusetts medical society",
    "taylor & francis",
    "sage",
    "wolters kluwer",
    "karger",
    "thieme",
    "american physical society",
    "acm",
    "national academy of sciences",
    "pnas",
    "royal society",
    "emerald",
    "de gruyter",
    "annual reviews",
];

/// True when the publisher name matches a recognized scholarly publisher (case-insensitive).
pub fn is_allowlisted(publisher: &str) -> bool {
    let publisher = publisher.to_lowercase();
    PUBLISHERS.iter().any(|token| publisher.contains(token))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowlist_matches_known_publishers() {
        assert!(is_allowlisted("Springer Nature"));
        assert!(is_allowlisted("ELSEVIER BV"));
        assert!(!is_allowlisted("Random Blog Co"));
    }
}
