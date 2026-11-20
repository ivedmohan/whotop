//! Linux: read /proc/net/{tcp,tcp6,udp,udp6} and map socket inodes to pids
//! through /proc/<pid>/fd. No privileges needed. Byte counters are not
//! available this way (that needs packet capture), so has_bytes is false.

use super::{Proto, RawConn};
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

pub fn collect() -> Result<Vec<RawConn>> {
    let inode_to_pid = socket_inodes();
    let mut out = Vec::new();
    for (file, proto, v6) in [
        ("/proc/net/tcp", Proto::Tcp, false),
        ("/proc/net/tcp6", Proto::Tcp, true),
        ("/proc/net/udp", Proto::Udp, false),
        ("/proc/net/udp6", Proto::Udp, true),
    ] {
        let Ok(text) = fs::read_to_string(file) else { continue };
        for line in text.lines().skip(1) {
            let f: Vec<&str> = line.split_whitespace().collect();
            if f.len() < 10 {
                continue;
            }
            let Some((local_ip, local_port)) = parse_hex_endpoint(f[1], v6) else { continue };
            let Some((remote_ip, remote_port)) = parse_hex_endpoint(f[2], v6) else { continue };
            let inode: u64 = f[9].parse().unwrap_or(0);
            let Some((pid, name)) = inode_to_pid.get(&inode) else { continue };
            let state = match proto {
                Proto::Tcp => tcp_state(f[3]).to_string(),
                Proto::Udp => String::new(),
            };
            let remote_ip = remote_ip.filter(|ip| !ip.is_unspecified());
            out.push(RawConn {
                pid: *pid,
                proc_name: name.clone(),
                proto,
                local_ip: local_ip.filter(|ip| !ip.is_unspecified()),
                local_port,
                remote_ip,
                remote_port,
                iface: String::new(),
                state,
                bytes_in: 0,
                bytes_out: 0,
                has_bytes: false,
            });
        }
    }
    Ok(out)
}

fn socket_inodes() -> HashMap<u64, (u32, String)> {
    let mut map = HashMap::new();
    let Ok(procs) = fs::read_dir("/proc") else { return map };
    for entry in procs.flatten() {
        let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() else { continue };
        let Ok(fds) = fs::read_dir(entry.path().join("fd")) else { continue };
        let name = fs::read_to_string(entry.path().join("comm"))
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        for fd in fds.flatten() {
            let Ok(target) = fs::read_link(fd.path()) else { continue };
            let t = target.to_string_lossy();
            if let Some(num) = t.strip_prefix("socket:[").and_then(|s| s.strip_suffix(']')) {
                if let Ok(inode) = num.parse::<u64>() {
                    map.insert(inode, (pid, name.clone()));
                }
            }
        }
    }
    map
}

/// "0100007F:1F90" -> 127.0.0.1:8080. v6 is four little-endian u32 words.
fn parse_hex_endpoint(s: &str, v6: bool) -> Option<(Option<IpAddr>, u16)> {
    let (ip, port) = s.split_once(':')?;
    let port = u16::from_str_radix(port, 16).ok()?;
    let ip = if v6 {
        if ip.len() != 32 {
            return None;
        }
        let mut bytes = [0u8; 16];
        for (i, chunk) in ip.as_bytes().chunks(8).enumerate() {
            let word = u32::from_str_radix(std::str::from_utf8(chunk).ok()?, 16).ok()?;
            bytes[i * 4..i * 4 + 4].copy_from_slice(&word.to_le_bytes());
        }
        IpAddr::V6(Ipv6Addr::from(bytes))
    } else {
        let word = u32::from_str_radix(ip, 16).ok()?;
        IpAddr::V4(Ipv4Addr::from(word.to_le_bytes()))
    };
    Some((Some(ip), port))
}

fn tcp_state(hex: &str) -> &'static str {
    match hex {
        "01" => "Established",
        "02" => "SynSent",
        "03" => "SynReceived",
        "04" => "FinWait1",
        "05" => "FinWait2",
        "06" => "TimeWait",
        "07" => "Closed",
        "08" => "CloseWait",
        "09" => "LastAck",
        "0A" => "Listen",
        "0B" => "Closing",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_v4() {
        let (ip, port) = parse_hex_endpoint("0100007F:1F90", false).unwrap();
        assert_eq!(ip, Some("127.0.0.1".parse().unwrap()));
        assert_eq!(port, 8080);
    }

    #[test]
    fn hex_v6_loopback() {
        let (ip, port) = parse_hex_endpoint("00000000000000000000000001000000:0016", true).unwrap();
        assert_eq!(ip, Some("::1".parse().unwrap()));
        assert_eq!(port, 22);
    }
}
