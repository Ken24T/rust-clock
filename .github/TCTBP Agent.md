# Rust Clock — TCTBP Agent

## Purpose

This agent governs milestone, checkpointing, publishing, handover, resume, sync, status, recovery, and deployment actions for Rust Clock.

Primary objective: no code is ever lost while keeping local and remote repository state validated, recoverable, and easy to resume on another machine.

This workflow is for explicit operator actions such as `ship`, `checkpoint`, `publish`, `handover`, `resume`, `deploy`, `status`, `abort`, `branch`, and `branch <name>`. It is not for normal feature implementation work.

Quick reference: see [TCTBP Cheatsheet.md](TCTBP%20Cheatsheet.md).

## Authoritative Precedence

- `.github/TCTBP.json` is the source of truth when this document and the JSON profile differ.
- This file explains behaviour and guard rails when the JSON profile does not capture enough safety context.
- `.github/TCTBP Cheatsheet.md` is the short operator summary.
- `.github/agents/TCTBP.agent.md` is the runtime entry point for explicit TCTBP trigger routing.
- `.github/copilot-instructions.md` contains repo-specific engineering guidance and should stay aligned with the workflow files and runtime files.

## Repo Profile

Rust Clock is a Rust + iced application with Linux-first desktop-widget behaviour and an early Windows baseline.

Repo-specific operational values that must be preserved:

- default branch: `main`
- version source: `Cargo.toml` field `package.version`
- tag format: plain semver tags such as `1.2.2`, not `v1.2.2`
- format gate: `cargo fmt -- --check`
- lint gate: `cargo clippy -- -D warnings`
- test gate: `cargo test`
- normal build gate: `cargo build`
- release build: `cargo build --release`
- release build policy: use the release build for explicit installation, packaging, or deployment work, not as the default SHIP gate
- preferred interactive review launcher: `bash ./scripts/run-dev-harness.sh`
- user-facing docs commonly reviewed: `README.md`, `docs/user-guide.md`, `docs/windows-installer.md`, `PLAN.md`, and feature-specific docs under `docs/`
- locale: Australian English for user-facing text and comments

## Core Invariants

1. Verification must pass before irreversible actions unless `.github/TCTBP.json` explicitly allows a docs/infra-only shortcut.
2. Problems must be zero before any release, publication-linked, or shared-state commit, unless `.github/TCTBP.json` explicitly allows a local-only checkpoint commit to preserve work first.
3. Protected Git actions such as push, force-push, branch deletion, history rewrite, or remote modification require explicit approval unless granted by the active workflow trigger.
4. Tags must correspond exactly to the version committed in `Cargo.toml` and point to the commit that introduced that version.
5. No-code-loss takes priority over workflow completion.
6. Do not use hard reset, destructive checkout, auto-rebase, or force-push as normal workflow shortcuts.
7. Keep versioned artefacts, workflow files, runtime files, and documentation aligned.
8. Use the normal build gate by default; reserve release builds for install, packaging, or deployment work.

If any invariant fails, stop and explain the blocker.

## Supported Triggers

Supported workflow triggers are:

- `ship`, `ship please`, `shipping`, `prepare release`
- `checkpoint`, `checkpoint please`
- `publish`, `publish please`
- `deploy`, `deploy please`
- `handover`, `handover please`
- `resume`, `resume please`
- `status`, `status please`
- `abort`
- `branch`
- `branch <new-branch-name>`

Do not treat a bare `tctbp` request as implicit permission to mutate repository state.

## Interactive Review Runs

- preferred launcher: `bash ./scripts/run-dev-harness.sh`
- the launcher may stop stale instances of this repo's `target/debug/rust-clock` before starting a fresh review session
- it must not target an installed runtime outside the repo build tree
- plain `cargo run` remains valid when the hygiene step is intentionally not wanted

## Docs/Infra-Only Detection

A changeset is docs-only or infrastructure-only only when every changed file matches the repo rules in `.github/TCTBP.json`, for example:

- `*.md`, `*.txt`, `*.rst`
- `docs/**`
- `.github/**`
- `packaging/**`
- `LICENSE*`, `CHANGELOG*`, `CONTRIBUTING*`

Build manifests, installer definitions, desktop entries, and runtime configuration are not docs-only by default just because they are text files.

## Publish Workflow

Trigger: `publish` / `publish please`

Purpose: safely publish the current clean branch to origin without creating a release, bumping a version, creating a tag, or updating handover metadata.

Key rules:

- stop if `HEAD` is detached
- stop if the working tree is dirty
- fetch origin before deciding whether a push is required
- create an upstream on first publication when the branch is otherwise clean and unpublished
- stop if the branch is behind or diverged from origin
- never create a version bump, tag, or metadata update as part of `publish`

## Checkpoint Workflow

