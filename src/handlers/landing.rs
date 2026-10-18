use worker::*;
use crate::utils::{accepts_html, render_landing_page};

const FALLBACK_ORIGIN: &str = "https://dropx.click";

fn origin_for_host(host: Option<&str>) -> String {
    match host {
        Some(host) if host.starts_with("localhost") => format!("http://{}", host),
        Some(host) => format!("https://{}", host),
        None => FALLBACK_ORIGIN.to_string(),
    }
}

/// Handles GET / requests with content negotiation.
///
/// Returns:
/// - HTML landing page if Accept header contains text/html (browsers)
/// - Plain text response otherwise (curl, API clients)
///
/// Sets Vary: Accept header for proper caching behavior.
pub async fn handle_landing(req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let headers = req.headers();

    let hostname = origin_for_host(headers.get("Host")?.as_deref());

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
    use super::*;

    #[test]
    fn localhost_hosts_get_http() {
        assert_eq!(origin_for_host(Some("localhost")), "http://localhost");
        assert_eq!(origin_for_host(Some("localhost:8787")), "http://localhost:8787");
    }

    #[test]
    fn other_hosts_get_https() {
        assert_eq!(origin_for_host(Some("dropx.click")), "https://dropx.click");
        assert_eq!(origin_for_host(Some("local.host")), "https://local.host");
    }

    #[test]
    fn missing_host_falls_back_to_the_public_origin() {
        assert_eq!(origin_for_host(None), "https://dropx.click");
    }
}
