//! Sampling thread. Every `interval` it refreshes CPU/memory/processes via
//! sysinfo and the socket list via the OS collector, computes per-socket
//! rates from the previous sample, and sends a `Snapshot` to the UI.

pub mod net;

use net::{Proto, RawConn};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};
use sysinfo::{Networks, ProcessesToUpdate, System};

#[derive(Debug, Clone)]
pub struct Conn {
    pub pid: u32,
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
    pub has_bytes: bool,
    /// n-th socket with the same 5-tuple in this sample (SO_REUSEPORT etc).
    pub ordinal: u32,
    pub rate_in: f64,
    pub rate_out: f64,
    /// Seconds since we first saw this socket; None if it predates whotop.
    pub age: Option<f64>,
}

impl Conn {
    pub fn is_listening(&self) -> bool {
        self.remote_ip.is_none() && self.local_port != 0
    }
    pub fn rate(&self) -> f64 {
        self.rate_in + self.rate_out
    }
    pub fn total(&self) -> u64 {
        self.bytes_in + self.bytes_out
    }
}

#[derive(Debug, Clone)]
pub struct Proc {
    pub pid: u32,
    pub name: String,
    pub cpu: f32,
    pub mem: u64,
    pub conns: usize,
    pub rate: f64,
}

#[derive(Debug, Clone, Default)]
pub struct Snapshot {
    pub cpu_total: f32,
    pub cpu_cores: Vec<f32>,
    pub load: (f64, f64, f64),
    pub mem_used: u64,
    pub mem_total: u64,
    pub swap_used: u64,
    pub swap_total: u64,
    pub net_in: f64,
    pub net_out: f64,
    pub conns: Vec<Conn>,
    pub procs: Vec<Proc>,
    pub net_error: Option<String>,
    pub sample_ms: u128,
}

pub fn spawn(interval: Duration) -> Receiver<Snapshot> {
    let (tx, rx) = mpsc::channel();
    thread::Builder::new()
        .name("collect".into())
        .spawn(move || run(tx, interval))
        .expect("spawn collector");
    rx
}

fn run(tx: Sender<Snapshot>, interval: Duration) {
    let mut sys = System::new();
    let mut networks = Networks::new_with_refreshed_list();
    sys.refresh_cpu_usage();
    sys.refresh_memory();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let mut prev: HashMap<_, (u64, u64, Instant)> = HashMap::new();
    let mut first_seen: HashMap<_, Option<Instant>> = HashMap::new();
    let mut first_sample = true;
    let mut last = Instant::now();
    thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL.max(Duration::from_millis(300)));
    loop {
        let started = Instant::now();
        let dt = started.duration_since(last).as_secs_f64().max(0.001);
        last = started;

        sys.refresh_cpu_usage();
        sys.refresh_memory();
        sys.refresh_processes(ProcessesToUpdate::All, true);
        networks.refresh(true);
        let (mut net_in, mut net_out) = (0u64, 0u64);
        for (_, data) in networks.iter() {
            net_in += data.received();
            net_out += data.transmitted();
        }

        let names: HashMap<u32, String> = sys
            .processes()
            .iter()
            .map(|(pid, p)| (pid.as_u32(), p.name().to_string_lossy().to_string()))
            .collect();

        let (conns, net_error) = match net::collect() {
            Ok(raw) => (build_conns(raw, &names, &mut prev, &mut first_seen, started, first_sample), None),
            Err(e) => (Vec::new(), Some(e.to_string())),
        };

        let mut per_pid: HashMap<u32, (usize, f64)> = HashMap::new();
        for c in &conns {
            let e = per_pid.entry(c.pid).or_default();
            e.0 += 1;
            e.1 += c.rate();
        }
        let mut procs: Vec<Proc> = sys
            .processes()
            .values()
            .map(|p| {
                let pid = p.pid().as_u32();
                let (conns, rate) = per_pid.get(&pid).copied().unwrap_or((0, 0.0));
                Proc { pid, name: p.name().to_string_lossy().to_string(), cpu: p.cpu_usage(), mem: p.memory(), conns, rate }
            })
            .collect();
        procs.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal));

        let load = System::load_average();
        let snap = Snapshot {
            cpu_total: sys.global_cpu_usage(),
            cpu_cores: sys.cpus().iter().map(|c| c.cpu_usage()).collect(),
            load: (load.one, load.five, load.fifteen),
            mem_used: sys.used_memory(),
            mem_total: sys.total_memory(),
            swap_used: sys.used_swap(),
            swap_total: sys.total_swap(),
            net_in: net_in as f64 / dt,
            net_out: net_out as f64 / dt,
            conns,
            procs,
            net_error,
            sample_ms: started.elapsed().as_millis(),
        };
        first_sample = false;
        if tx.send(snap).is_err() {
            return;
        }
        thread::sleep(interval.saturating_sub(started.elapsed()));
    }
}

type Key = ((u32, Proto, Option<IpAddr>, u16, Option<IpAddr>, u16), u32);

fn build_conns(
    raw: Vec<RawConn>,
    names: &HashMap<u32, String>,
    prev: &mut HashMap<Key, (u64, u64, Instant)>,
    first_seen: &mut HashMap<Key, Option<Instant>>,
    now: Instant,
    first_sample: bool,
) -> Vec<Conn> {
    let mut next = HashMap::with_capacity(raw.len());
    let mut seen = HashMap::with_capacity(raw.len());
    let mut out = Vec::with_capacity(raw.len());
    let mut ordinals: HashMap<_, u32> = HashMap::new();
    for r in raw {
        let ordinal = {
            let n = ordinals.entry(r.key()).or_insert(0);
            *n += 1;
            *n - 1
        };
        let key = (r.key(), ordinal);
        let (rate_in, rate_out) = match prev.get(&key) {
            Some((pin, pout, t)) => {
                let dt = now.duration_since(*t).as_secs_f64().max(0.001);
                (
                    r.bytes_in.saturating_sub(*pin) as f64 / dt,
                    r.bytes_out.saturating_sub(*pout) as f64 / dt,
                )
            }
            None => (0.0, 0.0),
        };
        let first = *first_seen.get(&key).unwrap_or(&if first_sample { None } else { Some(now) });
        next.insert(key.clone(), (r.bytes_in, r.bytes_out, now));
        seen.insert(key, first);
        // sysinfo has the full name; nettop truncates to 15 chars.
        let proc_name = names.get(&r.pid).cloned().unwrap_or(r.proc_name);
        out.push(Conn {
            pid: r.pid,
            proc_name,
            proto: r.proto,
            local_ip: r.local_ip,
            local_port: r.local_port,
            remote_ip: r.remote_ip,
            remote_port: r.remote_port,
            iface: r.iface,
            state: r.state,
            bytes_in: r.bytes_in,
            bytes_out: r.bytes_out,
            has_bytes: r.has_bytes,
            ordinal,
            rate_in,
            rate_out,
            age: first.map(|f| now.duration_since(f).as_secs_f64()),
        });
    }
    *prev = next;
    *first_seen = seen;
    out
}
