//! Drawing. btop-style boxes: cpu + mem/net on top, the connections table
//! with an explanation panel in the middle, processes at the bottom.

use crate::app::{App, Focus};
use crate::collect::net::Proto;
use crate::collect::Conn;
use crate::explain::{self, human_bytes, human_rate};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Clear, Paragraph, Row, Sparkline, Table, TableState, Wrap};
use ratatui::Frame;

const BORDER: Color = Color::Rgb(70, 80, 110);
const BORDER_FOCUS: Color = Color::Rgb(120, 190, 255);
const TITLE: Color = Color::Rgb(255, 170, 90);
const ACCENT: Color = Color::Rgb(120, 220, 170);
const IN: Color = Color::Rgb(110, 200, 255);
const OUT: Color = Color::Rgb(255, 150, 120);
const DIM: Color = Color::Rgb(120, 130, 150);
const SEL: Color = Color::Rgb(45, 55, 85);
const HINT: Color = Color::Rgb(200, 190, 120);

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let [top, mid, bot, foot] = Layout::vertical([
        Constraint::Length(9),
        Constraint::Fill(3),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(area);

    let [cpu, mem] = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(top);
    draw_cpu(f, app, cpu);
    draw_mem(f, app, mem);

    let [table, detail] = Layout::horizontal([Constraint::Fill(3), Constraint::Length(detail_width(mid))]).areas(mid);
    draw_conns(f, app, table);
    draw_detail(f, app, detail);
    draw_procs(f, app, bot);
    draw_footer(f, app, foot);
    if app.help {
        draw_help(f, area);
    }
}

fn detail_width(area: Rect) -> u16 {
    (area.width / 3).clamp(30, 46)
}

fn block(title: &str, focused: bool) -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(if focused { BORDER_FOCUS } else { BORDER }))
        .title(Line::from(vec![
            Span::styled(" ", Style::new()),
            Span::styled(title.to_string(), Style::new().fg(TITLE).bold()),
            Span::styled(" ", Style::new()),
        ]))
}

fn draw_cpu(f: &mut Frame, app: &App, area: Rect) {
    let s = &app.snap;
    let title = format!("cpu {:>3.0}%  load {:.2} {:.2} {:.2}", s.cpu_total, s.load.0, s.load.1, s.load.2);
    let b = block(&title, false);
    let inner = b.inner(area);
    f.render_widget(b, area);
    if inner.height < 2 {
        return;
    }
    let [spark, cores] = Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).areas(inner);
    let data: Vec<u64> = app.cpu_hist.iter().rev().take(spark.width as usize).rev().copied().collect();
    f.render_widget(
        Sparkline::default().data(data).max(100).style(Style::new().fg(ACCENT)),
        spark,
    );
    // Per-core mini bars, laid out in columns.
    let n = s.cpu_cores.len().max(1);
    let rows = cores.height.max(1) as usize;
    let cols = n.div_ceil(rows).max(1);
    let col_w = (cores.width as usize / cols).max(8);
    let mut lines: Vec<Line> = vec![Line::default(); rows];
    for (i, pct) in s.cpu_cores.iter().enumerate() {
        let row = i % rows;
        let bar_w = col_w.saturating_sub(8).max(1);
        let bar = bar(*pct as f64, bar_w);
        let color = heat(*pct);
        lines[row].spans.push(Span::styled(format!("{i:>2} "), Style::new().fg(DIM)));
        lines[row].spans.push(Span::styled(bar, Style::new().fg(color)));
        lines[row].spans.push(Span::styled(format!("{pct:>3.0}% "), Style::new().fg(color)));
    }
    f.render_widget(Paragraph::new(Text::from(lines)), cores);
}

fn heat(pct: f32) -> Color {
    if pct > 80.0 {
        Color::Rgb(255, 100, 100)
    } else if pct > 50.0 {
        Color::Rgb(255, 200, 90)
    } else {
        ACCENT
    }
}

