# whotop

**Who is your computer talking to?**

whotop is a terminal monitor in the spirit of btop, except the main panel is
not CPU. It is every network socket on your machine: which program owns it,
who is on the other end (as a company or hostname, not just an IP), what the
port is normally used for, and how many bytes are moving right now. Select a
row and the side panel explains it in plain English.

It was built for people who open Activity Monitor or run `lsof -i`, see a wall
of IP addresses, and wonder what any of it means.

## What you get

- A live table of every connection, sorted by what is moving data right now.
- "who" instead of an IP: Apple, Cloudflare, Google, GitHub, your local
  network, or a hostname from reverse DNS.
- "what" instead of a port number: HTTPS, DNS, Apple push, Postgres, Vite dev.
- A side panel that explains the selected row: is it encrypted, what the
  connection state means, which network interface it uses, and how long it has
  been open.
- CPU, memory and a process table, so you don't need a second tool open.
- No sudo, no kernel extension, nothing sent anywhere. It only reads.

## Where to start

- [Install](install.md), one line on macOS or Linux.
- [First run](first-run.md), what you are looking at.
- [Keys](keys.md), everything you can press.
- [What the explanations mean](explanations.md), the "who", "what" and state
  text, and where it comes from.

whotop is open source under the MIT license.
Source: [github.com/ivedmohan/whotop](https://github.com/ivedmohan/whotop).
