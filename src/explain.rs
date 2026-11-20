//! Plain-English knowledge: what a port is for, who owns an IP range,
//! and reverse DNS with a cache, so the UI can say "Chrome ⇄ google.com, HTTPS"
//! instead of "49152 -> 142.250.183.14:443".

use crate::collect::net::Proto;
use std::collections::{HashMap, HashSet};
use std::net::{IpAddr, Ipv4Addr};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

pub struct PortInfo {
    pub name: &'static str,
    pub what: &'static str,
    pub encrypted: Option<bool>,
}

const fn p(name: &'static str, what: &'static str, encrypted: Option<bool>) -> PortInfo {
    PortInfo { name, what, encrypted }
}

/// Well-known ports. Looked up by remote port first, then local port.
pub fn port_info(port: u16, proto: Proto) -> Option<PortInfo> {
    Some(match (port, proto) {
        (443, Proto::Tcp) => p("HTTPS", "Encrypted web traffic. Almost every website, API call and app update uses this.", Some(true)),
        (443, Proto::Udp) => p("HTTP/3 QUIC", "Encrypted web traffic over UDP. Google, YouTube, Cloudflare sites and newer apps prefer it.", Some(true)),
        (80, _) => p("HTTP", "Unencrypted web traffic. Anyone on the network path can read it. Mostly redirects to HTTPS nowadays.", Some(false)),
        (53, _) => p("DNS", "Looking up a name like github.com to find its IP address. Happens before almost every connection.", Some(false)),
        (853, _) => p("DNS/TLS", "Encrypted DNS lookups.", Some(true)),
        (5353, Proto::Udp) => p("mDNS/Bonjour", "Discovering devices on your local network: printers, AirPlay, other Macs.", Some(false)),
        (5223, Proto::Tcp) => p("Apple push", "Apple Push Notification service. Keeps iMessage, FaceTime and app notifications arriving.", Some(true)),
        (5228, _) | (5229, _) | (5230, _) => p("Google push", "Google's push notification channel (Firebase Cloud Messaging).", Some(true)),
        (22, Proto::Tcp) => p("SSH", "Encrypted remote shell, git over SSH, or an SSH tunnel.", Some(true)),
        (25, Proto::Tcp) => p("SMTP", "Sending email, unencrypted handshake.", Some(false)),
        (465, Proto::Tcp) => p("SMTPS", "Sending email, encrypted.", Some(true)),
        (587, Proto::Tcp) => p("SMTP submit", "Sending email through your provider, usually upgraded to TLS.", None),
        (993, Proto::Tcp) => p("IMAPS", "Mail app syncing your inbox, encrypted.", Some(true)),
        (995, Proto::Tcp) => p("POP3S", "Mail app downloading mail, encrypted.", Some(true)),
        (123, Proto::Udp) => p("NTP", "Clock sync. Your computer asking a time server what time it is.", Some(false)),
        (67, Proto::Udp) | (68, Proto::Udp) => p("DHCP", "Getting an IP address from your router.", Some(false)),
        (546, Proto::Udp) | (547, Proto::Udp) => p("DHCPv6", "Getting an IPv6 address from your router.", Some(false)),
        (1900, Proto::Udp) => p("SSDP/UPnP", "Discovering smart TVs, speakers and routers on the local network.", Some(false)),
        (3478, _) | (3479, _) | (19302, _) => p("STUN/TURN", "Video or voice call setup (WebRTC, FaceTime, Zoom, Meet). Finds a path through your router.", None),
        (1194, Proto::Udp) | (51820, Proto::Udp) => p("VPN", "OpenVPN or WireGuard tunnel.", Some(true)),
        (5432, Proto::Tcp) => p("PostgreSQL", "A Postgres database connection.", None),
        (3306, Proto::Tcp) => p("MySQL", "A MySQL or MariaDB database connection.", None),
        (6379, Proto::Tcp) => p("Redis", "A Redis connection.", Some(false)),
        (27017, Proto::Tcp) => p("MongoDB", "A MongoDB connection.", None),
        (5672, Proto::Tcp) => p("AMQP", "A RabbitMQ message queue connection.", None),
        (9092, Proto::Tcp) => p("Kafka", "A Kafka broker connection.", None),
        (3000, Proto::Tcp) | (3001, Proto::Tcp) => p("dev server", "A local development server (Next.js, Express, Rails use this by default).", Some(false)),
        (5173, Proto::Tcp) => p("Vite dev", "Vite's development server.", Some(false)),
        (8000, Proto::Tcp) | (8080, Proto::Tcp) | (8888, Proto::Tcp) => p("dev/proxy", "A common port for local web servers, proxies and Jupyter.", Some(false)),
        (4200, Proto::Tcp) => p("Angular dev", "Angular's development server.", Some(false)),
        (5000, Proto::Tcp) => p("Flask/AirPlay", "Flask's default port, also used by AirPlay Receiver on macOS.", None),
        (7000, Proto::Tcp) => p("AirPlay", "AirPlay Receiver on macOS, or a local app.", None),
        (9000, Proto::Tcp) => p("port 9000", "A local service port; which one depends on the process.", None),
        (11434, Proto::Tcp) => p("Ollama", "A local LLM server.", Some(false)),
        (1883, Proto::Tcp) => p("MQTT", "IoT messaging.", Some(false)),
        (8883, Proto::Tcp) => p("MQTTS", "IoT messaging, encrypted.", Some(true)),
        (445, Proto::Tcp) => p("SMB", "Windows / Mac file sharing.", None),
        (548, Proto::Tcp) => p("AFP", "Apple file sharing (legacy).", None),
        (3389, Proto::Tcp) => p("RDP", "Windows Remote Desktop.", Some(true)),
        (5900, Proto::Tcp) => p("VNC", "Remote screen control.", None),
        (49152..=65535, _) => p("ephemeral", "A random high port the OS picked for this side of the connection; the other end's port is the meaningful one.", None),
        _ => return None,
    })
}

