# Rust Clock

A classic analog clock desktop widget (desklet) for Linux, built with Rust and [iced](https://iced.rs).

## Features

- **Classic analog clock face** with Arabic numerals, tick marks, and three hands
- **Transparent, borderless window** that sits on your desktop
- **Lightweight** — minimal resource usage, single binary
- **Configurable** via TOML configuration file

## Building

### Prerequisites

- Rust stable toolchain (install via [rustup](https://rustup.rs))
- Linux with X11 or Wayland

### Build & Run

```bash
cargo run              # Debug build
cargo run --release    # Release build (recommended)
```

## Configuration

Configuration is stored at `~/.config/rust-clock/config.toml`:

```toml
size = 250
theme = "classic"
# position = [100, 100]  # Optional: fixed position
```

## Licence

MIT
