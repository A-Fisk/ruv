# ruv MVP plan

The MVP answers one question: **given a package name, install it and all its
dependencies into a local library directory**, entirely from Rust, on the current
machine (macOS arm64/x86_64 to start).

No lockfile, no R version management, no cache — just the core install loop working
end-to-end. Everything else builds on top of this.

---

## What the MVP does

```
ruv install ggplot2
```

1. Fetch the CRAN package index
2. Parse it to build an in-memory map of every package + its dependencies
3. Resolve the full dependency tree for `ggplot2`
4. Download the correct binary for the current OS
5. Extract each binary into `./ruv_lib/`

Running R with `R_LIBS=./ruv_lib Rscript -e "library(ggplot2)"` should work
at the end.

---

## CRAN package metadata — what it looks like

CRAN exposes a plain-text index at:

```
https://cloud.r-project.org/src/contrib/PACKAGES.gz
```

(For macOS binaries, the URL is R-version-specific, covered in Step 3.)

Each entry in `PACKAGES` looks like a mail-style header block:

```
Package: ggplot2
Version: 3.5.1
Depends: R (>= 3.3)
Imports: cli, glue, gtable, isoband, lifecycle, MASS, mgcv, rlang (>= 1.1.0),
        scales (>= 1.3.0), tibble, vctrs (>= 0.5.0), withr (>= 2.5.0)
Suggests: covr, dplyr, ...
License: MIT + file LICENSE
MD5sum: abc123...

Package: dplyr
Version: 1.1.4
...
```

Entries are separated by blank lines. The fields we care about are `Package`,
`Version`, `Imports`, and `Depends`. `Suggests` are optional and **not** installed.

---

## Step-by-step build plan

### Step 1 — Fetch and decompress PACKAGES.gz

- HTTP GET `https://cloud.r-project.org/src/contrib/PACKAGES.gz`
- Decompress the gzip stream
- Print the raw text to stdout

**Crates needed:**
```toml
reqwest = { version = "0.12", features = ["blocking"] }
flate2 = "1"
```

**Smoke test:** running `ruv` prints the first 50 lines of the PACKAGES file.

---

### Step 2 — Parse PACKAGES into a package index

Parse the text into a `HashMap<String, Package>` where:

```rust
struct Package {
    name: String,
    version: String,
    imports: Vec<String>,   // required at runtime
    depends: Vec<String>,   // also required (minus base R packages)
}
```

Rules:
- Split on blank lines to get individual entries
- Split each entry into `key: value` pairs
- For `Imports`/`Depends`, strip version constraints like `(>= 1.0)` — just
  keep the package name for now
- Ignore base R packages that ship with R itself: `base`, `utils`, `stats`,
  `graphics`, `grDevices`, `methods`, `datasets`, `tools`

**Smoke test:** look up `ggplot2` in the map and print its direct dependencies.

---

### Step 3 — Resolve the full dependency tree

Given a package name, walk the dependency graph and collect every package that
needs to be installed (direct + transitive).

```rust
fn resolve(root: &str, index: &HashMap<String, Package>) -> Vec<String>
```

Use a simple BFS/DFS with a visited set to avoid cycles and duplicates. Version
conflicts are out of scope for the MVP — just take whatever version CRAN has.

**Smoke test:** print the full resolved list for `ggplot2` (~30-40 packages).

---

### Step 4 — Find binary download URLs for macOS

CRAN binary packages for macOS live at a URL that depends on:
- The R major.minor version (e.g. `4.4`)
- The macOS arm64 vs x86_64 architecture

URL pattern:
```
https://cloud.r-project.org/bin/macosx/big-sur-arm64/contrib/4.4/{Package}_{Version}.tgz
https://cloud.r-project.org/bin/macosx/big-sur-x86_64/contrib/4.4/{Package}_{Version}.tgz
```

To detect which to use:
- Arch: `std::env::consts::ARCH` — `"aarch64"` → arm64, `"x86_64"` → x86_64
- R version: shell out to `Rscript -e 'cat(R.version$major, R.version$minor)'`
  or hardcode `4.4` for now

**Smoke test:** print the resolved download URLs for all packages in the list.

---

### Step 5 — Download and extract binaries into `./ruv_lib/`

For each resolved package:
1. HTTP GET the `.tgz` URL
2. Decompress + untar into `./ruv_lib/`

R binary packages are tarballs with a single top-level directory named after the
package. Extracting them all into the same `ruv_lib/` directory produces the
correct library structure.

**Crates needed:**
```toml
tar = "0.4"
```

Do downloads sequentially first, then parallelise with `rayon` or `tokio` once
it works.

**Final smoke test:**
```bash
R_LIBS=./ruv_lib Rscript -e "library(ggplot2); ggplot()"
```

---

## Dependency summary

```toml
[dependencies]
reqwest = { version = "0.12", features = ["blocking"] }
flate2 = "1"
tar = "0.4"
```

---

## What the MVP explicitly does NOT do

- No lockfile
- No cache (re-downloads every time)
- No parallel downloads
- No Linux support (macOS only for now)
- No version resolution (always takes latest from CRAN)
- No R version management
- No `Suggests` dependencies
- No source package compilation

These are all Phase 2+ from `project_plan.md`.

---

## Order of work

1. `cargo init` + get reqwest fetching PACKAGES.gz printing to stdout
2. Parse into HashMap, print ggplot2's direct deps
3. BFS resolver, print full dep list for ggplot2
4. Build download URLs, print them
5. Download + extract into ruv_lib/
6. Smoke test with Rscript
