# Rust Analog Clock Desklet — Implementation Plan

## Current Version: 0.1.0

**Target:** A highly customisable, moveable analog clock that sits on the Linux
desktop as a transparent widget. Primary DE: Cinnamon (Linux Mint). Deployed as
a standalone Rust binary using iced — no JavaScript, no DE-specific plugin APIs.

---

## Phase 0 — Project Scaffold ✅

> Branch: `main` | Tag: `0.1.0`

- [x] Git repository initialised
- [x] Cargo.toml with dependencies (iced, chrono, serde, toml, directories)
- [x] .gitignore, README.md, PLAN.md
- [x] Copilot instructions rewritten for Rust project
- [x] TCTBP.json updated with Cargo commands

---

## Phase 1 — Clock Rendering

> Branch: `feature/clock-rendering`

### 1a — Basic face (done)

- [x] iced application entry point (`src/main.rs`)
- [x] Clock face canvas rendering (`src/clock_face.rs`)
  - [x] Face circle with semi-transparent background
  - [x] 60 tick marks (hour marks thicker)
  - [x] Arabic numerals (1–12)
  - [x] Hour hand (short, thick)
  - [x] Minute hand (medium)
  - [x] Second hand (thin, red accent)
  - [x] Centre dot
- [x] Theme/colour definitions (`src/theme.rs`)
- [x] Configuration management (`src/config.rs`)

### 1b — Rendering polish

- [ ] Smooth second hand — 60 fps sweep via `window::frames()` subscription
- [ ] Subtle drop shadow on clock hands for depth
- [ ] Date display on the clock face (day-of-month, toggleable in config)
- [ ] Configurable clock size from config (50–500 px, default 250)
- [ ] Verify transparent, borderless window renders correctly on Cinnamon

---

## Phase 2 — Customisation

> Branch: `feature/customisation`

### Theme system

- [ ] `ClockTheme` loadable from TOML config sections
- [ ] All colours configurable: face, border, ticks, numerals, hands, centre dot
- [ ] Face background alpha/opacity as a separate config value (0.0–1.0)
- [ ] Numeral style option: Arabic, Roman, dots-only, none
- [ ] Hand style option: classic, modern (tapered), skeleton

### Built-in themes

- [ ] Classic — white face, dark hands, red second hand
- [ ] Dark — dark face, light hands, cyan second hand
- [ ] Minimal — no numerals, thin markers, grey tones
- [ ] Transparent — no face fill, outline-only ticks, ghost hands

### Config-driven sizing

- [ ] Window size from config (`size = 250`)
- [ ] Minimum 50 px, maximum 500 px
- [ ] Proportional scaling of all drawing elements

---

## Phase 3 — Interaction

> Branch: `feature/interaction`

### Window dragging

- [ ] Click-and-drag moves the clock window
- [ ] Position saved to config on mouse release
- [ ] Load saved position on startup (`position = [x, y]`)

### Right-click context menu

- [ ] Custom iced overlay menu on right-click
- [ ] Menu items:
  - [ ] Theme picker (sub-menu with available themes)
  - [ ] Size adjustment (small / medium / large / custom)
  - [ ] Toggle date display
  - [ ] Toggle smooth second hand
  - [ ] Always on top toggle
  - [ ] Quit
- [ ] Menu dismissed on click-away or Escape

### Settings dialog (stretch)

- [ ] Separate iced window for advanced settings
- [ ] Live preview of colour / theme changes
- [ ] Opened from context menu → "Settings…"

---

## Phase 4 — Desktop Layer Integration

> Branch: `feature/desktop-layer`

### X11 (Cinnamon / Xfce / MATE)

- [ ] Set `_NET_WM_WINDOW_TYPE_DESKTOP` to pin below all windows
- [ ] Set `_NET_WM_STATE_SKIP_TASKBAR` + `_NET_WM_STATE_SKIP_PAGER`
- [ ] Make sticky across all workspaces
- [ ] Investigate click-through for the transparent face area

### Wayland preparation

- [ ] `iced_layershell` integration for wlr-layer-shell compositors
- [ ] Background layer, no keyboard focus
- [ ] Conditional compilation or runtime detection

---

## Phase 5 — Multi-Clock & Timezones

> Branch: `feature/multi-clock`

- [ ] CLI flag `--timezone <tz>` (e.g. `Australia/Sydney`, `UTC`, `US/Eastern`)
- [ ] CLI flag `--config <path>` for per-instance configuration
- [ ] Each clock instance is a separate process with its own window & config
- [ ] Default timezone: system local (current behaviour)
- [ ] Timezone label displayed on the face (toggleable)
- [ ] XDG autostart support — launch multiple clocks on login via a wrapper script

---

## Phase 6 — Alarm & Chime

> Branch: `feature/alarm-chime`

- [ ] Hourly chime — play a sound file at the top of each hour
- [ ] Configurable chime sound (path to `.wav` / `.ogg`, or system bell)
- [ ] Configurable chime hours (e.g. only 8:00–22:00)
- [ ] Alarm times in config — one or more `[[alarm]]` sections
- [ ] Desktop notification via `notify-rust` when alarm fires
- [ ] Visual indicator on the clock face when an alarm is set
- [ ] Snooze support (5-minute re-fire)

---

## Phase 7 — Packaging & Distribution

> Branch: `feature/packaging`

- [ ] XDG `.desktop` file for application menu and autostart
- [ ] Application icon (`assets/rust-clock.svg`)
- [ ] AppImage build
- [ ] `.deb` package for Debian/Ubuntu/Mint
- [ ] AUR PKGBUILD (Arch Linux)
- [ ] Flatpak manifest (stretch)
- [ ] Release binary builds via GitHub Actions
- [ ] Linux Mint Software Manager submission (stretch)

---

## Design Decisions

| Decision | Rationale |
|----------|-----------|
| Standalone binary over Cinnamon desklet | Keeps Rust performance, iced rendering, and DE portability. No JS/CSS rewrite. |
| Right-click overlay menu over system tray | Self-contained within the clock window. No dependency on tray protocol support. |
| Separate processes for multi-clock | Simpler than multi-window iced. Each instance has its own config, crash isolation. |
| X11 first, Wayland later | Cinnamon on Mint uses X11 by default. Wayland support is future-proofing. |
| Config file as source of truth | All customisation persisted in `~/.config/rust-clock/config.toml`. GUI settings write back to this file. |
| 60 fps smooth second hand | Visually premium. Uses `window::frames()` — only redraws when frame is requested, minimal CPU. |
- Animated theme transitions
