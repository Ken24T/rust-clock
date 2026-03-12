# Rust Clock User Guide

## Welcome

Rust Clock is a small desktop clock for Linux with built-in alarms and timers. It is designed to sit quietly on the desktop, stay out of the way, and still be easy to adjust when you need it.

You can use it as:

- a live analog clock
- a simple timer for short reminders
- a basic alarm for a specific time later today or on another date

## Getting Started

Start the app with:

```bash
cargo run
```

If you want the faster day-to-day version, use:

```bash
cargo run --release
```

When the app starts, it tries to:

- open the clock where you last left it
- restore your saved settings
- restore any saved alarms and timers
- start a tray icon if your desktop supports one

## Using The Clock

The main clock window shows:

- the analog clock face
- hour and minute hands
- an optional second hand
- an optional weekday and date display
- an active reminder summary on medium and large clocks when a timer or alarm is running

### Basic Controls

- Left-click the clock face to drag it somewhere else on the desktop
- Right-click the clock face to open the settings window
- Press `Escape` to close the current settings or alarms window
- Press `Ctrl+Q` to quit the app

When you move the clock, its position is saved automatically.

## Settings Window

Right-clicking the clock opens a small settings window. This is where most everyday changes happen.

### Choose A Theme

The built-in themes are:

- `classic`: a traditional white clock face with dark hands
- `dark`: a darker look with lighter hands
- `minimal`: a cleaner style with less visual detail
- `transparent`: a lighter, more see-through look for the desktop

When you choose a theme, the clock updates straight away and saves the change for next time.

### Change The Size

The quick size options are:

- Small: `150`
- Medium: `250`
- Large: `350`

If you prefer, you can also set another size manually in the config file anywhere from `50` to `500`.

### Show Or Hide Details

You can turn these on or off from the settings window:

- Show Date
- Smooth Seconds
- Show Seconds

`Smooth Seconds` makes the second hand sweep more smoothly instead of stepping once each second.

### Open Alarms And Timers

The `Alarms & Timers` button opens the reminder window. If you already have active alarms or timers, the button shows how many are running.

## Alarms And Timers

Rust Clock includes a simple reminder panel for countdown timers and clock alarms.

If you have an active reminder and the clock is at Medium or Large size, the main clock face now shows the first active item as a short summary near the bottom of the dial.

### Quick Timers

If you want something fast, you can start one of these straight away:

- 1 min
- 5 min
- 10 min
- 15 min
- 30 min
- 1 hour

### Create A Timer

To create your own countdown timer:

1. Open `Alarms & Timers`.
2. Leave the mode set to `Timer`.
3. Add a label if you want one.
4. Add a message if you want one.
5. Enter the number of minutes.
6. Select `Add`.

Examples of labels:

- Tea
- Laundry
- Stretch break

### Create An Alarm

To set a reminder for a specific time:

1. Open `Alarms & Timers`.
2. Switch the mode to `Alarm`.
3. Add a label if you want one.
4. Add a message if you want one.
5. Enter the time as `HH:MM`.
6. Optionally enter the date as `YYYY-MM-DD`.
7. Select `Add`.

If you leave the date empty, Rust Clock uses today.

### Edit Or Remove A Reminder

- Select `✎` to load an existing timer or alarm back into the form
- Select `✕` to remove a timer or alarm
- Select `Clear Done` to clear reminders that have already fired

The list in the panel shows:

- the label
- how much time is left, or whether it is done
- whether it is a timer or an alarm
- a short preview of the message if one was added

### When A Reminder Goes Off

When a timer or alarm finishes, Rust Clock currently does two things:

- it plays the built-in beep sound
- it shows a desktop notification

At the moment, the reminder window does not offer separate sound-only or notification-only choices.

## Tray Icon

If your Linux desktop supports tray icons, Rust Clock can add one automatically.

From the tray menu, you can:

- open `Alarms & Timers`
- start a quick timer
- quit the app

Clicking the tray icon focuses the main clock window.

If no tray icon appears, the app can still be used normally.

## Where Settings Are Stored

Rust Clock saves its files here:

- `~/.config/rust-clock/config.toml` for app settings
- `~/.config/rust-clock/alarms.toml` for alarms and timers

In normal use, you do not need to edit these files by hand.

## Optional Manual Configuration

If you do want to customise the app more deeply, the main config file is `~/.config/rust-clock/config.toml`.

Available top-level settings include:

- `size`: clock size in logical pixels, from `50` to `500`
- `position`: saved clock position as `[x, y]`
- `theme`: built-in theme name
- `smooth_seconds`: `true` or `false`
- `show_date`: `true` or `false`
- `show_seconds`: `true` or `false`
- `theme_config`: optional full theme override section

### Example Custom Theme

```toml
theme = "dark"

[theme_config]
numeral_style = "dots"
hand_style = "skeleton"
border_width = 1.5
face_colour = [1.0, 1.0, 1.0, 0.08]
border_colour = [1.0, 1.0, 1.0, 0.30]
tick_colour = [1.0, 1.0, 1.0, 0.55]
numeral_colour = [1.0, 1.0, 1.0, 0.60]
hour_hand_colour = [1.0, 1.0, 1.0, 0.70]
minute_hand_colour = [1.0, 1.0, 1.0, 0.60]
second_hand_colour = [1.0, 0.35, 0.35, 0.70]
centre_dot_colour = [1.0, 0.35, 0.35, 0.70]
shadow_colour = [0.0, 0.0, 0.0, 0.15]
date_text_colour = [1.0, 1.0, 1.0, 0.50]
```

If `theme_config` is present, it overrides the named built-in theme.

## If Something Does Not Look Right

### No Desktop Notifications

Make sure `notify-send` is installed and available on your system.

### No Tray Icon

Some Linux desktop environments do not show tray icons in every session. The clock, settings window, and alarms panel should still work.

### Position Or Settings Are Not Being Saved

Check that Rust Clock can write to `~/.config/rust-clock/`.

### Wayland Looks Different From X11

That is normal for the current version. The X11-specific desktop placement features have not yet been replaced with full Wayland layer-shell support.