/// Who owns a public IP range. Coarse, hand-curated, enough for the usual
/// suspects; reverse DNS fills in the rest.
pub fn org_for_ip(ip: &IpAddr) -> Option<&'static str> {
    match ip {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            let in_range = |net: Ipv4Addr, bits: u32| {
                let mask = if bits == 0 { 0 } else { u32::MAX << (32 - bits) };
                (u32::from(*v4) & mask) == (u32::from(net) & mask)
            };
            let r = |a, b, c, d, bits| in_range(Ipv4Addr::new(a, b, c, d), bits);
            Some(match () {
                _ if o[0] == 17 => "Apple",
                _ if r(1, 1, 1, 0, 24) || r(1, 0, 0, 0, 24) => "Cloudflare DNS",
                _ if r(104, 16, 0, 0, 12) || r(172, 64, 0, 0, 13) || r(162, 158, 0, 0, 15) || r(188, 114, 96, 0, 20) || r(198, 41, 128, 0, 17) => "Cloudflare",
                _ if r(8, 8, 8, 0, 24) || r(8, 8, 4, 0, 24) => "Google DNS",
                _ if r(142, 250, 0, 0, 15) || r(172, 217, 0, 0, 16) || r(216, 58, 192, 0, 19) || r(74, 125, 0, 0, 16) || r(64, 233, 160, 0, 19) || r(173, 194, 0, 0, 16) || r(209, 85, 128, 0, 17) || r(35, 190, 0, 0, 17) || r(34, 64, 0, 0, 10) || r(35, 184, 0, 0, 13) => "Google",
                _ if r(140, 82, 112, 0, 20) || r(185, 199, 108, 0, 22) || r(143, 55, 64, 0, 20) || r(192, 30, 252, 0, 22) => "GitHub",
                _ if r(151, 101, 0, 0, 16) || r(199, 232, 0, 0, 16) => "Fastly (CDN)",
                _ if r(157, 240, 0, 0, 16) || r(31, 13, 64, 0, 18) || r(179, 60, 192, 0, 22) || r(163, 70, 128, 0, 17) => "Meta",
                _ if r(13, 107, 0, 0, 16) || r(20, 190, 128, 0, 18) || r(40, 126, 0, 0, 18) || r(52, 96, 0, 0, 12) || r(204, 79, 197, 0, 24) => "Microsoft",
                _ if r(3, 0, 0, 0, 8) || r(52, 0, 0, 0, 10) || r(54, 64, 0, 0, 10) || r(18, 128, 0, 0, 9) || r(34, 192, 0, 0, 10) || r(44, 192, 0, 0, 10) || r(13, 32, 0, 0, 12) || r(99, 84, 0, 0, 16) || r(65, 8, 0, 0, 14) || r(13, 224, 0, 0, 12) => "Amazon / AWS",
                _ if r(23, 32, 0, 0, 11) || r(23, 192, 0, 0, 11) || r(104, 64, 0, 0, 10) || r(184, 24, 0, 0, 13) || r(2, 16, 0, 0, 13) || r(95, 100, 0, 0, 15) => "Akamai (CDN)",
                _ if r(76, 76, 21, 0, 24) || r(76, 223, 0, 0, 16) => "Vercel",
                _ if r(162, 159, 0, 0, 16) => "Cloudflare",
                _ if r(146, 75, 0, 0, 16) => "Fastly (CDN)",
                _ if r(149, 154, 160, 0, 20) || r(91, 108, 4, 0, 22) || r(91, 108, 56, 0, 22) => "Telegram",
                _ if r(66, 22, 192, 0, 18) => "Discord",
                _ if r(160, 79, 104, 0, 23) => "Anthropic",
                _ if r(104, 18, 0, 0, 16) => "Cloudflare",
                _ => return None,
            })
        }
        IpAddr::V6(v6) => {
            let seg = v6.segments();
            let r = |a: u16, b: u16| seg[0] == a && (seg[1] & 0xff00) == (b & 0xff00);
            Some(match () {
                _ if r(0x2a03, 0x2880) || r(0x2a03, 0x2887) => "Meta",
                _ if r(0x2606, 0x4700) || r(0x2a06, 0x98c0) => "Cloudflare",
                _ if r(0x2607, 0x6bc0) => "Anthropic",
                _ if r(0x2600, 0x1901) || r(0x2600, 0x1900) => "Google Cloud",
                _ if r(0x2607, 0xf8b0) || r(0x2a00, 0x1450) || r(0x2001, 0x4860) || r(0x2404, 0x6800) || r(0x2800, 0x3f0) => "Google",
                _ if r(0x2620, 0x149) || r(0x2a01, 0xb740) || r(0x2403, 0x300) => "Apple",
                _ if r(0x2a04, 0x4e42) || r(0x2a04, 0x4e40) => "Fastly (CDN)",
                _ if r(0x2a01, 0x111) || r(0x2603, 0x1000) || r(0x2620, 0x1ec) => "Microsoft",
                _ if r(0x2600, 0x1f00) || r(0x2600, 0x9000) || r(0x2406, 0xda00) => "Amazon / AWS",
                _ if r(0x2a02, 0x26f0) || r(0x2600, 0x1400) => "Akamai (CDN)",
                _ => return None,
            })
        }
    }
}

