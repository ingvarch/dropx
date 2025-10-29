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
        accept.to_lowercase().contains("text/html")
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worker::Headers;

    #[test]
    fn test_accepts_html_with_html_accept() {
        let mut headers = Headers::new();
        headers
            .set("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .unwrap();
        assert!(accepts_html(&headers));
    }

    #[test]
    fn test_accepts_html_with_json_accept() {
        let mut headers = Headers::new();
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
        let mut headers = Headers::new();
        headers.set("Accept", "TEXT/HTML").unwrap();
        assert!(accepts_html(&headers));
    }
}
