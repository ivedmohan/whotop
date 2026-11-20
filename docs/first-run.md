# First run

Type `whotop` and press Enter. You get a full-screen view that updates once a
second. Press `q` to leave at any time.

## The layout

```
┌ cpu ─────────────────────────┐┌ mem ─────────────────────────┐
│ usage graph + one bar per core││ used / swap, net ↓ and ↑ graphs│
└──────────────────────────────┘└──────────────────────────────┘
┌ who  24 of 61 sockets ───────────────────┐┌ what is this? ──────┐
│ process   who          what   ↓/s  ↑/s ...││ plain-English panel  │
│ ▶ Safari  Cloudflare   HTTPS  12K  1.2K   ││ for the selected row │
│   node    localhost    dev server         ││                      │
└───────────────────────────────────────────┘└──────────────────────┘
┌ processes ───────────────────────────────────────────────────────┐
│ pid  name  cpu%  mem  socks  net/s                                │
└──────────────────────────────────────────────────────────────────┘
 j/k move  tab panel  s sort  / filter  l listening  ? help  q quit
```

- **Top left, cpu.** Total usage, load averages, a sparkline of recent usage,
  and one bar per core.
- **Top right, mem.** Memory and swap in use, and two sparklines for network
  download and upload across the whole machine.
- **Middle left, who.** The main table. One row per network socket.
- **Middle right, what is this?** Explains whatever row is selected.
- **Bottom, processes.** Every process with how many sockets it has and how
  much network traffic it is doing.
- **Footer.** The keys that matter right now.

## Your first 30 seconds

1. Look at the top rows of the **who** table. They are sorted by bytes per
   second, so whatever is moving data right now is at the top.
2. Press `j` to move down a row. The **what is this?** panel changes to
   describe that socket.
3. Press `l` to also show *listening* sockets: programs waiting for someone
   to connect to them. This is how you find a dev server or database you
   forgot was running.
4. Press `/` and type a name, like `node` or `apple`, to filter. `Esc` clears.
5. Press `?` for the key list.

## Beginner hints

By default the explanation panel includes a short story for each row, for
example what a "listening" socket is or why "localhost" means the program is
only talking to itself. Press `b` to switch those hints off once they get in
the way. Start with `whotop --no-hints` to have them off from the beginning.
