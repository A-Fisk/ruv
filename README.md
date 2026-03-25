# ruv

> A fast R package manager, written in Rust. The R equivalent of [`uv`](https://github.com/astral-sh/uv).

---

## Motivation

R package management is slow, fragmented, and requires too many tools:

- `install.packages()` — sequential, no caching, no lockfiles
- `renv` — lockfiles but slow restoration, no binary cache
- `pak` — faster installs, global cache, and CI-oriented lockfiles, but no project-level reproducibility workflow or R version management
- `rig` — R version management, but a separate tool

`ruv` replaces all of them with a single fast binary.

## Goals

- **Fast** — parallel downloads, global binary cache, hard-links into project libraries (zero-copy installs)
- **Reproducible** — lockfile-based installs, exact version pinning
- **Integrated** — package management + R version management in one tool
- **Simple** — one binary, familiar `uv`-style workflow

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

### Flags

```sh
ruv --verbose sync   # show per-package source (cache vs download)
```

## How it works

- **`ruv lock`** fetches the CRAN package index, resolves all transitive dependencies using the PubGrub algorithm, and writes `ruv.lock` with exact versions and the full dependency graph. Each package entry records its pinned version and the [RSPM](https://packagemanager.posit.co) `cran/latest` registry — the exact version in the filename (e.g. `ggplot2_3.5.1.tgz`) is the reproducibility guarantee.
- **`ruv sync`** reads `ruv.lock` directly — no CRAN fetch required — and installs packages into `.ruv/library/` from the pinned RSPM binary URLs. Packages are downloaded once to a global cache (`~/Library/Caches/ruv/` on macOS) and hard-linked into the project library, so repeated installs across projects are instant.

## Comparison

| | `install.packages` | `renv` | `pak` | **ruv** |
|---|---|---|---|---|
| Parallel downloads | ❌ | ❌ | ✅ | ✅ |
| Global binary cache | ❌ | ❌ | ✅ | ✅ |
| Lockfile | ❌ | ✅ | ⚠️ CI caching only | ✅ |
| Reproducible binary installs | ❌ | ⚠️ source only | ⚠️ PPM snapshots only | ✅ |
| Lock/sync separation | ❌ | ❌ | ❌ | ✅ |
| R version management | ❌ | ❌ | ❌ | 🚧 planned |

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
