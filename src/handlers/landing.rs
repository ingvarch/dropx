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
