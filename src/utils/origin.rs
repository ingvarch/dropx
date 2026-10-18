const FALLBACK_ORIGIN: &str = "https://dropx.click";
const LOCAL_HOSTNAMES: [&str; 2] = ["localhost", "127.0.0.1"];

/// Public origin for a Host header value: http for local hosts, https otherwise.
pub fn origin_for_host(host: Option<&str>) -> String {
    match host {
        Some(host) if is_local(host) => format!("http://{}", host),
        Some(host) => format!("https://{}", host),
        None => FALLBACK_ORIGIN.to_string(),
    }
}

fn is_local(host: &str) -> bool {
    let hostname = host.split(':').next().unwrap_or(host);
    LOCAL_HOSTNAMES.contains(&hostname)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_hosts_get_http_and_keep_the_port() {
        assert_eq!(origin_for_host(Some("localhost")), "http://localhost");
        assert_eq!(origin_for_host(Some("localhost:8787")), "http://localhost:8787");
        assert_eq!(origin_for_host(Some("127.0.0.1:8787")), "http://127.0.0.1:8787");
    }

    #[test]
    fn other_hosts_get_https() {
        assert_eq!(origin_for_host(Some("dropx.click")), "https://dropx.click");
        assert_eq!(origin_for_host(Some("local.host")), "https://local.host");
        assert_eq!(origin_for_host(Some("localhost.evil.com")), "https://localhost.evil.com");
    }

    #[test]
    fn missing_host_falls_back_to_the_public_origin() {
        assert_eq!(origin_for_host(None), "https://dropx.click");
    }
}
