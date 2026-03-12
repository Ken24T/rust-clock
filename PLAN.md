# Rust Clock — Implementation Plan

## Current Version: 0.8.0

**Target:** A moveable Linux desktop clock that behaves like a lightweight desklet, with live analog rendering, saved placement, configurable appearance, and built-in alarms/timers.

---

## Phase 0 — Project Scaffold ✅

> Branch: `main` | Tag: `0.1.0`

- [x] Git repository initialised
- [x] Cargo.toml with primary dependencies
- [x] README and plan documents added
- [x] Rust/iced project structure established

---

## Phase 1 — Clock Rendering ✅

> Branch: `feature/clock-rendering`

### 1a — Basic face

- [x] iced daemon entry point and window setup
- [x] Canvas-based analog clock face
- [x] Circular face and border ring
- [x] 60 tick marks with stronger hour markers
- [x] Hour, minute, and second hands
- [x] Centre dot
- [x] Theme and colour definitions
- [x] Configuration-backed startup state

### 1b — Rendering polish

- [x] Smooth second-hand mode via high-frequency timer updates
- [x] Drop shadow on hands and centre dot
- [x] Optional weekday and day-of-month display
- [x] Configurable clock size from `50` to `500`
- [x] Proportional scaling of drawing elements
- [ ] Manual verification pass on Cinnamon desktop behaviour

---

## Phase 2 — Customisation ✅

> Branch: `feature/customisation`

### Theme system

- [x] `ThemeConfig` loaded from TOML
- [x] Built-in named themes
- [x] Full colour overrides for face, border, ticks, numerals, hands, centre dot, shadow, and date text
- [x] Numeral style selection: Arabic, Roman, dots, none
- [x] Hand style selection: classic, modern, skeleton
- [ ] Separate dedicated opacity field beyond RGBA theme colours

### Built-in themes

- [x] Classic
- [x] Dark
- [x] Minimal
- [x] Transparent

### Size customisation

- [x] Size persisted in config
- [x] Small, Medium, and Large presets in the UI
- [x] Arbitrary config-driven size within bounds

---

## Phase 3 — Interaction ✅

> Branch: `feature/interaction`

### Window interaction

- [x] Left-click drag to move the clock window
- [x] Position saved after moves
- [x] Position restored on startup

### Control windows

- [x] Right-click opens a dedicated settings/control window
- [x] Theme selection controls
- [x] Size selection controls
- [x] Date visibility toggle
- [x] Smooth-seconds toggle
- [x] Second-hand visibility toggle
- [x] Alarms & Timers entry with active-count label
- [x] Close and quit actions
- [x] Escape key dismisses the current control window

### System tray

- [x] Linux tray icon when StatusNotifier support is available
- [x] Tray activation focuses the clock
- [x] Tray menu opens the alarms panel
- [x] Tray quick-timer actions
- [x] Tray quit action

### Deferred interaction work

- [ ] Advanced settings dialog
- [ ] Live theme preview editor
- [ ] Additional keyboard shortcuts beyond Escape and Ctrl+Q

---

## Phase 4 — Desktop Layer Integration

> Branch: `feature/desktop-layer`

### X11 (current state)

- [x] Skip taskbar
- [x] Skip pager
- [x] Sticky across workspaces
- [x] Below-state hint for desktop-like stacking
- [ ] `_NET_WM_WINDOW_TYPE_DESKTOP` instead of current utility-style hints
- [ ] Click-through transparent regions

### Wayland

- [x] Safe runtime path that skips X11-only hinting
- [ ] Layer-shell integration
- [ ] Background-layer placement and focus rules

---

## Phase 5 — Alarms & Timers ✅

> Branch: `feature/alarms-timers` | Tag: `v0.5.0`

### Core functionality

- [x] Alarm model for fixed-time alarms and countdown timers
- [x] Persistent alarm storage in `~/.config/rust-clock/alarms.toml`
- [x] Quick timer presets
- [x] Form-based creation of timers and fixed alarms
- [x] Optional label and notification message
- [x] Editing existing alarms/timers
- [x] Deleting alarms/timers
- [x] Clear fired items action
- [x] Generated beep alarm sound via `rodio`
- [x] Desktop notification via `notify-send`

### Remaining alarm work

- [ ] Hourly chime
- [ ] User-selectable alarm sound files through the UI
- [ ] Configurable chime hours
- [ ] Visual indicator on the face for pending alarms
- [ ] Snooze
- [ ] Recurring alarms

### Face-level visibility

- [ ] On-face active-item summary lane
- [ ] Alarm/timer hover details
- [ ] Compact overflow handling for multiple items
- [ ] Size-aware layout fallback for small clocks

Phase/slice planning for this work now lives in [docs/clock-face-visibility-plan.md](docs/clock-face-visibility-plan.md).

---

## Phase 6 — Multi-Clock & Timezones

> Branch: `feature/multi-clock`

- [ ] CLI flag for alternate timezone
- [ ] CLI flag for alternate config path
- [ ] Separate instances with independent config/state
- [ ] Visible timezone label on the face
- [ ] Autostart support for multiple instances

---

## Phase 7 — Packaging & Distribution

> Branch: `feature/packaging`

- [x] Desktop entry file in `assets/`
- [ ] Application icon asset
- [ ] AppImage
- [ ] Debian/Ubuntu package
- [ ] AUR package
- [ ] Flatpak manifest
- [ ] Release automation

---

## Documentation Status

- [x] README updated for the current shipped feature set
- [x] User guide added for controls, alarms, tray behaviour, and configuration
- [ ] Release/install guide once packaging artefacts exist

---

## Design Decisions

| Decision | Rationale |
|----------|-----------|
| Standalone binary over a DE-specific desklet API | Keeps the app portable across Linux desktop environments while preserving Rust + iced rendering. |
| Separate control windows instead of in-canvas forms | Simplifies interaction handling and keeps the clock canvas focused on display and basic pointer gestures. |
| Config file as the source of truth | UI changes persist cleanly and can also be edited manually. |
| X11-first desktop integration | Matches the primary target environment while leaving room for future Wayland work. |
| Alarm/timer controls inside the app | Keeps reminders tightly integrated with the clock instead of depending on an external companion utility. |
