# Distribution Setup for ruv

This document explains the distribution infrastructure that allows users to install ruv without Rust.

## Overview

ruv now uses automated binary distribution with:
- **Cargo-dist** configuration for building multi-platform binaries
- **GitHub Actions** workflow that builds and publishes releases
- **Shell installer script** for `curl | sh` installation
- **Homebrew formula** for macOS users

## How it works

### 1. Cargo.toml Configuration (`[package.metadata.dist]`)

The `[package.metadata.dist]` section in `Cargo.toml` tells cargo-dist:
- Which platforms to build for: `aarch64-apple-darwin` (macOS ARM), `x86_64-apple-darwin` (macOS Intel), `x86_64-unknown-linux-gnu` (Linux)
- To generate `shell` installer scripts
- To create `tarball` distributions
- To generate `sha256` checksums for verification

### 2. GitHub Actions Release Workflow (`.github/workflows/release.yml`)

When you push a tag like `v0.2.0`, GitHub Actions automatically:

1. **Builds binaries** for all three platforms using a matrix strategy
2. **Creates tarballs** containing the binary (e.g., `ruv-aarch64-apple-darwin.tar.gz`)
3. **Generates SHA256 checksums** for security verification
4. **Creates an `install.sh` script** in the release
5. **Uploads everything** to the GitHub Release page

Workflow jobs:
- `build-matrix`: Builds the binary for each platform
- `create-release`: Coordinates the release
- `publish`: Generates the installer script and publishes artifacts

### 3. Installation Methods

#### Method A: Shell installer (recommended for first-time users)

```bash
# Download and run the installer from the latest release
curl -LsSf https://github.com/A-Fisk/ruv/releases/latest/download/install.sh | sh

# Or install a specific version
curl -LsSf https://github.com/A-Fisk/ruv/releases/download/v0.2.0/install.sh | sh
```

What the installer does:
1. Detects your OS (macOS/Linux) and architecture (ARM/Intel/x86_64)
2. Downloads the correct pre-built binary tarball from GitHub Releases
3. Extracts it to `~/.local/bin/ruv`
4. Makes it executable
5. Prints instructions to add `~/.local/bin` to your `$PATH` if needed

#### Method B: Homebrew (for macOS users)

Once you set up a tap (homebrew-ruv repo):

```bash
brew tap A-Fisk/ruv
brew install ruv
```

The Homebrew formula (`formula/ruv.rb`) is a template that must be updated with actual SHA256 hashes when you release a new version.

#### Method C: Direct download

Visit https://github.com/A-Fisk/ruv/releases and download the tarball for your platform, then extract it.

## How to Create a Release

### Step 1: Update the version

In `Cargo.toml`, bump the version (e.g., from `0.1.0` to `0.2.0`):

```toml
[package]
version = "0.2.0"
```

### Step 2: Commit and create a Git tag

```bash
git add Cargo.toml
git commit -m "chore: bump version to 0.2.0"
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin v0.2.0
```

### Step 3: GitHub Actions takes over

The `release.yml` workflow automatically:
1. Detects the `v0.2.0` tag
2. Builds binaries for all platforms
3. Uploads them to the release page
4. Generates the installer script

You can watch progress at: https://github.com/A-Fisk/ruv/actions

### Step 4: Verify the release

Visit https://github.com/A-Fisk/ruv/releases/tag/v0.2.0 and verify:
- ✓ Three tarballs (aarch64, x86_64 macOS, x86_64 Linux)
- ✓ `install.sh` script
- ✓ SHA256SUMS file with checksums

### Step 5 (Optional): Update Homebrew formula

Update `formula/ruv.rb` with actual SHA256 hashes from the release:

```bash
# Get the hashes
cd /path/to/release/tarballs
sha256sum *

# Update formula/ruv.rb with the hashes
# Then create a homebrew-ruv tap repo and push the formula there
```

## User Experience

A user with no Rust installed can now:

```bash
# Install in <60 seconds
curl -LsSf https://github.com/A-Fisk/ruv/releases/latest/download/install.sh | sh

# Add to PATH (one-time)
export PATH="$HOME/.local/bin:$PATH"

# Use immediately
ruv --help
ruv install ggplot2
```

This is exactly the same experience as installing other modern CLI tools (like `uv`, `ripgrep`, `fd`, etc).

## Files Changed

- `Cargo.toml` — Added `[package.metadata.dist]` configuration
- `.github/workflows/release.yml` — New automated release workflow
- `install.sh` — Shell installer script (can be run locally or via `curl | sh`)
- `formula/ruv.rb` — Homebrew formula (template)

## Next Steps

1. Create your first release tag: `git tag -a v0.1.0 -m "Initial release" && git push origin v0.1.0`
2. Verify the workflow runs at https://github.com/A-Fisk/ruv/actions
3. Check the release page at https://github.com/A-Fisk/ruv/releases
4. Test the installer: `curl -LsSf https://github.com/A-Fisk/ruv/releases/latest/download/install.sh | sh`
5. (Optional) Set up a `homebrew-ruv` tap repository for Homebrew installs

## Troubleshooting

### Binary not found in tarball

Check that the GitHub Actions workflow creates the correct directory structure:
```
ruv-aarch64-apple-darwin/
└── bin/
    └── ruv
```

### Installer script fails to download

Verify:
- The release tag exists in GitHub
- The tarball filename matches what the installer expects
- You have internet connectivity

### SHA256 checksum mismatch

The installer doesn't verify checksums by default (for simplicity), but you can manually verify:
```bash
sha256sum -c SHA256SUMS
```

This file is in the GitHub Release.
