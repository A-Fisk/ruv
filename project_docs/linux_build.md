# Linux compatibility — build and verification

See GitHub issue #50 for the umbrella tracking issue.

## Implementation phases

### Phase 1 — `install.sh` POSIX fix
- Change shebang `#!/bin/bash` → `#!/bin/sh`
- Replace `[[ ":$PATH:" == *":$INSTALL_DIR:"* ]]` with a `case` statement (POSIX-compatible)
- Update Linux `TARGET` strings: `x86_64-unknown-linux-gnu` → `x86_64-unknown-linux-musl`, add `aarch64-unknown-linux-musl`

### Phase 2 — Musl static builds in CI
Musl builds are statically linked — no glibc dependency, runs on any Linux kernel ≥ 3.2.

- **`Cargo.toml`**: Switch `reqwest` from `native-tls` → `rustls` (required — musl can't link OpenSSL)
- **`release.yml`**: Replace `x86_64-unknown-linux-gnu` with `x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl`; add `musl-tools` / `gcc-aarch64-linux-gnu` install steps
- **`Cargo.toml` dist targets**: Update to match

### Phase 3 — Package installs work on Linux
`get_arch()` in `installer.rs` panicked on Linux. RSPM URL structure differs:
- macOS: `bin/macosx/big-sur-arm64/contrib/4.4/pkg.tgz`
- Linux: `bin/linux/ubuntu-jammy/contrib/4.4/pkg.tar.gz`

Refactored to `get_platform()` which reads `/etc/os-release` and maps to RSPM distro names
(`ubuntu-jammy`, `ubuntu-noble`, etc). Handles `.tgz` vs `.tar.gz` extension difference.

### Phase 4 — Linux R discovery paths
- Moved `/opt/homebrew/bin` behind `#[cfg(target_os = "macos")]`
- Added `/opt/R/*/bin/` for Posit/rig managed installations on Linux

### Phase 5 — Linux R auto-download (follow-on issue)
The current stub errors with "only supported on macOS". The right approach is Posit pre-built
`.deb` binaries from `cdn.posit.co/r/{distro}/pkgs/r-{version}_1_amd64.deb` — same binaries
rig uses, no sudo required (extract the `.deb` as an `ar` archive → unpack `data.tar.xz`).

This is a separate follow-on issue after #50 is closed.

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

---

## Manual verification via Docker

### Prerequisites

**On macOS with an M-series chip**, Docker containers run as `aarch64` by default.
Use `--platform linux/amd64` to test x86_64 behaviour, or omit it to test aarch64.

**Docker daemon**: requires Docker Desktop or Colima (`brew install colima && colima start`).
The plain `brew install docker` only installs the CLI — you need the daemon too.

---

### Setup — build the musl binaries locally

```bash
# Install targets (one-time)
rustup target add x86_64-unknown-linux-musl
rustup target add aarch64-unknown-linux-musl

# On macOS you need musl cross-compilers
brew install FiloSottile/musl-cross/musl-cross

# Build x86_64
CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=x86_64-linux-musl-gcc \
  cargo build --release --target x86_64-unknown-linux-musl

# Build aarch64
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-musl-gcc \
  cargo build --release --target aarch64-unknown-linux-musl
```

---

### Phase 1 — verify `install.sh` works under POSIX sh

```bash
# Force x86_64 on M-series Mac
docker run --rm --platform linux/amd64 ubuntu:22.04 bash -c "
  apt-get update -q && apt-get install -y dash curl
  curl -fsSL https://github.com/A-Fisk/ruv/releases/latest/download/install.sh | dash -s
"
```

---

### Phase 2 — verify musl binary runs on old glibc

```bash
# Ubuntu 22.04 (GLIBC_2.35) — main compatibility target
docker run --rm --platform linux/amd64 \
  -v $(pwd)/target/x86_64-unknown-linux-musl/release/ruv:/usr/local/bin/ruv \
  ubuntu:22.04 ruv --help

# Ubuntu 20.04 (GLIBC_2.31)
docker run --rm --platform linux/amd64 \
  -v $(pwd)/target/x86_64-unknown-linux-musl/release/ruv:/usr/local/bin/ruv \
  ubuntu:20.04 ruv --help

# Alpine (musl natively — good smoke test)
docker run --rm --platform linux/amd64 \
  -v $(pwd)/target/x86_64-unknown-linux-musl/release/ruv:/usr/local/bin/ruv \
  alpine:latest ruv --help

# aarch64 (native on M-series Mac, no --platform needed)
docker run --rm \
  -v $(pwd)/target/aarch64-unknown-linux-musl/release/ruv:/usr/local/bin/ruv \
  ubuntu:22.04 ruv --help
```

---

### Phase 3 — verify package install works on Linux

```bash
docker run --rm --platform linux/amd64 \
  -v $(pwd)/target/x86_64-unknown-linux-musl/release/ruv:/usr/local/bin/ruv \
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

---

### Phase 4 — verify R discovery finds system R

```bash
docker run --rm --platform linux/amd64 \
  -v $(pwd)/target/x86_64-unknown-linux-musl/release/ruv:/usr/local/bin/ruv \
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

---

### End-to-end sanity check (all phases)

```bash
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

docker build --platform linux/amd64 -f /tmp/ruv-test.Dockerfile .
```

A clean `docker build` with no errors = all phases verified.
