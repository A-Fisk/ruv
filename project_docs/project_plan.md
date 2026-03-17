# ruv — A uv-inspired package manager for R

> `ruv` is to R what `uv` is to Python: a single fast Rust binary that manages
> R versions, project libraries, dependency resolution, and reproducible installs.

---

## Motivation

| uv capability | R equivalent today | Gap |
|---|---|---|
| Fast parallel resolver | `pak` (partial) | No Rust-speed resolver |
| Global package cache | none | Every project re-downloads |
| Integrated version management | `rig` (separate tool) | Not unified |
| Lock files | `renv.lock` | Slow restore, no binary cache |
| Single binary, zero deps | nothing | R toolchain required to manage R |
| Ephemeral `uv run`-style execution | nothing | No ephemeral envs |

---

## Name

The CLI is `ruv`. (`rv` is already taken by a Ruby uv-equivalent.)

---

## Architecture

**Language:** Rust — the bottlenecks are I/O, network, and dependency resolution,
all of which Rust handles well. Same rationale as uv.

```
ruv/
├── ruv-cli/          # CLI entry point
├── ruv-resolver/     # PubGrub-based dependency resolution engine
├── ruv-installer/    # parallel download + package extraction
├── ruv-cache/        # global content-addressable package cache
├── ruv-rversion/     # R version download + management
├── ruv-lockfile/     # lockfile read/write
└── ruv-metadata/     # CRAN/Bioc/r-universe metadata fetching
```

---

## Project file format

For scripts and applications, a new `ruv.toml`:

```toml
[project]
name = "my-analysis"
r-version = ">=4.3"

[dependencies]
ggplot2 = ">=3.4"
dplyr = "*"
"bioc:DESeq2" = "3.18"

[sources]
cran = "https://cloud.r-project.org"
bioc = "https://bioconductor.org"
```

For R packages, read deps directly from the existing `DESCRIPTION` file so that
existing packages work without modification.

---

## Lock file format

```toml
# ruv.lock — generated, do not edit

[[package]]
name = "ggplot2"
version = "3.5.1"
source = "cran"
sha256 = "abc123..."
deps = ["scales", "rlang", "gtable"]

[[package]]
name = "rlang"
version = "1.1.4"
source = "cran"
sha256 = "def456..."
deps = []
```

---

## Dependency resolver

R dependency types have a priority ordering: `Depends > Imports > LinkingTo > Suggests`.
The resolver handles:

1. **CRAN metadata** — fetch `PACKAGES.gz` from CRAN mirrors, parse into an in-memory index
2. **Bioconductor** — versioned releases tied to R version (e.g., Bioc 3.18 → R 4.3)
3. **GitHub / r-universe** — fetch `DESCRIPTION` directly from repo
4. **Conflict resolution** — R only uses `>= x.y` lower bounds on CRAN, which simplifies
   the version constraint space vs. Python

**Algorithm:** PubGrub (same as uv and poetry) — well-suited to R's constraint structure.

---

## Package installer

CRAN provides pre-built binaries for Windows (`.zip`) and macOS (`.tgz`). The approach:

- **Parallel downloads** — current R tooling is largely sequential
- **Global content-addressable cache** at `~/.cache/ruv/` — if `ggplot2 3.5.1` is
  cached, never re-download across projects
- **Hard-link from cache into project library** — zero-copy installs (same as uv)
- **Source fallback** — compile from source when no binary exists, cache the compiled result

For source compilation, delegate to R itself (unavoidable), but cache the output.

