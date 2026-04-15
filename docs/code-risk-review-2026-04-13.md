# Code Risk Review - 2026-04-13

Scope: read-only review of the current `chore/bug-fixes-2` branch on 13 April 2026.

Constraints for this pass:

- no runtime code changes were made
- findings were verified against the current branch where practical
- `cargo test` passed with 69/69 tests green
- `cargo clippy -- -D warnings` passed

Branch caveat:

- this branch was `0 ahead / 14 behind` `main` at review time
- some risks listed here may already be fixed on `main`
- before implementing fixes on this branch, compare each item with `main` and prefer merging or reusing existing fixes over re-solving the same bug locally

## Findings Summary

| ID | Severity | Area | Evidence | Risk summary |
| --- | --- | --- | --- | --- |
| R1 | High | Linux window geometry | `src/platform/linux.rs::work_area_for_point` | Uses the full X root screen instead of the active monitor work area, which can place the clock or popup windows off-screen on multi-monitor setups. |
| R2 | High | Startup and saved placement | `src/main.rs::WindowMoved`, `src/main.rs::main_window_settings`, `src/main.rs::apply_startup_window_hints` | Position is saved directly from move events and startup only applies initial settings plus hints, so stale or invalid coordinates can survive restarts and monitor changes. |
| R3 | High | Reminder resume logic | `src/alarm/mod.rs::resume_after_restart` | Countdown resumption is based on wall-clock timestamps, so DST jumps or manual clock changes can resume timers too early, too late, or immediately. |
| R4 | High | Reminder persistence on exit | `src/main.rs::Message::Quit`, `src/alarm/manager.rs::save_for_shutdown` | Shutdown snapshotting only happens on the explicit quit path, so crashes or forced termination can lose active countdown state. |
| R5 | Medium | Tick workload and long-run stability | `src/main.rs::subscription`, `src/main.rs::Message::Tick` | Smooth seconds runs at about 60 fps and each tick also checks alarms, refreshes active-item state, and updates hover windows. |
| R6 | Medium | Hover window state safety | `src/main.rs::update_hover_window` | Hover window opening relies on `expect(...)` for content state and can panic if message ordering or window-open behaviour drifts. |
| R7 | Medium | Tray lifecycle robustness | `src/main.rs::poll_tray_commands`, `src/tray.rs` Windows backend | Tray receiver disconnects and Windows shutdown paths are not fully hardened, so quit and teardown can be brittle after tray failure. |
| R8 | Low | User-visible failure reporting | `src/platform/linux.rs::send_notification`, `src/alarm/sound.rs`, `src/config.rs::save` | Notification, audio, and config write failures are mostly stderr-only, leaving users with silent degradation. |
| R9 | Low | Recurrence search bounds | `src/alarm/mod.rs::RecurrenceRule::next_after` | The 400-day search limit is arbitrary and can turn malformed or unusual schedules into silent `None` results. |

## Detailed Findings And Plans

### R1. Linux work-area detection ignores real monitor geometry

Evidence:

- `src/platform/linux.rs::work_area_for_point` returns the full X screen bounds, not the containing monitor or nearest active work area
- `src/main.rs::clamp_clock_position` and `src/main.rs::popup_position` trust that result for clock and popup placement

Why it matters:

- on asymmetric multi-monitor layouts, the clock can be clamped into invisible regions between monitors or beyond the visible desktop edge
- the same issue affects the settings window and hover detail window

Plan:

1. Replace the root-screen fallback with monitor-aware lookup using X11 RandR monitor data.
2. Select the containing monitor when the clock centre is inside one.
3. Fall back to the nearest monitor when the saved point is in a gap or on a removed display.
4. Keep a root-screen fallback only when monitor enumeration fails entirely.
5. Add Linux tests for containing-monitor, nearest-monitor, and monitor-gap cases.

Validation:

- automated tests around monitor selection
- manual verification on a three-monitor layout with gaps
- restart and suspend/resume checks for the main clock and popup windows

