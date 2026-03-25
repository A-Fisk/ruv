use crate::index::Package;
use crate::installer::get_r_version;
use crate::version::RVersion;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

const RSPM_BASE: &str = "https://packagemanager.posit.co/cran";

/// Write ruv.lock from the pubgrub-resolved map of package → version.
/// All packages use RSPM/latest as their registry — the exact version in the
/// filename is the reproducibility guarantee, not the snapshot date.
/// The R version used at lock time is recorded in [manifest] so sync can use
/// the same version regardless of what R is on PATH at sync time.
pub fn write_lockfile(
    roots: &[String],
    resolved: &HashMap<String, RVersion>,
    index: &HashMap<String, Package>,
) {
    write_lockfile_to(Path::new("ruv.lock"), roots, resolved, index);
    println!("wrote ruv.lock");
}

fn write_lockfile_to(
    path: &Path,
    roots: &[String],
    resolved: &HashMap<String, RVersion>,
    index: &HashMap<String, Package>,
) {
    let mut sorted_roots = roots.to_vec();
    sorted_roots.sort();

    let r_ver = get_r_version();

    let mut out = String::from("# ruv.lock — generated, do not edit\n\nversion = 1\n\n");
    out.push_str("[manifest]\n");
    out.push_str(&format!("r_version = \"{}\"\n", r_ver));
    out.push_str("dependencies = [");
    out.push_str(
        &sorted_roots
            .iter()
            .map(|d| format!("\"{}\"", d))
            .collect::<Vec<_>>()
            .join(", "),
    );
    out.push_str("]\n\n");

    let mut sorted_names: Vec<&String> = resolved.keys().collect();
    sorted_names.sort();

    for name in &sorted_names {
        // Use the original version string from the index (preserves dashes e.g. "2.23-26").
        // Fall back to RVersion Display only if somehow not in the index.
        let version_str = index
            .get(*name)
            .map(|p| p.version.as_str())
            .unwrap_or_else(|| "0");
        let registry = format!("{}/latest", RSPM_BASE);
        out.push_str("[[package]]\n");
        out.push_str(&format!("name = \"{}\"\n", name));
        out.push_str(&format!("version = \"{}\"\n", version_str));
        out.push_str(&format!("source = {{ registry = \"{}\" }}\n", registry));
        // write deps that are also in the resolved set
        if let Some(pkg) = index.get(*name)
            && !pkg.deps.is_empty()
        {
            let mut sorted_dep_names: Vec<&str> =
                pkg.deps.iter().map(|d| d.name.as_str()).collect();
            sorted_dep_names.sort();
            let resolved_deps: Vec<&str> = sorted_dep_names
                .into_iter()
                .filter(|d| resolved.contains_key(*d))
                .collect();
            if !resolved_deps.is_empty() {
                out.push_str("dependencies = [");
                out.push_str(
                    &resolved_deps
                        .iter()
                        .map(|d| format!("{{ name = \"{}\" }}", d))
                        .collect::<Vec<_>>()
                        .join(", "),
                );
                out.push_str("]\n");
            }
        }
        out.push('\n');
    }
    std::fs::write(path, out).unwrap();
}

/// Reads ruv.lock and returns the list of locked (name, version, registry_url) triples.
pub fn read_lockfile() -> Vec<(String, String, String)> {
    let text =
        std::fs::read_to_string("ruv.lock").expect("no ruv.lock found — run `ruv lock` first");
    parse_lockfile(&text)
}

/// Reads the R version recorded in the [manifest] section of ruv.lock.
/// Returns None if the lockfile predates r_version recording (older format).
pub fn read_lockfile_r_version() -> Option<String> {
    let text = std::fs::read_to_string("ruv.lock").ok()?;
    let lf: LockfileHeader = toml::from_str(&text).ok()?;
    lf.manifest.r_version
}

/// Returns true if the lockfile exists and its manifest deps match the given roots.
pub fn lockfile_is_fresh(roots: &[String]) -> bool {
    let Ok(text) = std::fs::read_to_string("ruv.lock") else {
        return false;
    };
    let Ok(lf) = toml::from_str::<LockfileHeader>(&text) else {
        return false;
    };
    let mut locked = lf.manifest.dependencies.clone();
    locked.sort();
    let mut current = roots.to_vec();
    current.sort();
    locked == current
}

fn parse_lockfile(text: &str) -> Vec<(String, String, String)> {
    #[derive(Deserialize)]
    struct RawLockfile {
        #[serde(default)]
        package: Vec<LockedPackage>,
    }
    #[derive(Deserialize)]
    struct LockedPackage {
        name: String,
        version: String,
        #[serde(default)]
        source: LockedSource,
        #[serde(default)]
        #[allow(dead_code)]
        dependencies: Vec<toml::Value>,
    }
    #[derive(Deserialize)]
    struct LockedSource {
        #[serde(default = "default_registry")]
        registry: String,
    }
    impl Default for LockedSource {
        fn default() -> Self {
            LockedSource {
                registry: default_registry(),
            }
        }
    }
    fn default_registry() -> String {
        format!("{}/latest", RSPM_BASE)
    }
    let lf: RawLockfile = toml::from_str(text).expect("failed to parse ruv.lock");
    lf.package
        .into_iter()
        .map(|p| (p.name, p.version, p.source.registry))
        .collect()
}

#[derive(Deserialize)]
struct LockfileHeader {
    manifest: Manifest,
}

#[derive(Deserialize)]
struct Manifest {
    #[serde(default)]
    r_version: Option<String>,
    dependencies: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::Package;
    use crate::version::RVersion;

