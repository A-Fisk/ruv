# ruv

> A fast R package manager, written in Rust. The R equivalent of [`uv`](https://github.com/astral-sh/uv).

---

## Motivation

R package workflows often combine multiple tools:

- `install.packages()` for direct installs
- `renv` for project lockfiles
- `pak` for fast dependency solving and installs
- `rig` for R version management

`ruv` aims to provide a single fast CLI with lock/sync workflows and integrated R runtime selection.

## Goals

- **Fast** — parallel downloads, global binary cache, hard-links into project libraries (zero-copy installs)
- **Reproducible** — lockfile-based installs, exact version pinning
- **Integrated** — package management + R version management in one tool
- **Simple** — one binary, familiar `uv`-style workflow

## Installation

```sh
curl -LsSf https://github.com/A-Fisk/ruv/releases/latest/download/install.sh | sh
```

Then add `~/.local/bin` to your `$PATH` if it isn't already:

```sh
export PATH="$HOME/.local/bin:$PATH"
```

For manual downloads, see the [releases page](https://github.com/A-Fisk/ruv/releases).

To build from source (requires Rust):

```sh
cargo install --path .
```

## Usage

### Project setup

Create an `ruv.toml` in your project directory:

```toml
[project]
name = "my-analysis"
version = "0.1.0"
r-version = ">=4.3"
dependencies = [
    "ggplot2",
    "dplyr",
]
```

### Commands

```sh
# Create ruv.toml in the current directory
ruv init

# Resolve dependencies and write ruv.lock
ruv lock

# Install exact versions from ruv.lock
ruv sync

# Install a package and its dependencies (without modifying ruv.toml)
ruv install ggplot2

# Launch an interactive R console with the project library
ruv run R

# Run a script using the project library
ruv run Rscript analysis.R
ruv run -- -e "library(ggplot2)"
```

### Typical workflow

```sh
# First time — resolve and install
ruv lock
ruv sync

# After editing ruv.toml — re-resolve and reinstall
ruv lock
ruv sync

# Colleague clones the repo — restore exactly from lockfile
ruv sync
```

### Development workflow

```sh
# Build and run the CLI while developing
cargo run -- init
cargo run -- lock
cargo run -- sync
cargo run -- run R

# Run test suite
cargo test
```

For local manual testing with a real dependency set, see `project_docs/dev_testing.md`.

### Flags

```sh
ruv --verbose sync   # show per-package source (cache vs download)
```

## How it works

- **`ruv lock`** fetches the CRAN package index, resolves all transitive dependencies using the PubGrub algorithm, and writes `ruv.lock` with exact versions and the full dependency graph. Each package entry records its pinned version and the [RSPM](https://packagemanager.posit.co) `cran/latest` registry — the exact version in the filename (e.g. `ggplot2_3.5.1.tgz`) is the reproducibility guarantee.
- **`ruv sync`** reads `ruv.lock` directly — no CRAN fetch required — and installs packages into `.ruv/library/` from the pinned RSPM binary URLs. Packages are downloaded once to a global cache (`~/Library/Caches/ruv/` on macOS) and hard-linked into the project library, so repeated installs across projects are instant.

## Comparison

Sourced from each tool's official documentation (as of Mar 2026). ⚠️ = partial or limited support.

| | `install.packages` | `renv` | `pak` | `rv` | **ruv** |
|---|---|---|---|---|---|
| Lockfile | ❌ | ✅ | ✅ [`lockfile_create`](https://pak.r-lib.org/reference/lockfile_create.html) | ✅ | ✅ |
| Explicit lock + sync commands | ❌ | ❌ | ⚠️ separate functions | ✅ `plan`/`sync` | ✅ `lock`/`sync` |
| Global package cache | ❌ | ✅ | ✅ | ✅ (v0.18+) | ✅ |
| Fast parallel installs | ❌ | ⚠️ | ✅ | ✅ | ✅ |
| `r_version` in project config | ❌ | ❌ | ❌ | ✅ (selects installed R) | ✅ (selects + downloads R) |
| `ruv run` / script runner | ❌ | ❌ | ❌ | ❌ | ✅ |
| renv migration | ❌ | n/a | ❌ | ✅ `migrate renv` | 🚧 planned |
| System dependency hints | ❌ | ✅ `sysreqs()` | ✅ | ✅ `sysdeps` | 🚧 planned |
| Single binary, no R required to install | n/a | n/a | n/a | ✅ | ✅ |

## ruv vs rv

[`rv`](https://github.com/A2-ai/rv) is the closest comparison to `ruv` — both are Rust-based R package managers with lockfiles, global caches, and `r_version` support.

Key differences based on their docs:

- **R runtime**: `rv` selects an already-installed R matching `r_version`; `ruv` can also download and install R automatically if it isn't found.
- **Script runner**: `ruv run` launches R or Rscript with the project library set — `rv` has no equivalent execution wrapper.
- **Lock workflow**: `rv` uses `plan`/`sync` (plan is a dry-run preview); `ruv` uses a separate `lock` step that writes the lockfile, then `sync` installs from it.
- **Maturity**: `rv` is further along (v0.20, renv migration, sysdeps, shell activation). `ruv` is earlier-stage.

## Status

Working MVP on macOS (arm64 + x86_64). Active development — see the [GitHub issues](https://github.com/A-Fisk/ruv/issues) for the roadmap.

**What works:**
- `ruv lock` — PubGrub dependency resolution + write lockfile with exact versions and RSPM/latest URLs
- `ruv sync` — restore from lockfile using pinned RSPM binaries (no CRAN fetch on warm runs)
- `ruv install` — one-off package install
- `ruv run` — run scripts with the project library
- Version constraint solving (`>=`, `==`, `<=`, `<`) including pinning to older versions via crandb
- Global package cache with hard-linking
- 54 unit tests, CI on GitHub Actions

**Coming next:**
- `ruv add` / `ruv remove`
- Bioconductor package support
- R version management

## Development

Requires Rust (install via [rustup](https://rustup.rs)):

```sh
git clone https://github.com/A-Fisk/ruv
cd ruv
cargo build
cargo test
```

## License

MIT