### R2. Saved position can remain invalid across restarts

Evidence:

- `src/main.rs::Message::WindowMoved` saves raw window coordinates directly to config
- startup opens the main window with `main_window_settings(...)`, then only retries desktop hints via `ApplyStartupHints`
- this branch does not have a dedicated startup layout reapply step after the main window opens

Why it matters:

- monitor changes, suspend/resume, or WM-reported out-of-bounds positions can persist a bad anchor
- a later launch may appear at the wrong size or off-screen even if the saved config is otherwise valid

Plan:

1. Clamp and normalise persisted main-window coordinates before saving them.
2. Introduce a single layout helper that computes both size and clamped position from config.
3. Reapply move and resize explicitly after opening the main window instead of relying on initial settings alone.
4. Reuse the same helper for startup, manual size changes, and any future restore path.
5. Add tests for off-screen saved coordinates and layout clamping.

Validation:

- save an invalid position in config and verify restart recovery
- resize, move, quit, restart, and confirm stable placement
- suspend/resume on the reference Linux setup

### R3. Resume-after-restart uses wall-clock time and is vulnerable to time jumps

Evidence:

- `src/alarm/mod.rs::resume_after_restart` computes remaining time from `target - paused_at`, then reapplies it relative to `resumed_at`
- both values are local wall-clock timestamps, not monotonic elapsed time

Why it matters:

- DST transitions and manual clock adjustments can distort elapsed time
- countdown timers and repeating intervals are the most exposed reminder types

Plan:

1. Add focused tests that simulate DST forward/backward transitions and manual clock jumps.
2. Change restart persistence for countdown reminders to store remaining seconds directly at shutdown, rather than recomputing from local timestamps on restart.
3. Keep one-shot calendar alarms on local wall-clock semantics, but make the distinction explicit in code and tests.
4. Decide the intended behaviour for large forward jumps and clock rollbacks, then encode that in tests before changing logic.

Validation:

- unit tests for DST forward jump, DST backward jump, and manual clock rollback
- manual smoke test with a paused timer across a controlled system-time change

### R4. Active reminder state is only snapshotted on explicit quit

Evidence:

- `src/main.rs::Message::Quit` calls `self.alarm_manager.save_for_shutdown()`
- there is no equivalent lifecycle snapshot on unexpected termination paths

Why it matters:

- the app cannot survive `SIGKILL`, power loss, or a crash cleanly with the current design
- the current shutdown model narrows the recovery window to graceful exit only

Plan:

1. Define the intended recovery guarantee first: graceful quit only, or best-effort recovery after abnormal exit.
2. If best-effort recovery is required, persist enough countdown state during normal operation so an extra shutdown snapshot is not the only recovery path.
3. Consider writing suspended countdown state when timers are created, paused, resumed, or materially updated.
4. Document any remaining hard limits, especially around `SIGKILL`.

Validation:

- forced termination test while a timer is running
- restart verification for both one-shot timers and repeating intervals

### R5. Smooth-seconds mode couples animation cadence to app work

Evidence:

- `src/main.rs::subscription` uses `Duration::from_millis(16)` when smooth seconds are enabled
- `src/main.rs::Message::Tick` performs alarm checks, face sync, and hover-window updates on every tick

Why it matters:

- the app does more than animation on each frame, so smooth seconds multiplies unrelated work
- this increases the chance of long-run CPU churn and timing-related UI issues

Plan:

1. Decide whether smooth seconds is a visual-only feature or a whole-app high-frequency mode.
2. If it is visual-only, separate animation refresh from reminder polling.
3. If one timer must remain shared, reduce the cadence and measure CPU before and after.
4. Add a manual overnight validation checklist for smooth-seconds mode.

Validation:

- overnight runtime on the Linux reference environment
- CPU sampling with and without smooth seconds
- hover and tray interactions during long-run idle time

### R6. Hover window opening depends on `expect(...)`

Evidence:

- `src/main.rs::update_hover_window` stores content in `self.hover_window_content`, then immediately re-reads it through `expect(...)` when opening the window

