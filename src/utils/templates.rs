const ANON_HOURLY_LIMIT: &str = env!("ANON_HOURLY_LIMIT");
const ANON_DAILY_LIMIT: &str = env!("ANON_DAILY_LIMIT");
const MAX_FILE_SIZE_MB: &str = env!("MAX_FILE_SIZE_MB");
const DEFAULT_RETENTION_DAYS: &str = env!("DEFAULT_RETENTION_DAYS");

/// Renders the landing page HTML template with dynamic values.
///
/// Replaces template placeholders with actual values:
/// - {{HOSTNAME}} - The domain/hostname of the service
/// - {{HOURLY_LIMIT}} - Anonymous hourly upload limit
/// - {{DAILY_LIMIT}} - Anonymous daily upload limit
/// - {{MAX_FILE_SIZE}} - Maximum file size in MB
/// - {{RETENTION_DAYS}} - Default retention period in days
pub fn render_landing_page(hostname: &str) -> String {
    const TEMPLATE: &str = include_str!("../templates/index.html");

    TEMPLATE
        .replace("{{HOSTNAME}}", hostname)
        .replace("{{HOURLY_LIMIT}}", ANON_HOURLY_LIMIT)
        .replace("{{DAILY_LIMIT}}", ANON_DAILY_LIMIT)
        .replace("{{MAX_FILE_SIZE}}", MAX_FILE_SIZE_MB)
        .replace("{{RETENTION_DAYS}}", DEFAULT_RETENTION_DAYS)
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

        assert!(!result.contains("{{HOSTNAME}}"), "HOSTNAME placeholder should be replaced");
        assert!(!result.contains("{{HOURLY_LIMIT}}"), "HOURLY_LIMIT placeholder should be replaced");
        assert!(!result.contains("{{DAILY_LIMIT}}"), "DAILY_LIMIT placeholder should be replaced");
        assert!(!result.contains("{{MAX_FILE_SIZE}}"), "MAX_FILE_SIZE placeholder should be replaced");
        assert!(!result.contains("{{RETENTION_DAYS}}"), "RETENTION_DAYS placeholder should be replaced");

        assert!(result.contains(&format!("{} uploads per hour", ANON_HOURLY_LIMIT)));
        assert!(result.contains(&format!("{} uploads per day", ANON_DAILY_LIMIT)));
        assert!(result.contains(&format!("Files up to {} MB", MAX_FILE_SIZE_MB)));
        assert!(result.contains(&format!("{} days retention", DEFAULT_RETENTION_DAYS)));
    }
}
