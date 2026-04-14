# Rust Clock

Rust Clock is an analog clock widget built with Rust and [iced](https://iced.rs). Linux remains the reference platform today, with transparent desklet-style behaviour, alarms, timers, and tray controls. Windows is now treated as an early baseline target with a simpler floating-widget interpretation.

## Current Functionality

- Live analog clock face with hour, minute, and optional second hands
- Optional weekday and day-of-month display on the clock face
- Active reminder summary shown on the clock face for medium and large clocks, with compact multi-item overflow
- Hover reminder detail that stays readable off the dial on X11 and Windows, with inline detail on Wayland
- Transparent, borderless main window intended to sit below normal windows
- Drag-to-move clock placement with saved position
- Saved placement clamped back onto the nearest active monitor on restart when displays change
- Right-click settings window for theme, size, and display toggles
- Four built-in themes: Classic, Dark, Minimal, Transparent
- Separate clock opacity control so any built-in theme can be tuned from subtle transparency to fully opaque
- Optional custom theme configuration through TOML
- Preset-based size tuning with Small, Medium, and Large plus bounded relative adjustment
- Alarm and timer management panel with one-shot and recurring reminders, create/edit/delete controls, and quick presets
- Platform tray integration with focus, quick timer, alarm panel, and quit actions where supported
- Persistent configuration and alarm storage in the XDG config directory

## Platform Support

Rust Clock is still Linux-first, but the project is now being structured for a safe Windows/Linux split.

- Linux on X11 is the reference implementation today
- Linux applies X11 window hints to stay below other windows, skip the taskbar, skip the pager, and stay sticky across workspaces
- Wayland uses a dedicated layer-shell main clock surface on compositors that advertise layer-shell support, so the visible clock does not appear as a normal dock or taskbar application window there
- Wayland compositors without layer-shell support fall back to the existing safe tray-only startup path when tray support is available
- On that fallback path, choosing `Show Clock` opens a normal application window, so a taskbar or dock icon is expected
- Wayland still opens settings and alarms windows as normal compositor-managed windows, and keeps reminder hover detail inline on the clock face when the layer-shell path is active
- Windows currently uses a simpler floating-widget model: transparent borderless main window, movable clock, and always-on-top control windows
- Windows toast notifications are available through the platform layer
- Windows tray support is available for focus, alarms panel, quick timers, and quit
- Linux-style desktop-layer semantics are not implemented on Windows
- Tray support is currently implemented on Linux and Windows through separate platform backends

## Build And Run

### Prerequisites

- Rust stable toolchain
- A Linux desktop session for the full reference experience, or Windows for the early floating-widget baseline
- `notify-send` available on Linux if you want desktop alarm notifications
- Optional on Windows: `RUST_CLOCK_WINDOWS_AUMID` if you want to provide a packaged AppUserModelID instead of the development fallback used for toast notifications

### Commands

```bash
cargo build
cargo run
cargo build --release
cargo test
cargo clippy -- -D warnings
```

For interactive review runs, prefer:

```bash
bash ./scripts/run-dev-harness.sh
```

That launcher stops only stale instances of this repo's `target/debug/rust-clock` binary before starting a fresh debug session. It does not touch the installed runtime in `~/.local/bin`.

For a user-local install that also shows up reliably in desktop launchers, build the release binary and run:

```bash
./scripts/install-linux-user-local.sh
```

That install helper installs the desktop entry under `~/.local/share/applications` and writes absolute `Exec` and `TryExec` paths to `~/.local/bin/rust-clock`, which avoids launcher discovery issues on systems where `~/.local/bin` is not on the desktop session `PATH`.

## Windows Installer

Rust Clock now includes a Windows installer definition based on Inno Setup.

### Installer Prerequisites

- Windows with PowerShell
- Rust stable toolchain
- Inno Setup 6

You can install Inno Setup with `winget install JRSoftware.InnoSetup`.

### Build The Installer

```powershell
pwsh -File .\installer\windows\build-installer.ps1
```

This script will:

- build `target\release\rust-clock.exe`
- compile the installer from `installer\windows\rust-clock.iss`
- place the versioned setup executable under `dist\windows\`

The installer uses a per-user install directory under `%LocalAppData%\Programs\Rust Clock`, creates a Start Menu shortcut, and can optionally add desktop and startup shortcuts.

## Everyday Use

- Left-click the clock face to drag and reposition the widget
- Right-click the clock face to open the settings window
- Press `Escape` to dismiss the current control window
- Press `Ctrl+Q` to quit

### Settings Window

The right-click settings window currently lets you:

- switch between the built-in themes
- adjust clock opacity independently of the selected theme
- choose Small, Medium, or Large size presets
- fine-tune the selected size preset up or down in bounded 10% steps
- toggle the date display
- toggle smooth seconds
- toggle second-hand visibility
- open the Alarms & Timers panel
- close the menu or quit the app

### Alarms And Timers

The alarms panel supports:

- quick timer presets: 1 min, 5 min, 10 min, 15 min, 30 min, 1 hour
- custom countdown timers entered in minutes, either once or on a repeating interval
- fixed alarms for a specific local time and optional date
- recurring alarms for daily, weekdays-only, weekly, and custom weekday schedules
- labels and optional notification messages
- pause and resume controls for running reminders directly in the active list
- editing existing alarms and timers
- deleting entries
- clearing fired items

When an alarm fires, the current app behaviour is to play the built-in generated beep pattern and send a desktop notification.

When you quit Rust Clock, active timers and alarms pause instead of continuing in the background. On the next launch, countdown timers and repeating interval timers resume from the remaining time they had when you quit, while one-shot alarms that were missed during shutdown are treated as missed rather than firing late.

While Rust Clock is running, countdown timers and repeating interval reminders also keep lightweight restart snapshots on disk. That means an unexpected stop or crash can usually recover close to the last saved remaining time instead of dropping those reminders entirely.

When reminders are active, the clock face keeps a compact summary on the dial. On X11 and Windows, hovering those reminders opens a separate reminder detail window instead of layering long callouts over the clock face itself. On Wayland with layer-shell support, the same detail is shown inline on the clock face.

Paused reminders stay available in the alarms panel for quick resumption, but they are removed from the live clock-face reminder summary until resumed.

The alarms panel now separates currently running reminders from paused ones and shows compact running/paused counts so it is easier to resume the right item without mixing it into the live list.

If multiple reminders are paused, the panel also offers a `Resume All` action so you can restore the live list in one step.

Likewise, the panel offers `Pause All` for currently running reminders, while reminders that are already done no longer show a misleading pause action.

The detached reminder detail surface now follows low-contrast face themes more closely, including the transparent and minimal built-in looks.

### Tray Menu

When the tray icon is available, it provides:

- focus/raise the clock window
- open the Alarms & Timers panel
- start quick timers
- quit the app

On Wayland compositors with layer-shell support, the main clock opens as a desktop-layer surface. On compositors without that protocol, Rust Clock falls back to tray-only startup when tray support is available, and `Show Clock` reopens the clock as a normal application window. In either case, use the tray icon or the tray menu's `Show Clock` action to reopen the clock after closing it.

## Configuration Files

Rust Clock stores its data under the XDG config directory:

- `~/.config/rust-clock/config.toml`
- `~/.config/rust-clock/alarms.toml`

Example configuration:

```toml
size = 250
size_preset = "medium"
size_adjust_percent = 0
opacity_percent = 100
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
- [docs/windows-installer.md](docs/windows-installer.md) for Windows installer build and install notes
- [PLAN.md](PLAN.md) for implementation status and planned work

User-facing and supporting documents now live under `docs/` unless they are part of the core repo entry points.

## Current Limitations

- No hourly chime, snooze, or multi-clock support yet
- No dedicated settings dialog beyond the current control windows
- Wayland auxiliary windows still use normal compositor-managed windows; only the main clock uses desktop-layer behaviour on compositors that advertise layer-shell support
- Windows tray support uses a platform-specific backend and should be treated as an early baseline rather than final polished packaging behaviour
- Linux remains the only platform with desklet-style taskbar, pager, and workspace integration, using X11 hints or Wayland layer-shell depending on the session
- Windows notifications currently use a development-friendly AppUserModelID fallback unless `RUST_CLOCK_WINDOWS_AUMID` is set by packaging or the runtime environment

## Licence

MIT
