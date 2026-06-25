use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

/// 校验 IP 格式（支持纯 IP / CIDR）
pub fn is_valid_ip(s: &str) -> bool {
    // 分离 CIDR 前缀
    let (ip_part, cidr) = match s.split_once('/') {
        Some((ip, mask)) => (ip, Some(mask)),
        None => (s, None),
    };

    let is_v4 = Ipv4Addr::from_str(ip_part).is_ok();
    let is_v6 = Ipv6Addr::from_str(ip_part).is_ok();
    if !is_v4 && !is_v6 {
        return false;
    }

    // 校验 CIDR 前缀（如果有）
    if let Some(mask) = cidr {
        if let Ok(n) = mask.parse::<u8>() {
            if is_v4 { n <= 32 } else { n <= 128 }
        } else {
            false
        }
    } else {
        true
    }
}

/// 解析 IP 列表（兼容旧格式 string[] 和新格式 {ip,remark}[] ），过滤无效 IP
pub fn parse_valid_ips(json: &str) -> Vec<String> {
    // 试试新格式：[{"ip":"...","remark":"..."}, ...]
    #[derive(serde::Deserialize)]
    struct IpEntry { ip: String }
    if let Ok(entries) = serde_json::from_str::<Vec<IpEntry>>(json) {
        return entries.into_iter()
            .map(|e| e.ip)
            .filter(|ip| is_valid_ip(ip))
            .collect();
    }
    // 回退旧格式：["ip1", "ip2", ...]
    serde_json::from_str::<Vec<String>>(json)
        .unwrap_or_default()
        .into_iter()
        .filter(|ip| is_valid_ip(ip))
        .collect()
}