/// Where an address lives, in words a beginner understands.
pub fn scope_label(ip: &IpAddr) -> Option<&'static str> {
    match ip {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            if v4.is_loopback() {
                Some("this computer (localhost)")
            } else if o[0] == 10 || (o[0] == 172 && (16..=31).contains(&o[1])) || (o[0] == 192 && o[1] == 168) {
                Some("your local network")
            } else if o[0] == 169 && o[1] == 254 {
                Some("link-local (no router)")
            } else if v4.is_multicast() {
                Some("multicast (everyone on the LAN)")
            } else if v4.is_broadcast() {
                Some("broadcast")
            } else {
                None
            }
        }
        IpAddr::V6(v6) => {
            if v6.is_loopback() {
                Some("this computer (localhost)")
            } else if (v6.segments()[0] & 0xffc0) == 0xfe80 {
                Some("link-local (your LAN)")
            } else if (v6.segments()[0] & 0xfe00) == 0xfc00 {
                Some("your local network")
            } else if v6.is_multicast() {
                Some("multicast (everyone on the LAN)")
            } else {
                None
            }
        }
    }
}

pub fn is_ipv6(ip: &IpAddr) -> bool {
    matches!(ip, IpAddr::V6(_))
}

/// Reverse DNS in a background thread. `name_for` never blocks: it returns
/// the cached answer or queues a lookup and returns None.
pub struct Resolver {
    cache: HashMap<IpAddr, Option<String>>,
    pending: HashSet<IpAddr>,
    tx: Sender<IpAddr>,
    rx: Receiver<(IpAddr, Option<String>)>,
}

impl Resolver {
    pub fn new() -> Self {
        let (tx, work_rx) = mpsc::channel::<IpAddr>();
        let (res_tx, rx) = mpsc::channel();
        thread::Builder::new()
            .name("rdns".into())
            .spawn(move || {
                for ip in work_rx {
                    let name = dns_lookup::lookup_addr(&ip)
                        .ok()
                        .filter(|n| n.parse::<IpAddr>().is_err());
                    if res_tx.send((ip, name)).is_err() {
                        break;
                    }
                }
            })
            .expect("spawn rdns thread");
        Self { cache: HashMap::new(), pending: HashSet::new(), tx, rx }
    }