**Linux note:** CRAN does not provide Linux binaries. Default to
[Posit Public Package Manager](https://packagemanager.posit.co) as the CRAN mirror on
Linux, which provides pre-built Linux binaries and is the key unlock for Linux speed.

---

## R version management

Integrate what `rig` does, built into `ruv`:

- Download R from official CRAN/R-project mirrors
- Manage multiple R versions under `~/.local/share/ruv/r-versions/`
- `.r-version` file per project (like `.python-version`)
- Auto-install the required R version on `ruv sync` if missing

---

## CLI design

```
# Dependency management
ruv add ggplot2                  # add + install, update lockfile
ruv add ggplot2@3.4.0            # pin specific version
ruv add bioc:DESeq2              # Bioconductor package
ruv add gh:tidyverse/ggplot2     # GitHub package
ruv remove ggplot2

# Project sync
ruv sync                         # install from lockfile (fast path: binary cache)
ruv install                      # alias for sync

# Running R
ruv run Rscript analysis.R       # run with project library, auto-sync if needed
ruv run -- -e "library(dplyr)"   # run R expression

# R version management
ruv r install 4.4.1              # download + install R version
ruv r list                       # list installed R versions
ruv r pin 4.4.1                  # write .r-version for this project
ruv r default 4.4.1              # set system-wide default

# Cache management
ruv cache clean                  # prune old/unused packages
ruv cache info                   # show cache size and stats
```

---

## Implementation phases

### Phase 1 — Core installer (MVP)

- CRAN metadata fetching and parsing (`PACKAGES.gz`)
- Parallel binary package download and extraction
- Global cache at `~/.cache/ruv/`
- Basic project library management (`.ruv/library/`)
- Simple lockfile (record installed packages, no full resolver yet)

### Phase 2 — Resolver

- PubGrub-based dependency resolution
- Full deterministic lockfile generation
- `ruv sync` restores exactly from lockfile
- Bioconductor source support

### Phase 3 — R version management

- Download and install R versions from official mirrors
- `.r-version` file support
- Auto-select correct R for project on `ruv sync` / `ruv run`

### Phase 4 — Developer experience

- `ruv run` with ephemeral per-script environments (like `uv run`)
- GitHub and r-universe package sources
- `ruv add` / `ruv remove` properly update lockfile
- Shell completions (bash, zsh, fish)
- `ruv import renv` migration path for existing `renv` projects

### Phase 5 — Polish

- Source package compilation caching on Linux
- System dependency hints (like `pak`'s sysreqs detection)
- `ruv publish` to submit packages to CRAN / r-universe
- IDE integration hooks for RStudio and Positron

---

## Key technical risks

### 1. Linux binaries
The largest scope risk. CRAN does not provide Linux binaries. Defaulting to Posit Package
Manager mitigates this, but introduces a dependency on a third-party service. A longer-term
option is building a binary compilation + caching layer, but that is significant scope.

### 2. R's package loading conventions
Unlike Python wheels, R packages use `.so`/`.dll` files loaded via R's own mechanism.
Installation must respect R's `lib.loc` path conventions exactly or packages will silently
fail to load.

### 3. Bioconductor versioning
Bioconductor releases are tightly coupled to specific R versions. The resolver needs to
maintain and consult a compatibility matrix (e.g., Bioc 3.18 → R 4.3.x only).

### 4. renv adoption / migration
Many existing R projects use `renv`. An `ruv import renv` command that reads
`renv.lock` and produces an `ruv.lock` is essential for adoption.

---

## Competitive landscape

| Tool | Speed | Lock file | R version mgmt | Binary cache |
|---|---|---|---|---|
| `install.packages` | slow | no | no | no |
| `pak` | fast (parallel) | no | no | no |
| `renv` | slow | yes | no | no |
| `rig` | n/a | n/a | yes | n/a |
| **ruv** | **fast** | **yes** | **yes** | **yes** |

---

## Success criteria

- `ruv sync` on a cold cache is at least 5× faster than `renv::restore()` on the same
  lockfile
- `ruv sync` on a warm cache (all packages cached) completes in under 2 seconds
- Single statically-linked binary with no R installation required to install ruv itself
- Full round-trip: `ruv add` → `ruv.lock` committed → colleague runs `ruv sync` →
  identical library
