# Platforms

| platform | status | per-socket byte rates |
|---|---|---|
| macOS, Apple Silicon | tested daily | yes |
| macOS, Intel | built in CI, not hand-tested | yes |
| Linux, x86_64 | compiles, lists sockets, not tested on real hardware yet | no |
| Linux, arm64 | same as x86_64 | no |
| Windows | not supported yet | |

## macOS

The full experience. Rates, totals and interface names for every socket come
from the kernel's own per-socket counters, the same source Activity Monitor
uses, and are readable without sudo. Tested on macOS 26 on Apple Silicon.

## Linux

The connection table, owning process, remote address, port, state, "who" and
"what" all work by reading `/proc`. What is missing is the **↓/s**, **↑/s**
and **total** columns, because Linux does not keep per-socket byte counters
anywhere an unprivileged program can read them. Getting them means packet
capture, which needs `CAP_NET_RAW` or root, and is on the list.

The machine-wide download and upload sparklines in the mem panel do work on
Linux.

If you run it on Linux, please
[open an issue](https://github.com/ivedmohan/whotop/issues) with what you
see, even if it is just "it works".

## Windows

Not yet. The socket collector has a macOS and a Linux implementation and
nothing for Windows, so the crate does not build there. The data is
available through the `GetExtendedTcpTable` and `GetExtendedUdpTable` APIs
and the terminal side already works on Windows Terminal, so this is a
contained piece of work.

## Terminal requirements

Any terminal that supports colour and Unicode box drawing. iTerm2, Terminal.app,
Ghostty, kitty, Alacritty, WezTerm, Windows Terminal and the VS Code terminal
all do. Columns hide themselves on narrow windows and come back above about
100 and 150 columns wide.
