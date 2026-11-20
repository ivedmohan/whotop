//! macOS: parse one CSV sample from `nettop`, which exposes the kernel's
//! per-socket byte counters to any user (same source Activity Monitor uses).

use super::{parse_endpoint, Proto, RawConn};
use anyhow::{Context, Result};
use std::process::Command;

pub fn collect() -> Result<Vec<RawConn>> {
    let out = Command::new("/usr/bin/nettop")
        .args(["-n", "-x", "-L", "1", "-J", "interface,state,bytes_in,bytes_out"])
        .output()
        .context("running nettop")?;
    if !out.status.success() {
        anyhow::bail!("nettop exited with {}", out.status);
    }
    Ok(parse(&String::from_utf8_lossy(&out.stdout)))
}

/// nettop CSV layout (with -J interface,state,bytes_in,bytes_out):
/// ```text
/// ,interface,state,bytes_in,bytes_out,
/// apsd.370,,,5473,12346,
/// tcp4 192.168.31.242:56761<->17.242.13.4:5223,en0,Established,5473,12346,
/// ```
/// A process row is followed by its socket rows.
pub fn parse(csv: &str) -> Vec<RawConn> {
    let mut conns = Vec::new();
    let mut cur: Option<(u32, String)> = None;
    for line in csv.lines() {
        if line.starts_with(',') || line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split(',').collect();
        let first = cols[0];
        if let Some((proto, rest)) = first.split_once(' ') {
            let proto = match proto {
                "tcp4" | "tcp6" => Proto::Tcp,
                "udp4" | "udp6" => Proto::Udp,
                _ => continue,
            };
            let Some((pid, name)) = &cur else { continue };
            let Some((l, r)) = rest.split_once("<->") else { continue };
            let Some((local_ip, local_port)) = parse_endpoint(l) else { continue };
            let Some((remote_ip, remote_port)) = parse_endpoint(r) else { continue };
            if local_port == 0 && remote_ip.is_none() {
                continue; // unbound UDP socket, nothing to say about it
            }
            let get = |i: usize| cols.get(i).copied().unwrap_or("");
            let bytes_in = get(3).parse().unwrap_or(0);
            let bytes_out = get(4).parse().unwrap_or(0);
            conns.push(RawConn {
                pid: *pid,
                proc_name: name.clone(),
                proto,
                local_ip,
                local_port,
                remote_ip,
                remote_port,
                iface: get(1).to_string(),
                state: get(2).to_string(),
                bytes_in,
                bytes_out,
                has_bytes: true,
            });
        } else if let Some((name, pid)) = first.rsplit_once('.') {
            if let Ok(pid) = pid.parse::<u32>() {
                cur = Some((pid, name.to_string()));
            }
        }
    }
    conns
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
,interface,state,bytes_in,bytes_out,
configd.340,,,0,864,
udp6 *.546<->*.*,,,0,864,
apsd.370,,,5473,12346,
tcp4 192.168.31.242:56761<->17.242.13.4:5223,en0,Established,5473,12346,
nesessionmanage.377,,,0,0,
udp4 *:*<->*:*,,,,,
";

    #[test]
    fn parses_process_then_sockets() {
        let c = parse(SAMPLE);
        assert_eq!(c.len(), 2, "{c:?}");
        assert_eq!(c[0].pid, 340);
        assert_eq!(c[0].proc_name, "configd");
        assert!(c[0].remote_ip.is_none() && c[0].local_port == 546);
        assert_eq!(c[1].pid, 370);
        assert_eq!(c[1].proto, Proto::Tcp);
        assert_eq!(c[1].remote_port, 5223);
        assert_eq!(c[1].state, "Established");
        assert_eq!(c[1].bytes_out, 12346);
        assert_eq!(c[1].iface, "en0");
    }
}
