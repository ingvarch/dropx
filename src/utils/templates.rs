use crate::utils::constants::{
    ANON_HOURLY_LIMIT, ANON_DAILY_LIMIT,
    ANON_MAX_FILE_SIZE, ANON_RETENTION_DAYS,
    BYTES_PER_MB, MAX_BUFFERED_UPLOAD_SIZE,
};

/// Renders the landing page HTML template with dynamic values.
///
/// Replaces template placeholders with actual values:
/// - {{STYLES}} - Inlined CSS from styles.css
/// - {{HOSTNAME}} - The domain/hostname of the service
/// - {{HOURLY_LIMIT}} - Anonymous hourly upload limit
/// - {{DAILY_LIMIT}} - Anonymous daily upload limit
/// - {{MAX_FILE_SIZE}} - Maximum file size in MB
/// - {{RETENTION_DAYS}} - Default retention period in days
/// - {{MAX_BUFFERED_UPLOAD_SIZE}} - Largest upload without Content-Length, in MB
pub fn render_landing_page(hostname: &str) -> String {
    const TEMPLATE: &str = include_str!("../templates/index.html");
    const STYLES: &str = include_str!("../templates/styles.css");

    let max_file_size_mb = ANON_MAX_FILE_SIZE / BYTES_PER_MB;
    let max_buffered_upload_mb = MAX_BUFFERED_UPLOAD_SIZE / BYTES_PER_MB;

    TEMPLATE
        .replace("{{STYLES}}", STYLES)
        .replace("{{HOSTNAME}}", hostname)
        .replace("{{HOURLY_LIMIT}}", &ANON_HOURLY_LIMIT.to_string())
        .replace("{{DAILY_LIMIT}}", &ANON_DAILY_LIMIT.to_string())
        .replace("{{MAX_FILE_SIZE}}", &max_file_size_mb.to_string())
        .replace("{{RETENTION_DAYS}}", &ANON_RETENTION_DAYS.to_string())
        .replace("{{MAX_BUFFERED_UPLOAD_SIZE}}", &max_buffered_upload_mb.to_string())
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

    #[test]
    fn test_render_landing_page_replaces_all_placeholders() {
        let result = render_landing_page("https://dropx.click");

        assert!(!result.contains("{{STYLES}}"), "STYLES placeholder should be replaced");
        assert!(!result.contains("{{HOSTNAME}}"), "HOSTNAME placeholder should be replaced");
        assert!(!result.contains("{{HOURLY_LIMIT}}"), "HOURLY_LIMIT placeholder should be replaced");
        assert!(!result.contains("{{DAILY_LIMIT}}"), "DAILY_LIMIT placeholder should be replaced");
        assert!(!result.contains("{{MAX_FILE_SIZE}}"), "MAX_FILE_SIZE placeholder should be replaced");
        assert!(!result.contains("{{RETENTION_DAYS}}"), "RETENTION_DAYS placeholder should be replaced");

        let max_file_size_mb = ANON_MAX_FILE_SIZE / BYTES_PER_MB;

        assert!(result.contains(&format!("{} uploads per hour", ANON_HOURLY_LIMIT)));
        assert!(result.contains(&format!("{} uploads per day", ANON_DAILY_LIMIT)));
        assert!(result.contains(&format!("Files up to {} MB", max_file_size_mb)));
        assert!(result.contains(&format!("{} days retention", ANON_RETENTION_DAYS)));

        assert!(result.contains(".hero"), "Styles should be inlined");
        assert!(result.contains("--primary"), "CSS variables should be present");
    }

    #[test]
    fn render_landing_page_leaves_no_placeholder() {
        let result = render_landing_page("https://dropx.click");

        assert!(!result.contains("{{"), "unreplaced placeholder in the landing page");
    }

    #[test]
    fn render_landing_page_takes_the_pipe_upload_limit_from_the_constant() {
        let result = render_landing_page("https://dropx.click");

        let limit_mb = MAX_BUFFERED_UPLOAD_SIZE / BYTES_PER_MB;
        assert!(result.contains(&format!("Files over {} MB", limit_mb)));
        assert!(result.contains(&format!("so it is limited to {} MB", limit_mb)));
    }
}
