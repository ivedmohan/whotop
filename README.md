# whotop

Who is your computer talking to?

A terminal monitor in the spirit of btop, except the main panel is not CPU.
It is every network socket on the machine: which program owns it, who is on
the other end (as a company or hostname, not just an IP), what the port is
normally used for, and how many bytes are moving right now. Select a row and
the side panel explains it in plain English.

Built for people who open Activity Monitor or `lsof -i`, see a wall of IP
addresses, and wonder what any of it means.

## Status

Early. Works on macOS today, Linux collector written but untested on real
hardware. Not packaged yet.

## Run it

```sh
cargo run --release
```

Options: `--interval <seconds>` (default 1), `--no-hints`.

## Keys

| key | action |
|---|---|
| `j` / `k`, arrows | move |
| `tab` | switch between connections and processes |
| `s` | sort: rate, total, process, newest |
| `/` | filter by process, host, company, port or protocol; `esc` clears |
| `l` | also show listening sockets |
| `i` | only sockets with traffic right now |
| `b` | beginner hints on / off |
| `?` | help |
| `q` | quit |

## How it works

- **macOS**: one `nettop -L 1` sample per tick. The kernel exposes per-socket
  byte counters to any user through the same mechanism Activity Monitor uses,
  so there is no sudo. Rates are the difference between two samples.
- **Linux**: `/proc/net/{tcp,udp}{,6}` joined to `/proc/<pid>/fd` for the
  owning process. No privileges needed. Byte counters are not available this
  way; that needs packet capture and is not implemented yet.
- CPU, memory and the process table come from `sysinfo`. The UI is `ratatui`.
- "Who" is a small hand-curated table of well-known IP ranges (Apple, Google,
  Cloudflare, GitHub, AWS, Meta, Microsoft, Fastly, Akamai and a few more),
  falling back to reverse DNS in a background thread, falling back to the IP.
- "What" is a table of well-known ports with a one-line explanation and
  whether the traffic is normally encrypted.

## License

MIT.
