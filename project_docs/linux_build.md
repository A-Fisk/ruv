# Linux compatibility — build and verification

See GitHub issue #50 for the umbrella tracking issue.

## Implementation phases

### Phase 1 — `install.sh` POSIX fix (5 min, zero risk)
- Change shebang `#!/bin/bash` → `#!/bin/sh`
- Replace line 89 `[[ ":$PATH:" == *":$INSTALL_DIR:"* ]]` with a `case` statement (POSIX-compatible)
- Update the Linux `TARGET` string from `x86_64-unknown-linux-gnu` → `x86_64-unknown-linux-musl` (coupled to Phase 2)

### Phase 2 — Musl static build in CI (the core fix)
This is the real glibc fix. The binary currently links against `GLIBC_2.39` (Ubuntu 24.04 runner)
but Ubuntu 22.04 only has `GLIBC_2.35`. A musl build is statically linked — runs on any Linux kernel ≥ 3.2.

- **`Cargo.toml`**: Switch `reqwest` from `native-tls` → `rustls-tls` (required — musl can't link OpenSSL at build time)
- **`release.yml`**: Replace `x86_64-unknown-linux-gnu` with `x86_64-unknown-linux-musl`, add `musl-tools` install step
- **`Cargo.toml` dist targets**: Update the targets list to match

### Phase 3 — Package installs work on Linux
`get_arch()` in `installer.rs` currently panics on Linux. RSPM URL structure is different:
- macOS: `bin/macosx/big-sur-arm64/contrib/4.4/pkg.tgz`
- Linux: `bin/linux/ubuntu-jammy/contrib/4.4/pkg.tar.gz`

Refactor `get_arch()` → `get_platform_path()` that reads `/etc/os-release` on Linux and maps to
RSPM distro names (`ubuntu-jammy`, `ubuntu-noble`, etc). Also handle the `.tgz` vs `.tar.gz`
extension difference.

### Phase 4 — Linux R discovery paths
`find_r_installations()` searches `/opt/homebrew/bin` (macOS only — move behind
`#[cfg(target_os = "macos")]`) and misses Linux-specific paths:
- `/opt/R/*/bin/` — where Posit/rig installs R versions
- `/usr/bin/R` — already covered

### Phase 5 — Linux R auto-download (follow-on issue)
The current stub just errors with "only supported on macOS". The right approach is **Posit
pre-built `.deb` binaries** from `cdn.posit.co/r/{distro}/pkgs/r-{version}_1_amd64.deb` — same
binaries rig uses, no sudo required (extract the `.deb` as an `ar` archive → unpack `data.tar.xz`).

This is more work so should be its own issue after #50 is closed.

### Dependency order

```
Phase 1 (install.sh)
    ↓
Phase 2 (musl build) ← reqwest→rustls is a hard dependency
    ↓
Phase 3 (RSPM Linux URLs) ← needed for ruv sync to work on Linux
Phase 4 (R discovery paths) ← independent, low risk
    ↓
Phase 5 (auto-download R on Linux) ← separate follow-on issue
```

Phases 1–4 together close #50 and make ruv fully functional on Linux (install, sync, run).
Phase 5 is a separate issue for R version management on Linux.

---

## Manual verification via Docker

### Setup — build the musl binary locally first
```bash
# Install the musl target (one-time)
rustup target add x86_64-unknown-linux-musl

# On macOS you need a musl cross-compiler
brew install FiloSottile/musl-cross/musl-cross

# Build
CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=x86_64-linux-musl-gcc \
  cargo build --release --target x86_64-unknown-linux-musl
```

### Phase 1 — verify `install.sh` works under POSIX sh
```bash
# Run the install script under dash (Ubuntu's /bin/sh) — catches any bash-isms
docker run --rm ubuntu:22.04 bash -c "
  apt-get install -y dash curl > /dev/null 2>&1
  curl -fsSL https://github.com/A-Fisk/ruv/releases/latest/download/install.sh | dash -s
"
```

### Phase 2 — verify musl binary runs on old glibc
```bash
# Ubuntu 22.04 has GLIBC_2.35 — this is the main compatibility target
docker run --rm -v $(pwd)/target/x86_64-unknown-linux-musl/release/ruv:/usr/local/bin/ruv \
  ubuntu:22.04 ruv --help

# Also test on older Ubuntu 20.04 (GLIBC_2.31)
docker run --rm -v $(pwd)/target/x86_64-unknown-linux-musl/release/ruv:/usr/local/bin/ruv \
  ubuntu:20.04 ruv --help

# And Alpine (musl natively — good smoke test)
docker run --rm -v $(pwd)/target/x86_64-unknown-linux-musl/release/ruv:/usr/local/bin/ruv \
  alpine:latest ruv --help
```

### Phase 3 — verify package install works on Linux
```bash
docker run --rm -v $(pwd)/target/x86_64-unknown-linux-musl/release/ruv:/usr/local/bin/ruv \
  ubuntu:22.04 bash -c "
    apt-get update -q && apt-get install -y r-base
    mkdir /test && cd /test
    cat > ruv.toml << 'EOF'
[project]
name = \"test\"
version = \"0.1.0\"
dependencies = [\"jsonlite\"]
EOF
    ruv lock
    ruv sync
    ruv run -e 'library(jsonlite); cat(\"OK\n\")'
  "
```

### Phase 4 — verify R discovery finds system R
```bash
docker run --rm -v $(pwd)/target/x86_64-unknown-linux-musl/release/ruv:/usr/local/bin/ruv \
  ubuntu:22.04 bash -c "
    apt-get update -q && apt-get install -y r-base
    mkdir /test && cd /test
    cat > ruv.toml << 'EOF'
[project]
name = \"test\"
version = \"0.1.0\"
r-version = \"4\"
dependencies = []
EOF
    ruv lock && ruv sync
  "
# Should print: Using R 4.x.x (/usr/bin)
# Should NOT print: R not found locally, downloading...
```

### End-to-end sanity check (all phases)
```bash
# Minimal Dockerfile for a clean reproducible test
cat > /tmp/ruv-test.Dockerfile << 'EOF'
FROM ubuntu:22.04
RUN apt-get update && apt-get install -y r-base && rm -rf /var/lib/apt/lists/*
COPY target/x86_64-unknown-linux-musl/release/ruv /usr/local/bin/ruv
WORKDIR /project
RUN cat > ruv.toml << 'TOML'
[project]
name = "test"
version = "0.1.0"
r-version = "4"
dependencies = ["jsonlite", "dplyr"]
TOML
RUN ruv lock && ruv sync
RUN ruv run -e 'library(jsonlite); library(dplyr); cat("all packages OK\n")'
EOF

docker build -f /tmp/ruv-test.Dockerfile .
```

A clean `docker build` with no errors = all phases verified.
