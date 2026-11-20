# Reading the table

Each row in the **who** table is one socket: one endpoint of one network
conversation, owned by one process.

## Columns

| column | meaning |
|---|---|
| **process** | the program that owns the socket, and its pid in the detail panel |
| **who** | the other end. A scope like `localhost` or `your local network`, a company like `Apple`, a hostname from reverse DNS, or the bare IP if nothing else is known |
| **where** | the remote IP and port (shown on wider terminals) |
| **state** | TCP state such as `Established` or `TimeWait` (shown on very wide terminals) |
| **what** | what the remote port is normally used for: `HTTPS`, `DNS`, `Postgres` |
| **↓/s** | bytes per second coming in to this socket, right now |
| **↑/s** | bytes per second going out |
| **total** | bytes moved in both directions since the socket opened |
| **age** | how long whotop has seen the socket. `old` means it was already open when whotop started |

Byte counts use short units: `512B`, `2.0K`, `15M`, `1.2G`.

The table title tells you what you are looking at: `who  24 of 61 sockets`
means 24 shown, 61 exist, the rest are hidden by the current filter, by the
listening toggle, or by the active-only toggle.

## Which sockets are hidden by default

- **Listening sockets** are hidden until you press `l`. A listening socket is
  a program waiting for connections, like a web server on port 3000 or Postgres
  on 5432. They are quiet by nature, so they would just add noise.
- **Idle sockets** are shown by default. Press `i` to see only sockets with
  traffic in the last tick, which collapses the table down to what is actually
  busy.

## The detail panel

Select a row and **what is this?** on the right shows:

- the process and pid
- who it is talking to, the full address, and the hostname when one resolves
- the port, its name, and whether that traffic is normally encrypted
- the TCP state, in words
- which network interface carries it: Wi-Fi, Ethernet, a VPN tunnel, or the
  loopback interface for localhost
- bytes in and out, in total and right now
- how long the socket has been open
- with hints on, a short story putting all of that together

See [What the explanations mean](explanations.md) for the vocabulary.

## The processes table

Press `Tab` to move focus to the bottom table. It lists every process with
CPU, memory, how many sockets it owns, and its total network rate. It is the
quickest way to answer "which program is using my bandwidth".