    fn make_index(entries: &[(&str, &str)]) -> HashMap<String, Package> {
        entries
            .iter()
            .map(|(name, version)| {
                (
                    name.to_string(),
                    Package {
                        version: version.to_string(),
                        deps: vec![], // no deps needed for lockfile format tests
                    },
                )
            })
            .collect()
    }

    fn make_resolved(entries: &[(&str, &str)]) -> HashMap<String, RVersion> {
        entries
            .iter()
            .map(|(name, version)| (name.to_string(), RVersion::parse(version).unwrap()))
            .collect()
    }

    #[test]
    fn test_write_lockfile_format() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let index = make_index(&[("ggplot2", "3.5.1"), ("rlang", "1.1.4")]);
        let resolved = make_resolved(&[("ggplot2", "3.5.1"), ("rlang", "1.1.4")]);
        let roots = vec!["ggplot2".to_string()];

        write_lockfile_to(tmp.path(), &roots, &resolved, &index);

        let contents = std::fs::read_to_string(tmp.path()).unwrap();
        assert!(contents.contains("version = 1"));
        assert!(contents.contains("[manifest]"));
        assert!(contents.contains("dependencies = [\"ggplot2\"]"));
        assert!(contents.contains("[[package]]"));
        assert!(contents.contains("name = \"ggplot2\""));
        assert!(contents.contains("version = \"3.5.1\""));
    }

    #[test]
    fn test_write_lockfile_uses_rspm_latest() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let index = make_index(&[("ggplot2", "3.5.1")]);
        let resolved = make_resolved(&[("ggplot2", "3.5.1")]);
        let roots = vec!["ggplot2".to_string()];

        write_lockfile_to(tmp.path(), &roots, &resolved, &index);

        let contents = std::fs::read_to_string(tmp.path()).unwrap();
        assert!(
            contents.contains(
                "source = { registry = \"https://packagemanager.posit.co/cran/latest\" }"
            )
        );
    }

    #[test]
    fn test_write_lockfile_preserves_dash_version() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let index = make_index(&[("nlme", "2.23-26")]);
        let resolved = make_resolved(&[("nlme", "2.23-26")]);
        let roots = vec!["nlme".to_string()];

        write_lockfile_to(tmp.path(), &roots, &resolved, &index);

        let contents = std::fs::read_to_string(tmp.path()).unwrap();
        assert!(contents.contains("version = \"2.23-26\""));
        assert!(!contents.contains("2.23.26"));
    }

    #[test]
    fn test_write_lockfile_sorted() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let index = make_index(&[("zzz", "1.0"), ("aaa", "2.0")]);
        let resolved = make_resolved(&[("zzz", "1.0"), ("aaa", "2.0")]);
        let roots = vec!["zzz".to_string(), "aaa".to_string()];

        write_lockfile_to(tmp.path(), &roots, &resolved, &index);

        let contents = std::fs::read_to_string(tmp.path()).unwrap();
        let aaa_pos = contents.find("\"aaa\"").unwrap();
        let zzz_pos = contents.find("\"zzz\"").unwrap();
        assert!(aaa_pos < zzz_pos);
    }

    #[test]
    fn test_parse_lockfile_roundtrip() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let index = make_index(&[("ggplot2", "3.5.1"), ("rlang", "1.1.4")]);
        let resolved = make_resolved(&[("ggplot2", "3.5.1"), ("rlang", "1.1.4")]);
        let roots = vec!["ggplot2".to_string()];

        write_lockfile_to(tmp.path(), &roots, &resolved, &index);

        let text = std::fs::read_to_string(tmp.path()).unwrap();
        let mut parsed = parse_lockfile(&text);
        parsed.sort_by(|a, b| a.0.cmp(&b.0));

        assert_eq!(parsed[0].0, "ggplot2");
        assert_eq!(parsed[0].1, "3.5.1");
        assert_eq!(parsed[0].2, "https://packagemanager.posit.co/cran/latest");
        assert_eq!(parsed[1].0, "rlang");
        assert_eq!(parsed[1].1, "1.1.4");
    }

    #[test]
    fn test_write_lockfile_records_r_version() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let index = make_index(&[("ggplot2", "3.5.1")]);
        let resolved = make_resolved(&[("ggplot2", "3.5.1")]);
        let roots = vec!["ggplot2".to_string()];

        write_lockfile_to(tmp.path(), &roots, &resolved, &index);

        let contents = std::fs::read_to_string(tmp.path()).unwrap();
        // r_version line should be present in [manifest] and be non-empty
        assert!(contents.contains("r_version = \""));
    }

    #[test]
    fn test_parse_r_version_from_lockfile() {
        let text = r#"
version = 1

[manifest]
r_version = "4.4"
dependencies = ["ggplot2"]

[[package]]
name = "ggplot2"
version = "3.5.1"
source = { registry = "https://packagemanager.posit.co/cran/latest" }
"#;
        let lf: LockfileHeader = toml::from_str(text).unwrap();
        assert_eq!(lf.manifest.r_version, Some("4.4".to_string()));
    }

    #[test]
    fn test_parse_r_version_missing_is_none() {
        // Old lockfile format without r_version field — should not fail to parse
        let text = r#"
version = 1

[manifest]
dependencies = ["ggplot2"]

[[package]]
name = "ggplot2"
version = "3.5.1"
source = { registry = "https://packagemanager.posit.co/cran/latest" }
"#;
        let lf: LockfileHeader = toml::from_str(text).unwrap();
        assert_eq!(lf.manifest.r_version, None);
    }
}
