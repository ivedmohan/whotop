# FAQ and troubleshooting

## `whotop: command not found` right after installing

The installer put the binary in `~/.local/bin` and added that folder to your
shell's startup file, but the shell you are typing in started before that.
Either open a new terminal or run:

```sh
export PATH="$HOME/.local/bin:$PATH"
```

## The ↓/s and ↑/s columns are empty

On Linux that is expected, see [Platforms](platforms.md). On macOS it can
happen for the first second after launch, before there are two samples to
compare, and for sockets that have genuinely moved nothing.

## Why is almost everything talking to Apple or Cloudflare?

Because it is. macOS itself keeps a handful of connections to Apple open for
push notifications, iCloud and updates, and a large share of websites and APIs
sit behind Cloudflare. The "who" column names the network operator, not
necessarily the service.

## What are all these TimeWait rows?

Connections that just closed. The OS holds the slot for about a minute as a
safety measure. They are harmless and they go away on their own. Press `i` to
hide anything without live traffic.

## I see a dev server or database I forgot about

That is one of the main reasons whotop exists. Press `l` to show listening
sockets, find the process name and pid, and stop it with your usual tool,
for example `kill <pid>` or `brew services stop postgresql`.

## Does whotop send anything anywhere?

No. The only network activity whotop itself generates is reverse DNS lookups
for addresses already on screen, which go to whatever DNS server your system
already uses. There is no telemetry, no update check, no config file.

## Is it safe to run on a work machine?

It reads the same information Activity Monitor, `lsof` and `netstat` show,
and runs as your normal user. It cannot see the content of any connection,
only the addresses and byte counts.

## It looks wrong in my terminal

Columns hide below 100 and 150 characters of width, so widen the window.
Garbled box characters usually mean the terminal font lacks Unicode box
drawing; most modern terminal fonts have it.

## How do I report a bug?

[github.com/ivedmohan/whotop/issues](https://github.com/ivedmohan/whotop/issues).
Include your OS, the terminal, and a screenshot if it is a display problem.
