# Rust Analog Clock Desklet – Copilot Instructions

## Project Overview

A classic analog clock desktop widget (desklet) for Linux, built with **Rust + iced**. Renders as a transparent, borderless window sitting on the desktop layer.

**Implementation plan:** See PLAN.md in the repo root for current tasks and progress.

## Project Structure

| Path | Purpose |
|------|---------|
| `src/main.rs` | Entry point, iced application setup, message loop |
| `src/clock_face.rs` | Canvas-based analog clock rendering |
| `src/theme.rs` | Clock colour and style definitions |
| `src/config.rs` | Configuration load/save (TOML, XDG paths) |
| `src/platform/` | Platform-specific window management (X11, Wayland) |
| `assets/` | Desktop entry, icons |
| `config/` | Default configuration template |
| `.github/` | Copilot instructions, TCTBP workflow |

## Development Commands

```bash
cargo build                    # Debug build
cargo run                      # Run in debug mode
cargo build --release          # Release build (optimised, stripped)
cargo test                     # Run unit tests
cargo clippy -- -D warnings    # Lint (zero warnings policy)
cargo fmt                      # Format code
cargo fmt -- --check           # Check formatting without modifying
```

**Rust edition:** 2021
**MSRV:** Stable Rust (latest stable recommended)

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `iced` (with `canvas`) | GUI framework + custom 2D canvas drawing |
| `chrono` | Local time access |
| `serde` + `toml` | Configuration serialisation |
| `directories` | XDG config path resolution |

## Code Patterns

### iced Application

- Use the free-function pattern: `iced::application(title, update, view)`
- Application builder with `.subscription()`, `.window()`, `.antialiasing()`
- Initialise with `.run_with()` for custom setup

### Canvas Rendering

- Implement `canvas::Program` for the clock face
- Use `canvas::Cache` for geometry caching
- Clear cache on each tick to redraw hands
- All drawing via `canvas::Frame` methods (`fill`, `stroke`, `fill_text`)

### Configuration

- TOML format at `~/.config/rust-clock/config.toml`
- Use `directories::ProjectDirs` for XDG path resolution
- Graceful fallback to defaults on missing/invalid config

### Platform Abstraction

- `src/platform/mod.rs` dispatches between X11 and Wayland
- X11: Set `_NET_WM_WINDOW_TYPE_DESKTOP` for desktop-layer behaviour
- Wayland: Use `wlr-layer-shell` for background layer

## Critical Rules

1. **Zero `clippy` warnings** – `cargo clippy -- -D warnings` must pass before any commit
2. **No `unsafe`** without documented justification and a `// SAFETY:` comment
3. **Regional language** – Use Australian English spelling in all UI text and comments (e.g., "colour", "centre", "initialise")
4. **Version sync** – Keep version aligned between `Cargo.toml` and git tags
5. **Keep files under ~300 lines** – Split by responsibility when files grow large
6. **Handle errors gracefully** – No panics in production paths; use `Result` and meaningful error messages

## File Organisation

- `src/main.rs` – App entry point, message types, iced wiring
- `src/clock_face.rs` – `ClockFace` struct, `canvas::Program` impl, drawing methods
- `src/theme.rs` – `ClockTheme` struct with colour definitions
- `src/config.rs` – `AppConfig` struct, TOML load/save, XDG paths
- `src/platform/mod.rs` – Platform detection and dispatch
- `src/platform/x11.rs` – X11-specific window hints
- `src/platform/wayland.rs` – Wayland layer-shell integration

## When Generating Code

- Prefer strict typing and exhaustive matches
- Use `iced` widget and canvas APIs idiomatically
- Handle loading, empty, and error states explicitly
- Keep drawing code clean — extract helper methods for face, hands, ticks
- Log errors meaningfully with `eprintln!` (no logging framework yet)
- Use `f32` for all drawing coordinates and angles
- Angles measured in radians, clockwise from 12 o'clock position

## Branch Naming

- `feature/<name>` – New features
- `fix/<name>` – Bug fixes
- `docs/<name>` – Documentation updates
- `infrastructure/<name>` – Build/CI/tooling changes

## Shipping Workflow

For SHIP/TCTBP activation, steps, approvals, and versioning rules, see [TCTBP Agent.md](TCTBP%20Agent.md).
