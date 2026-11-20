# Contributing and releases

## Building from source

Needs a Rust toolchain, 1.85 or newer for the 2024 edition.

```sh
git clone https://github.com/ivedmohan/whotop
cd whotop
cargo run
```

`cargo test` runs the unit tests for the explanation tables, the address
parsers and the nettop and `/proc` parsers. To type-check the Linux collector
from a Mac:

```sh
rustup target add x86_64-unknown-linux-gnu
cargo check --target x86_64-unknown-linux-gnu
```

## Layout

```
src/main.rs           argument parsing and the event loop
src/app.rs            state: selection, sort, filter, toggles, history
src/ui.rs             all drawing
src/explain.rs        ports, IP ranges, scopes, reverse DNS, number formatting
src/collect/mod.rs    the sampling thread and snapshot types
src/collect/net/      socket collectors: macos.rs (nettop), linux.rs (/proc)
docs/                 this documentation
```

## Good first contributions

- Add IP ranges to the company table in `src/explain.rs`, with a link to
  where the range is published.
- Add ports to the port table, with a one-line plain-English description.
- Test on Linux and report what you see.
- A Windows collector, see [Platforms](platforms.md).

## Releases

Releases are built by GitHub Actions using `cargo-dist`. To cut one:

1. Bump `version` in `Cargo.toml`, run `cargo update -p whotop`, commit.
2. Tag it: `git tag v0.1.2 && git push origin main v0.1.2`.
3. CI builds four binaries, the shell installer, checksums and the Homebrew
   formula, attaches them to a GitHub release, and pushes the formula to
   `ivedmohan/homebrew-tap`.
4. Run `cargo publish` for crates.io.

The workflow file is generated. Change `dist-workspace.toml` and run
`dist generate` instead of editing it by hand.