fn bar(pct: f64, width: usize) -> String {
    let filled = ((pct / 100.0) * width as f64).round() as usize;
    let filled = filled.min(width);
    "▇".repeat(filled) + &"▁".repeat(width - filled)
}

fn draw_mem(f: &mut Frame, app: &App, area: Rect) {
    let s = &app.snap;
    let b = block("mem", false);
    let inner = b.inner(area);
    f.render_widget(b, area);
    if inner.height < 2 {
        return;
    }
    let [memr, swapr, hintr, nin, nout] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(2),
        Constraint::Length(2),
    ])
    .areas(inner);
    let mem_pct = if s.mem_total > 0 { s.mem_used as f64 * 100.0 / s.mem_total as f64 } else { 0.0 };
    let swap_pct = if s.swap_total > 0 { s.swap_used as f64 * 100.0 / s.swap_total as f64 } else { 0.0 };
    let label_w = 24usize;
    let bar_w = (inner.width as usize).saturating_sub(label_w + 4).max(4);
    let row = |name: &str, used: u64, total: u64, pct: f64, color: Color| {
        Line::from(vec![
            Span::styled(format!("{name:<5}"), Style::new().fg(DIM)),
            Span::styled(format!("{:>5} / {:<5}", human_bytes(used), human_bytes(total)), Style::new().fg(Color::White)),
            Span::styled(format!(" {} ", bar(pct, bar_w)), Style::new().fg(color)),
            Span::styled(format!("{pct:>3.0}%"), Style::new().fg(color)),
        ])
    };
    f.render_widget(Paragraph::new(row("used", s.mem_used, s.mem_total, mem_pct, heat(mem_pct as f32))), memr);
    if s.swap_total > 0 {
        f.render_widget(Paragraph::new(row("swap", s.swap_used, s.swap_total, swap_pct, Color::Rgb(180, 140, 255))), swapr);
    } else {
        f.render_widget(Paragraph::new(Span::styled("swap  none", Style::new().fg(DIM))), swapr);
    }
    let hint = if !app.hints {
        ""
    } else if s.swap_total > 0 && swap_pct > 70.0 {
        "swap mostly full: RAM is tight, the OS is paging to disk"
    } else if mem_pct > 90.0 {
        "memory nearly full: the OS is about to swap to disk"
    } else {
        "the OS uses spare memory as cache, so high usage alone is not a problem"
    };
    f.render_widget(Paragraph::new(Span::styled(hint, Style::new().fg(HINT).italic())), hintr);

    let net_row = |f: &mut Frame, area: Rect, label: &str, rate: f64, hist: &std::collections::VecDeque<u64>, color: Color| {
        let [lbl, spark] = Layout::horizontal([Constraint::Length(16), Constraint::Fill(1)]).areas(area);
        let text = Text::from(vec![
            Line::from(Span::styled(label.to_string(), Style::new().fg(DIM))),
            Line::from(Span::styled(human_rate(rate), Style::new().fg(color).bold())),
        ]);
        f.render_widget(Paragraph::new(text), lbl);
        let data: Vec<u64> = hist.iter().rev().take(spark.width as usize).rev().copied().collect();
        f.render_widget(Sparkline::default().data(data).style(Style::new().fg(color)), spark);
    };
    net_row(f, nin, "net ↓ download", s.net_in, &app.in_hist, IN);
    net_row(f, nout, "net ↑ upload", s.net_out, &app.out_hist, OUT);
}

/// "who" column: org, else reverse DNS, else scope, else the bare IP.
fn who(app: &mut App, c: &Conn) -> (String, Style) {
    let Some(ip) = c.remote_ip else {
        return (format!("listening on {}", if c.local_ip.is_none() { "all interfaces" } else { "this address" }), Style::new().fg(DIM));
    };
    if let Some(s) = explain::scope_label(&ip) {
        return (s.to_string(), Style::new().fg(DIM));
    }
    if let Some(org) = explain::org_for_ip(&ip) {
        return (org.to_string(), Style::new().fg(Color::White));
    }
    if let Some(name) = app.resolver.name_for(&ip) {
        return (explain::short_host(name), Style::new().fg(Color::White));
    }
    (ip.to_string(), Style::new().fg(DIM))
}

