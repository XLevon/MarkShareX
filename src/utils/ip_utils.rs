use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

fn normalize_ip(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(ipv6) => ipv6
            .to_ipv4_mapped()
            .map(IpAddr::V4)
            .unwrap_or(IpAddr::V6(ipv6)),
        ipv4 => ipv4,
    }
}

fn canonicalize_rule(rule: &str) -> Option<String> {
    let (ip_part, prefix) = match rule.trim().split_once('/') {
        Some((ip, mask)) => (ip, Some(mask.parse::<u8>().ok()?)),
        None => (rule.trim(), None),
    };
    let raw_ip = IpAddr::from_str(ip_part).ok()?;

    match prefix {
        None => Some(normalize_ip(raw_ip).to_string()),
        Some(prefix) => match raw_ip {
            IpAddr::V4(ip) if prefix <= 32 => Some(format!("{ip}/{prefix}")),
            IpAddr::V6(ip) if prefix <= 128 => {
                if let Some(mapped) = ip.to_ipv4_mapped() {
                    if prefix >= 96 {
                        return Some(format!("{mapped}/{}", prefix - 96));
                    }
                }
                Some(format!("{ip}/{prefix}"))
            }
            _ => None,
        },
    }
}

/// 校验 IP 格式（支持纯 IP / CIDR）。
pub fn is_valid_ip(value: &str) -> bool {
    canonicalize_rule(value).is_some()
}

fn ipv4_matches(network: Ipv4Addr, candidate: Ipv4Addr, prefix: u8) -> bool {
    let mask = if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - prefix)
    };
    u32::from(network) & mask == u32::from(candidate) & mask
}

fn ipv6_matches(network: Ipv6Addr, candidate: Ipv6Addr, prefix: u8) -> bool {
    let mask = if prefix == 0 {
        0
    } else {
        u128::MAX << (128 - prefix)
    };
    u128::from(network) & mask == u128::from(candidate) & mask
}

/// 判断客户端 IP 是否命中一个纯 IP 或 CIDR 规则。
pub fn ip_matches_rule(rule: &str, client_ip: &str) -> bool {
    let Some(rule) = canonicalize_rule(rule) else {
        return false;
    };
    let Some(candidate) = IpAddr::from_str(client_ip).ok().map(normalize_ip) else {
        return false;
    };

    let Some((network, prefix)) = rule.split_once('/') else {
        return IpAddr::from_str(&rule).ok().map(normalize_ip) == Some(candidate);
    };
    let Ok(prefix) = prefix.parse::<u8>() else {
        return false;
    };
    let Ok(network) = IpAddr::from_str(network) else {
        return false;
    };

    match (network, candidate) {
        (IpAddr::V4(network), IpAddr::V4(candidate)) => ipv4_matches(network, candidate, prefix),
        (IpAddr::V6(network), IpAddr::V6(candidate)) => ipv6_matches(network, candidate, prefix),
        (IpAddr::V6(network), IpAddr::V4(candidate)) => {
            ipv6_matches(network, candidate.to_ipv6_mapped(), prefix)
        }
        (IpAddr::V4(_), IpAddr::V6(_)) => false,
    }
}

/// 解析 IP 列表（兼容旧格式 string[] 和新格式 {ip,remark}[]），过滤无效 IP并规范化。
pub fn parse_valid_ips(json: &str) -> Vec<String> {
    #[derive(serde::Deserialize)]
    struct IpEntry {
        ip: String,
    }

    if let Ok(entries) = serde_json::from_str::<Vec<IpEntry>>(json) {
        return entries
            .into_iter()
            .filter_map(|entry| canonicalize_rule(&entry.ip))
            .collect();
    }

    serde_json::from_str::<Vec<String>>(json)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|ip| canonicalize_rule(&ip))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapped_ipv6_and_ipv4_addresses_match_bidirectionally() {
        assert!(ip_matches_rule("::ffff:203.0.113.7", "203.0.113.7"));
        assert!(ip_matches_rule("203.0.113.7", "::ffff:203.0.113.7"));
    }

    #[test]
    fn mapped_ipv6_rules_are_canonicalized_when_loaded() {
        assert_eq!(
            parse_valid_ips(r#"["::ffff:203.0.113.7"]"#),
            vec!["203.0.113.7"]
        );
    }

    #[test]
    fn cidr_rules_match_their_address_family() {
        assert!(ip_matches_rule("203.0.113.0/24", "203.0.113.7"));
        assert!(!ip_matches_rule("203.0.113.0/24", "203.0.114.7"));
        assert!(ip_matches_rule("2001:db8::/32", "2001:db8::7"));
    }
}
