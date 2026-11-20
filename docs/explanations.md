# What the explanations mean

whotop replaces raw numbers with words wherever it can. This page is the
dictionary.

## "who": the other end

In order of preference, the **who** column shows:

1. **A scope**, when the address is not on the internet at all:
   - `this computer (localhost)`: the program is talking to another program
     on the same machine. A browser talking to a dev server, an app talking
     to a local database. Nothing leaves the computer.
   - `your local network`: another device on your Wi-Fi or LAN, like a
     printer, a phone, or a NAS.
   - `link-local (no router)`: an address that only works on the local link,
     common for IPv6 neighbour discovery and AirDrop.
   - `multicast (everyone on the LAN)`: a packet addressed to every device on
     the network, used for discovery (Bonjour, SSDP).
2. **A company**, when the IP falls in a well-known range. whotop ships a
   small hand-maintained table covering Apple, Google, Google Cloud, Google
   DNS, Cloudflare, Cloudflare DNS, GitHub, Fastly, Akamai, Amazon / AWS,
   Microsoft, Meta, Vercel, Telegram, Discord and Anthropic. A match means
   the address is operated by that company, not necessarily that the service
   you are using belongs to them: a lot of the internet is served from AWS
   and Cloudflare.
3. **A hostname**, from a reverse DNS lookup done in a background thread. It
   is shortened to the interesting part, so `ec2-3-5-140-2.compute-1.amazonaws.com`
   becomes `amazonaws.com (AWS)`.
4. **The IP address**, if nothing else is known. Long IPv6 addresses are
   trimmed in the table and shown in full in the detail panel.

## "what": the port

Ports are numbered doors. Most well-known services always use the same
number, so the number tells you what kind of conversation it is. whotop knows
these, with a one-line description and whether the traffic is normally
encrypted:

| port | name | normally |
|---|---|---|
| 443 TCP | HTTPS | encrypted web, APIs, app updates |
| 443 UDP | HTTP/3 QUIC | encrypted web over UDP, Google and Cloudflare sites |
| 80 | HTTP | unencrypted web, mostly redirects to HTTPS now |
| 53 | DNS | looking up a name to find its IP |
| 853 | DNS/TLS | encrypted DNS |
| 5353 UDP | mDNS/Bonjour | finding printers, AirPlay, other Macs nearby |
| 5223 | Apple push | iMessage, FaceTime and app notifications |
| 5228–5230 | Google push | Firebase Cloud Messaging |
| 22 | SSH | remote shell, git over SSH |
| 25 / 465 / 587 | SMTP | sending mail |
| 993 / 995 | IMAPS / POP3S | mail app syncing |
| 123 UDP | NTP | clock sync |
| 67 / 68, 546 / 547 | DHCP, DHCPv6 | getting an IP from the router |
| 1900 UDP | SSDP/UPnP | discovering TVs, speakers, routers |
| 3478 / 3479 / 19302 | STUN/TURN | video and voice call setup |
| 1194 / 51820 UDP | VPN | OpenVPN, WireGuard |
| 5432, 3306, 6379, 27017 | Postgres, MySQL, Redis, MongoDB | databases |
| 5672, 9092 | AMQP, Kafka | message queues |
| 3000 / 3001 | dev server | Next.js, Express, Rails defaults |
| 5173 | Vite dev | Vite |
| 4200 | Angular dev | Angular |
| 8000 / 8080 / 8888 | dev/proxy | local servers, proxies, Jupyter |
| 5000, 7000 | Flask/AirPlay, AirPlay | Flask, or AirPlay Receiver on macOS |
| 11434 | Ollama | local LLM server |
| 1883 / 8883 | MQTT / MQTTS | IoT messaging |
| 445, 548 | SMB, AFP | file sharing |
| 3389, 5900 | RDP, VNC | remote desktop |
| 49152–65535 | ephemeral | a random port the OS picked for this side |

Anything else shows as `tcp/1234` or `udp/1234`.

**Ephemeral ports** confuse everyone. When your browser connects to a
website, the website's side is port 443 and your side is a random high number
like 52314. The random one carries no meaning. whotop always labels the remote
port, and the detail panel says so when the local one is ephemeral.

## "encrypted"

The detail panel says `yes`, `no`, or `depends`:

- **yes**: the protocol on that port is always encrypted (HTTPS, SSH, IMAPS).
  Someone on the network can see *that* you talked to the host, not what you
  said.
- **no**: plain text on the wire (HTTP, DNS, mDNS). Normal for local network
  chatter, worth noticing for anything crossing the internet.
- **depends**: the port is used both ways, or the app decides. Database ports
  and `STARTTLS` mail are like this.

whotop infers this from the port number. It does not inspect traffic, so it
can be wrong for a program using a standard port in a non-standard way.

## TCP states

| state | in words |
|---|---|
| Established | open, data can flow both ways |
| Listen | waiting for incoming connections |
| SynSent | trying to connect, waiting for an answer. Stuck here means unreachable host or a firewall dropping packets |
| TimeWait | already closed. The OS keeps the slot for about a minute so late packets can't confuse a new connection. Harmless, and you will see many |
| CloseWait | the other side hung up but this app hasn't closed its end. Lots of these from one program can mean a leak |
| FinWait1 / FinWait2 | this side is closing and waiting for the other side to finish |
| LastAck / Closing | both sides finishing the close handshake |

UDP has no states. Packets are just sent, with no handshake and no guarantee
of arrival, which is fine for DNS, video and games.

## Interfaces

The `via` line names the network interface the socket uses, with a label for
the ones whotop recognises:

- **en0**: Wi-Fi on most Macs, the normal path to the internet.
- **lo0**: loopback, the computer talking to itself. Never leaves the machine.
- **utun**: a VPN tunnel. Traffic through here is encrypted to the VPN server
  regardless of the port.
- **awdl**: AirDrop and peer-to-peer Wi-Fi.
- **bridge**: virtual machine networking.

Other interfaces show their raw name. Linux interface names are shown as-is
for now.
