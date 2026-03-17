# Findings

## Issue Analysis

### #47 — CI pull_request trigger missing `main`
Current `.github/workflows/release.yml` (or ci.yml) only has `development` in `pull_request.branches`.
Fix: add `main`. One-liner change.

### #48 — Installation instructions
README currently has no install section. Shell installer exists at `scripts/install.sh` and is attached to releases. Fix: add `## Installation` section with curl command and PATH note.

### #40 — project_docs dir
Root currently has: `mvp_plan.md`, `project_plan.md`, `test_coverage.md`, `DISTRIBUTION.md`, `task_plan.md`, `findings.md`, `progress.md`.
Proposal: move all into `project_docs/` (or `docs/project/`).

### #50 — Linux compatibility
Two bugs:
1. `install.sh` line 89 uses `[[` (bash-specific) but runs under `/bin/sh` via `sh -s`. Fix: `[ ... ]`.
2. Binary links against `GLIBC_2.39` but Ubuntu 22.04 only has `GLIBC_2.35`.
   Fix: build a `x86_64-unknown-linux-musl` target (statically linked) in CI.

### #51 — Inaccurate README comparisons
- `pak` does support lockfiles (`pak::lockfile_create()` etc.)
- Need to audit full comparison table and be precise about what's *different* vs just *missing*.

### #52 — Differences from `rv` (A2-ai/rv)
`rv` is a Rust-based R package manager that supports:
- Declaring repos in config
- Snapshot/restore workflow
Research needed: does it support R version management? Virtual environments?
Likely positioning: `ruv` focuses on R version management (like `pyenv`+`pip` combined); `rv` is more pak-like.

### #49 — R console via `ruv run R`
`ruv run` passes args to R, so `ruv run R` should work if R is on PATH or managed by ruv.
Need to verify the current implementation handles this correctly.

### #29 — Distribution status
Binaries exist for macOS arm64 + x86_64. Linux binary exists but has glibc issue (#50).
Installer script has sh-compat bug (#50). Once #50 is fixed, #29 can likely be closed.

## Key constraints / risks
- musl build requires `x86_64-unknown-linux-musl` target added to CI matrix
- Homebrew (#43) should wait until Linux binary is stable (don't want a broken bottle)
- renv compat (#42) is open-ended — decision needed on scope before implementing
