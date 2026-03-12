# Windows/Linux Port Plan

## Purpose

This document turns the cross-platform exploration into an implementation-ready plan.

The guiding rule for this work is:

- preserve current Linux behaviour wherever it is already working
- separate platform-specific code clearly before adding Windows behaviour
- prefer omission over fragile parity when a Linux feature does not translate cleanly to Windows

## Delivery Model

Work is split into phases and slices.

- a **phase** is a major stage of the port
- a **slice** is a small, reviewable unit that should be safe to complete and ship independently

## Phase 0 — Guard Rails And Baseline

### Intent

Define the success criteria and safety boundaries before changing architecture or behaviour.

### Slices

1. Preservation targets, Windows acceptance targets, non-goals, capability matrix, and Linux validation checklist
2. Manual regression checklist for Linux runtime behaviour
3. Establish workflow guard rails for phased work, including validation and push policy

### Slice 1 Status

This document completes Slice 1.

### Slice 2 Status

This document completes Slice 2 through the Linux validation checklist below.

### Slice 3 Status

This slice is completed by the current workflow rules used for this branch:

- normal phase validation uses format, clippy, tests, and the standard build
- release builds are not required unless explicitly requested for installation or deployment work
- commits may happen within a phase as slices are completed
- branch pushes happen only after the full phase is complete

## Linux Preservation Targets

These are the behaviours that should be treated as the Linux baseline and must not regress without deliberate review.

### Main clock window

- transparent and borderless window
- opens at saved size and position
- desklet-like stacking behaviour on Linux
- remains movable by dragging the clock face
- saves new position when moved

### Control windows

- settings window opens correctly from the clock
- alarms/timers window opens correctly from the clock and tray
- control windows remain usable and focused
- Escape dismisses the current control window

### Clock behaviour

- clock face renders correctly
- theme selection works
- size changes work
- date and second-hand toggles work
- smooth second-hand mode works

### Alarm and timer behaviour

- quick timers work
- custom timers work
- fixed alarms work
- edit and delete actions work
- fired items can be cleared
- alarm sound playback works
- notification delivery works on Linux

### Platform integrations

- tray icon works when Linux tray support is available
- tray commands are received and handled correctly
- current Linux notification path works
- current Linux window hint behaviour still applies

## Windows Acceptance Targets

These define the first useful Windows milestone. They are intentionally narrower than Linux behaviour.

### Required for first Windows milestone

- app builds on Windows
- main clock window opens
- clock face renders correctly
- clock window can be moved
- settings window opens and closes correctly
- alarms/timers window opens and closes correctly
- config persists correctly
- alarms and timers still function internally
- alarm sound playback works

### Not required for first Windows milestone

- exact Linux desklet behaviour
- tray support
- native Windows notifications
- taskbar hiding
- sticky-all-workspaces behaviour
- click-through transparent regions

## Explicit Non-Goals For Early Windows Work

The following should not be forced into the first Windows implementation:

- exact X11-style below-all-windows behaviour
- pager/taskbar semantics copied from Linux
- sticky desktop-widget semantics across virtual desktops
- any native window hack that is difficult to explain or maintain
- any refactor that changes Linux UX and Windows behaviour in the same slice

If a feature cannot be implemented cleanly on Windows, it should remain unsupported there until a safe design exists.

## Capability Matrix

The app should move toward an explicit capability model rather than assuming every platform behaves like Linux.

### Capability: transparent borderless main window

- Linux: required
- Windows: required
- Notes: core product behaviour

### Capability: movable floating widget window

- Linux: required
- Windows: required
- Notes: this is the minimum shared widget experience

### Capability: desklet/background-layer placement

- Linux: required
- Windows: optional / unsupported initially
- Notes: Linux reference behaviour; Windows should not fake this unsafely

### Capability: hidden from taskbar/pager

- Linux: required where currently supported
- Windows: optional / unsupported initially
- Notes: do not treat as cross-platform baseline

### Capability: sticky across workspaces/desktops

- Linux: required where currently supported
- Windows: optional / unsupported initially
- Notes: platform-dependent and likely low-value early on

### Capability: tray support

- Linux: required
- Windows: optional for first milestone, desirable later
- Notes: should be abstracted as a service

### Capability: native alarm notifications

- Linux: required
- Windows: optional for first milestone, desirable later
- Notes: should be abstracted as a service

### Capability: alarm sound playback

- Linux: required
- Windows: required
- Notes: should remain part of baseline functionality

### Capability: saved config and alarm persistence

- Linux: required
- Windows: required
- Notes: already expected to be portable

## Working Rules For Implementation

To reduce regression risk while working from Windows:

1. No slice should change both Linux-specific behaviour and shared app behaviour unless the change is purely an extraction with no intended behaviour change.
2. Linux-specific code should move behind a platform interface before Windows implementations are added.
3. Unsupported Windows features should be represented as explicit capability gaps, not partial emulations.
4. Each slice should be small enough that its Linux regression risk can be reasoned about from code review alone.
5. Ship after each successful slice so the branch history stays granular and recoverable.