fn what(c: &Conn) -> String {
    let port = if c.remote_ip.is_some() { c.remote_port } else { c.local_port };
    match explain::port_info(port, c.proto) {
        Some(p) if p.name != "ephemeral" => p.name.to_string(),
        _ => match explain::port_info(c.local_port, c.proto) {
            Some(p) if c.remote_ip.is_some() && p.name != "ephemeral" => p.name.to_string(),
            _ => format!("{}/{}", c.proto.label(), port),
        },
    }
}

fn draw_conns(f: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.focus == Focus::Conns;
    let visible: Vec<Conn> = app.visible_conns().into_iter().cloned().collect();
    let total = app.snap.conns.len();
    let mut title = format!(
        "who  {} of {} sockets  sort:{}{}{}",
        visible.len(),
        total,
        app.sort.label(),
        if app.show_listening { "  +listening" } else { "" },
        if app.show_idle { "" } else { "  active only" },
    );
    if app.typing || !app.filter.is_empty() {
        title.push_str(&format!("  filter:{}{}", app.filter, if app.typing { "▏" } else { "" }));
    }
    let b = block(&title, focused);
    let inner = b.inner(area);
    f.render_widget(b, area);
    if let Some(err) = &app.snap.net_error {
        f.render_widget(Paragraph::new(format!("could not read sockets: {err}")).fg(OUT).wrap(Wrap { trim: true }), inner);
        return;
    }
    if visible.is_empty() {
        let msg = if app.ticks == 0 { "sampling…" } else { "nothing matches" };
        f.render_widget(Paragraph::new(msg).fg(DIM), inner);
        return;
    }
    let has_bytes = visible.iter().any(|c| c.has_bytes);
    let wide = inner.width >= 100;
    let xwide = inner.width >= 150;
    let mut head = vec!["process", "who"];
    if wide {
        head.push("where");
    }
    if xwide {
        head.push("state");
    }
    head.extend(["what", "↓/s", "↑/s", "total"]);
    if wide {
        head.push("age");
    }
    let header = Row::new(head).style(Style::new().fg(DIM).add_modifier(Modifier::BOLD));
    let rows: Vec<Row> = visible
        .iter()
        .map(|c| {
            let (who_s, who_style) = who(app, c);
            let where_s = match c.remote_ip {
                Some(ip) => format!("{}:{}", compact_ip(&ip), c.remote_port),
                None => format!("{}:{}", c.local_ip.map(|i| compact_ip(&i)).unwrap_or_else(|| "*".into()), c.local_port),
            };
            let rate_in = if c.rate_in > 0.0 { Span::styled(human_rate(c.rate_in), Style::new().fg(IN)) } else { Span::styled("-", Style::new().fg(DIM)) };
            let rate_out = if c.rate_out > 0.0 { Span::styled(human_rate(c.rate_out), Style::new().fg(OUT)) } else { Span::styled("-", Style::new().fg(DIM)) };
            let active = c.rate() > 0.0;
            let mut cells = vec![
                Line::from(Span::styled(c.proc_name.clone(), if active { Style::new().fg(ACCENT).bold() } else { Style::new().fg(Color::White) })),
                Line::from(Span::styled(who_s, who_style)),
            ];
            if wide {
                cells.push(Line::from(Span::styled(where_s, Style::new().fg(DIM))));
            }
            if xwide {
                let st = if c.is_listening() { "Listen".to_string() } else if c.state.is_empty() { "-".to_string() } else { c.state.clone() };
                cells.push(Line::from(Span::styled(st, Style::new().fg(DIM))));
            }
            cells.extend([
                Line::from(Span::styled(what(c), Style::new().fg(Color::Rgb(200, 200, 255)))),
                Line::from(rate_in),
                Line::from(rate_out),
                Line::from(Span::styled(if has_bytes { human_bytes(c.total()) } else { "-".into() }, Style::new().fg(DIM))),
            ]);
            if wide {
                cells.push(Line::from(Span::styled(age(c.age), Style::new().fg(DIM))));
            }
            Row::new(cells)
        })
        .collect();
    let mut widths = vec![Constraint::Length(if inner.width >= 130 { 20 } else { 14 }), Constraint::Fill(1)];
    if wide {
        widths.push(Constraint::Length(19));
    }
    if xwide {
        widths.push(Constraint::Length(11));
    }
    widths.extend([Constraint::Length(12), Constraint::Length(7), Constraint::Length(7), Constraint::Length(6)]);
    if wide {
        widths.push(Constraint::Length(4));
    }
    let table = Table::new(rows, widths)
        .header(header)
        .column_spacing(1)
        .row_highlight_style(Style::new().bg(SEL).add_modifier(Modifier::BOLD))
        .highlight_symbol(if focused { "▶ " } else { "  " });
    let mut state = TableState::default().with_selected(Some(app.conn_sel));
    f.render_stateful_widget(table, inner, &mut state);
}

