# Rust Clock

Rust Clock is a Linux desktop analog clock desklet built with Rust and [iced](https://iced.rs). It runs as a transparent, borderless window with desktop-style behaviour and also includes alarms, timers, and tray controls.

## Current Functionality

- Live analog clock face with hour, minute, and optional second hands
- Optional weekday and day-of-month display on the clock face
- Transparent, borderless main window intended to sit below normal windows
- Drag-to-move clock placement with saved position
- Right-click settings window for theme, size, and display toggles
- Four built-in themes: Classic, Dark, Minimal, Transparent
- Optional custom theme configuration through TOML
- Alarm and timer management panel with create, edit, delete, and quick presets
- Linux system tray integration with focus, quick timer, alarm panel, and quit actions
- Persistent configuration and alarm storage in the XDG config directory

## Platform Support

Rust Clock is currently Linux-focused.

- X11 is the best-supported environment today
- The app applies X11 window hints to stay below other windows, skip the taskbar, skip the pager, and stay sticky across workspaces
- Wayland runs without the X11-specific hinting, but dedicated layer-shell support is not implemented yet
- The tray icon is available only where StatusNotifier/AppIndicator support is present

## Build And Run

### Prerequisites

- Rust stable toolchain
- A Linux desktop session
- `notify-send` available on the system if you want desktop alarm notifications

### Commands

```bash
cargo build
cargo run
cargo build --release
cargo test
cargo clippy -- -D warnings
```

## Everyday Use

- Left-click the clock face to start an OS-level drag and reposition the widget
- Right-click the clock face to open the settings window
- Press `Escape` to dismiss the current control window
- Press `Ctrl+Q` to quit

### Settings Window

The right-click settings window currently lets you:

- switch between the built-in themes
- choose Small, Medium, or Large size presets
- toggle the date display
- toggle smooth seconds
- toggle second-hand visibility
- open the Alarms & Timers panel
- close the menu or quit the app

### Alarms And Timers

The alarms panel supports:

- quick timer presets: 1 min, 5 min, 10 min, 15 min, 30 min, 1 hour
- custom countdown timers entered in minutes
- fixed alarms for a specific local time and optional date
- labels and optional notification messages
- editing existing alarms and timers
- deleting entries
- clearing fired items

When an alarm fires, the current app behaviour is to play the built-in generated beep pattern and send a desktop notification.

### Tray Menu

When the tray icon is available, it provides:

- focus/raise the clock window
- open the Alarms & Timers panel
- start quick timers
- quit the app

## Configuration Files

Rust Clock stores its data under the XDG config directory:

- `~/.config/rust-clock/config.toml`
- `~/.config/rust-clock/alarms.toml`

Example configuration:

```toml
size = 250
theme = "classic"
smooth_seconds = true
show_date = true
show_seconds = true
# position = [100, 100]

[theme_config]
numeral_style = "roman"
hand_style = "modern"
border_width = 2.0
face_colour = [0.12, 0.12, 0.15, 0.92]
border_colour = [0.40, 0.40, 0.45, 1.0]
tick_colour = [0.70, 0.70, 0.75, 1.0]
numeral_colour = [0.90, 0.90, 0.92, 1.0]
hour_hand_colour = [0.95, 0.95, 0.95, 1.0]
minute_hand_colour = [0.80, 0.80, 0.85, 1.0]
second_hand_colour = [0.10, 0.85, 0.85, 1.0]
centre_dot_colour = [0.10, 0.85, 0.85, 1.0]
shadow_colour = [0.0, 0.0, 0.0, 0.35]
date_text_colour = [0.70, 0.70, 0.72, 1.0]
```

`theme_config` overrides the named theme preset when present.

## Documentation

- [docs/README.md](docs/README.md) for documentation organisation
- [docs/user-guide.md](docs/user-guide.md) for a workflow-oriented user guide
- [PLAN.md](PLAN.md) for implementation status and planned work

User-facing and supporting documents now live under `docs/` unless they are part of the core repo entry points.

## Current Limitations

- No hourly chime, snooze, recurring alarms, or multi-clock support yet
- No on-face alarm summary or hover callout yet
- No dedicated settings dialog beyond the current control windows
- Wayland desktop-layer integration is still pending

## Licence

MIT