Trigger: `checkpoint` / `checkpoint please`

Purpose: create a durable local-only checkpoint commit on the current branch without changing version, tags, metadata, or remote state.

Key rules:

- stop if `HEAD` is detached
- stop if the working tree is clean
- stop if the working tree has unresolved conflicts or if a merge, rebase, cherry-pick, or revert is in progress
- stage the current non-ignored tracked and untracked changes on the current branch
- create a clearly marked local-only commit using the configured checkpoint message prefix
- do not run heavyweight verification gates as a blocker for this workflow
- if diagnostics are already available, they may be reported for awareness only
- end with a concise four-column table covering the previous `HEAD`, new checkpoint commit, resulting working-tree state, upstream sync state, and explicit local-only outcome
- never push, create a tag, bump version, update handover metadata, or change branches as part of `checkpoint`

## Branch Workflow

Trigger: `branch` or `branch <new-branch-name>`

Purpose: close out the current branch safely and either stop on `main` or create the next branch without losing code.

Key rules:

- stop if `HEAD` is detached
- validate the requested branch name before mutating anything when a new branch was requested
- stop if the target branch already exists locally or on origin
- stop if the source branch is dirty and SHIP is declined
- stop if the source branch is ahead, behind, diverged, or otherwise unpublished relative to its upstream
- fast-forward local `main` when clean and behind origin
- ask for explicit confirmation before merging a non-default branch back into `main`
- treat merge-to-`main` as the expected default outcome, but stop if that merge is explicitly declined
- verify the source branch tip is reachable from `main` before optional cleanup
- require explicit approval for push and branch deletion

Never use stash, reset, rebase, force-push, or destructive checkout as part of the branch workflow.

## Handover Workflow

Trigger: `handover` / `handover please`

Purpose: safely checkpoint and publish the current work branch at end of day, then refresh the handover metadata branch so another machine can resume from a deterministic shared state.

Scope:

- syncs the current work branch
- syncs relevant tags when needed
- maintains the metadata branch `tctbp/handover-state`
- does not attempt to reconcile every branch in the repository
- does not merge the current work branch into `main` as part of ordinary multi-machine sync

Handover metadata:

- metadata branch: `tctbp/handover-state`
- metadata file: `.github/TCTBP_STATE.json`
- metadata is refreshed after the current branch is safely published
- the metadata branch is never treated as a work branch candidate

Key safety rules:

- stop if `HEAD` is detached
- preserve dirty unpublished work through a durable checkpoint when necessary
- allow fast-forward only when local is clean and behind
- stop on divergence rather than guessing
- never auto-merge or auto-rebase as part of reconciliation
- update the metadata branch using a secondary worktree or another non-destructive mechanism

## Resume Workflow

Trigger: `resume` / `resume please`

Purpose: restore the intended work branch at start of day by consulting handover metadata first, switching safely when needed, and reconciling only through non-destructive checkout and fast-forward operations.

Key safety rules:

- stop if `HEAD` is detached
- consult metadata before arbitrary branch-recency inference
- prefer metadata over an arbitrary clean non-default branch
- create a local tracking branch from remote when the intended branch is published but missing locally
- allow fast-forward only when local is clean and behind
- stop when local is ahead, diverged, or ambiguous instead of publishing during `resume`

## Status Workflow

Trigger: `status` / `status please`

Purpose: provide a read-only operator snapshot of the repo.

Behaviour:

- fetch remote state first
- render a four-column table using `Origin`, `Local`, `Status`, and `Action(s)`
- include branch/upstream state, head commit, default-branch state, tag state, ahead/behind counts, working tree state, version source, metadata state, and whether `resume`, `checkpoint`, `publish`, `ship`, or `handover` is recommended
- never mutate the repo from `status`

## Abort Workflow

Trigger: `abort`

Purpose: inspect and recover safely from a partially completed workflow.

Check for states such as:

- version bumped without matching tag
- tag created but not pushed
- branch pushed while handover metadata is stale
- metadata pushed while the target branch is unpublished
- merge in progress
- local/remote tag drift
- changelog updated without a matching version bump

Abort must inspect first, propose recovery second, and execute only explicitly approved actions.

## Deploy Workflow

Trigger: `deploy` / `deploy please`

Purpose: build a runtime-ready artefact or packaging output and install or publish it safely.

General rules:

- stop if `HEAD` is detached
- require a clean working tree
- require a synced branch
- use `cargo build --release` for deployment work
- review packaging and install docs impact before mutating deployment targets
- validate the deployed result rather than merely copying files

Repo-specific deploy targets:

### `linux-local-runtime`

- build: `cargo build --release`
- install: `sudo install -Dm755 target/release/rust-clock /usr/local/bin/rust-clock`
- post-deploy validation: compare `sha256sum target/release/rust-clock /usr/local/bin/rust-clock`