fn compact_ip(ip: &std::net::IpAddr) -> String {
    let s = ip.to_string();
    if s.len() > 15 {
        format!("{}…", &s[..10])
    } else {
        s
    }
}

fn age(secs: Option<f64>) -> String {
    let Some(secs) = secs else { return "old".into() };
    if secs < 60.0 {
        format!("{secs:.0}s")
    } else if secs < 3600.0 {
        format!("{:.0}m", secs / 60.0)
    } else {
        format!("{:.1}h", secs / 3600.0)
    }
}

fn draw_detail(f: &mut Frame, app: &mut App, area: Rect) {
    let b = block("what is this?", false);
    let inner = b.inner(area);
    f.render_widget(b, area);
    let Some(c) = app.selected_conn().cloned() else {
        f.render_widget(Paragraph::new("select a connection with j/k").fg(DIM), inner);
        return;
    };
    let label = |s: &str| Span::styled(format!("{s:<10}"), Style::new().fg(DIM));
    let val = |s: String| Span::styled(s, Style::new().fg(Color::White));
    let mut lines: Vec<Line> = Vec::new();
    let (who_s, _) = who(app, &c);
    lines.push(Line::from(vec![
        Span::styled(c.proc_name.clone(), Style::new().fg(ACCENT).bold()),
        Span::styled(format!(" (pid {})", c.pid), Style::new().fg(DIM)),
    ]));
    if c.is_listening() {
        lines.push(Line::from(vec![label("is"), val("waiting for connections".into())]));
    } else {
        lines.push(Line::from(vec![label("talks to"), val(who_s.clone())]));
    }
    if let Some(ip) = c.remote_ip {
        lines.push(Line::from(vec![label("address"), val(ip.to_string())]));
        if let Some(name) = app.resolver.name_for(&ip) {
            if explain::org_for_ip(&ip).is_some() {
                lines.push(Line::from(vec![label("hostname"), val(name.to_string())]));
            }
        }
        if explain::is_ipv6(&ip) {
            lines.push(Line::from(vec![label(""), Span::styled("IPv6 address (the newer, longer kind)", Style::new().fg(DIM))]));
        }
    }
    let port = if c.remote_ip.is_some() { c.remote_port } else { c.local_port };
    let info = explain::port_info(port, c.proto).filter(|p| p.name != "ephemeral" || c.remote_ip.is_none())
        .or_else(|| explain::port_info(c.local_port, c.proto).filter(|p| p.name != "ephemeral"));
    let proto_name = match c.proto { Proto::Tcp => "TCP", Proto::Udp => "UDP" };
    match &info {
        Some(p) => {
            lines.push(Line::from(vec![label("port"), val(format!("{port} {proto_name} = {}", p.name))]));
            let enc = match p.encrypted {
                Some(true) => ("yes", ACCENT),
                Some(false) => ("no, readable on the network", OUT),
                None => ("depends on the app", DIM),
            };
            lines.push(Line::from(vec![label("encrypted"), Span::styled(enc.0, Style::new().fg(enc.1))]));
        }
        None => lines.push(Line::from(vec![label("port"), val(format!("{port} {proto_name} (not a well-known port)"))])),
    }
    if !c.state.is_empty() {
        lines.push(Line::from(vec![label("state"), val(c.state.clone())]));
    }
    if !c.iface.is_empty() {
        lines.push(Line::from(vec![label("via"), val(iface_label(&c.iface))]));
    }
    if c.has_bytes {
        lines.push(Line::from(vec![
            label("traffic"),
            Span::styled(format!("↓{} ", human_bytes(c.bytes_in)), Style::new().fg(IN)),
            Span::styled(format!("↑{}", human_bytes(c.bytes_out)), Style::new().fg(OUT)),
            Span::styled(format!("  now ↓{} ↑{}", human_rate(c.rate_in), human_rate(c.rate_out)), Style::new().fg(DIM)),
        ]));
    } else {
        lines.push(Line::from(vec![label("traffic"), Span::styled("byte counts need packet capture on this OS", Style::new().fg(DIM))]));
    }
    lines.push(Line::from(vec![
        label("seen for"),
        val(match c.age {
            Some(_) => age(c.age),
            None => "longer than whotop has been running".into(),
        }),
    ]));

    if app.hints {
        lines.push(Line::default());
        let mut story = String::new();
        if c.is_listening() {
            story.push_str(&format!(
                "{} opened port {} and is waiting for something to connect to it. ",
                c.proc_name, c.local_port
            ));
            if let Some(p) = &info {
                story.push_str(p.what);
            }
            if c.local_ip.map(|ip| ip.is_loopback()).unwrap_or(false) {
                story.push_str(" It only accepts connections from this computer.");
            } else {
                story.push_str(" Other devices on the network could reach it.");
            }
        } else {
            story.push_str(&format!("{} is talking to {}. ", c.proc_name, who_s));
            if c.remote_ip.map(|ip| ip.is_loopback()).unwrap_or(false) {
                story.push_str("This never leaves your machine: it is two programs on this computer talking to each other through the network stack. ");
            }
            if let Some(p) = &info {
                story.push_str(p.what);
                story.push(' ');
            }
            story.push_str(state_story(&c.state, c.proto));
        }
        lines.push(Line::from(Span::styled(story, Style::new().fg(HINT))));
    }
    f.render_widget(Paragraph::new(Text::from(lines)).wrap(Wrap { trim: false }), inner);
}

