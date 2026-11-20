# How it works

whotop is a single Rust binary, about 1 MB, with no runtime dependencies. It
reads, it never writes, and it never makes a network request except for
reverse DNS lookups of the addresses it already sees.

## Sampling

A background thread takes one snapshot per interval and sends it to the UI
thread over a channel. The UI thread redraws when a snapshot arrives and
handles keys in between, so the interface never stalls on a slow sample.

Each snapshot contains CPU usage per core and load averages, memory and swap,
machine-wide network throughput, the socket list, and the process table.

## Where the socket list comes from

**macOS** runs `/usr/bin/nettop -n -x -L 1` with a column list once per
sample. nettop prints every socket with its owning process, interface, TCP
state and cumulative bytes in and out. The kernel maintains those counters for
every socket and exposes them to any user, which is why there is no sudo. A
sample takes about 80 ms. Rates are the difference between two consecutive
samples divided by the interval.

nettop truncates process names, so whotop replaces them with the full name
from the process table using the pid. Sockets that share an identical
address pair, which happens with `SO_REUSEPORT`, are told apart by order.

**Linux** parses `/proc/net/tcp`, `tcp6`, `udp` and `udp6`, which list every
socket with its inode, and joins that to `/proc/<pid>/fd`, where each
process's open sockets appear as links named `socket:[inode]`. That gives the
owner of every socket without privileges. There are no byte counters in those
files.

## Where the words come from

- **Scope** (`localhost`, `your local network`, link-local, multicast) is
  decided from the address itself.
- **Company** comes from a hand-maintained list of IP ranges in
  `src/explain.rs`. It is deliberately small and favours the networks a
  developer laptop talks to all day. Pull requests adding ranges are welcome,
  with a source for the range.
- **Hostname** comes from a reverse DNS lookup. Lookups run on a separate
  thread with a cache so they never block drawing, and a row shows the IP
  until its name arrives.
- **Port names and descriptions** are a match table in the same file.
- **Interface labels** map names like `en0`, `utun3` and `lo0` to words.

## CPU, memory, processes

These come from the `sysinfo` crate. CPU per core, memory, swap, process
names, CPU and memory per process. whotop adds socket counts and network
rates per process by grouping the socket list by pid.

## The UI

`ratatui` for widgets and layout, `crossterm` for the terminal. Sparklines keep
240 samples of history. The layout is responsive: the detail panel, the
`where`, `state` and `age` columns, and the width of the process column all
depend on the terminal size.

## Cost

Measured on an M-series Mac at the default one-second interval: roughly 3 to
5 percent of one core, mostly the nettop call, and 11 to 12 MB of memory.
