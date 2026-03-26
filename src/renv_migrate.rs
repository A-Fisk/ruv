/// Parser and ruv.toml writer for `renv.lock` migration.
///
/// renv.lock is a JSON file with the structure:
/// ```json
/// {
///   "R": { "Version": "4.4.0", "Repositories": [{"Name": "CRAN", "URL": "..."}] },
///   "Packages": {
///     "ggplot2": { "Package": "ggplot2", "Source": "Repository", "Repository": "CRAN", ... },
///     ...
///   }
/// }
/// ```
///
/// We extract:
/// - R version (major.minor only, per ruv convention)
/// - Repository aliases + URLs
/// - Package names where Source == "Repository" (CRAN-like)
///
/// GitHub/local/other sources are skipped with a warning — they require
/// additional ruv config not expressible in a plain migration.
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

const CONFIG_FILE: &str = "ruv.toml";

#[derive(Deserialize)]
struct RenvLock {
    #[serde(rename = "R")]
    r: RenvR,
    #[serde(rename = "Packages", default)]
    packages: HashMap<String, RenvPackage>,
}

#[derive(Deserialize)]
struct RenvR {
    #[serde(rename = "Version")]
    version: String,
    #[serde(rename = "Repositories", default)]
    repositories: Vec<RenvRepository>,
}

#[derive(Deserialize)]
struct RenvRepository {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "URL")]
    url: String,
}

#[derive(Deserialize)]
struct RenvPackage {
    #[serde(rename = "Package")]
    package: String,
    #[serde(rename = "Source", default)]
    source: String,
}

/// The result of parsing an renv.lock.
#[derive(Debug)]
pub struct MigrateResult {
    /// R major.minor version string, e.g. "4.4"
    pub r_version: String,
    /// Repositories as (alias, url) pairs, in order
    pub repositories: Vec<(String, String)>,
    /// Package names successfully migrated (Source == "Repository")
    pub packages: Vec<String>,
    /// Package names skipped (non-CRAN source), with reason
    pub skipped: Vec<(String, String)>,
}

/// Parse `renv.lock` at `path` into a `MigrateResult`.
pub fn parse_renv_lock(path: &Path) -> Result<MigrateResult, String> {
    let text = std::fs::read_to_string(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            format!(
                "could not find {} — run this command from a directory containing renv.lock",
                path.display()
            )
        } else {
            format!("failed to read {}: {}", path.display(), e)
        }
    })?;

    let lock: RenvLock = serde_json::from_str(&text)
        .map_err(|e| format!("failed to parse {}: {}", path.display(), e))?;

    // Truncate R version to major.minor
    let r_version = lock
        .r
        .version
        .splitn(3, '.')
        .take(2)
        .collect::<Vec<_>>()
        .join(".");

    let repositories: Vec<(String, String)> = lock
        .r
        .repositories
        .into_iter()
        .map(|r| (r.name, r.url))
        .collect();

    let mut packages = Vec::new();
    let mut skipped = Vec::new();

    let mut sorted_pkgs: Vec<RenvPackage> = lock.packages.into_values().collect();
    sorted_pkgs.sort_by(|a, b| a.package.cmp(&b.package));

    for pkg in sorted_pkgs {
        match pkg.source.as_str() {
            "Repository" | "" => packages.push(pkg.package),
            "GitHub" => skipped.push((
                pkg.package,
                "GitHub source — add manually as a GitHub dependency once ruv supports it"
                    .to_string(),
            )),
            other => skipped.push((
                pkg.package,
                format!("unsupported source '{}' — add manually", other),
            )),
        }
    }

    Ok(MigrateResult {
        r_version,
        repositories,
        packages,
        skipped,
    })
}

/// Write a `ruv.toml` from a `MigrateResult`.
/// Errors if `ruv.toml` already exists in the current directory.
pub fn write_ruv_toml(project_name: &str, result: &MigrateResult) -> Result<(), String> {
    write_ruv_toml_to(Path::new(CONFIG_FILE), project_name, result)
}