fn iface_label(iface: &str) -> String {
    match iface {
        "lo0" | "lo" => "loopback (never leaves this computer)".into(),
        i if i.starts_with("en0") => "en0 (Wi-Fi on most Macs)".into(),
        i if i.starts_with("utun") => format!("{i} (VPN tunnel)"),
        i if i.starts_with("awdl") => format!("{i} (AirDrop / peer-to-peer Wi-Fi)"),
        i if i.starts_with("bridge") => format!("{i} (virtual machine bridge)"),
        i => i.to_string(),
    }
}

fn state_story(state: &str, proto: Proto) -> &'static str {
    match (proto, state) {
        (Proto::Udp, _) => "UDP has no connection state: packets are just sent, no handshake, no guarantee they arrive. Good for DNS, video and games.",
        (_, "Established") => "The connection is open and data can flow both ways.",
        (_, "TimeWait") => "Already closed. The OS keeps the slot for about a minute so stray late packets can't confuse a new connection. Harmless.",
        (_, "CloseWait") => "The other side hung up but this app hasn't closed its end yet. A lot of these can mean a leak in the app.",
        (_, "SynSent") => "Trying to connect, waiting for the other side to answer. If it stays here, the host is unreachable or a firewall is dropping it.",
        (_, "FinWait1") | (_, "FinWait2") => "This side is closing the connection and waiting for the other side to finish.",
        (_, "LastAck") | (_, "Closing") => "Both sides are finishing the close handshake.",
        (_, "Listen") => "Waiting for incoming connections.",
        _ => "",
    }
}

