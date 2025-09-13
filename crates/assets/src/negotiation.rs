use crate::encoding::Encoding;
use fluent_langneg::{NegotiationStrategy, negotiate_languages};
use icu_locid::LanguageIdentifier;

/// Negotiates the best encoding from the Accept-Encoding header
pub fn negotiate_encoding(accept_encoding: &str, available_encodings: &[Encoding]) -> Encoding {
    // Parse the Accept-Encoding header
    let mut preferences = Vec::new();

    for part in accept_encoding.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        let (encoding_name, quality) = if let Some((name, q_part)) = part.split_once(';') {
            let quality = parse_quality(q_part.trim()).unwrap_or(1.0);
            (name.trim(), quality)
        } else {
            (part, 1.0)
        };

        // Skip zero-quality encodings
        if quality <= 0.0 {
            continue;
        }

        preferences.push((encoding_name, quality));
    }

    // Sort by quality (highest first)
    preferences.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Find the first matching encoding
    for (encoding_name, _) in preferences {
        if encoding_name == "*" {
            // Wildcard - return most preferred available
            return available_encodings
                .iter()
                .min_by_key(|encoding| encoding.preference_order())
                .copied()
                .unwrap_or(Encoding::Identity);
        }
        for &available in available_encodings {
            if matches_encoding(encoding_name, available) {
                return available;
            }
        }
    }

    // Fallback: return the most preferred available encoding
    available_encodings
        .iter()
        .min_by_key(|encoding| encoding.preference_order())
        .copied()
        .unwrap_or(Encoding::Identity)
}

/// Negotiates the best language from the Accept-Language header
pub fn negotiate_language(
    accept_language: &str,
    available_languages: &[LanguageIdentifier],
) -> Option<LanguageIdentifier> {
    if available_languages.is_empty() {
        return None;
    }

    // Parse the Accept-Language header into LanguageIdentifiers
    let requested: Vec<LanguageIdentifier> = accept_language
        .split(',')
        .filter_map(|lang_part| {
            let lang_tag = lang_part.split(';').next()?.trim();
            lang_tag.parse().ok()
        })
        .collect();

    if requested.is_empty() {
        return None;
    }

    // Use fluent-langneg for proper language negotiation
    let supported: Vec<&LanguageIdentifier> = available_languages.iter().collect();
    let _default_language = &available_languages[0]; // First available as default

    let result = negotiate_languages(
        &requested,
        &supported,
        None, // No default language for strict matching
        NegotiationStrategy::Filtering,
    );

    result.into_iter().next().cloned().cloned()
}

/// Parses a quality value from a q-parameter (e.g., "q=0.8")
fn parse_quality(q_part: &str) -> Option<f32> {
    if let Some(q_value) = q_part.strip_prefix("q=") {
        q_value.parse().ok()
    } else {
        None
    }
}

/// Checks if an encoding name from the Accept-Encoding header matches an available encoding
fn matches_encoding(encoding_name: &str, available: Encoding) -> bool {
    match encoding_name.to_lowercase().as_str() {
        "br" => available == Encoding::Brotli,
        "brotli" => available == Encoding::Brotli,
        "gzip" => available == Encoding::Gzip,
        "deflate" => available == Encoding::Gzip, // Treat deflate as gzip
        "identity" => available == Encoding::Identity,
        "*" => true, // Wildcard matches any encoding
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icu_locid::langid;

    #[test]
    fn test_negotiate_encoding_basic() {
        let available = [Encoding::Identity, Encoding::Gzip, Encoding::Brotli];

        assert_eq!(negotiate_encoding("br", &available), Encoding::Brotli);

        assert_eq!(negotiate_encoding("gzip", &available), Encoding::Gzip);

        assert_eq!(
            negotiate_encoding("identity", &available),
            Encoding::Identity
        );
    }

    #[test]
    fn test_negotiate_encoding_with_quality() {
        let available = [Encoding::Identity, Encoding::Gzip, Encoding::Brotli];

        // Higher quality should win
        assert_eq!(
            negotiate_encoding("gzip; q=0.8, br; q=0.9", &available),
            Encoding::Brotli
        );

        // Default quality is 1.0
        assert_eq!(
            negotiate_encoding("gzip, br; q=0.5", &available),
            Encoding::Gzip
        );
    }

    #[test]
    fn test_negotiate_encoding_fallback() {
        let available = [Encoding::Gzip, Encoding::Identity];

        // Request brotli but it's not available, should fallback to most preferred
        assert_eq!(
            negotiate_encoding("br", &available),
            Encoding::Gzip // Has preference order 1 vs Identity's 2
        );
    }

    #[test]
    fn test_negotiate_encoding_wildcard() {
        let available = [Encoding::Identity, Encoding::Brotli];

        assert_eq!(
            negotiate_encoding("*", &available),
            Encoding::Brotli // Most preferred
        );
    }

    #[test]
    fn test_negotiate_language_basic() {
        let available = [langid!("en"), langid!("fr"), langid!("de")];

        assert_eq!(negotiate_language("fr", &available), Some(langid!("fr")));

        assert_eq!(
            negotiate_language("es", &available),
            None // No matching language, no fallback in this case
        );
    }

    #[test]
    fn test_negotiate_language_with_region() {
        let available = [langid!("en"), langid!("fr")];

        // en-US should match en
        assert_eq!(negotiate_language("en-US", &available), Some(langid!("en")));
    }

    #[test]
    fn test_negotiate_language_multiple() {
        let available = [langid!("en"), langid!("fr"), langid!("de")];

        // Should prefer first match in requested order
        assert_eq!(
            negotiate_language("es, fr, en", &available),
            Some(langid!("fr"))
        );
    }

    #[test]
    fn test_negotiate_language_empty_available() {
        let available: [LanguageIdentifier; 0] = [];

        assert_eq!(negotiate_language("en", &available), None);
    }

    #[test]
    fn test_parse_quality() {
        assert_eq!(parse_quality("q=0.8"), Some(0.8));
        assert_eq!(parse_quality("q=1.0"), Some(1.0));
        assert_eq!(parse_quality("q=0"), Some(0.0));
        assert_eq!(parse_quality("charset=utf-8"), None);
        assert_eq!(parse_quality("q=invalid"), None);
    }
}