pub fn write_ruv_toml_to(
    path: &Path,
    project_name: &str,
    result: &MigrateResult,
) -> Result<(), String> {
    if path.exists() {
        return Err(format!(
            "{} already exists — remove it first or run from a clean directory",
            path.display()
        ));
    }

    let mut out = format!(
        "# migrated from renv.lock by ruv migrate renv\n\
         [project]\n\
         name = \"{}\"\n\
         version = \"0.1.0\"\n\
         r-version = \"{}\"\n",
        project_name, result.r_version
    );

    if !result.repositories.is_empty() {
        out.push_str("\nrepositories = [\n");
        for (alias, url) in &result.repositories {
            out.push_str(&format!(
                "    {{ alias = \"{}\", url = \"{}\" }},\n",
                alias, url
            ));
        }
        out.push_str("]\n");
    }

    out.push_str("\ndependencies = [\n");
    for pkg in &result.packages {
        out.push_str(&format!("    \"{}\",\n", pkg));
    }
    out.push_str("]\n");

    std::fs::write(path, out).map_err(|e| format!("failed to write {}: {}", path.display(), e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp_lock(content: &str) -> tempfile::NamedTempFile {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f
    }

    const SAMPLE_LOCK: &str = r#"{
  "R": {
    "Version": "4.4.0",
    "Repositories": [
      { "Name": "CRAN", "URL": "https://cloud.r-project.org" }
    ]
  },
  "Packages": {
    "ggplot2": {
      "Package": "ggplot2",
      "Version": "3.5.1",
      "Source": "Repository",
      "Repository": "CRAN"
    },
    "dplyr": {
      "Package": "dplyr",
      "Version": "1.1.4",
      "Source": "Repository",
      "Repository": "CRAN"
    }
  }
}"#;

    #[test]
    fn test_parse_r_version_truncated_to_major_minor() {
        let f = write_temp_lock(SAMPLE_LOCK);
        let result = parse_renv_lock(f.path()).unwrap();
        assert_eq!(result.r_version, "4.4");
    }

    #[test]
    fn test_parse_repositories() {
        let f = write_temp_lock(SAMPLE_LOCK);
        let result = parse_renv_lock(f.path()).unwrap();
        assert_eq!(result.repositories.len(), 1);
        assert_eq!(result.repositories[0].0, "CRAN");
        assert_eq!(result.repositories[0].1, "https://cloud.r-project.org");
    }

    #[test]
    fn test_parse_packages_sorted() {
        let f = write_temp_lock(SAMPLE_LOCK);
        let result = parse_renv_lock(f.path()).unwrap();
        assert_eq!(result.packages, vec!["dplyr", "ggplot2"]);
        assert!(result.skipped.is_empty());
    }

    #[test]
    fn test_parse_skips_github_source() {
        let lock = r#"{
  "R": { "Version": "4.4.0", "Repositories": [] },
  "Packages": {
    "mypkg": { "Package": "mypkg", "Version": "0.1.0", "Source": "GitHub" }
  }
}"#;
        let f = write_temp_lock(lock);
        let result = parse_renv_lock(f.path()).unwrap();
        assert!(result.packages.is_empty());
        assert_eq!(result.skipped.len(), 1);
        assert_eq!(result.skipped[0].0, "mypkg");
        assert!(result.skipped[0].1.contains("GitHub"));
    }

    #[test]
    fn test_parse_skips_unknown_source() {
        let lock = r#"{
  "R": { "Version": "4.3.1", "Repositories": [] },
  "Packages": {
    "localpkg": { "Package": "localpkg", "Version": "0.1.0", "Source": "Local" }
  }
}"#;
        let f = write_temp_lock(lock);
        let result = parse_renv_lock(f.path()).unwrap();
        assert!(result.packages.is_empty());
        assert_eq!(result.skipped[0].0, "localpkg");
    }

    #[test]
    fn test_parse_missing_file_gives_clear_error() {
        let err = parse_renv_lock(Path::new("/nonexistent/renv.lock")).unwrap_err();
        assert!(err.contains("could not find"), "got: {}", err);
    }

    #[test]
    fn test_write_ruv_toml_content() {
        let result = MigrateResult {
            r_version: "4.4".to_string(),
            repositories: vec![(
                "CRAN".to_string(),
                "https://cloud.r-project.org".to_string(),
            )],
            packages: vec!["dplyr".to_string(), "ggplot2".to_string()],
            skipped: vec![],
        };
        let tmp = tempfile::NamedTempFile::new().unwrap();
        // write_ruv_toml_to errors if file exists, so remove it first
        let path = tmp.path().with_extension("ruv.toml");
        write_ruv_toml_to(&path, "my-project", &result).unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("r-version = \"4.4\""));
        assert!(contents.contains("alias = \"CRAN\""));
        assert!(contents.contains("\"dplyr\""));
        assert!(contents.contains("\"ggplot2\""));
    }

    #[test]
    fn test_write_ruv_toml_errors_if_exists() {
        let result = MigrateResult {
            r_version: "4.4".to_string(),
            repositories: vec![],
            packages: vec![],
            skipped: vec![],
        };
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let err = write_ruv_toml_to(tmp.path(), "proj", &result).unwrap_err();
        assert!(err.contains("already exists"), "got: {}", err);
    }
}
