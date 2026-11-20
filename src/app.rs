//! UI state and key handling. Drawing lives in ui.rs.

use crate::collect::net::Proto;
use crate::collect::{Conn, Snapshot};
use crate::explain::{self, Resolver};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::VecDeque;
use std::net::IpAddr;

pub const HISTORY: usize = 240;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Conns,
    Procs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sort {
    Rate,
    Total,
    Proc,
    Age,
}

impl Sort {
    pub fn next(self) -> Self {
        match self {
            Sort::Rate => Sort::Total,
            Sort::Total => Sort::Proc,
            Sort::Proc => Sort::Age,
            Sort::Age => Sort::Rate,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Sort::Rate => "rate",
            Sort::Total => "total",
            Sort::Proc => "process",
            Sort::Age => "newest",
        }
    }
}

type Key = (u32, Proto, Option<IpAddr>, u16, Option<IpAddr>, u16, u32);

fn key_of(c: &Conn) -> Key {
    (c.pid, c.proto, c.local_ip, c.local_port, c.remote_ip, c.remote_port, c.ordinal)
}

pub struct App {
    pub snap: Snapshot,
    pub cpu_hist: VecDeque<u64>,
    pub in_hist: VecDeque<u64>,
    pub out_hist: VecDeque<u64>,
    pub resolver: Resolver,
    pub focus: Focus,
    pub sort: Sort,
    pub show_listening: bool,
    pub show_idle: bool,
    pub hints: bool,
    pub help: bool,
    pub filter: String,
    pub typing: bool,
    pub conn_sel: usize,
    pub conn_key: Option<Key>,
    pub proc_sel: usize,
    pub should_quit: bool,
    pub ticks: u64,
}

impl App {
    pub fn new(hints: bool) -> Self {
        Self {
            snap: Snapshot::default(),
            cpu_hist: VecDeque::with_capacity(HISTORY),
            in_hist: VecDeque::with_capacity(HISTORY),
            out_hist: VecDeque::with_capacity(HISTORY),
            resolver: Resolver::new(),
            focus: Focus::Conns,
            sort: Sort::Rate,
            show_listening: false,
            show_idle: true,
            hints,
            help: false,
            filter: String::new(),
            typing: false,
            conn_sel: 0,
            conn_key: None,
            proc_sel: 0,
            should_quit: false,
            ticks: 0,
        }
    }

    pub fn apply(&mut self, snap: Snapshot) {
        push(&mut self.cpu_hist, snap.cpu_total.round() as u64);
        push(&mut self.in_hist, snap.net_in as u64);
        push(&mut self.out_hist, snap.net_out as u64);
        self.snap = snap;
        self.ticks += 1;
        // Warm the reverse-DNS cache for everything on screen.
        let ips: Vec<IpAddr> = self.snap.conns.iter().filter_map(|c| c.remote_ip).collect();
        for ip in ips {
            let _ = self.resolver.name_for(&ip);
        }
        self.resolver.drain();
        self.resync_selection();
    }

    /// Keep the cursor on the same socket when the list reorders.
    fn resync_selection(&mut self) {
        let keys: Vec<Key> = self.visible_conns().iter().map(|c| key_of(c)).collect();
        if let Some(key) = &self.conn_key {
            if let Some(i) = keys.iter().position(|k| k == key) {
                self.conn_sel = i;
                return;
            }
        }
        self.conn_sel = self.conn_sel.min(keys.len().saturating_sub(1));
        self.conn_key = keys.get(self.conn_sel).cloned();
    }

    pub fn visible_conns(&self) -> Vec<&Conn> {
        let f = self.filter.to_lowercase();
        let mut v: Vec<&Conn> = self
            .snap
            .conns
            .iter()
            .filter(|c| self.show_listening || !c.is_listening())
            .filter(|c| self.show_idle || c.rate() > 0.0 || c.age.is_some_and(|a| a < 3.0))
            .filter(|c| f.is_empty() || self.matches(c, &f))
            .collect();
        match self.sort {
            Sort::Rate => v.sort_by(|a, b| b.rate().total_cmp(&a.rate()).then(b.total().cmp(&a.total()))),
            Sort::Total => v.sort_by(|a, b| b.total().cmp(&a.total())),
            Sort::Proc => v.sort_by(|a, b| a.proc_name.to_lowercase().cmp(&b.proc_name.to_lowercase()).then(b.rate().total_cmp(&a.rate()))),
            Sort::Age => v.sort_by(|a, b| a.age.unwrap_or(f64::MAX).total_cmp(&b.age.unwrap_or(f64::MAX))),
        }
        v
    }

