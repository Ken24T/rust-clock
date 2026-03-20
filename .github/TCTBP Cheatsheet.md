# TCTBP Developer Cheatsheet

Short operator reference for the Rust Clock TCTBP workflows.

Use this file for the quick view.
Use [TCTBP Agent.md](TCTBP%20Agent.md) for the full workflow rules and guard rails.

## Core Rule

- No code is ever lost while syncing local and remote state.
- Do not use destructive shortcuts as part of normal workflow execution.
- If a workflow hits divergence, ambiguity, or a failed invariant, it should stop rather than guess.

## Repo Gates

- Format check: `cargo fmt -- --check`
- Test: `cargo test`
- Lint: `cargo clippy -- -D warnings`
- Normal build gate: `cargo build`
- Runtime or deployment build: `cargo build --release`

## Triggers

### `ship` / `ship please` / `shipping` / `tctbp` / `prepare release`

Purpose:
Formal source release workflow.

Attempts to:
- preflight the repo state
- run verification gates
- confirm zero problems
- assess docs impact
- bump version when required
- commit the release changes
- create the version tag
- push the current branch

Use when:
- you want a formal shipped version
- version/tag state needs to be updated
- the repo should be published as a release milestone

Notes:
- Uses the normal build gate by default, not the release build
- Patch bump happens on every ship unless the changes are docs-only or infrastructure-only
- Release build is reserved for installation, packaging, or deployment scenarios

### `deploy` / `deploy please`

Purpose:
Build a runtime-ready artefact and install or package it for the target environment.

Attempts to:
- preflight the repo state and deployment target
- require a clean tree and synced branch
- optionally run `ship` first if repo policy requires it
- run verification gates
- assess docs impact for packaging/runtime/install changes
- run the runtime build path
- perform repo-specific install or packaging steps
- run post-deploy validation checks
- summarise the deployed result

Use when:
- the runtime should be updated on the local machine
- a packaged deployable artefact should be produced
- you need the release build rather than the dev build

Repo-specific deploy targets:

1. `linux-user-local`
   - installs `target/release/rust-clock` to `~/.local/bin/rust-clock`
   - installs [assets/rust-clock.desktop](../assets/rust-clock.desktop) to `~/.local/share/applications/rust-clock.desktop`
   - validates both files exist afterwards

2. `windows-installer`
   - runs `pwsh -File .\installer\windows\build-installer.ps1`
   - validates `dist/windows` exists afterwards

Current deploy policy:
- `requireCleanTree: true`
- `requireSyncedBranch: true`
- `requireShipFirst: false`
- `migrationCommand: null`

### `handover` / `handover please`

Purpose:
Safely reconcile local and remote branch state so another machine can continue from the same latest validated work.

Attempts to:
- preflight current branch, tree, and upstream state
- fetch and compare with `origin`
- preserve dirty work by staging and committing if needed
- run verification when a commit or merge is needed
- assess docs impact
- ship if repo policy requires it
- fast-forward when safe
- stop on divergence or ambiguity
- push the current branch and relevant tags when appropriate
- summarise the sync result

Use when:
- you are moving between machines
- the current branch should be safely published to origin
- you want the repo reconciled without risking local work loss

Never does:
- auto-rebase
- hard reset
- destructive checkout
- force-push

### `branch <new-branch-name>`

Purpose:
Close out current work cleanly and start the next branch.

Attempts to:
- assess whether the current branch should be shipped first
- merge current branch into local `main`
- create and switch to the new branch from updated local `main`

Use when:
- the current line of work is complete enough to close out locally
- you want a clean next branch starting from updated `main`

## Docs Impact Reminder

Review docs when the change touches:
- user-visible features
- UI or interaction
- config or settings
- packaging or metadata
- roadmap or status

Repo-specific docs commonly reviewed:
- [README.md](../README.md)
- [docs/user-guide.md](../docs/user-guide.md)
- [PLAN.md](../PLAN.md)
- [Cargo.toml](../Cargo.toml)
- [assets/rust-clock.desktop](../assets/rust-clock.desktop)

## Deployment Notes

- `cargo build` is the normal verification build
- `cargo build --release` is for runtime install, packaging, or deployment work
- Deployment should validate the installed result, not just copy files
- Deployment should preserve recoverability where practical

## Approval Model

- `ship` may create local commit and tag state as part of the workflow
- `handover` grants approval to push the current branch and relevant tags for that workflow only
- `deploy` grants approval to run repo-defined deployment commands for that workflow only
- Any other remote push still requires explicit approval unless already covered by the active workflow

## Quick Choice

- Need a release version/tag: use `ship`
- Need the runtime installed or packaged: use `deploy`
- Need to sync safely across machines: use `handover`
- Need to start the next branch: use `branch <new-branch-name>`