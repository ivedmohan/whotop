# Install

whotop runs on macOS (Apple Silicon and Intel) and Linux (x86_64 and arm64).
Windows is not supported yet, see [Platforms](platforms.md).

No sudo is needed for any of these.

## One line, any Mac or Linux

```sh
curl -fsSL https://github.com/ivedmohan/whotop/releases/latest/download/whotop-installer.sh | sh
```

The script detects your OS and CPU, downloads the matching prebuilt binary
from the GitHub release, verifies its checksum, and places it in
`~/.local/bin`. It adds that folder to your shell's PATH if it is not already
there. Open a new terminal, or run `export PATH="$HOME/.local/bin:$PATH"`,
then:

```sh
whotop
```

To install somewhere else, or to keep the script from touching your shell
config:

```sh
WHOTOP_INSTALL_DIR=/opt/tools/bin sh ./whotop-installer.sh
WHOTOP_NO_MODIFY_PATH=1 sh ./whotop-installer.sh
```

## Homebrew

```sh
brew install ivedmohan/tap/whotop
```

This uses a personal tap, a small repository of formulas at
`github.com/ivedmohan/homebrew-tap`, because whotop is too new for
homebrew-core. Updates arrive through `brew upgrade` like any other formula.

## cargo (Rust users)

```sh
cargo install whotop
```

Builds from source, which takes about a minute, and installs to
`~/.cargo/bin`. This is the path to use on a platform there is no prebuilt
binary for.

## Manual download

Grab a tarball from the
[releases page](https://github.com/ivedmohan/whotop/releases). Each file has
a `.sha256` next to it. Unpack it and put the `whotop` binary anywhere on your
PATH.

## Uninstall

| installed with | remove with |
|---|---|
| curl installer | `rm ~/.local/bin/whotop` |
| Homebrew | `brew uninstall whotop` |
| cargo | `cargo uninstall whotop` |

whotop writes no config files or caches.
