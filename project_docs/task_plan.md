# Task Plan: Issue Triage & Prioritisation

## Goal
Work through open GitHub issues in priority order — quick wins first, then infrastructure, then core features, then polish.

## Decisions
- Tackle issues in phases by effort/impact ratio
- Quick housekeeping and CI issues first (unblocks everything else)
- Linux fix before Homebrew (distribution completeness)
- Core version management features before package source expansion
- Long-term polish / docs website deferred until core is stable

---

## Phase 1: Quick Wins (< 30 min each)

- [x] #47 — CI: add `main` to `pull_request` branches trigger (5-min YAML edit)
- [x] #48 — Add installation instructions to README
- [x] #40 — Create `project_docs/` dir, move root .md planning files
- [ ] #32 — Explain development workflow better in docs (`cargo run -- sync` etc.)
- [ ] #49 — Investigate / respond: does `ruv run R` launch a REPL?

## Phase 2: Documentation Accuracy

- [ ] #51 — Audit README feature comparison table; fix inaccurate claims (pak lockfiles etc.)
- [ ] #52 — Research `rv` (A2-ai/rv), document differences; update README or respond

## Phase 3: Distribution & Linux

- [ ] #50 — Fix install.sh `[[` → `[` (sh-compat); build musl target for glibc compat
- [ ] #29 — Verify full install flow for macOS + Linux; close if complete after #50 fix
- [ ] #43 — Homebrew tap: create `homebrew-ruv` formula / tap

## Phase 4: Core Features — Version Management

- [ ] #11 — `.r-version` file support
- [ ] #10 — Download & install R versions from official mirrors
- [ ] #12 — Auto-select correct R on `ruv sync` / `ruv run`

## Phase 5: Core Features — Package Sources & Dev Experience

- [ ] #15 — `ruv add` / `ruv remove` properly update lockfile
- [ ] #14 — GitHub and r-universe package sources
- [ ] #9  — Bioconductor source support
- [ ] #13 — `ruv run` with ephemeral per-script environments (like `uv run`)
- [ ] #42 — renv compatibility / lockfile interop
- [ ] #17 — `ruv import` renv migration path

## Phase 6: Polish & Long-term

- [ ] #41 — Build mdBook documentation website
- [ ] #16 — Shell completions (bash, zsh, fish)
- [ ] #25 — Windows compatibility
- [ ] #21 — IDE integration hooks (RStudio / Positron)
- [ ] #20 — `ruv publish` for CRAN / r-universe
- [ ] #19 — System dependency hints (sysreqs)
- [ ] #18 — Source package compilation caching on Linux

---

## Status
🟢 Phase 1 in progress (3/5 done)
