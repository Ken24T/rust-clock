# Rust Clock — TCTBP Cheatsheet

Short operator reference for the Rust Clock workflows.

Use this file for the quick view.
Use [TCTBP Agent.md](TCTBP%20Agent.md) for the full workflow rules and guard rails.

## Core Rule

- No code is ever lost while syncing local and remote state.
- Do not use destructive shortcuts as part of normal workflow execution.
- If a workflow hits divergence, ambiguity, failed verification, or stale release state, it should stop rather than guess.

## Repo Gates

Repo gates for this repository:

- Format check: `cargo fmt -- --check`
- Test: `cargo test`
- Lint: `cargo clippy -- -D warnings`
- Normal build gate: `cargo build`
- Release build: `cargo build --release`

Release-build rule:

- `cargo build --release` is for explicit installation, packaging, or deployment work.
- Normal SHIP uses the normal build gate by default.

## Version And Tags

- Version source: `Cargo.toml` field `package.version`
- Tag format: plain semver, for example `1.1.2`
- Do not normalise this repo to `v1.1.2` tags unless explicitly requested.

## Triggers

### `ship` / `ship please` / `shipping` / `prepare release`

Purpose:
Formal source release workflow.

Attempts to:

- preflight the repo state
- show a concise origin-vs-local snapshot table before mutating anything
- run verification gates
- confirm zero problems
- assess docs impact
- bump version when required
- commit the release changes
- create the version tag
- push the current branch

Notes:

- starts with a four-column table: `Origin`, `Local`, `Status`, `Action(s)`
- uses `cargo build`, not `cargo build --release`, as the default build gate
- patch bump happens on every ship unless the changes are docs-only or infrastructure-only
- first ship on a `feature/` branch gets a minor bump
- stops if the branch is dirty, missing an upstream, behind origin, diverged from origin, or on detached `HEAD`

### `handover` / `handover please`

Purpose:
Safely reconcile the working branch with `origin` so you can stop on one machine and resume on another.

Scope:

- syncs the active work branch
- syncs relevant tags when needed
- maintains `tctbp/handover-state`
- does not reconcile every branch in the repository
- does not merge into `main` as part of normal machine-to-machine sync

Handover metadata:

- branch: `tctbp/handover-state`
- file: `.github/TCTBP_STATE.json`

Notes:

- prefers valid handover metadata over arbitrary clean-branch recency
- can checkpoint dirty unpublished work before verification strands it
- fast-forwards when behind and clean
- stops on divergence or ambiguity
- ends with a concise four-column table and a one-line completion summary

### `deploy` / `deploy please`

Purpose:
Run an explicit install or packaging deployment target.

Repo-specific deploy targets:

- `linux-local-runtime`
  - build: `cargo build --release`
  - install: `sudo install -Dm755 target/release/rust-clock /usr/local/bin/rust-clock`
  - validate: compare `sha256sum target/release/rust-clock /usr/local/bin/rust-clock`
- `windows-installer`
  - build/package: `pwsh -File .\installer\windows\build-installer.ps1`
  - expected output: versioned installer under `dist/windows/`

Notes:

- requires a clean, synced branch
- stops on detached `HEAD`
- reviews packaging/install docs when packaging behaviour changes

### `status` / `status please`

Purpose:
Read-only operator snapshot of branch state, sync status, tags, version source, and recommended next steps.

Notes:

- fetches first
- uses the fuller four-column table: `Origin`, `Local`, `Status`, `Action(s)`
- includes current branch, default branch, working tree, version source, tag state, ahead/behind state, and handover relevance

### `abort`

Purpose:
Inspect and recover from a partially completed SHIP, sync, or deploy workflow.

Use when:

- version, tag, merge, or push state looks inconsistent
- branch publication and handover metadata disagree
- a previous workflow stopped mid-way

Recovery expectations:

- inspect first
- preserve unpublished work before cleanup when needed
- never rewrite history or force-push without explicit extra confirmation

### `branch <new-branch-name>`

Purpose:
Close out current work cleanly and start the next branch.

Attempts to:

- assess whether the current branch should be shipped first
- stop if `HEAD` is detached
- stop if the requested new branch name is invalid or already exists locally or remotely
- stop instead of switching if the current branch is dirty and SHIP is declined
- stop instead of guessing if the source branch or local `main` is diverged
- stop if the source branch is ahead, behind, or unpublished relative to its upstream
- merge the current branch into local `main` when the current branch is not already `main`
- skip the merge step when the workflow already starts on `main`
- create and switch to the new branch from updated local `main`

## Docs Impact Reminder

Review docs when the change touches:

- user-visible features
- UI or interaction
- config or settings
- packaging or metadata
- roadmap or status

Repo-specific docs commonly reviewed:

- `README.md`
- `docs/user-guide.md`
- `docs/windows-installer.md`
- `PLAN.md`
- `docs/clock-face-visibility-plan.md`
- `.github/TCTBP Agent.md`
- `.github/TCTBP Cheatsheet.md`
- `.github/copilot-instructions.md`

## Deployment Notes

- Linux runtime deployment is a local install/update workflow, not a source release.
- Windows packaging currently means building the installer artefact, not publishing it automatically.
- Validate the deployed result, not just the build step.

## Approval Model

- `ship` may create local commit and tag state as part of the workflow
- `handover` grants approval to push the target branch and relevant tags for that workflow only
- `deploy` grants approval to run the repo-defined deployment commands for that workflow only
- any other remote push still requires explicit approval unless already covered by the active workflow

## Quick Choice

- Need a release version or tag: use `ship`
- Need to stop on one machine and resume on another safely: use `handover`
- Need the local runtime installed or the Windows installer built: use `deploy`
- Need a quick repo state check: use `status`
- Need to recover from a partial workflow state: use `abort`
- Need to start the next branch: use `branch <new-branch-name>`