use worker::Headers;

/// Checks if the request accepts HTML content based on the Accept header.
/// This implements HTTP content negotiation following the pattern used by transfer.sh.
///
/// Returns true if the Accept header contains "text/html", false otherwise.
/// If no Accept header is present, returns false (default to API response).
pub fn accepts_html(headers: &Headers) -> bool {
    if let Ok(Some(accept)) = headers.get("Accept") {
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

    // These tests require WASM runtime and are only compiled for wasm32 target
    #[cfg(target_arch = "wasm32")]
    mod wasm_tests {
        use super::*;
        use worker::Headers;

        #[test]
        fn test_accepts_html_with_html_accept() {
            let headers = Headers::new();
            headers
                .set("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
                .unwrap();
            assert!(accepts_html(&headers));
        }

        #[test]
        fn test_accepts_html_with_json_accept() {
            let headers = Headers::new();
            headers.set("Accept", "application/json").unwrap();
            assert!(!accepts_html(&headers));
        }

        #[test]
        fn test_accepts_html_without_accept_header() {
            let headers = Headers::new();
            assert!(!accepts_html(&headers));
        }

        #[test]
        fn test_accepts_html_case_insensitive() {
            let headers = Headers::new();
            headers.set("Accept", "TEXT/HTML").unwrap();
            assert!(accepts_html(&headers));
        }
    }
}
