# Command line options

```
whotop [--interval <seconds>] [--no-hints]
```

| option | default | meaning |
|---|---|---|
| `--interval <s>` | `1` | seconds between samples. Accepts decimals; clamped to the range 0.25 to 60 |
| `--no-hints` | off | start with beginner hints hidden; `b` toggles them at any time |
| `-h`, `--help` | | print usage and exit |

A shorter interval makes the rates twitchier and costs a little more CPU,
since each sample on macOS runs one `nettop` call. The footer shows how long
the last sample took, for example `sample 84ms`.

There is no config file. Everything is a key or a flag.
