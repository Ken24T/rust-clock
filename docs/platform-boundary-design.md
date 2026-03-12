# Platform Boundary Design

## Purpose

This document completes Phase 1 of the Windows/Linux port plan.

Its job is to define:

- the platform-specific service boundaries
- what remains in shared app logic
- what should move into platform modules later
- how unsupported features should degrade cleanly
- the extraction order for future code slices

This phase does not change runtime behaviour.

## Design Goals

The design is optimised for safety, not cleverness.

Priority order:

1. preserve current Linux behaviour
2. keep platform-specific code out of shared app flow
3. make Windows support incremental
4. treat unsupported features as explicit capability gaps
5. avoid abstractions that hide important platform differences

## Proposed Module Shape

The target structure should be:

- `src/platform/mod.rs`
- `src/platform/linux.rs`
- `src/platform/windows.rs`
- `src/platform/fallback.rs` or equivalent no-op implementation if helpful

This does not mean every service must be implemented immediately on every platform. It means there should be one clear home for platform-specific behaviour.

## Proposed Platform Services

### 1. Platform capabilities

Purpose:

- tell the app which platform-specific behaviours are available

Suggested responsibilities:

- tray support available or not
- native notifications available or not
- desktop/background-layer behaviour available or not
- sticky workspace behaviour available or not
- taskbar hiding available or not

Why this matters:

- the app should adapt based on capabilities instead of assuming Linux semantics everywhere

### 2. Main window policy

Purpose:

- describe how the main clock window should be configured and adjusted for the current platform

Suggested responsibilities:

- initial window level choice
- post-open native adjustments
- startup hint retry behaviour if needed

Current code that belongs here later:

- `main_window_settings`
- `apply_startup_window_hints`
- `apply_main_window_hints`
- `apply_linux_window_hints`
- X11 atom/hint helpers

### 3. Control window policy

Purpose:

- describe how settings/alarm windows should behave on the current platform

Suggested responsibilities:

- initial window level choice
- post-open focus/hint handling
- positioning adjustments if platform-specific work becomes needed later

Current code that belongs here later:

- `control_window_settings`
- `apply_control_window_hints`
- `apply_utility_window_hints`

### 4. Notification service

Purpose:

- deliver alarm notifications through a platform-native path when available

Suggested responsibilities:

- send summary/body notification
- report failure cleanly without crashing the app
- expose whether notifications are supported

Current code that belongs here later:

- `send_notification`

### 5. Tray service

Purpose:

- expose tray availability and command delivery without leaking backend details into shared app logic

Suggested responsibilities:

- start tray service if supported
- provide command receiver/channel for the app
- support clean shutdown

Current code that belongs here later:

- most of `src/tray.rs`

## Ownership Model

### Shared app layer should own

- app state
- messages and message handling
- theme selection
- clock rendering state
- alarm/timer state and persistence
- settings and panel UI
- formatting of user-visible alarm labels where not backend-specific

### Platform layer should own

- native window hints and special window behaviour
- tray implementation details
- notification delivery implementation
- platform capability reporting
- platform-specific startup hooks

### Shared app layer may coordinate but should not implement

- whether a platform service is invoked
- how tray commands map to app messages
- whether a capability changes what is shown or enabled in the UI

That means the app can still decide policy, but the platform layer should perform the native work.

## Current Responsibility Map

### In `main.rs`

Shared logic currently mixed with platform concerns:

- app initialisation and message loop
- control-window routing
- alarm/timer update handling
- platform-specific window setup
- platform-specific notification delivery

This file is currently the main place where separation is needed.

### In `tray.rs`

Current state:

- Linux implementation exists
- non-Linux path is a no-op fallback

This is closer to the desired separation already, but it still needs to be presented as a platform service rather than a Linux-first special case.

## Fallback Behaviour Rules

These rules are critical for a safe port.

### Rule 1: missing feature must be explicit

If a platform does not support a feature, the implementation should return a clear no-op or `unsupported` capability rather than attempting partial emulation.

### Rule 2: unsupported must still be stable

If tray or notifications are unavailable, the app must still run normally.

### Rule 3: unsupported does not imply hidden failure

The app should not silently assume that a platform integration succeeded when it was skipped.

### Rule 4: Linux remains the reference behaviour

Linux implementations should preserve the current user experience unless there is a deliberate product decision to change it.

## Windows Interpretation Policy

For Windows, the design should assume:

- transparent borderless movable widget is the baseline
- desklet/background-layer semantics are optional, not mandatory
- tray and notifications are desirable but not required for the earliest usable milestone
- unsupported Linux-only behaviour should remain unsupported on Windows until a safe implementation exists

This keeps the port honest and low-fragility.

## Phase 1 Slices

### Slice 1: Service boundary definition

Deliverables:

- service list
- ownership model
- capability model

Status:

- completed by this document

### Slice 2: Current-code extraction map

Deliverables:

- identify which functions move later
- identify which ones remain shared
- define the order of extraction to minimise Linux regression risk

Status:

- completed by the extraction plan below

### Slice 3: Fallback and Windows interpretation rules

Deliverables:

- explicit degradation policy
- Windows non-goal rules for unsafe parity

Status:

- completed by this document

## Planned Extraction Order

The safest future order is:

1. extract notification delivery behind a platform interface
2. extract startup/control window hooks behind a platform interface
3. move Linux-specific native hint code into a Linux platform module
4. adapt tray into a platform service interface
5. add explicit capability reporting
6. only then introduce Windows implementations

Why this order:

- notifications are narrowly scoped and easier to isolate first
- window hint extraction is higher-risk and should happen after one smaller success
- tray already has some separation and can be adapted after the other boundaries are clear

## Functions To Keep Shared

These should remain in shared app logic unless later evidence suggests otherwise:

- alarm form parsing and validation
- app message handling for shared behaviour
- config persistence
- clock-face drawing
- alarm/timer list management
- control-window content switching

## Functions To Move Behind Platform Services

These are the current candidates for later extraction:

- `main_window_settings`
- `control_window_settings`
- `apply_startup_window_hints`
- `apply_control_window_hints`
- `apply_main_window_hints`
- `apply_utility_window_hints`
- `apply_linux_window_hints`
- `apply_x11_window_hints`
- `intern_atom`
- `send_notification`
- tray startup/shutdown and backend-specific menu construction

## Phase 1 Exit Criteria

Phase 1 is complete when:

- platform services are defined conceptually
- ownership boundaries are documented
- fallback behaviour is documented
- extraction order is documented
- no runtime behaviour has been changed yet

## Summary

Phase 1 defines the shape of the safe port.

The key conclusion is that the app should gain a narrow platform-services layer, not a broad abstraction that tries to erase platform differences. Linux remains the reference implementation, Windows support is added incrementally, and unsupported behaviour is treated as an explicit capability gap rather than a half-working imitation.