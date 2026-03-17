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

## R Version Management — Design Research

### The uv analogy
uv creates `.venv/bin/python` as a symlink to a managed Python.
When `uv run` executes, it uses `.venv/bin/python` directly — not the system python.
R equivalent: `.ruv/bin/Rscript` → managed or system Rscript, set up per project.

### Current state in ruv
- `r-version = ">=4.3"` is written to `ruv.toml` by `ruv init` but the field is **not parsed** — `ProjectConfig` has no `r_version` field, so it's silently ignored.
- `ruv run` hardcodes `Command::new("Rscript")` — always uses whatever is on PATH.
- `.ruv/` only contains `library/`.

### Proposed `.ruv/` layout
```
.ruv/
  library/        # packages (existing)
  bin/
    R             # symlink → selected R binary
    Rscript       # symlink → selected Rscript binary
```

### Phase A — Discovery & symlinking (no download required)
1. Parse `r-version` from `ruv.toml` into `ProjectConfig`
2. New `src/r_version.rs` module:
   - `find_r_installations() -> Vec<(RVersion, PathBuf)>` — probe standard locations
   - `select_r(constraint: &str) -> Result<PathBuf, String>` — pick best match
   - `setup_r_symlinks(r_bin_dir: &Path) -> Result<(), String>` — write `.ruv/bin/R` + `.ruv/bin/Rscript`
   - `project_rscript() -> Option<PathBuf>` — return `.ruv/bin/Rscript` if set up
3. `ruv sync` calls `setup_r_symlinks` when `r-version` is set
4. `ruv run` uses `.ruv/bin/Rscript` if present, else falls back to system `Rscript`

**R search paths (macOS):**
- `/Library/Frameworks/R.framework/Versions/*/Resources/bin/R`
- `/usr/local/bin/R`, `/usr/bin/R`
- `~/.local/share/ruv/r/{version}/bin/R` (managed by ruv, Phase B)

**Get version from binary:** parse `R --version` output: `R version 4.4.2 (2024-10-31)`

### Phase B — Download & install R versions (#10)
New CLI: `ruv r install <version>`, `ruv r list`

**macOS download URLs:**
- ARM64:  `https://cran.r-project.org/bin/macosx/big-sur-arm64/base/R-{version}-arm64.pkg`
- x86_64: `https://cran.r-project.org/bin/macosx/big-sur-x86_64/base/R-{version}-x86_64.pkg`

**Extraction (no sudo):**
1. Shell out: `pkgutil --expand R.pkg /tmp/r-expanded/`
2. Payload inside is a gzipped cpio archive
3. Shell out: `cd ~/.local/share/ruv/r/{version} && cpio -iz < /tmp/r-expanded/R-fw.pkg/Payload`
4. Result: `~/.local/share/ruv/r/{version}/Library/Frameworks/R.framework/...`

**Linux:** Defer Phase B for Linux — CRAN provides no standalone binary tarballs. Options: musl static build, apt/yum integration, or rig-style deb download. Needs separate investigation.

### Files to create/modify
| File | Change |
|------|--------|
| `src/r_version.rs` | New module: discovery, selection, symlinking, download |
| `src/config.rs` | Add `r_version: Option<String>` to `ProjectConfig` |
| `src/main.rs` | Wire `sync` → symlink setup; `run` → use `.ruv/bin/Rscript`; add `ruv r` subcommand |
| `src/cache.rs` | Add `r_versions_dir()` path helper |

## Key constraints / risks
- musl build requires `x86_64-unknown-linux-musl` target added to CI matrix
- Homebrew (#43) should wait until Linux binary is stable (don't want a broken bottle)
- renv compat (#42) is open-ended — decision needed on scope before implementing
