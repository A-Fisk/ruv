# Test coverage

## Summary

| Module | Functions | Tested | Untested |
|---|---|---|---|
| `cache.rs` | 5 | 4 | 1 |
| `config.rs` | 2 | 1 | 1 |
| `index.rs` | 3 | 1 | 2 |
| `installer.rs` | 4 | 0 | 4 |
| `lockfile.rs` | 1 | 0 | 1 |
| `resolver.rs` | 2 | 2 | 0 |
| `main.rs` | 1 | 0 | 1 |

---

## `cache.rs`

| Function | Visibility | Test | Type | Notes |
|---|---|---|---|---|
| `cache_dir` | pub | none | — | Returns platform cache dir — depends on OS env, not worth unit testing |
| `package_cache_path` | pub | `test_package_cache_path_format` | Unit | Verifies `name_version` path format is correct |
| `is_cached` | pub | `test_is_cached_returns_false_when_missing` | Integration | Checks `.exists()` on a real (temp) path |
| `hard_link_into_library` | pub | none | — | Thin wrapper around `hard_link_dir` — covered indirectly by `hard_link_dir` tests |
| `hard_link_dir` | private | `test_hard_link_dir_copies_files`, `test_hard_link_dir_creates_true_hard_links` | Integration | Uses `tempfile` to create real dirs; verifies files exist and share inodes |

---

## `config.rs`

| Function | Visibility | Test | Type | Notes |
|---|---|---|---|---|
| `read_config` | pub | none | — | Reads from `arrrv.toml` on disk — needs a fixture file to test properly |
| `parse_dep_name` | pub | `test_parse_dep_name_with_gte`, `test_parse_dep_name_no_version`, `test_parse_dep_name_with_spaces`, `test_parse_dep_name_preserves_dots_and_dashes` | Unit | Pure string function, fully covered across 4 cases |

---

## `index.rs`

| Function | Visibility | Test | Type | Notes |
|---|---|---|---|---|
| `parse_packages` | pub | `test_parse_single_package`, `test_parse_strips_version_constraints`, `test_parse_filters_base_packages`, `test_parse_multiple_packages` | Unit | Pure function taking a string — well covered |
| `parse_from_bytes` | private | none | — | Thin wrapper over `parse_packages` + gzip decode — covered indirectly |
| `fetch_cran_index` | pub | none | — | Makes a network call and touches the filesystem — requires network or mocking to test |

---

## `installer.rs`

| Function | Visibility | Test | Type | Notes |
|---|---|---|---|---|
| `get_arch` | pub | none | — | Wraps `std::env::consts::ARCH` — effectively a constant at runtime |
| `get_r_version` | pub | none | — | Shells out to `Rscript` — requires R installed; worth an integration test |
| `build_urls` | pub | none | — | Good candidate for a unit test: given a fake index, verify URL format |
| `download_and_install` | pub | none | — | Network + filesystem + parallelism — hard to unit test; needs an integration test with a mock server |

---

## `lockfile.rs`

| Function | Visibility | Test | Type | Notes |
|---|---|---|---|---|
| `write_lockfile` | pub | none | — | Writes to disk — good candidate for an integration test using `tempfile` |

---

## `resolver.rs`

| Function | Visibility | Test | Type | Notes |
|---|---|---|---|---|
| `resolve` | pub | `test_resolve_transitive_deps`, `test_resolve_deduplicates`, `test_resolve_excludes_root`, `test_resolve_unknown_package_returns_empty` | Unit | Pure function with fake index — well covered |
| `resolve_all` | pub | `test_resolve_all_unions_results` | Unit | Covered with a basic union case — could add more edge cases |

---

## `main.rs`

| Function | Visibility | Test | Type | Notes |
|---|---|---|---|---|
| `main` | private | none | — | CLI entry point — tested manually via `cargo run` |

---

## Gaps worth closing

In priority order:

1. **`build_urls`** — pure-ish function (depends on arch/R version), but can be tested
   by passing a fake index and asserting URL format. Medium priority.

2. **`write_lockfile`** — straightforward integration test with `tempfile`. Low effort,
   good value.

3. **`read_config`** — integration test using a temp `arrrv.toml` fixture. Low effort.

4. **`get_r_version`** — integration test that shells out to R. Only runs if R is installed,
   so should be gated or placed in a separate test that can be skipped in CI.

5. **`download_and_install`** — hardest to test. Requires either a mock HTTP server
   (e.g. `mockito` crate) or a live network call. Low priority for now.