Why it matters:

- the panic path is narrow, but it is unnecessary and tied to transient UI state ordering
- this is the kind of bug that only appears under rapid pointer movement or odd window-open timing

Plan:

1. Remove the `expect(...)` dependency and build hover-window settings from the local `content` value already in scope.
2. Keep the state update as a separate assignment once the window open path is decided.
3. Add a focused regression test or manual repro steps for rapid hover open/close transitions.

Validation:

- repeated hover enter/leave cycles
- rapid move between summary rows and overflow indicators

### R7. Tray teardown is not fully hardened

Evidence:

- `src/main.rs::poll_tray_commands` drops the receiver on disconnect but does not clearly retire the tray handle at the same time
- the Windows tray backend in `src/tray.rs` ignores `PostThreadMessageW(...)` failure in `shutdown()`

Why it matters:

- Linux may limp along after tray failure, but shutdown behaviour becomes less predictable
- Windows is more exposed because tray lifetime is thread-driven and uses `unsafe` Win32 message-loop code

Plan:

1. Treat tray disconnect as a first-class degraded mode and retire all tray-side state consistently.
2. Make shutdown idempotent and resilient if the tray thread is already gone.
3. Add platform-specific manual teardown checks, especially for Windows rapid open/quit cycles.
4. Keep Linux and Windows failure modes documented separately because the implementations differ materially.

Validation:

- forced tray-backend failure followed by app quit
- repeated launch/quit cycles on Windows

### R8. User-visible error reporting is thin

Evidence:

- `src/platform/linux.rs::send_notification` only logs notification failures
- `src/alarm/sound.rs` only logs audio failures
- `src/config.rs::save` returns an error, while `src/main.rs::save_config` reduces that to stderr logging

Why it matters:

- reminders can appear to fail silently even when the app logic is still running
- settings changes can be lost without any persistent UI signal

Plan:

1. Decide which failures deserve immediate UI feedback versus debug logging only.
2. Add a lightweight user-visible diagnostics surface for persistence and notification failures.
3. Include at least one manual test each for missing `notify-send`, audio output failure, and config write failure.

Validation:

- remove `notify-send` from `PATH`
- simulate config write failure with an unwritable config directory
- verify that the user gets actionable feedback

### R9. Recurrence search limit is arbitrary

Evidence:

- `src/alarm/mod.rs::RecurrenceRule::next_after` searches only `0..=400` days ahead

Why it matters:

- normal schedules should still resolve, but malformed or future rule changes can turn this into a silent failure point

Plan:

1. Replace the magic number with a named constant and document why it exists.
2. Decide whether the bound should vary by recurrence type or disappear in favour of a logically guaranteed search strategy.
3. Add tests for corrupted or edge-case schedule data.

Validation:

- unit tests for empty or malformed weekday lists and long search windows

## Recommended Execution Order

1. Reconcile this branch with `main` and re-check which items still reproduce here.
2. Fix geometry and startup restore first (`R1`, `R2`) because they affect launch usability directly.
3. Harden reminder persistence and restart logic next (`R3`, `R4`) because they affect user trust and reminder correctness.
4. Reduce or separate smooth-seconds workload (`R5`) before doing further long-run stability testing.
5. Remove panic and teardown traps in transient UI and tray code (`R6`, `R7`).
6. Improve diagnostics and low-priority schedule robustness last (`R8`, `R9`).

## Areas That Look Relatively Healthy

- alarm and theme serialisation round-trip tests are present and currently passing
- config loading falls back to defaults cleanly on parse/read failure
- corrupted alarms files are backed up before the manager falls back to an empty state
- alarm form parsing already rejects several obvious invalid inputs before persistence

## Suggested Next Slice

If the next step is implementation rather than another review pass, use a narrow sequence:

1. merge or compare against `main`
2. land the geometry and startup restore fixes together
3. add restart-time tests before changing reminder persistence semantics
4. only then revisit long-run smooth-seconds behaviour
