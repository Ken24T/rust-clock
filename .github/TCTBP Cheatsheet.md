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

## Dev Harness Review

- Preferred review launcher: `bash ./scripts/run-dev-harness.sh`
- This launcher stops only stale instances of this repo's `target/debug/rust-clock`
- It must not kill installed runtimes outside the repo build tree
- Use plain `cargo run` when you explicitly do not want the hygiene step

## Version And Tags

- Version source: `Cargo.toml` field `package.version`
- Tag format: plain semver, for example `1.2.2`
- Do not normalise this repo to `v1.2.2` tags unless explicitly requested.

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
- patch bump behaviour is controlled by `versioning.patchEveryShip` and `versioning.patchEveryShipForDocsInfrastructureOnly` in `.github/TCTBP.json`
- in this repo, docs-only and infrastructure-only ships do not bump by default
- first ship on a `feature/` branch gets a minor bump
- may publish a clean branch that has no upstream yet by creating the upstream on the first ship push
- stops if the branch is dirty, behind origin, diverged from origin, or on detached `HEAD`

### `publish` / `publish please`

Purpose:
Safely publish the current branch to `origin` without release semantics.

Attempts to:

- preflight the current branch state
- fetch and compare local versus origin
- allow first publication by creating the upstream when needed
- push the current branch when it is clean and ahead
- verify that the branch is now synced

Notes:

- does not bump version
- does not create a tag
- does not update handover metadata
- stops if the branch is dirty, behind, diverged, or detached

### `checkpoint` / `checkpoint please`

Purpose:
Create a durable local-only checkpoint commit on the current branch without release or sync side effects.

Attempts to:

- preflight the current branch and working tree state
- stop if `HEAD` is detached, the tree is clean, conflicts exist, or a merge/rebase/cherry-pick/revert is in progress
- stage the current non-ignored tracked and new files
- create a clearly marked non-release local commit
- end with a concise four-column table showing the pre-checkpoint commit, the new checkpoint commit, resulting sync state, and explicit local-only outcome
- confirm that nothing was pushed, tagged, or handed over

Notes:

- ends with a concise four-column table: `Origin`, `Local`, `Status`, `Action(s)`
- the table should show the actual pre-checkpoint commit and the new checkpoint commit, not only the final SHA
- does not push
- does not bump version
- does not create a tag
- does not update handover metadata
- does not reconcile with origin
- may leave the branch ahead of or further diverged from origin because it is local-only
- handover may reuse a recent matching checkpoint commit instead of creating another one

### `handover` / `handover please`

Purpose:
Safely checkpoint and publish the current work branch at the end of a session, then refresh handover metadata so another machine can resume deterministically.

Scope:

- syncs the current work branch
- syncs relevant tags when needed
- maintains `tctbp/handover-state`
- does not reconcile every branch in the repository
- does not merge into `main` as part of normal machine-to-machine sync

Handover metadata:

- branch: `tctbp/handover-state`
- file: `.github/TCTBP_STATE.json`

Notes:

- can checkpoint dirty unpublished work before verification strands it
- fast-forwards when behind and clean
- stops on divergence or ambiguity
- ends with a concise four-column table and a one-line completion summary

### `resume` / `resume please`

Purpose:
Safely restore the intended work branch at the start of a session.

Attempts to:

- fetch and inspect remote state
- read the handover metadata branch first
- prefer metadata over arbitrary branch-recency guesses
- create a local tracking branch from the intended remote branch when needed
- fast-forward a clean branch when origin is ahead
- stop on ambiguity, divergence, or any case that would require publication

Notes:

- does not publish
- does not update metadata
- does not create a release
- stops if switching branches would be destructive or if local/remote state is ambiguous

### `deploy` / `deploy please`

Purpose:
Run an explicit install or packaging deployment target.

Repo-specific deploy targets:

- `linux-local-runtime`
  - build: `cargo build --release`
  - install: `sudo install -Dm755 target/release/rust-clock /usr/local/bin/rust-clock`
  - validate: compare `sha256sum target/release/rust-clock /usr/local/bin/rust-clock`
- `linux-user-local`
  - build: `cargo build --release`
  - install binary: `install -Dm755 target/release/rust-clock ~/.local/bin/rust-clock`
  - install desktop entry: `install -Dm644 assets/rust-clock.desktop ~/.local/share/applications/rust-clock.desktop`
  - validate: confirm both installed files exist
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
- includes current branch, default branch, working tree, version source, tag state, ahead/behind state, metadata relevance, and whether `resume`, `checkpoint`, `publish`, `ship`, or `handover` is recommended

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

### `branch` / `branch <new-branch-name>`

Purpose:
Close out current work cleanly, optionally starting the next branch.

Attempts to:

- assess whether the current branch should be shipped first
- stop if `HEAD` is detached
- stop if the requested new branch name is invalid or already exists locally or remotely
- stop instead of switching if the current branch is dirty and SHIP is declined
- stop instead of guessing if the source branch or local `main` is diverged
- stop if the source branch is ahead, behind, or unpublished relative to its upstream
- recommend `publish`, `handover`, or `ship` first when the source branch is not yet published or synced
- ask for explicit confirmation before merging the current non-default branch back into `main`
- merge the current branch into local `main` when the current branch is not already `main`
- skip the merge step when the workflow already starts on `main`
- create and switch to the new branch from updated local `main` when a new branch name was supplied
- allow closeout-only mode that leaves the repo on updated `main` when the user runs bare `branch`

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
- `checkpoint` grants approval only for the local checkpoint commit it creates
- `publish` grants approval to push the current branch for that workflow only
- `handover` grants approval to push the current branch, metadata branch, and relevant tags for that workflow only
- `deploy` grants approval to run the repo-defined deployment commands for that workflow only
- any other remote push still requires explicit approval unless already covered by the active workflow

## Quick Choice

- Need a release version or tag: use `ship`
- Need a durable local-only save before deciding whether to publish or hand over: use `checkpoint`
- Need to sync a clean branch without release or metadata side effects: use `publish`
- Need to stop on one machine and resume on another safely: use `handover`
- Need to restore the last handed-over branch before starting work: use `resume`
- Need the local runtime installed or the Windows installer built: use `deploy`
- Need a quick repo state check: use `status`
- Need to recover from a partial workflow state: use `abort`
- Need to close out the current branch or start the next branch: use `branch` or `branch <new-branch-name>`