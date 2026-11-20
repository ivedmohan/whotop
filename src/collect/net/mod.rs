//! Per-process network connections, one implementation per OS.

use std::net::IpAddr;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
pub use macos::collect;
#[cfg(target_os = "linux")]
pub use linux::collect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Proto {
    Tcp,
    Udp,
}

impl Proto {
    pub fn label(self) -> &'static str {
        match self {
            Proto::Tcp => "tcp",
            Proto::Udp => "udp",
        }
    }
}

/// One socket as reported by the OS. Byte counts are cumulative since the
/// socket was opened (macOS) or unavailable (0 on Linux without capture).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RawConn {
    pub pid: u32,
    /// Process name as the OS reports it (may be truncated on macOS).
    pub proc_name: String,
    pub proto: Proto,
    pub local_ip: Option<IpAddr>,
    pub local_port: u16,
    pub remote_ip: Option<IpAddr>,
    pub remote_port: u16,
    pub iface: String,
    pub state: String,
    pub bytes_in: u64,
    pub bytes_out: u64,
    /// Whether the OS gave us byte counters at all.
    pub has_bytes: bool,
}

impl RawConn {
    /// Identity of a socket across samples, used to compute rates.
    pub fn key(&self) -> (u32, Proto, Option<IpAddr>, u16, Option<IpAddr>, u16) {
        (
            self.pid,
            self.proto,
            self.local_ip,
            self.local_port,
            self.remote_ip,
            self.remote_port,
        )
    }
}

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
/// Parse "ip:port" (v4) or "ip.port" (v6 as nettop prints it) or "*:port".
/// Returns (ip, port); ip is None for wildcard.
pub fn parse_endpoint(s: &str) -> Option<(Option<IpAddr>, u16)> {
    let s = s.trim();
    // nettop uses ':' for v4 and '.' for v6 port separators.
    let (host, port) = if let Some(idx) = s.rfind(':').filter(|&i| !s[..i].contains(':')) {
        (&s[..idx], &s[idx + 1..])
    } else if let Some(idx) = s.rfind('.') {
        (&s[..idx], &s[idx + 1..])
    } else {
        return None;
    };
    let port: u16 = if port == "*" { 0 } else { port.parse().ok()? };
    let host = host.split('%').next().unwrap_or(host);
    let ip = if host == "*" || host.is_empty() {
        None
    } else {
        Some(host.parse().ok()?)
    };
    Some((ip, port))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoints() {
        assert_eq!(
            parse_endpoint("192.168.31.242:56761"),
            Some((Some("192.168.31.242".parse().unwrap()), 56761))
        );
        assert_eq!(parse_endpoint("*:57114"), Some((None, 57114)));
        assert_eq!(parse_endpoint("*:*"), Some((None, 0)));
        assert_eq!(parse_endpoint("*.546"), Some((None, 546)));
        assert_eq!(
            parse_endpoint("fe80::1%en0.5353"),
            Some((Some("fe80::1".parse().unwrap()), 5353))
        );
        assert_eq!(
            parse_endpoint("2a03:2880:f12f:83:face:b00c:0:25de.443"),
            Some((Some("2a03:2880:f12f:83:face:b00c:0:25de".parse().unwrap()), 443))
        );
    }
}
