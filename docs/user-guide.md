# Rust Clock User Guide

## Welcome

Rust Clock is a small desktop clock for Linux with built-in alarms and timers. It is designed to sit quietly on the desktop, stay out of the way, and still be easy to adjust when you need it.

You can use it as:

- a live analog clock
- a simple timer for short reminders
- a basic alarm for a specific time later today or on another date
- a recurring reminder for daily routines, weekly events, and repeating intervals

## Getting Started

On Windows, install Rust Clock with the provided setup program.

After installation, you can start it from:

- the Start Menu entry: `Rust Clock`
- the optional desktop shortcut, if you chose that during install
- the optional startup shortcut, if you chose launch-on-sign-in during install

The installer places the app under `%LocalAppData%\Programs\Rust Clock` and keeps your normal user settings and alarms outside the install folder.

If you are using Rust Clock from source on a development machine instead of from the installer, refer to the repo-level setup notes in the main README.

When the app starts, it tries to:

- open the clock where you last left it
- restore your saved settings
- restore any saved alarms and timers
- start a tray icon if your desktop supports one

If your display layout changed since the last run, Rust Clock clamps the saved clock position back onto the nearest active monitor instead of blindly reopening at stale off-screen coordinates.

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

### Tune Opacity

The settings window also has an `Opacity` control that works independently from the theme preset.

That means you can use combinations such as:

- `dark` at `50%` opacity
- `classic` at `95%` opacity
- `transparent` at a slightly stronger or lighter opacity than its default look

Opacity changes apply live without closing the settings window.

### Change The Size

The quick size options are:

- Small: `150`
- Medium: `250`
- Large: `350`

After choosing one of those presets, you can fine-tune it up or down in `10%` steps from the settings window.

Examples:

- Small `+10%`
- Medium `-20%`
- Large `+30%`

Rust Clock keeps the size tuning within bounded sane ranges, and the settings window shows the current effective pixel size as feedback.

If you prefer, you can still inspect the resulting size value in the config file, with the effective clock size kept within `50` to `500`.

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

If you have an active reminder, the main clock face now keeps some reminder presence at every supported size. Medium and Large clocks show up to two active items as short summaries near the bottom of the dial. If more reminders are active, the last visible line shows a compact `+N more` overflow indicator.

Intermediate clock sizes switch to a reduced single-line summary so the face stays readable without dropping the overlay entirely. The smallest clock sizes show a compact active-count badge instead of item-by-item text.

Hovering a visible reminder summary opens a separate reminder detail window beside the clock instead of drawing the full callout over the dial. Hovering the `+N more` suffix shows aggregate detail for the hidden reminders in that same detached window, and the minimal count badge also uses the detached reminder window for its listed detail view.

That detached reminder surface is now blended to match the active clock-face theme more closely, so the transparent and minimal themes keep a lighter-touch overlay instead of switching to a strong opaque panel.

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
3. Choose `Once` for a one-shot timer or `Repeats` for a repeating interval timer.
4. Add a label if you want one.
5. Add a message if you want one.
6. Enter the number of minutes.
7. Select `Add`.

Examples of labels:

- Tea
- Laundry
- Stretch break

### Create An Alarm

To set a reminder for a specific time:

1. Open `Alarms & Timers`.
2. Switch the mode to `Alarm`.
3. Choose one of the schedule types:
	- `Once` for a one-shot alarm
	- `Daily` for every day
	- `Weekdays` for Monday to Friday
	- `Weekly` for one weekday each week
	- `Custom Days` for a selected set of weekdays
4. Add a label if you want one.
5. Add a message if you want one.
6. Enter the time as `HH:MM`.
7. If you chose `Once`, optionally enter the date as `YYYY-MM-DD`.
8. If you chose `Weekly`, select the weekday.
9. If you chose `Custom Days`, select one or more weekdays.
10. Select `Add`.

If you leave the date empty, Rust Clock uses today.

### Edit Or Remove A Reminder

- Select `Pause` to temporarily stop a running reminder without deleting it
- Select `Resume` to continue a paused reminder
- Select `✎` to load an existing timer or alarm back into the form
- Select `✕` to remove a timer or alarm
- Select `Clear Done` to clear reminders that have already fired

The list in the panel shows:

- the label
- how much time is left, or whether it is done
- whether it is a timer or an alarm
- the recurrence pattern when a reminder repeats
- a short preview of the message if one was added

Countdown timers show how much time was left when paused and resume from that point. Fixed-time alarms show that they were paused before their scheduled time, while recurring alarms show that their schedule is paused until resumed.

Running and paused reminders are shown in separate sections inside the panel so paused items stay manageable without looking like they are still live on the clock face.

If you have several paused reminders, use `Resume All` to move them all back into the running list at once.

If several reminders are still running, use `Pause All` to pause every eligible reminder together. One-shot reminders that are already done no longer offer a pause action.

### When A Reminder Goes Off

When a timer or alarm finishes, Rust Clock currently does two things:

- it plays the built-in beep sound
- it shows a desktop notification

At the moment, the reminder window does not offer separate sound-only or notification-only choices.

When you quit Rust Clock with `Ctrl+Q` or the tray `Quit` action, the app stops running rather than leaving reminders active in the background. Existing countdown timers and repeating interval timers resume from their remaining time when you start the app again. One-shot alarms that would have fired while the app was closed are marked as missed and do not fire late after restart.

Rust Clock now also keeps lightweight recovery snapshots for live countdown reminders while it is running. If the app stops unexpectedly, countdown timers and repeating interval reminders can usually resume close to the last saved remaining time instead of being lost altogether.

## Tray Icon

Rust Clock can add a tray icon when the current platform session supports it.

From the tray menu, you can:

- open `Alarms & Timers`
- start a quick timer
- quit the app

Clicking the tray icon focuses the main clock window.

If no tray icon appears, the app can still be used normally.

## Where Settings Are Stored

Rust Clock saves its files in your normal user configuration area.

On Linux, the files are:

- `~/.config/rust-clock/config.toml` for app settings
- `~/.config/rust-clock/alarms.toml` for alarms and timers

On Windows, the app uses the matching per-user configuration location resolved by the operating system.

In normal use, you do not need to edit these files by hand.

## Optional Manual Configuration

If you do want to customise the app more deeply, the main config file is `~/.config/rust-clock/config.toml`.

Available top-level settings include:

- `size`: clock size in logical pixels, from `50` to `500`
- `size_preset`: `small`, `medium`, or `large`
- `size_adjust_percent`: bounded relative adjustment around the selected preset
- `opacity_percent`: global clock opacity percentage
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

On Linux, make sure `notify-send` is installed and available on your system.

### No Tray Icon

Some desktop environments do not show tray icons in every session. The clock, settings window, and alarms panel should still work.

### Position Or Settings Are Not Being Saved

Check that Rust Clock can write to your normal per-user configuration directory.

### Wayland Looks Different From X11

That is normal for the current version. The X11-specific desktop placement features have not yet been replaced with full Wayland layer-shell support.
