mod app;
mod collect;
mod explain;
mod ui;

use anyhow::Result;
use app::App;
use ratatui::crossterm::event::{self, Event};
use std::time::Duration;

const USAGE: &str = "whotop: who is your computer talking to?

Usage: whotop [--interval <seconds>] [--no-hints]

  --interval <s>   sampling interval, default 1
  --no-hints       start with beginner hints off (toggle with b)
  -h, --help       this text
";

fn main() -> Result<()> {
    let mut interval = 1.0f64;
    let mut hints = true;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--interval" => {
                interval = args.next().and_then(|v| v.parse().ok()).unwrap_or(1.0);
            }
            "--no-hints" => hints = false,
            "-h" | "--help" => {
                print!("{USAGE}");
                return Ok(());
            }
            other => {
                eprintln!("unknown argument: {other}\n{USAGE}");
                std::process::exit(2);
            }
        }
    }
    let interval = Duration::from_secs_f64(interval.clamp(0.25, 60.0));

    let rx = collect::spawn(interval);
    let mut app = App::new(hints);
    let mut terminal = ratatui::init();
    let result = run(&mut terminal, &mut app, rx);
    ratatui::restore();
    result
}

fn run(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut App,
    rx: std::sync::mpsc::Receiver<collect::Snapshot>,
) -> Result<()> {
    loop {
        let mut got = false;
        while let Ok(snap) = rx.try_recv() {
            app.apply(snap);
            got = true;
        }
        if !got {
            app.resolver.drain();
        }
        terminal.draw(|f| ui::draw(f, app))?;
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(k) if k.is_press() => app.on_key(k),
                _ => {}
            }
        }
        if app.should_quit {
            return Ok(());
        }
    }
}
