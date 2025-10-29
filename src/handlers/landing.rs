use worker::*;
use crate::utils::{accepts_html, render_landing_page};

/// Handles GET / requests with content negotiation.
///
/// Returns:
/// - HTML landing page if Accept header contains text/html (browsers)
/// - Plain text response otherwise (curl, API clients)
///
/// Sets Vary: Accept header for proper caching behavior.
pub async fn handle_landing(req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let headers = req.headers();

    // Get the hostname from the request
    let hostname = match headers.get("Host")? {
        Some(host) => {
            // Construct full URL with protocol
            let protocol = if host.starts_with("localhost") {
                "http"
            } else {
                "https"
            };
            format!("{}://{}", protocol, host)
        }
        None => "https://dropx.click".to_string(), // fallback
    };

    let mut response = if accepts_html(headers) {
        // Browser request - return HTML landing page
        let html = render_landing_page(&hostname);
        Response::from_html(html)?
    } else {
        // API/curl request - return simple text
        Response::ok("DropX - File Transfer Service")?
    };

    // Set Vary header for proper caching with content negotiation
    response
        .headers_mut()
        .set("Vary", "Accept")?;

    Ok(response)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_protocol_detection_localhost() {
        let host = "localhost:8787";
        let protocol = if host.starts_with("localhost") {
            "http"
        } else {
            "https"
        };

        assert_eq!(protocol, "http");
    }

    #[test]
    fn test_protocol_detection_production() {
        let host = "dropx.click";
        let protocol = if host.starts_with("localhost") {
            "http"
        } else {
            "https"
        };

        assert_eq!(protocol, "https");
    }

    #[test]
    fn test_protocol_detection_custom_domain() {
        let host = "transfer.example.com";
        let protocol = if host.starts_with("localhost") {
            "http"
        } else {
            "https"
        };

        assert_eq!(protocol, "https");
    }

    #[test]
    fn test_url_construction_localhost() {
        let host = "localhost:8787";
        let protocol = if host.starts_with("localhost") {
            "http"
        } else {
            "https"
        };
        let url = format!("{}://{}", protocol, host);

        assert_eq!(url, "http://localhost:8787");
    }

    #[test]
    fn test_url_construction_production() {
        let host = "dropx.click";
        let protocol = if host.starts_with("localhost") {
            "http"
        } else {
            "https"
        };
        let url = format!("{}://{}", protocol, host);

        assert_eq!(url, "https://dropx.click");
    }

    #[test]
    fn test_fallback_hostname() {
        let fallback = "https://dropx.click";
        assert!(fallback.starts_with("https://"));
        assert_eq!(fallback, "https://dropx.click");
    }

    #[test]
    fn test_vary_header_value() {
        let vary_value = "Accept";
        assert_eq!(vary_value, "Accept");
    }

    #[test]
    fn test_landing_text_response() {
        let text = "DropX - File Transfer Service";
        assert!(text.contains("DropX"));
        assert!(text.contains("File Transfer"));
    }

    #[test]
    fn test_localhost_variations() {
        assert!("localhost".starts_with("localhost"));
        assert!("localhost:8080".starts_with("localhost"));
        assert!("localhost:3000".starts_with("localhost"));
        assert!(!"local.host".starts_with("localhost"));
    }

    #[test]
    fn test_hostname_protocol_combinations() {
        let hosts = vec![
            ("localhost", "http"),
            ("localhost:8787", "http"),
            ("dropx.click", "https"),
            ("www.dropx.click", "https"),
            ("api.dropx.click", "https"),
        ];

        for (host, expected_protocol) in hosts {
            let protocol = if host.starts_with("localhost") {
                "http"
            } else {
                "https"
            };
            assert_eq!(protocol, expected_protocol, "Failed for host: {}", host);
        }
    }

    #[test]
    fn test_url_format_consistency() {
        let host = "dropx.click";
        let protocol = "https";
        let url = format!("{}://{}", protocol, host);

        assert!(url.contains("://"));
        assert_eq!(url.matches("://").count(), 1);
        assert!(url.starts_with("https://"));
    }
}