fn draw_procs(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::Procs;
    let b = block(&format!("processes  {}", app.snap.procs.len()), focused);
    let inner = b.inner(area);
    f.render_widget(b, area);
    let header = Row::new(vec!["pid", "name", "cpu%", "mem", "socks", "net/s"]).style(Style::new().fg(DIM).add_modifier(Modifier::BOLD));
    let rows: Vec<Row> = app
        .snap
        .procs
        .iter()
        .map(|p| {
            Row::new(vec![
                Line::from(Span::styled(p.pid.to_string(), Style::new().fg(DIM))),
                Line::from(Span::styled(p.name.clone(), Style::new().fg(Color::White))),
                Line::from(Span::styled(format!("{:.1}", p.cpu), Style::new().fg(heat(p.cpu)))),
                Line::from(Span::styled(human_bytes(p.mem), Style::new().fg(DIM))),
                Line::from(Span::styled(if p.conns > 0 { p.conns.to_string() } else { "-".into() }, Style::new().fg(DIM))),
                Line::from(Span::styled(human_rate(p.rate), Style::new().fg(if p.rate > 0.0 { IN } else { DIM }))),
            ])
        })
        .collect();
    let widths = [
        Constraint::Length(7),
        Constraint::Fill(1),
        Constraint::Length(6),
        Constraint::Length(7),
        Constraint::Length(5),
        Constraint::Length(8),
    ];
    let table = Table::new(rows, widths)
        .header(header)
        .row_highlight_style(Style::new().bg(SEL).add_modifier(Modifier::BOLD))
        .highlight_symbol(if focused { "▶ " } else { "  " });
    let mut state = TableState::default().with_selected(Some(app.proc_sel));
    f.render_stateful_widget(table, inner, &mut state);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let key = |k: &str, d: &str| {
        vec![
            Span::styled(format!(" {k} "), Style::new().fg(Color::Black).bg(TITLE)),
            Span::styled(format!(" {d}  "), Style::new().fg(DIM)),
        ]
    };
    let mut spans = Vec::new();
    for (k, d) in [
        ("q", "quit"),
        ("j/k", "move"),
        ("tab", "panel"),
        ("s", "sort"),
        ("/", "filter"),
        ("l", "listening"),
        ("i", "active only"),
        ("b", if app.hints { "hints on" } else { "hints off" }),
        ("?", "help"),
    ] {
        spans.extend(key(k, d));
    }
    spans.push(Span::styled(format!("sample {}ms", app.snap.sample_ms), Style::new().fg(BORDER)));
    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn draw_help(f: &mut Frame, area: Rect) {
    let w = 64.min(area.width.saturating_sub(4));
    let h = 22.min(area.height.saturating_sub(2));
    let rect = Rect::new((area.width - w) / 2, (area.height - h) / 2, w, h);
    f.render_widget(Clear, rect);
    let text = "\
whotop shows every network socket on this machine, which
program owns it, who it is talking to, and how much data
is moving. Select a row to get a plain-English explanation.

  j / k, arrows   move          g / G    top / bottom
  tab             switch panel  s        change sort
  /               filter text   esc      clear filter
  l               show listening sockets too
  i               only sockets with traffic right now
  b               beginner hints on / off
  q               quit

Reading the table:
  who     the company or hostname behind the IP
  what    what the port is normally used for
  ↓/s ↑/s bytes per second in / out, right now
  total   bytes since the socket opened

press any key to close";
    let b = block("help", true);
    f.render_widget(Paragraph::new(text).block(b).wrap(Wrap { trim: false }), rect);
}
