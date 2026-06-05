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

    Ok(trimmed.trim_end_matches('/').to_string())
}

pub(crate) fn normalize_relative_path(value: Option<&str>, default: &str) -> Result<String, String> {
    let path = value
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .unwrap_or(default);
    if path.is_empty() || path.chars().any(|ch| ch.is_control()) {
        return Err("Invalid path".to_string());
    }
    let lower = path.to_ascii_lowercase();
    if lower.starts_with("http://")
        || lower.starts_with("https://")
        || path.starts_with("//")
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

fn is_private_or_reserved_host(host: &str) -> bool {
    let normalized = host.trim_matches(|ch| ch == '[' || ch == ']' || ch == '.');
    if normalized.eq_ignore_ascii_case("localhost")
        || normalized.to_ascii_lowercase().ends_with(".localhost")
    {
        return true;
    }

    match normalized.parse::<IpAddr>() {
        Ok(IpAddr::V4(ip)) => {
            ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || ip.is_broadcast()
                || is_ipv4_documentation(ip)
                || ip.is_unspecified()
        }
        Ok(IpAddr::V6(ip)) => {
            ip.is_loopback()
                || ip.is_unspecified()
                || ip.is_unique_local()
                || ip.is_unicast_link_local()
                || is_ipv6_documentation(ip)
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