    fn matches(&self, c: &Conn, f: &str) -> bool {
        if c.proc_name.to_lowercase().contains(f) || c.pid.to_string() == f {
            return true;
        }
        if let Some(ip) = c.remote_ip {
            if ip.to_string().contains(f) {
                return true;
            }
            if explain::org_for_ip(&ip).map(|o| o.to_lowercase().contains(f)).unwrap_or(false) {
                return true;
            }
            if let Some(Some(name)) = self.resolver_peek(&ip) {
                if name.to_lowercase().contains(f) {
                    return true;
                }
            }
        }
        let port = if c.remote_ip.is_some() { c.remote_port } else { c.local_port };
        if port.to_string() == f {
            return true;
        }
        explain::port_info(port, c.proto).map(|p| p.name.to_lowercase().contains(f)).unwrap_or(false)
    }

    fn resolver_peek(&self, ip: &IpAddr) -> Option<Option<&str>> {
        self.resolver.peek(ip)
    }

    pub fn selected_conn(&self) -> Option<&Conn> {
        self.visible_conns().get(self.conn_sel).copied()
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        if self.typing {
            match key.code {
                KeyCode::Esc => {
                    self.typing = false;
                    self.filter.clear();
                }
                KeyCode::Enter => self.typing = false,
                KeyCode::Backspace => {
                    self.filter.pop();
                }
                KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => self.filter.push(c),
                _ => {}
            }
            self.conn_sel = 0;
            self.conn_key = None;
            self.resync_selection();
            return;
        }
        if self.help {
            self.help = false;
            return;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => self.should_quit = true,
            KeyCode::Char('?') => self.help = true,
            KeyCode::Tab => {
                self.focus = match self.focus {
                    Focus::Conns => Focus::Procs,
                    Focus::Procs => Focus::Conns,
                }
            }
            KeyCode::Char('j') | KeyCode::Down => self.move_sel(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_sel(-1),
            KeyCode::PageDown => self.move_sel(10),
            KeyCode::PageUp => self.move_sel(-10),
            KeyCode::Home | KeyCode::Char('g') => self.move_sel(i64::MIN / 2),
            KeyCode::End | KeyCode::Char('G') => self.move_sel(i64::MAX / 2),
            KeyCode::Char('s') => {
                self.sort = self.sort.next();
                self.resync_selection();
            }
            KeyCode::Char('l') => {
                self.show_listening = !self.show_listening;
                self.resync_selection();
            }
            KeyCode::Char('i') => {
                self.show_idle = !self.show_idle;
                self.resync_selection();
            }
            KeyCode::Char('b') => self.hints = !self.hints,
            KeyCode::Char('/') => {
                self.typing = true;
                self.focus = Focus::Conns;
            }
            _ => {}
        }
    }

    fn move_sel(&mut self, delta: i64) {
        match self.focus {
            Focus::Conns => {
                let n = self.visible_conns().len();
                if n == 0 {
                    return;
                }
                let sel = (self.conn_sel as i64 + delta).clamp(0, n as i64 - 1) as usize;
                let key = self.visible_conns().get(sel).map(|c| key_of(c));
                self.conn_sel = sel;
                self.conn_key = key;
            }
            Focus::Procs => {
                let n = self.snap.procs.len();
                if n == 0 {
                    return;
                }
                self.proc_sel = (self.proc_sel as i64 + delta).clamp(0, n as i64 - 1) as usize;
            }
        }
    }
}

fn push(h: &mut VecDeque<u64>, v: u64) {
    if h.len() == HISTORY {
        h.pop_front();
    }
    h.push_back(v);
}