## Validation Policy For This Effort

Unless explicitly overridden for packaging, installation, or deployment work, the standard validation set for this cross-platform effort is:

- `cargo fmt -- --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- `cargo build`

`cargo build --release` is reserved for explicit release, installation, packaging, or deployment scenarios.

## Push Policy For This Effort

The current branch workflow for this effort is:

- slices may be completed and committed within a phase
- the branch should only be pushed after the whole phase is complete
- phase completion should include a brief summary of what the phase delivered

## Phase 2 — Narrow Service Extraction

### Intent

Move platform-sensitive behaviour behind explicit platform services without changing intended Linux behaviour.

### Slices

1. Extract notification delivery behind a platform module
2. Extract shared window-policy hooks behind a platform module
3. Move Linux native hint internals fully into the Linux platform implementation
4. Adapt tray startup behind the same platform-oriented boundary
5. Introduce explicit capability reporting for unsupported features

### Slice 1 Status

Completed.

### Slice 2 Status

Completed by moving shared window setting adjustments and post-open hint hooks into the platform module while preserving the existing Linux X11 behaviour.

### Slice 3 Status

Completed by moving the Linux window-hint internals into the Linux platform implementation instead of leaving them in the shared app entry file.

### Slice 4 Status

Completed by routing tray startup and shutdown through the platform boundary instead of importing the tray backend directly in the shared app flow.

### Slice 5 Status

Completed by introducing an explicit platform capability model and using it in shared app startup/subscription decisions for tray polling and window-hint retries.

## Phase 2 Summary

Phase 2 is complete.

It delivered:

- notification delivery behind a platform service
- window setting and hint hooks behind a platform service
- Linux X11 hint internals moved out of the shared app entry file
- tray startup routed through the platform boundary
- explicit capability reporting for unsupported platform features

## Phase 3 — Explicit Windows Baseline

### Intent

Turn Windows from a passive fallback target into an explicit early-runtime baseline with deliberate default window behaviour, while preserving Linux semantics.

### Slices

1. Move initial window level policy fully under platform ownership
2. Define the first Windows baseline window behaviour in code and docs
3. Refresh user-facing platform support wording to match the new baseline

### Slice 1 Status

Completed by removing Linux-style window levels from the shared app defaults and making each platform implementation set its own main and control window levels.

### Slice 2 Status

Completed by making Windows use a normal floating main window and always-on-top control windows instead of inheriting Linux desklet-style defaults.

### Slice 3 Status

Completed by updating the platform-support documentation to describe Windows as an early baseline build/runtime target rather than an unspecified non-Linux fallback.

## Phase 3 Summary

Phase 3 is complete.

It delivered:

- platform-owned initial window levels
- an explicit Windows default window policy
- updated platform-support wording for the current cross-platform state

## Phase 4 — Windows Notification Baseline

### Intent

Restore native notification delivery on Windows without disturbing Linux behaviour, while keeping the desktop-app identity limitation explicit.

### Slices

1. Add a Windows-only toast notification dependency
2. Implement Windows toast delivery behind the existing platform notification service
3. Document the temporary AppUserModelID fallback and how packaging can override it later

### Slice 1 Status

Completed by adding a Windows-target-only toast notification dependency instead of introducing a cross-platform notification crate into the shared dependency set.

### Slice 2 Status

Completed by implementing Windows toast delivery in the Windows platform module and marking the notifications capability as available there.

### Slice 3 Status

Completed by documenting the default desktop-app identity fallback and the `RUST_CLOCK_WINDOWS_AUMID` override for future packaged builds.

## Phase 4 Summary

Phase 4 is complete.

It delivered:

- Windows toast notification support behind the existing platform seam
- Windows notifications capability enabled in the platform capability model
- an explicit path for future packaged builds to supply a real Windows AppUserModelID

## Phase 5 — Windows Tray Baseline

### Intent

Restore basic tray controls on Windows without disturbing the Linux tray backend or pushing tray state into shared app logic.

### Slices

1. Add Windows-only tray and Win32 message-loop dependencies
2. Implement a Windows tray backend on its own message-loop thread
3. Expose the tray through the existing platform service and enable the Windows tray capability
4. Document the current Windows tray scope and remaining limitations

### Slice 1 Status

Completed by adding Windows-target-only tray and Win32 support dependencies instead of introducing new shared GUI dependencies for all platforms.

### Slice 2 Status

Completed by implementing a Windows tray backend that owns its menu and tray icon on a dedicated Win32 message-loop thread and forwards only `TrayCommand` values back to shared app logic.

### Slice 3 Status

Completed by wiring the Windows tray backend into the existing platform service and enabling the Windows tray capability.

### Slice 4 Status

Completed by updating the platform support documentation to describe the new Windows tray baseline and its remaining limitations.

## Phase 5 Summary

Phase 5 is complete.

It delivered:

- Windows tray support behind the existing tray/platform seams
- a dedicated Win32 tray thread that keeps tray state out of shared app logic
- Windows tray capability enabled in the platform capability model

## Phase 6 — Windows Integration Hardening

### Intent

Harden the new Windows integrations with deterministic helper tests and a concrete manual validation checklist, without changing intended runtime behaviour.

### Slices

1. Extract Windows-specific helper logic into directly testable functions
2. Add Windows-targeted unit tests for tray command mapping and toast AppUserModelID selection
3. Document a concrete Windows manual validation checklist for the current runtime baseline

### Slice 1 Status

Completed by extracting Windows tray menu ID parsing and Windows toast AppUserModelID selection into focused helper functions.

### Slice 2 Status

Completed by adding Windows-targeted unit tests around those helpers so the Windows platform layer has deterministic coverage for its key non-UI branches.

### Slice 3 Status

Completed by documenting the Windows manual validation checklist below.

## Phase 6 Summary

Phase 6 is complete.

It delivered:

- testable Windows helper functions for tray command mapping and toast AppUserModelID selection
- Windows-targeted unit coverage for those helpers
- a concrete manual validation checklist for the current Windows runtime baseline

## Windows Validation Checklist

This checklist should be used on a live Windows session after Windows-sensitive phases.

### Startup and windowing

- app launches successfully
- main clock window is transparent and borderless
- main clock window starts as a normal floating widget
- clock window can be dragged and new position is saved
- settings window opens and stays usable
- alarms/timers window opens and stays usable

### Notifications

- a fired alarm shows a Windows toast notification
- `RUST_CLOCK_WINDOWS_AUMID` override works when provided
- missing or blank `RUST_CLOCK_WINDOWS_AUMID` still falls back to the development toast path

### Tray

- tray icon appears
- left-click on the tray focuses the clock
- tray menu opens
- tray "Alarms & Timers" opens the panel
- tray quick timers create the expected timers
- tray quit exits the app cleanly

## Linux Validation Checklist

This checklist should be used later on a Linux environment after each platform-sensitive phase.

### Startup and windowing

- app launches successfully
- main clock window is transparent and borderless
- main clock window is positioned correctly
- main clock window shows expected desklet-like stacking behaviour
- control windows open in sensible positions

### Interaction

- left-drag still moves the clock window
- right-click still opens settings
- Escape still dismisses the active control window
- Ctrl+Q still quits

### Settings and rendering

- built-in themes still apply correctly
- size presets still resize the clock correctly
- date toggle still affects face rendering
- smooth seconds still updates as expected
- second-hand toggle still works

### Alarms and timers

- quick timers can be added
- fixed alarms can be created
- existing alarms can be edited
- items can be removed
- fired items can be cleared
- alarm sound is audible
- alarm notification is delivered

### Tray

- tray icon appears when supported
- tray menu items work
- tray quick timers work
- tray focus action works

## Exit Criteria For Phase 0

Phase 0 is complete when:

- preservation targets are documented
- Windows acceptance targets are documented
- non-goals are explicit
- capability expectations are explicit
- Linux validation checklist exists for later use
- validation and push policy for the phased work are explicit

### Phase 0 Status

Phase 0 is now defined as complete in planning terms. The remaining action is to validate, commit, and push the phase baseline on this branch.

## Phase 1 — Platform Boundary Design

### Intent

Define the platform service boundaries, ownership model, fallback rules, and extraction order before any runtime refactor begins.

### Slices

1. Service boundary definition
2. Current-code extraction map
3. Fallback and Windows interpretation rules

### Slice Status

Phase 1 is documented and completed by [platform-boundary-design.md](platform-boundary-design.md).

### Exit Criteria

- platform services are defined
- ownership boundaries are defined
- fallback behaviour is defined
- extraction order is defined
- no runtime behaviour changes are introduced in this phase

## Next Phase Preview

Phase 2 should begin code extraction with the narrowest safe service boundary first, ideally notifications or another small platform-specific seam, while preserving Linux runtime behaviour.

## Phase 2 — Narrow Service Extraction

### Intent

Begin code extraction by moving the smallest platform-specific seams behind explicit platform services without intentionally changing Linux runtime behaviour.

### Slices

1. Notification service extraction
2. Window-hook extraction
3. Linux native hint extraction into platform modules
4. Tray service adaptation
5. Capability reporting introduction

### Slice Status

Slice 1 is completed by the current notification-service extraction.

### Exit Criteria

- notification delivery is behind a platform-facing boundary
- Linux notification behaviour is preserved
- unsupported non-Linux platforms remain stable
- no broader runtime refactor is coupled to this slice