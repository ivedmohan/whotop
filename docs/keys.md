# Keys

| key | what it does |
|---|---|
| `j` / `k`, `↓` / `↑` | move the selection |
| `g` / `G`, `Home` / `End` | jump to top / bottom |
| `PgUp` / `PgDn` | move ten rows |
| `Tab` | switch focus between the connections table and the processes table |
| `s` | change sort: rate → total → process → newest → rate |
| `/` | start typing a filter; `Enter` keeps it, `Esc` clears it |
| `l` | also show listening sockets |
| `i` | only show sockets with traffic right now |
| `b` | beginner hints on / off |
| `?` | help overlay, any key closes it |
| `q`, `Esc`, `Ctrl-C` | quit |

## Sorting

- **rate** (default): bytes per second, in plus out, highest first. What is
  busy right now floats to the top.
- **total**: bytes moved since the socket opened. Finds the big downloads.
- **process**: alphabetical by program name, so one program's sockets sit
  together.
- **newest**: the socket that appeared most recently first. Useful when you
  just clicked something and want to see what it opened.

## Filtering

The filter matches against the process name, pid, the remote IP, the "who"
label (company or hostname), the port number, and the port's name. All of
these work:

```
/safari        every socket Safari owns
/cloudflare    everything going to Cloudflare
/443           everything on port 443
/https         same thing, by name
/localhost     only loopback traffic
```

The table title shows `filter:…` while one is active.

## Selection follows the socket

The selected row is pinned to a socket, not to a row number. When the sort
order changes because traffic moved, your selection stays on the same
connection. When that socket closes, the selection stays at the same position.
