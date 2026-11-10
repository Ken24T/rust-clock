# Monitor placement recovery

Rust Clock now keeps the user's Linux/X11 monitor and top/right offsets through
startup, temporary display loss and reconnection. The absolute `position` remains
available for older settings and temporary fallback onto an active display.
Only a deliberate clock-face drag updates the chosen monitor and corner. A drag
that overlaps a monitor layout change is discarded rather than saving an
automatic desktop move.

The monitor snapshot is checked every two seconds for the lifetime of the app.
Automatic window moves also request a restore, so window-manager adjustments
after a layout notification cannot leave the clock on the wrong display. Failed
monitor queries preserve both the last valid snapshot and the saved preference.
The existing startup window-hint retries remain in place.

For example, on ARAGORN's portrait display:

```toml
[monitor_placement]
monitor = "DP-4"
right = 12
top = 12
```

The name is the X11 RandR monitor name. Moving a cable to a different connector
may change it; dragging the clock onto the intended display updates the saved
name. Windows and other backends retain their existing placement behaviour.

## Design review

Retain `src/main.rs` as the existing window/message orchestrator. The added
messages wire monitor checks and drag completion into that responsibility.
Monitor-relative coordinates and the recovery/drag state machine live together
in `src/placement.rs`, with display-independent regression tests. X11 monitor
and pointer queries remain in the Linux platform implementation. This keeps
display I/O out of the state machine without splitting the existing app flow.

The September local placement changes in `src/main.rs` were preserved in a
recovery copy before editing. Their protection against fallback placements
overwriting the user's anchor is strengthened by saving only deliberate drags.

## Verification

Regression tests cover a missing/reconnected monitor, recovery after the startup
period, changes to monitor coordinates and clock size, automatic window moves,
explicit drag completion, interrupted drags and settings serialisation. Runtime
acceptance also checks startup placement, automatic-move recovery and persistence
after restarting the installed app.

Run the existing alarm-manager unit suite with an isolated `XDG_CONFIG_HOME`:
some tests persist alarm fixtures through the normal save path. The original
empty alarm file was restored and verified byte-for-byte after detecting this
side effect during this change's initial test run.
