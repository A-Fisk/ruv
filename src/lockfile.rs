use std::collections::HashMap;
use std::path::Path;
use crate::index::Package;

pub fn write_lockfile(packages: &[String], index: &HashMap<String, Package>) {
    write_lockfile_to(Path::new("arrrv.lock"), packages, index);
    println!("wrote arrrv.lock");
}

fn write_lockfile_to(path: &Path, packages: &[String], index: &HashMap<String, Package>) {
    let mut out = String::from("# arrrv.lock — generated, do not edit\n\n");
    let mut sorted = packages.to_vec();
    sorted.sort();
    for name in &sorted {
        if let Some(pkg) = index.get(name) {
            out.push_str("[[package]]\n");
            out.push_str(&format!("name = \"{}\"\n", name));
            out.push_str(&format!("version = \"{}\"\n\n", pkg.version));
        }
    }
    std::fs::write(path, out).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::Package;

    fn make_index(entries: &[(&str, &str)]) -> HashMap<String, Package> {
        entries.iter().map(|(name, version)| {
            (name.to_string(), Package { version: version.to_string(), deps: vec![] })
        }).collect()
    }

    #[test]
    fn test_write_lockfile_format() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let index = make_index(&[("ggplot2", "3.5.1"), ("rlang", "1.1.4")]);
        let packages = vec!["ggplot2".to_string(), "rlang".to_string()];

        write_lockfile_to(tmp.path(), &packages, &index);

        let contents = std::fs::read_to_string(tmp.path()).unwrap();
        assert!(contents.contains("[[package]]"));
        assert!(contents.contains("name = \"ggplot2\""));
        assert!(contents.contains("version = \"3.5.1\""));
    }

    #[test]
    fn test_write_lockfile_sorted() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let index = make_index(&[("zzz", "1.0"), ("aaa", "2.0")]);
        let packages = vec!["zzz".to_string(), "aaa".to_string()];

        write_lockfile_to(tmp.path(), &packages, &index);

        let contents = std::fs::read_to_string(tmp.path()).unwrap();
        let aaa_pos = contents.find("\"aaa\"").unwrap();
        let zzz_pos = contents.find("\"zzz\"").unwrap();
        assert!(aaa_pos < zzz_pos);
    }

    #[test]
    fn test_write_lockfile_skips_unknown_packages() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let index = make_index(&[("ggplot2", "3.5.1")]);
        let packages = vec!["ggplot2".to_string(), "unknown-pkg".to_string()];

        write_lockfile_to(tmp.path(), &packages, &index);

        let contents = std::fs::read_to_string(tmp.path()).unwrap();
        assert!(!contents.contains("unknown-pkg"));
    }
}
