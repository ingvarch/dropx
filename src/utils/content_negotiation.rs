use worker::Headers;
use crate::utils::constants::{HEADER_ACCEPT, MIME_JSON};

/// How a response that has a text and a JSON form is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseFormat {
    Text,
    Json,
}

impl ResponseFormat {
    /// JSON when the Accept value contains `application/json`, plain text otherwise.
    pub fn from_accept(accept: Option<&str>) -> Self {
        match accept {
            Some(value) if value.to_lowercase().contains(MIME_JSON) => ResponseFormat::Json,
            _ => ResponseFormat::Text,
        }
    }
}

/// Checks if the request accepts HTML content based on the Accept header.
/// This implements HTTP content negotiation following the pattern used by transfer.sh.
///
/// Returns true if the Accept header contains "text/html", false otherwise.
/// If no Accept header is present, returns false (default to API response).
pub fn accepts_html(headers: &Headers) -> bool {
    if let Ok(Some(accept)) = headers.get(HEADER_ACCEPT) {
        // Check if the Accept header contains text/html
        // Accept header format: "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
        accept_value_contains_html(&accept)
    } else {
        false
    }
}

/// Helper function to check if an Accept header value accepts HTML
/// This is the core logic extracted for easier testing
fn accept_value_contains_html(accept_value: &str) -> bool {
    accept_value.to_lowercase().contains("text/html")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accept_value_contains_html() {
        assert!(accept_value_contains_html("text/html"));
        assert!(accept_value_contains_html(
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
        ));
        assert!(accept_value_contains_html("TEXT/HTML"));
        assert!(accept_value_contains_html("application/xhtml+xml,text/html"));
    }

    #[test]
    fn test_accept_value_does_not_contain_html() {
        assert!(!accept_value_contains_html("application/json"));
        assert!(!accept_value_contains_html("application/xml"));
        assert!(!accept_value_contains_html("*/*"));
        assert!(!accept_value_contains_html("text/plain"));
    }

    #[test]
    fn test_accept_value_case_insensitive() {
        assert!(accept_value_contains_html("TEXT/HTML"));
        assert!(accept_value_contains_html("Text/Html"));
        assert!(accept_value_contains_html("text/HTML"));
    }

    #[test]
    fn test_accept_value_with_quality_values() {
        assert!(accept_value_contains_html("text/html;q=0.9"));
        assert!(accept_value_contains_html("application/json,text/html;q=0.8"));
    }

    #[test]
    fn missing_accept_gives_text() {
        assert_eq!(ResponseFormat::from_accept(None), ResponseFormat::Text);
    }

    #[test]
    fn accept_without_json_gives_text() {
        for accept in ["*/*", "text/plain", "text/html", "application/xml", ""] {
            assert_eq!(ResponseFormat::from_accept(Some(accept)), ResponseFormat::Text, "{accept}");
        }
    }

    #[test]
    fn accept_with_json_gives_json() {
        for accept in [
            "application/json",
            "application/json, */*;q=0.5",
            "text/html,application/json;q=0.8",
            "APPLICATION/JSON",
            "Application/Json",
        ] {
            assert_eq!(ResponseFormat::from_accept(Some(accept)), ResponseFormat::Json, "{accept}");
        }
    }
}