### `linux-user-local`

- build: `cargo build --release`
- install binary: `install -Dm755 target/release/rust-clock ~/.local/bin/rust-clock`
- install desktop entry: `install -Dm644 assets/rust-clock.desktop ~/.local/share/applications/rust-clock.desktop`
- post-deploy validation: confirm both installed files exist

### `windows-installer`

- build/package: `pwsh -File .\installer\windows\build-installer.ps1`
- expected output: versioned installer under `dist/windows/`
- review: `docs/windows-installer.md`, `installer/windows/build-installer.ps1`, `installer/windows/rust-clock.iss`

If the requested deployment target is not one of these explicit cases, stop and ask rather than guessing.

## SHIP Workflow

Trigger: `ship` / `ship please` / `shipping` / `prepare release`

Purpose: create a formal shipped version only from a clean, fetched branch.

Workflow order:

1. preflight
2. verify
3. problems
4. docs impact
5. bump
6. commit
7. changelog when present
8. tag
9. push

Preflight guard rails:

- fetch origin when needed
- stop if `HEAD` is detached
- allow first publication from a clean unpublished branch
- stop if the branch is behind or diverged from origin
- stop if the working tree is dirty
- render a release-focused four-column snapshot table before mutating anything

Verify and build policy:

- normal SHIP gate: `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `cargo test`, `cargo build`
- use `cargo build --release` only when the user explicitly requests installation or deployment work, or when the deploy workflow requires it
- docs/infra-only changes may skip heavyweight code gates according to `.github/TCTBP.json`, but still require editor diagnostics and docs impact assessment

Versioning rules:

- patch bump behaviour is controlled by `.github/TCTBP.json`
- in this repo, docs-only and infrastructure-only ships do not bump by default
- first SHIP on a `feature/` branch gets a minor bump instead of a patch bump
- major bump only by explicit instruction
- apply version changes to `Cargo.toml` before committing

Tagging rules:

- use plain semver tags such as `1.2.2`
- one tag per shipped commit
- skip tagging when no version bump occurs

Docs impact rules:

- `README.md`, `docs/user-guide.md`, and `PLAN.md` for user-visible changes
- `docs/windows-installer.md`, `installer/windows/build-installer.ps1`, and `installer/windows/rust-clock.iss` for Windows packaging changes
- `assets/rust-clock.desktop` for Linux desktop integration changes

## Repo-Specific Preservation Notes

When updating these workflow files, preserve the following local choices unless the user explicitly changes them:

- plain semver release tags with no `v` prefix
- `Cargo.toml` as version source
- `cargo build` as the default SHIP build gate
- `cargo build --release` only for explicit deployment/install work
- the dev harness launcher and its stale-process protections
- Linux and Windows deployment targets and docs paths
- Australian English conventions

Preflight guard rails:

- fetch origin when needed
- stop if `HEAD` is detached
- allow first publication from a clean unpublished branch
- stop if the branch is behind or diverged from origin
- stop if the working tree is dirty
- render a release-focused four-column snapshot table before mutating anything

Verify and build policy:

- normal SHIP gate: `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `cargo test`, `cargo build`
- use `cargo build --release` only when the user explicitly requests installation, packaging, or deployment work, or when the deploy workflow requires it
- docs/infra-only changes may skip heavyweight code gates according to `.github/TCTBP.json`, but still require editor diagnostics and docs impact assessment

Versioning rules:

- patch bump on every SHIP except docs-only or infrastructure-only changes
- first SHIP on a `feature/` branch gets a minor bump instead of a patch bump
- major bump only by explicit instruction
- apply version changes to `Cargo.toml` before committing

Tagging rules:

- use plain semver tags like `1.2.2`
- one tag per shipped commit
- skip tagging when no version bump occurs

Docs impact rules:

- `README.md` and `docs/user-guide.md` for user-visible, UI, or settings changes
- `docs/windows-installer.md` and installer assets for packaging changes
- `PLAN.md` and feature-specific planning docs for roadmap/status changes
- if no docs changes are required, record `No docs impact` with a short reason

## Summary Table Consistency

For SHIP, handover, and status tables:

- columns must be `Origin`, `Local`, `Status`, and `Action(s)`
- use `n/a` when there is no meaningful origin-side value
- keep `Status` diagnostic, not narrative
- keep `Action(s)` concrete and short

## Repo-Specific Preservation Notes

When updating these workflow files, preserve the following local choices unless the user explicitly changes them:

- plain numeric release tags instead of `v`-prefixed tags
- `Cargo.toml` as version source
- `cargo build` as the default SHIP build gate
- `cargo build --release` only for explicit deployment/install/packaging work
- Rust + iced project structure and Windows installer assets under `installer/windows/`
- docs paths under `docs/` plus `PLAN.md`
- Australian English conventions