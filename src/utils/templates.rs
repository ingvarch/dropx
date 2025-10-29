/// Renders the landing page HTML template with dynamic values.
///
/// Replaces template placeholders with actual values:
/// - {{HOSTNAME}} - The domain/hostname of the service
pub fn render_landing_page(hostname: &str) -> String {
    const TEMPLATE: &str = include_str!("../templates/index.html");

    TEMPLATE.replace("{{HOSTNAME}}", hostname)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_landing_page_replaces_hostname() {
        let result = render_landing_page("https://dropx.click");
        assert!(result.contains("https://dropx.click"));
        assert!(!result.contains("{{HOSTNAME}}"));
    }

    #[test]
    fn test_render_landing_page_contains_html_structure() {
        let result = render_landing_page("https://example.com");
        assert!(result.contains("<!DOCTYPE html>"));
        assert!(result.contains("<title>DropX"));
        assert!(result.contains("curl -T"));
    }
}
