# Windows And Cross-Platform Exploration

## Purpose

This note captures the first pass at what needs to happen to move Rust Clock from a Linux-focused desklet to a genuinely cross-platform application, starting with Windows.

This is an exploration document only. It does not commit the project to a specific implementation yet.

## Current State

Today, the app is best understood as:

- a mostly cross-platform `iced` UI and canvas renderer
- a cross-platform config and alarm data model
- Linux-specific desktop integration around window placement, tray integration, and notifications

That split is useful because it means the clock rendering and alarm logic are already in reasonably portable shape, while the platform work is concentrated in a smaller set of features.

## What Already Looks Portable

These areas are good candidates to keep with minimal change:

- analog clock drawing and themes
- alarm/timer data model and persistence
- config loading and saving via `directories`
- alarm panel and settings window UI built with `iced`
- audio playback through `rodio`

In other words, most of the application logic is not the real blocker.

## What Is Linux-Specific Today

The main cross-platform blockers are:

### 1. Desktop-layer window behaviour

The app currently aims to behave like a Linux desklet:

- transparent and borderless
- below normal windows
- hidden from taskbar/pager
- sticky across workspaces

That exact behaviour does not map cleanly to Windows. Windows can support transparent and borderless windows, but the "desktop widget" behaviour will need a Windows-specific interpretation.

### 2. X11 window hints in the main app

The current implementation applies X11-specific window hints directly in the app code. Those calls need to move behind a clearer platform abstraction.

### 3. Tray integration

The tray implementation currently uses `ksni`, which is Linux-specific.

### 4. Desktop notifications

Alarm notifications are currently sent via `notify-send`, which is Linux-specific.

### 5. Product language and packaging

The current docs and metadata still describe the app as Linux-focused. That is accurate today, but it will need to evolve once Windows support is real.

## The First Big Product Decision

Before writing Windows code, the project should decide what the app is on Windows.

There are two realistic options:

### Option A: Same core app, platform-native behaviour

On Windows, Rust Clock would still be an always-available analog clock with alarms and timers, but the window behaviour would be adapted for Windows rather than forcing Linux desklet semantics.

That probably means:

- transparent or semi-transparent borderless window
- movable floating widget
- optional always-on-top or normal floating behaviour
- tray support
- Windows-native notifications

This is the most practical path.

### Option B: Try to reproduce full desklet semantics everywhere

This would attempt to keep the app below normal windows or attached to the desktop layer on every platform.

That is much riskier and more platform-specific. It is likely to cost a lot more for much less product value.

### Recommended direction

Option A is the better path. The app should keep the same identity and core features across platforms, but allow platform-specific window behaviour where necessary.

## Recommended Technical Direction

### 1. Introduce a real platform layer

The codebase already hints at a platform split in the instructions, but most of the current behaviour still lives directly in `main.rs`.

The next step should be to create a real abstraction for:

- startup window configuration
- platform-specific post-open window handling
- tray support
- notifications
- platform capability detection

Suggested shape:

- `src/platform/mod.rs`
- `src/platform/linux.rs`
- `src/platform/windows.rs`
- optional `src/platform/fallback.rs`

### 2. Separate core app behaviour from desktop integration

The core app should decide things like:

- current theme
- clock size
- alarms/timers
- whether a control window is open

The platform layer should decide things like:

- how a window is positioned or hinted
- how notifications are shown
- whether a tray icon exists

This separation will make Windows support much easier.

### 3. Define capabilities instead of assuming Linux behaviour

Instead of assuming every platform can do the same thing, define capabilities such as:

- supports tray
- supports background desktop layer
- supports platform notifications
- supports sticky-all-workspaces behaviour

Then the app can adapt the UX accordingly.

## Windows-Specific Workstreams

### Window behaviour

Questions to answer:

- Should the clock be always-on-top on Windows, or just a normal floating widget?
- Should there be a toggle for click-through or taskbar presence?
- Should Windows have a different default window level from Linux?

Recommended first implementation:

- transparent borderless movable window
- sensible default floating behaviour
- no attempt to emulate Linux desktop-layer placement in the first slice

### Notifications

The app needs a Windows-compatible notification path.

Possible direction:

- add a notifier abstraction
- implement Linux with `notify-send`
- implement Windows with a native crate or Windows notification API wrapper

### Tray

The app will need a cross-platform tray strategy.

Possible direction:

- abstract tray operations behind a small internal interface
- keep `ksni` for Linux
- add a Windows tray implementation separately

### Packaging

Windows support also means deciding how the app is shipped:

- release binary only
- zip bundle
- installer
- auto-start integration later

This is secondary to getting the app running well, but it should be tracked early.

## Likely Sequence Of Work

The safest order looks like this:

1. Refactor platform-specific behaviour out of `main.rs`
2. Add notification and tray abstractions
3. Make the app build cleanly with a Windows target
4. Get the main clock window running on Windows with acceptable default behaviour
5. Restore alarms, notifications, and tray support on Windows
6. Revisit packaging and user-facing documentation

## Definition Of A Good First Windows Milestone

A realistic first Windows milestone would be:

- app builds on Windows
- main analog clock window opens and redraws correctly
- window can be moved
- settings window opens
- alarms and timers still function internally
- audio alarm playback works
- Windows-specific tray and notification support may still be incomplete

That would be enough to prove the architecture is viable.

## Risks

The main risks are:

- spending too much effort trying to force Linux desktop semantics onto Windows
- mixing platform code deeper into app logic instead of extracting it
- choosing a tray or notification library that works on one platform but blocks another
- letting docs and product language drift while the platform story changes

## Suggested Next Branch Goals

For this exploration branch, the next concrete tasks should probably be:

1. document the desired Windows behaviour in product terms
2. extract the existing Linux-specific window and notification logic behind traits or modules
3. make the app compile cleanly with explicit platform fallbacks
4. only then start a minimal Windows implementation

## Summary

Making Rust Clock cross-platform looks feasible.

The core clock, config, alarms, and rendering are already close to portable. The real work is around desktop integration: window semantics, tray behaviour, notifications, and packaging. The best approach is to accept platform-native behaviour where needed, rather than trying to clone the Linux desklet model exactly on Windows.