# Rust Analog Clock Desklet — Implementation Plan

## Current Version: 0.6.4

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

## Phase 3 — Interaction ✅

> Branch: `feature/interaction`

### Window dragging ✅

- [x] Click-and-drag moves the clock window (OS-level drag via `window::drag`)
- [x] Position saved to config on window move event
- [x] Load saved position on startup (`position = [x, y]`)

### Right-click context menu ✅

- [x] Custom iced overlay menu on right-click (centred `stack` panel)
- [x] Menu items:
  - [x] Theme picker (Classic / Dark / Minimal / Transparent buttons)
  - [x] Size adjustment (Small 150 / Medium 250 / Large 350)
  - [x] Toggle date display (✓ indicator)
  - [x] Toggle smooth second hand (✓ indicator)
  - [x] Quit (red-tinted button)
- [x] Menu dismissed on click-away (left-click starts drag) or Escape key
- [ ] Always on top toggle (deferred to Phase 4 — desktop layer)

### Settings dialog (stretch — deferred)

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

## Phase 6 — Alarm & Chime (core complete) ✅

> Branch: `feature/alarms-timers` | Tag: `v0.5.0`

- [x] Alarm data model — `AlarmKind` (AtTime, Timer), `AlertAction` (Sound, Notification, Both)
- [x] Alarm manager with persistence (`~/.config/rust-clock/alarms.toml`)
- [x] Quick timer presets (1m / 5m / 10m / 15m / 30m / 1h) from overlay panel
- [x] Audio playback via `rodio` (sine-wave beeps, custom sound file support)
- [x] Desktop notification via `notify-rust` when alarm fires
- [x] "Alarms & Timers" button in right-click context menu (with active count badge)
- [x] Separate overlay panel for alarm management
- [x] **v0.5.3**: Alarm form with text inputs (label, notification message)
- [x] **v0.5.3**: Timer mode (duration in minutes) and Alarm mode (specific time/date)
- [x] **v0.5.3**: Edit existing alarms via ✎ button — form populates from alarm
- [ ] Hourly chime — play a sound file at the top of each hour
- [ ] Configurable chime sound (path to `.wav` / `.ogg`, or system bell)
- [ ] Configurable chime hours (e.g. only 8:00–22:00)
- [ ] Visual indicator on the clock face when an alarm is set
- [ ] Snooze support (5-minute re-fire)
- [ ] Recurring alarms

### Face-level alarm visibility

- [ ] Add an on-face active-item summary lane centred above the `6` position
- [ ] Render active alarms and timers in a compact, glanceable format that does not compete with the hands or numerals
- [ ] Prioritise what is shown when space is limited:
  - imminently due items first
  - then remaining active timers
  - then future alarms
- [ ] Remove timers/alarms from the face summary immediately once they expire or fire
- [ ] Define a compact summary format for each visible item:
  - timer: short label plus remaining time (for example `Tea 04:12`)
  - alarm: short label plus target time (for example `Call 14:30`)
  - untitled items fall back to `Timer` / `Alarm`
- [ ] Cap the number of rendered items and add overflow handling such as `+2 more`
- [ ] Use distinct visual treatment for timers vs alarms without adding clutter
- [ ] Add size-aware fallbacks so small clock sizes collapse to fewer items or a count-only summary
- [ ] Ensure summaries remain legible across built-in themes and do not overlap the date or centre area

### Hover and callout behaviour

- [ ] Add hit-testing for each rendered face summary item so hover can target a specific alarm or timer
- [ ] Show a callout on mouseover with full item details:
  - type (`Alarm` / `Timer`)
  - full label
  - target date/time or remaining duration
  - notification message when present
  - alert mode (`Sound`, `Notification`, `Both`)
- [ ] Keep the callout as lightweight as possible: no dedicated settings window, no persistent state, no click actions
- [ ] Anchor the callout close to the hovered summary while keeping it inside the clock face bounds where possible
- [ ] Dismiss the callout immediately when the pointer leaves the clock face
- [ ] Make the callout strictly hover-only and read-only so it does not conflict with drag or right-click interactions
- [ ] Add keyboard-free behaviour only for now; detailed editing remains in the existing controls window

### Implementation approach

- [ ] Compute a dedicated face-summary view model in application state from the current `AlarmManager` data on each tick
- [ ] Keep rendering and interaction responsibilities separate:
  - `ClockApp` owns summary data, hover state, and callout state
  - `ClockFace` renders summaries and reports hover targets
  - the main clock window renders the callout inline as a lightweight hover element
- [ ] Extend canvas event handling to react to pointer movement without breaking existing drag and context-menu behaviour
- [ ] Add geometry bookkeeping for summary bounding boxes so hover can be resolved deterministically
- [ ] Treat the whole feature as ephemeral display state only; no summary or callout interaction should navigate to editing controls
- [ ] Reuse existing alarm formatting helpers where possible so panel and face descriptions stay consistent
- [ ] Add tests around summary ordering and formatting to keep behaviour stable as alarms/timers evolve
- [ ] Manually verify behaviour at multiple clock sizes and with overlapping states:
  - one timer
  - multiple timers
  - mixed timers and alarms
  - item expiring and disappearing immediately
  - no active items

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
| Alarm/timer visibility on the face | The clock should expose active time-sensitive items without forcing the controls window open; summaries stay compact and details appear on hover. |
- Animated theme transitions
