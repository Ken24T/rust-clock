# Rust Analog Clock Desklet — Implementation Plan

## Current Version: 0.1.0

## Phase 0 — Project Scaffold

- [x] Git repository initialised
- [x] Cargo.toml with dependencies
- [x] .gitignore
- [x] README.md
- [x] PLAN.md
- [x] Copilot instructions rewritten for Rust project
- [x] TCTBP.json updated with Cargo commands

## Phase 1 — Basic Clock Rendering

- [x] iced application entry point (`src/main.rs`)
- [x] Clock face canvas rendering (`src/clock_face.rs`)
  - [x] Face circle with semi-transparent background
  - [x] 60 tick marks (hour marks thicker)
  - [x] Arabic numerals (1–12)
  - [x] Hour hand (short, thick)
  - [x] Minute hand (medium)
  - [x] Second hand (thin, red)
  - [x] Centre dot
- [x] Theme/colour definitions (`src/theme.rs`)
- [x] Configuration management (`src/config.rs`)
- [ ] Transparent, borderless window verified
- [ ] Build and verify renders correctly

## Phase 2 — Desktop Widget Behaviour

- [ ] Platform abstraction (`src/platform/`)
- [ ] X11: `_NET_WM_WINDOW_TYPE_DESKTOP` window hint
- [ ] X11: Skip taskbar and pager
- [ ] Wayland: `wlr-layer-shell` background layer integration
- [ ] Runtime platform detection and dispatch

## Phase 3 — Configuration & Polish

- [ ] Draggable window with position saving
- [ ] Config file hot-reload or restart detection
- [ ] Multiple theme support (classic, minimal, dark)
- [ ] Window size from config
- [ ] Command-line flags (`--help`, `--version`, `--config`)

## Phase 4 — Packaging & Distribution

- [ ] XDG desktop entry (`assets/rust-clock.desktop`)
- [ ] Application icon (`assets/rust-clock.svg`)
- [ ] XDG autostart support
- [ ] AUR PKGBUILD (Arch Linux)
- [ ] Release binary builds

## Future Ideas

- Smooth second hand (sub-second updates)
- Alarm/chime support
- Multiple timezone clocks
- Per-monitor positioning
- Animated theme transitions
