# Clock Face Alarm And Timer Visibility Plan

## Purpose

This document turns the new face-level alarm and timer visibility work into an implementation-ready plan.

The guiding rule for this work is:

- preserve the existing clock face as the default interaction surface
- add active-item visibility without making the face feel busy or fragile
- keep each slice small enough to validate and ship independently
- prefer simple fallback behaviour over dense rendering on small clocks

## Delivery Model

Work is split into phases and slices.

- a **phase** is a major stage of the feature
- a **slice** is a small, reviewable unit that should be safe to complete and ship independently

## Current Constraints

The current code shape creates four immediate boundaries for this feature:

- `AlarmManager` already owns the live alarm list and active-item counting, but it does not yet expose a face-oriented projection of pending items
- `ClockFace` currently renders only theme and time state, with no alarm/timer data or hover state
- the canvas `update` path only publishes drag and context-menu actions, so hover details require new pointer-state handling
- small clock sizes already constrain numerals and date layout, so any new overlay must degrade gracefully below the current medium-size default

## UX Targets

The face-level visibility feature should satisfy these user-facing outcomes:

- active timers and alarms are visible from the main clock window without opening the alarms panel
- the default presentation stays compact and readable at a glance
- additional detail appears only when the pointer is over a specific item affordance
- multiple active items collapse cleanly instead of overlapping text around the dial
- small clocks favour legibility over parity with larger layouts

## Phase 0 — Data And Rendering Boundary

### Intent

Define the smallest shared data model and drawing seam needed to show active reminders on the clock face without committing to final hover behaviour or dense layout rules.

### Slices

1. Introduce a compact face-visible reminder view model derived from active alarms and timers
2. Thread that view model into `ClockFace` without changing current face rendering when there are no active items
3. Establish a dedicated overlay drawing seam separate from the static face and hands

### Slice Status

Phase 0 is now completed by the current active-item projection, app-to-face wiring, and dedicated overlay seam.

The clock face now receives a sorted, compact summary list of active alarms and timers, and rendering has a dedicated overlay seam above the face and hands. The seam is still intentionally inert, so there is not yet any visible summary lane.

### Exit Criteria

- the clock face can receive a stable list of active reminder summaries
- the new seam does not alter existing rendering when no items are active
- follow-on slices can change overlay layout without revisiting app-state ownership

## Phase 1 — Summary Lane Baseline

### Intent

Add the first user-visible on-face summary lane with a conservative layout that works for one or two active items and keeps the rest of the clock readable.

### Slices

1. Choose and implement the baseline lane position and typography rules for medium and large clock sizes
2. Render the first active item with a concise summary made from label, kind, or remaining time as appropriate
3. Extend the lane to show a second visible item and a count-based overflow indicator for the remainder

### Slice Status

Phase 1 is now completed by the current bottom-centre summary lane for medium and large clocks.

The lane now shows up to two active reminders with a compact `+N more` overflow indicator on the last visible line, while small clocks still omit the lane entirely.

### Exit Criteria

- at least one active timer or alarm is visible on the clock face
- the lane remains legible against existing themes
- the lane does not collide materially with the date block or centre-hand readability on standard sizes
- overflow is represented intentionally instead of by clipped or overlapping text

## Phase 2 — Hover Details And Hit Testing

### Intent

Layer in pointer-aware detail without weakening the existing drag and context-menu interactions on the clock face.

### Slices

1. Add face-level hit targets for summary-lane items and overflow affordances
2. Track hover state in the canvas program and invalidate rendering only when the hover target changes
3. Render a compact hover detail treatment that shows the fuller label and timing detail for the targeted item

### Slice Status

Phase 2 is now completed by the current hit-region model, canvas hover-state tracking, and bounded hover-detail treatment.

The clock face can now distinguish visible item rows and the overflow suffix as separate logical targets, track the hovered target with redraws only when that target changes, and render a compact on-face hover detail panel. Overflow hover now shows aggregate detail instead of trying to enumerate hidden items.

### Exit Criteria

- hover details work without breaking left-drag or right-click menu behaviour
- hover state stays stable as the clock ticks
- detail rendering remains bounded to the face and does not require a separate window

## Phase 3 — Small-Size Fallbacks And Polish

### Intent

Make the feature robust across the supported clock-size range, especially the smaller sizes where the baseline lane may not fit comfortably.

### Slices

1. Define size breakpoints for full lane, reduced lane, and minimal indicator modes
2. Implement the reduced and minimal fallback layouts for small clocks
3. Review theme contrast and edge cases such as long labels, near-expiry timers, and mixed alarm/timer sets

### Slice Status

Slices 1 and 2 are now completed by the explicit overlay layout-mode selector plus the first reduced and minimal fallback treatments.

Medium and large clocks keep the existing full summary lane. Intermediate radii now fall back to a single-line reduced lane that still supports item hover and overflow targeting, while smaller radii show a compact active-count indicator instead of dropping the overlay entirely.

### Exit Criteria

- small clocks use an intentional fallback instead of a squeezed full layout
- long labels and multiple items remain readable or degrade predictably
- the feature behaves consistently across existing built-in themes

## Guard Rails

The following constraints should hold across all slices:

1. No slice should change alarm firing logic or persistence unless the slice is explicitly about face-visible projection data.
2. No slice should combine hover behaviour and fallback-layout work unless the earlier summary-lane baseline is already stable.
3. Rendering changes should remain local to the clock face and app-state wiring, not the alarm panel UI.
4. Each slice should be small enough that format, clippy, tests, and a normal build can validate it before shipping.
5. Ship after each successful slice once the validation gates are green.

## Validation Expectations

Normal slice validation should use:

- `cargo fmt -- --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- `cargo build`

Release builds are not required for this feature unless the work is explicitly coupled to installation, packaging, or deployment.

## Manual Review Checklist

Each user-visible slice should be checked against these behaviours:

- the clock still drags correctly from non-overlay regions
- right-click still opens the settings window
- alarms and timers still fire as expected
- no overlay is shown when there are no active items
- one active item is readable at a glance
- multiple active items show deliberate overflow behaviour
- small clock sizes use the intended fallback mode

## Next Slice Recommendation

Proceed with Phase 3, Slice 3:

- review theme contrast across the new reduced and minimal treatments
- tighten edge cases such as long labels, near-expiry timers, and mixed alarm/timer sets
- decide whether any small-mode copy or sizing needs final polish before closing the feature

That keeps the next slice focused on polish and edge-case review now that all size tiers have a deliberate fallback.