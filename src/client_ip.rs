//! Resolve the real client IP when running behind a reverse proxy (e.g. Caddy).
//!
//! `X-Forwarded-For` is only honoured when the TCP peer is a configured trusted
//! proxy. The header is then read right to left and the first address that is not
//! itself a trusted proxy is the client. A client connecting directly can therefore
//! never spoof its IP by sending its own `X-Forwarded-For`.

use std::net::IpAddr;

use axum::http::HeaderMap;

/// A set of trusted proxy networks (single IPs or CIDR ranges).
#[derive(Debug, Clone, Default)]
pub struct TrustedProxies {
    nets: Vec<(IpAddr, u8)>,
}

impl TrustedProxies {
    /// Parse entries like `127.0.0.1`, `172.16.0.0/12` or `fd00::/8`.
    pub fn parse(entries: &[String]) -> anyhow::Result<Self> {
        let mut nets = Vec::with_capacity(entries.len());
        for entry in entries {
            let entry = entry.trim();
            let (addr, prefix) = match entry.split_once('/') {
                Some((a, p)) => (a, Some(p)),
                None => (entry, None),
            };
            let ip: IpAddr = addr
                .parse()
                .map_err(|_| anyhow::anyhow!("invalid trusted proxy address: {entry}"))?;
            let max = if ip.is_ipv4() { 32 } else { 128 };
            let prefix = match prefix {
                Some(p) => p
                    .parse::<u8>()
                    .ok()
                    .filter(|&p| p <= max)
                    .ok_or_else(|| anyhow::anyhow!("invalid prefix length in: {entry}"))?,
                None => max,
            };
            nets.push((ip.to_canonical(), prefix));
        }
        Ok(Self { nets })
    }

    pub fn is_empty(&self) -> bool {
        self.nets.is_empty()
    }

    /// True if `ip` falls inside any trusted network.
    pub fn contains(&self, ip: IpAddr) -> bool {
        let ip = ip.to_canonical();
        self.nets
            .iter()
            .any(|&(net, prefix)| in_network(ip, net, prefix))
    }

    /// Return the client IP for a request arriving from `peer`.
    pub fn resolve(&self, peer: IpAddr, headers: &HeaderMap) -> IpAddr {
        let peer = peer.to_canonical();
        if !self.contains(peer) {
            return peer;
        }
        // All X-Forwarded-For values, in order, flattened.
        let hops: Vec<&str> = headers
            .get_all("x-forwarded-for")
            .iter()
            .filter_map(|v| v.to_str().ok())
            .flat_map(|v| v.split(','))
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        let mut last_trusted = peer;
        for hop in hops.iter().rev() {
            match hop.parse::<IpAddr>() {
                Ok(ip) => {
                    let ip = ip.to_canonical();
                    if !self.contains(ip) {
                        return ip;
                    }
                    last_trusted = ip;
                }
                // Malformed entry: anything further left is untrustworthy.
                Err(_) => return last_trusted,
            }
        }
        last_trusted
    }
}

fn in_network(ip: IpAddr, net: IpAddr, prefix: u8) -> bool {
    match (ip, net) {
        (IpAddr::V4(ip), IpAddr::V4(net)) => {
            let mask = u32::MAX.checked_shl(32 - u32::from(prefix)).unwrap_or(0);
            (u32::from(ip) & mask) == (u32::from(net) & mask)
        }
        (IpAddr::V6(ip), IpAddr::V6(net)) => {
            let mask = u128::MAX.checked_shl(128 - u32::from(prefix)).unwrap_or(0);
            (u128::from(ip) & mask) == (u128::from(net) & mask)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proxies(list: &[&str]) -> TrustedProxies {
        TrustedProxies::parse(&list.iter().map(|s| s.to_string()).collect::<Vec<_>>()).unwrap()
    }

    fn xff(value: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert("x-forwarded-for", value.parse().unwrap());
        h
    }

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    #[test]
    fn no_trusted_proxies_uses_peer_and_ignores_header() {
        let tp = TrustedProxies::default();
        assert_eq!(
            tp.resolve(ip("172.18.0.2"), &xff("1.2.3.4")),
            ip("172.18.0.2")
        );
    }

    #[test]
    fn untrusted_peer_cannot_spoof() {
        let tp = proxies(&["172.16.0.0/12"]);
        assert_eq!(tp.resolve(ip("8.8.8.8"), &xff("1.2.3.4")), ip("8.8.8.8"));
    }

    #[test]
    fn trusted_peer_uses_forwarded_client() {
        let tp = proxies(&["172.16.0.0/12"]);
        assert_eq!(
            tp.resolve(ip("172.18.0.2"), &xff("203.0.113.7")),
            ip("203.0.113.7")
        );
    }

    #[test]
    fn takes_rightmost_untrusted_hop() {
        // Client sent a forged "6.6.6.6"; Caddy appended the real client.
        let tp = proxies(&["172.16.0.0/12", "10.0.0.0/8"]);
        let h = xff("6.6.6.6, 203.0.113.7, 10.0.0.5");
        assert_eq!(tp.resolve(ip("172.18.0.2"), &h), ip("203.0.113.7"));
    }

    #[test]
    fn multiple_header_lines_are_joined() {
        let tp = proxies(&["172.16.0.0/12"]);
        let mut h = HeaderMap::new();
        h.append("x-forwarded-for", "6.6.6.6".parse().unwrap());
        h.append("x-forwarded-for", "203.0.113.7".parse().unwrap());
        assert_eq!(tp.resolve(ip("172.18.0.2"), &h), ip("203.0.113.7"));
    }

    #[test]
    fn malformed_hop_stops_the_walk() {
        let tp = proxies(&["172.16.0.0/12"]);
        let h = xff("203.0.113.7, garbage");
        assert_eq!(tp.resolve(ip("172.18.0.2"), &h), ip("172.18.0.2"));
    }

    #[test]
    fn missing_header_falls_back_to_peer() {
        let tp = proxies(&["172.16.0.0/12"]);
        assert_eq!(
            tp.resolve(ip("172.18.0.2"), &HeaderMap::new()),
            ip("172.18.0.2")
        );
    }

    #[test]
    fn ipv4_mapped_ipv6_peer_matches_ipv4_range() {
        let tp = proxies(&["127.0.0.1"]);
        assert_eq!(
            tp.resolve(ip("::ffff:127.0.0.1"), &xff("203.0.113.7")),
            ip("203.0.113.7")
        );
    }

    #[test]
    fn ipv6_ranges() {
        let tp = proxies(&["fd00::/8"]);
        assert!(tp.contains(ip("fd12::1")));
        assert!(!tp.contains(ip("2001:db8::1")));
    }

    #[test]
    fn zero_prefix_and_bad_entries() {
        assert!(proxies(&["0.0.0.0/0"]).contains(ip("8.8.8.8")));
        assert!(TrustedProxies::parse(&["nope".into()]).is_err());
        assert!(TrustedProxies::parse(&["10.0.0.0/33".into()]).is_err());
    }
}
