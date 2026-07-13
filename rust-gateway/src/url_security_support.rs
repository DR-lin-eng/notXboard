use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use url::Url;

pub(crate) fn normalize_http_url(value: &str, block_private_hosts: bool) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().any(|ch| ch.is_control()) {
        return Err("Invalid URL".to_string());
    }

    let parsed = Url::parse(trimmed).map_err(|_| "Invalid URL".to_string())?;
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Err("URL must use http or https".to_string());
    }
    let Some(host) = parsed.host_str() else {
        return Err("URL must include a host".to_string());
    };
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err("URL userinfo is not allowed".to_string());
    }
    if block_private_hosts && is_private_or_reserved_host(host) {
        return Err("Private or reserved hosts are not allowed".to_string());
    }

    Ok(parsed.to_string().trim_end_matches('/').to_string())
}

pub(crate) fn normalize_relative_path(value: Option<&str>, default: &str) -> Result<String, String> {
    let path = value
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .unwrap_or(default);
    if path.is_empty()
        || path
            .chars()
            .any(|ch| ch.is_control() || ch.is_whitespace() || matches!(ch, '"' | '<' | '>' | '`'))
    {
        return Err("Invalid path".to_string());
    }
    let lower = path.to_ascii_lowercase();
    if lower.starts_with("http://")
        || lower.starts_with("https://")
        || path.starts_with("//")
        || path.contains(':')
        || path.contains('\\')
        || path.contains('?')
        || path.contains('#')
    {
        return Err("Path must be relative to the payment gateway host".to_string());
    }

    Ok(if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    })
}

pub(crate) fn join_http_url_path(
    base_url: &str,
    path: Option<&str>,
    default_path: &str,
    block_private_hosts: bool,
) -> Result<String, String> {
    let base_url = normalize_http_url(base_url, block_private_hosts)?;
    let path = normalize_relative_path(path, default_path)?;
    let mut base = Url::parse(&base_url).map_err(|_| "Invalid URL".to_string())?;
    base.set_query(None);
    base.set_fragment(None);
    if !base.path().ends_with('/') {
        let directory_path = format!("{}/", base.path());
        base.set_path(&directory_path);
    }
    let base_path = base.path().to_string();
    let joined = base
        .join(path.trim_start_matches('/'))
        .map_err(|_| "Invalid URL path".to_string())?;
    if joined.origin() != base.origin()
        || !matches!(joined.scheme(), "http" | "https")
        || !joined.path().starts_with(&base_path)
    {
        return Err("Path must remain on the payment gateway host".to_string());
    }
    Ok(joined.to_string())
}

pub(crate) fn is_private_or_reserved_host(host: &str) -> bool {
    let normalized = host.trim_matches(|ch| ch == '[' || ch == ']' || ch == '.');
    if normalized.eq_ignore_ascii_case("localhost")
        || normalized.to_ascii_lowercase().ends_with(".localhost")
    {
        return true;
    }

    match normalized.parse::<IpAddr>() {
        Ok(IpAddr::V4(ip)) => {
            let octets = ip.octets();
            ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || ip.is_broadcast()
                || ip.is_multicast()
                || is_ipv4_documentation(ip)
                || ip.is_unspecified()
                || octets[0] == 0
                || (octets[0] == 100 && (64..=127).contains(&octets[1]))
                || (octets[0] == 198 && matches!(octets[1], 18 | 19))
                || (octets[0] == 192 && octets[1] == 0 && octets[2] == 0)
                || octets[0] >= 240
        }
        Ok(IpAddr::V6(ip)) => {
            let segments = ip.segments();
            ip.is_loopback()
                || ip.is_unspecified()
                || ip.is_unique_local()
                || ip.is_unicast_link_local()
                || ip.is_multicast()
                || is_ipv6_documentation(ip)
                || (segments[0] & 0xffc0) == 0xfec0
                || (segments[0] == 0x0064 && segments[1] == 0xff9b)
                || (segments[0] == 0x2001 && segments[1] < 0x0200)
                || segments[0] == 0x2002
        }
        Err(_) => false,
    }
}

fn is_ipv4_documentation(ip: Ipv4Addr) -> bool {
    let octets = ip.octets();
    matches!(
        octets,
        [192, 0, 2, _] | [198, 51, 100, _] | [203, 0, 113, _]
    )
}

fn is_ipv6_documentation(ip: Ipv6Addr) -> bool {
    let segments = ip.segments();
    segments[0] == 0x2001 && segments[1] == 0x0db8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_payment_paths_reject_active_or_cross_origin_inputs() {
        for path in [
            "https://evil.example/submit",
            "//evil.example/submit",
            "javascript:alert(1)",
            "/pay/\"/><script>alert(1)</script>",
            "/pay/submit php",
            "../submit.php",
        ] {
            assert!(join_http_url_path(
                "https://pay.example.test/gateway/",
                Some(path),
                "/submit.php",
                false,
            )
            .is_err(), "accepted unsafe path: {path}");
        }
    }

    #[test]
    fn payment_path_join_preserves_the_gateway_directory() {
        assert_eq!(
            join_http_url_path(
                "https://credit.linux.do/epay",
                Some("/pay/submit.php"),
                "/fallback.php",
                false,
            )
            .unwrap(),
            "https://credit.linux.do/epay/pay/submit.php"
        );
    }
}
