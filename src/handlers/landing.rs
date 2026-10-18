use worker::*;
use crate::utils::{accepts_html, origin_for_host, render_landing_page};
use crate::utils::constants::HEADER_HOST;

/// Handles GET / requests with content negotiation.
///
/// Returns:
/// - HTML landing page if Accept header contains text/html (browsers)
/// - Plain text response otherwise (curl, API clients)
///
/// Sets Vary: Accept header for proper caching behavior.
pub async fn handle_landing(req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let headers = req.headers();

    let hostname = origin_for_host(headers.get(HEADER_HOST)?.as_deref());

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
