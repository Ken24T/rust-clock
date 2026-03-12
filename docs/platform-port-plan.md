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