    pub fn drain(&mut self) {
        while let Ok((ip, name)) = self.rx.try_recv() {
            self.pending.remove(&ip);
            self.cache.insert(ip, name);
        }
    }

    pub fn name_for(&mut self, ip: &IpAddr) -> Option<&str> {
        if scope_label(ip).is_some() {
            return None;
        }
        if !self.cache.contains_key(ip) && self.pending.insert(*ip) {
            let _ = self.tx.send(*ip);
        }
        self.cache.get(ip).and_then(|n| n.as_deref())
    }
}

/// Trim a reverse-DNS name down to something readable:
/// "ec2-3-5-140-2.ap-south-1.compute.amazonaws.com" -> "amazonaws.com"
/// "bom12s20-in-f14.1e100.net" -> "1e100.net (Google)".
pub fn short_host(name: &str) -> String {
    let parts: Vec<&str> = name.trim_end_matches('.').split('.').collect();
    let n = parts.len();
    if n < 2 {
        return name.to_string();
    }
    // Keep three labels for ccTLD style like co.uk / com.au.
    let keep = if n >= 3 && parts[n - 2].len() <= 3 && parts[n - 1].len() == 2 { 3 } else { 2 };
    let base = parts[n - keep..].join(".");
    match base.as_str() {
        "1e100.net" => "1e100.net (Google)".to_string(),
        "icloud.com" | "apple.com" | "aaplimg.com" => format!("{base} (Apple)"),
        "amazonaws.com" => "amazonaws.com (AWS)".to_string(),
        _ => base,
    }
}

pub fn human_bytes(b: u64) -> String {
    const UNITS: [&str; 5] = ["B", "K", "M", "G", "T"];
    let mut v = b as f64;
    let mut i = 0;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{b}B")
    } else if v < 10.0 {
        format!("{v:.1}{}", UNITS[i])
    } else {
        format!("{v:.0}{}", UNITS[i])
    }
}

pub fn human_rate(bps: f64) -> String {
    if bps < 1.0 {
        return "-".into();
    }
    format!("{}/s", human_bytes(bps as u64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orgs() {
        assert_eq!(org_for_ip(&"17.242.13.4".parse().unwrap()), Some("Apple"));
        assert_eq!(org_for_ip(&"142.250.183.14".parse().unwrap()), Some("Google"));
        assert_eq!(org_for_ip(&"140.82.114.4".parse().unwrap()), Some("GitHub"));
        assert_eq!(org_for_ip(&"1.1.1.1".parse().unwrap()), Some("Cloudflare DNS"));
        assert_eq!(org_for_ip(&"2a03:2880:f12f:83:face:b00c:0:25de".parse().unwrap()), Some("Meta"));
        assert_eq!(org_for_ip(&"2607:6bc0::10".parse().unwrap()), Some("Anthropic"));
        assert_eq!(org_for_ip(&"2600:1901:0:9e23::".parse().unwrap()), Some("Google Cloud"));
        assert_eq!(org_for_ip(&"203.0.113.9".parse().unwrap()), None);
    }

    #[test]
    fn scopes() {
        assert_eq!(scope_label(&"127.0.0.1".parse().unwrap()), Some("this computer (localhost)"));
        assert_eq!(scope_label(&"192.168.31.1".parse().unwrap()), Some("your local network"));
        assert_eq!(scope_label(&"8.8.8.8".parse().unwrap()), None);
    }

    #[test]
    fn hosts() {
        assert_eq!(short_host("bom12s20-in-f14.1e100.net"), "1e100.net (Google)");
        assert_eq!(short_host("ec2-3-5-140-2.ap-south-1.compute.amazonaws.com"), "amazonaws.com (AWS)");
        assert_eq!(short_host("foo.bbc.co.uk"), "bbc.co.uk");
    }

    #[test]
    fn bytes() {
        assert_eq!(human_bytes(512), "512B");
        assert_eq!(human_bytes(2048), "2.0K");
        assert_eq!(human_bytes(15 * 1024 * 1024), "15M");
    }
}

impl Resolver {
    /// Cached answer without queueing a lookup. Outer None = not looked up yet.
    pub fn peek(&self, ip: &IpAddr) -> Option<Option<&str>> {
        self.cache.get(ip).map(|n| n.as_deref())
    }
}
