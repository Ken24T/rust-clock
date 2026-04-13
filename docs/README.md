# Documentation

This folder holds supporting documentation for Rust Clock.

## Current Documents

- [user-guide.md](user-guide.md): end-user guide for everyday use, alarms, timers, tray behaviour, and optional manual configuration
- [windows-installer.md](windows-installer.md): Windows installer build, output, and installation behaviour
- [windows-cross-platform.md](windows-cross-platform.md): exploration notes for bringing Rust Clock beyond Linux, starting with Windows
- [platform-port-plan.md](platform-port-plan.md): phased porting plan, guard rails, capability matrix, and validation checklists for safe Windows/Linux work
- [platform-boundary-design.md](platform-boundary-design.md): Phase 1 design for platform service boundaries, ownership, fallbacks, and extraction order
- [clock-face-visibility-plan.md](clock-face-visibility-plan.md): phased plan for showing active alarms and timers directly on the clock face
- [code-risk-review-2026-04-13.md](code-risk-review-2026-04-13.md): read-only risk review and remediation plan for the `chore/bug-fixes-2` branch

## Structure

As a rule of thumb:

- keep `README.md` in the repo root as the main entry point
- keep `PLAN.md` in the repo root as the project status and implementation roadmap
- place user guides, install notes, architecture notes, packaging guides, screenshots, and similar supporting material in `docs/`

## Suggested Future Additions

- `packaging.md`
- `architecture.md`
- `screenshots/`