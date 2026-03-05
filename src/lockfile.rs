use crate::index::Package;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

pub fn write_lockfile(roots: &[String], packages: &[String], index: &HashMap<String, Package>) {
    write_lockfile_to(Path::new("arrrv.lock"), roots, packages, index);
    println!("wrote arrrv.lock");
}

fn write_lockfile_to(
    path: &Path,
    roots: &[String],
    packages: &[String],
    index: &HashMap<String, Package>,
) {
    let mut sorted_roots = roots.to_vec();
    sorted_roots.sort();

    let mut out = String::from("# arrrv.lock — generated, do not edit\n\nversion = 1\n\n");
    out.push_str("[manifest]\n");
    out.push_str("dependencies = [");
    out.push_str(
        &sorted_roots
            .iter()
            .map(|d| format!("\"{}\"", d))
            .collect::<Vec<_>>()
            .join(", "),
    );
    out.push_str("]\n\n");

    let mut sorted = packages.to_vec();
    sorted.sort();
    for name in &sorted {
        if let Some(pkg) = index.get(name) {
            out.push_str("[[package]]\n");
            out.push_str(&format!("name = \"{}\"\n", name));
            out.push_str(&format!("version = \"{}\"\n", pkg.version));
            if !pkg.deps.is_empty() {
                let mut sorted_deps = pkg.deps.clone();
                sorted_deps.sort();
                // only include deps that are in the resolved package list
                let resolved_deps: Vec<_> = sorted_deps
                    .iter()
                    .filter(|d| packages.contains(d))
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
    }
    std::fs::write(path, out).unwrap();
}

/// Reads arrrv.lock and returns the list of locked (name, version) pairs.
pub fn read_lockfile() -> Vec<(String, String)> {
    let text = std::fs::read_to_string("arrrv.lock")
        .expect("no arrrv.lock found — run `arrrv lock` first");
    parse_lockfile(&text)
}

/// Returns true if the lockfile exists and its manifest deps match the given roots.
pub fn lockfile_is_fresh(roots: &[String]) -> bool {
    let Ok(text) = std::fs::read_to_string("arrrv.lock") else {
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

fn parse_lockfile(text: &str) -> Vec<(String, String)> {
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
        #[allow(dead_code)]
        dependencies: Vec<toml::Value>, // present but not used during sync
    }
    let lf: RawLockfile = toml::from_str(text).expect("failed to parse arrrv.lock");
    lf.package
        .into_iter()
        .map(|p| (p.name, p.version))
        .collect()
}

#[derive(Deserialize)]
struct LockfileHeader {
    manifest: Manifest,
}

#[derive(Deserialize)]
struct Manifest {
    dependencies: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::Package;

    fn make_index(entries: &[(&str, &str)]) -> HashMap<String, Package> {
        entries
            .iter()
            .map(|(name, version)| {
                (
                    name.to_string(),
                    Package {
                        version: version.to_string(),
                        deps: vec![],
                    },
                )
            })
            .collect()
    }

    #[test]
    fn test_write_lockfile_format() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let index = make_index(&[("ggplot2", "3.5.1"), ("rlang", "1.1.4")]);
        let roots = vec!["ggplot2".to_string()];
        let packages = vec!["ggplot2".to_string(), "rlang".to_string()];

        write_lockfile_to(tmp.path(), &roots, &packages, &index);

        let contents = std::fs::read_to_string(tmp.path()).unwrap();
        assert!(contents.contains("version = 1"));
        assert!(contents.contains("[manifest]"));
        assert!(contents.contains("dependencies = [\"ggplot2\"]"));
        assert!(contents.contains("[[package]]"));
        assert!(contents.contains("name = \"ggplot2\""));
        assert!(contents.contains("version = \"3.5.1\""));
    }

    #[test]
    fn test_write_lockfile_sorted() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let index = make_index(&[("zzz", "1.0"), ("aaa", "2.0")]);
        let roots = vec!["zzz".to_string(), "aaa".to_string()];
        let packages = vec!["zzz".to_string(), "aaa".to_string()];

        write_lockfile_to(tmp.path(), &roots, &packages, &index);

        let contents = std::fs::read_to_string(tmp.path()).unwrap();
        let aaa_pos = contents.find("\"aaa\"").unwrap();
        let zzz_pos = contents.find("\"zzz\"").unwrap();
        assert!(aaa_pos < zzz_pos);
    }

    #[test]
    fn test_write_lockfile_skips_unknown_packages() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let index = make_index(&[("ggplot2", "3.5.1")]);
        let roots = vec!["ggplot2".to_string()];
        let packages = vec!["ggplot2".to_string(), "unknown-pkg".to_string()];

        write_lockfile_to(tmp.path(), &roots, &packages, &index);

        let contents = std::fs::read_to_string(tmp.path()).unwrap();
        assert!(!contents.contains("unknown-pkg"));
    }

    #[test]
    fn test_parse_lockfile_roundtrip() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let index = make_index(&[("ggplot2", "3.5.1"), ("rlang", "1.1.4")]);
        let roots = vec!["ggplot2".to_string()];
        let packages = vec!["ggplot2".to_string(), "rlang".to_string()];

        write_lockfile_to(tmp.path(), &roots, &packages, &index);

        let text = std::fs::read_to_string(tmp.path()).unwrap();
        let mut parsed = parse_lockfile(&text);
        parsed.sort();

        assert_eq!(
            parsed,
            vec![
                ("ggplot2".to_string(), "3.5.1".to_string()),
                ("rlang".to_string(), "1.1.4".to_string()),
            ]
        );
    }
}
