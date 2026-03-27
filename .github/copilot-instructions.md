# Rust Clock – Copilot Instructions

## Project Overview

Rust Clock is a Linux-first analog clock widget built with Rust and `iced`, with alarms, timers, tray integration, and desktop-widget behaviour on Linux. Windows is supported as an early cross-platform baseline with a simpler floating-widget interpretation and installer packaging.

Implementation status lives in `PLAN.md`. Feature-specific planning and design notes live under `docs/`.

## Current Structure

| Path | Purpose |
|------|---------|
| `src/main.rs` | `iced::daemon` entry point, window orchestration, message loop, workflow wiring |
| `src/clock_face/` | Clock face canvas rendering, drawing helpers, and reminder overlay layout |
| `src/alarm/` | Alarm model, persistence, and sound support |
| `src/alarm_panel.rs` | Alarm and timer control window UI |
| `src/context_menu.rs` | Settings/control window UI |
| `src/hover_panel.rs` | Detached reminder hover-detail window |
| `src/platform/` | Platform abstraction and OS-specific policy hooks |
| `src/tray.rs` | Tray service integration routed through platform support |
| `src/config.rs` | Config loading/saving and user preference state |
| `src/theme.rs` | Built-in themes, colour config, and window chrome styling |
| `src/app_icon.rs` | Embedded icon/resource helpers |
| `docs/` | User guide, Windows installer notes, platform plans, feature plans |
| `installer/windows/` | Windows installer script and Inno Setup definition |
| `assets/` | Desktop entry and related Linux app metadata |
| `.github/` | Copilot instructions, TCTBP runtime/workflow files, prompts, and optional hook assets |

## TCTBP Runtime Surface

The Rust Clock TCTBP runtime and workflow surface lives in:

- `.github/agents/TCTBP.agent.md`
- `.github/TCTBP.json`
- `.github/TCTBP Agent.md`
- `.github/TCTBP Cheatsheet.md`
- `.github/copilot-instructions.md`
- `.github/prompts/Install TCTBP Agent Infrastructure Into Another Repository.prompt.md`
- optional hook layer: `.github/hooks/tctbp-safety.json` and `scripts/tctbp-pretool-hook.js`

Keep these files aligned when the workflow or runtime entry points change.

The consolidated cross-repo application prompt is expected to be discoverable through the explicit local-only trigger `reconcile-tctbp <absolute-target-repo-path>`.

## Development Commands

```bash
cargo build                    # Debug build
cargo run                      # Run in debug mode
cargo build --release          # Release build for install/deploy/packaging work
cargo test                     # Run unit tests
cargo clippy -- -D warnings    # Lint (zero warnings policy)
cargo fmt                      # Format code
cargo fmt -- --check           # Check formatting without modifying
pwsh -File .\installer\windows\build-installer.ps1   # Windows installer packaging
```

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `iced` with `canvas` and `tokio` | GUI framework, daemon entry, and custom canvas rendering |
| `chrono` | Local time, reminder targets, and display formatting |
| `serde` + `toml` | Config and alarm serialisation |
| `directories` | XDG config path resolution |
| `rodio` | Generated reminder/alarm sound playback |
| `x11rb` | Linux X11 window policy and desktop-style hints |
| `ksni` | Linux StatusNotifier tray support |
| `winrt-notification`, `tray-icon`, `windows-sys` | Windows notifications, tray support, and platform integration |

## Code Patterns

### Application model

- Use `iced::daemon(...)` with free functions for title, update, and view wiring.
- Open the main clock window explicitly with `window::open(...)` during startup.
- Keep auxiliary UI in dedicated windows rather than piling more controls into the main canvas.

### Canvas and clock-face rendering

- Keep clock rendering in `src/clock_face/`.
- Use `canvas::Program` and `canvas::Cache` for drawing and redraw invalidation.
- Keep long layout and overlay logic split into helper modules instead of growing `main.rs`.
- Treat reminder summaries and hover-detail behaviour as clock-face concerns, not settings-window concerns.

### Platform boundaries

- Put OS-specific logic in `src/platform/` and keep policy selection centralised.
- Keep Linux-specific window-hint behaviour out of generic UI modules.
- Keep Windows-specific installer and packaging logic in `installer/windows/` and related docs.

### Persistence and config

- Config lives in `~/.config/rust-clock/config.toml` on Linux through `directories::ProjectDirs`.
- Alarm state lives in `~/.config/rust-clock/alarms.toml`.
- Favour graceful fallback to defaults on missing or invalid user data.

## Critical Repo Rules

1. `cargo clippy -- -D warnings` must pass before any commit.
2. `cargo test` must pass before any SHIP or handover checkpoint.
3. Use Australian English in user-facing text and comments.
4. Keep `Cargo.toml` version and shipped tags aligned.
5. Preserve the repo’s plain semver tag convention such as `1.1.2` unless explicitly changed.
6. Do not introduce `unsafe` without a documented justification and a `// SAFETY:` comment.
7. Prefer focused files and extract modules when logic becomes hard to scan.
8. Keep `.github/agents/TCTBP.agent.md`, `.github/TCTBP.json`, `.github/TCTBP Agent.md`, `.github/TCTBP Cheatsheet.md`, `.github/copilot-instructions.md`, and any installed hook files aligned when workflow guidance changes.

## Documentation Expectations

When behaviour changes, review the repo docs that match the change:

- `README.md` for high-level shipped behaviour
- `docs/user-guide.md` for user-visible controls and workflows
- `docs/windows-installer.md` for Windows packaging/install behaviour
- `PLAN.md` for implementation status and remaining work
- feature-specific docs under `docs/` when a change lands in that feature area

## Branch Naming

- `feature/<name>` for new features
- `fix/<name>` for bug fixes
- `docs/<name>` for documentation updates
- `infrastructure/<name>` for build, CI, tooling, or workflow changes

## TCTBP Workflow

For SHIP, publish, handover, resume, deploy, status, abort, and branch transition rules, use:

- `.github/TCTBP.json` as the authoritative machine-readable profile
- `.github/TCTBP Agent.md` for workflow guard rails and interpretation
- `.github/TCTBP Cheatsheet.md` for quick operator guidance
- `.github/agents/TCTBP.agent.md` as the runtime trigger-routing entry point

For this repo, treat `cargo build` as the normal verification build and reserve `cargo build --release` for explicit installation, packaging, or deployment work.