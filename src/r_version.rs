use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::version::{RVersion, VersionReq};

/// A discovered R installation: its version and path to the `bin/` directory.
#[derive(Debug, Clone)]
pub struct RInstallation {
    pub version: RVersion,
    pub bin_dir: PathBuf,
}

impl RInstallation {
    pub fn r_path(&self) -> PathBuf {
        self.bin_dir.join("R")
    }

    pub fn rscript_path(&self) -> PathBuf {
        self.bin_dir.join("Rscript")
    }
}

/// Probe standard locations and return all R installations found, sorted
/// descending by version (highest first).
pub fn find_r_installations() -> Vec<RInstallation> {
    let mut bin_dirs: Vec<PathBuf> = Vec::new();

    // macOS: R.framework versioned directories (standard CRAN install)
    #[cfg(target_os = "macos")]
    {
        let fw = Path::new("/Library/Frameworks/R.framework/Versions");
        if let Ok(entries) = std::fs::read_dir(fw) {
            for entry in entries.flatten() {
                let bin = entry.path().join("Resources/bin");
                if bin.join("R").exists() {
                    bin_dirs.push(bin);
                }
            }
        }
    }

    // Common PATH locations
    for dir in &["/usr/local/bin", "/usr/bin", "/opt/homebrew/bin"] {
        let bin = PathBuf::from(dir);
        if bin.join("R").exists() {
            bin_dirs.push(bin);
        }
    }

    // Managed by ruv: ~/.local/share/ruv/r/{version}/bin/
    if let Some(data_dir) = dirs::data_local_dir() {
        let managed = data_dir.join("ruv").join("r");
        if let Ok(entries) = std::fs::read_dir(&managed) {
            for entry in entries.flatten() {
                let bin = entry.path().join("bin");
                if bin.join("R").exists() {
                    bin_dirs.push(bin);
                }
            }
        }
    }

    let mut seen: HashSet<String> = HashSet::new();
    let mut installations: Vec<RInstallation> = bin_dirs
        .into_iter()
        .filter_map(|bin_dir| {
            let version = probe_r_version(&bin_dir.join("R"))?;
            let key = version.to_string();
            if seen.insert(key) {
                Some(RInstallation { version, bin_dir })
            } else {
                None
            }
        })
        .collect();

    // Highest version first so select_r picks the best match naturally
    installations.sort_by(|a, b| b.version.cmp(&a.version));
    installations
}

/// Run `R --version` and extract the version number.
fn probe_r_version(r_bin: &Path) -> Option<RVersion> {
    let output = std::process::Command::new(r_bin)
        .arg("--version")
        .output()
        .ok()?;
    // Some R versions write to stderr, some to stdout
    let text = String::from_utf8_lossy(&output.stdout);
    let text = if text.trim().is_empty() {
        String::from_utf8_lossy(&output.stderr).into_owned()
    } else {
        text.into_owned()
    };
    parse_r_version_output(&text)
}

/// Parse "R version 4.4.2 (2024-10-31) -- ..." → RVersion(4, 4, 2)
fn parse_r_version_output(text: &str) -> Option<RVersion> {
    let line = text.lines().next()?;
    let rest = line.strip_prefix("R version ")?;
    let version_str = rest.split_whitespace().next()?;
    RVersion::parse(version_str)
}

/// Find the best installed R satisfying `constraint` (e.g. `">=4.3"`).
pub fn select_r(constraint: &str) -> Result<RInstallation, String> {
    let req = VersionReq::parse(constraint)
        .ok_or_else(|| format!("could not parse r-version constraint: {}", constraint))?;

    let installations = find_r_installations();
    if installations.is_empty() {
        return Err(
            "no R installations found — install R from https://cran.r-project.org".to_string(),
        );
    }

    // Sorted descending, so first match is the highest satisfying version
    installations
        .into_iter()
        .find(|i| req.matches(&i.version))
        .ok_or_else(|| {
            format!(
                "no R installation satisfies {} — install R from https://cran.r-project.org",
                constraint
            )
        })
}

/// Create `.ruv/bin/R` and `.ruv/bin/Rscript` symlinks pointing at `installation`.
pub fn setup_r_symlinks(installation: &RInstallation) -> Result<(), String> {
    let bin_dir = Path::new(".ruv/bin");
    std::fs::create_dir_all(bin_dir).map_err(|e| format!("failed to create .ruv/bin: {}", e))?;

    make_symlink(&installation.r_path(), &bin_dir.join("R"))?;
    make_symlink(&installation.rscript_path(), &bin_dir.join("Rscript"))?;
    Ok(())
}

#[cfg(unix)]
fn make_symlink(target: &Path, link: &Path) -> Result<(), String> {
    // Remove stale link if present
    if link.symlink_metadata().is_ok() {
        std::fs::remove_file(link)
            .map_err(|e| format!("failed to remove {}: {}", link.display(), e))?;
    }
    std::os::unix::fs::symlink(target, link)
        .map_err(|e| format!("failed to create symlink {}: {}", link.display(), e))
}

#[cfg(not(unix))]
fn make_symlink(_target: &Path, _link: &Path) -> Result<(), String> {
    Err("R symlinks are not supported on this platform yet".to_string())
}

/// Return `.ruv/bin/Rscript` if project symlinks are set up, else `None`.
pub fn project_rscript() -> Option<PathBuf> {
    let p = PathBuf::from(".ruv/bin/Rscript");
    if p.exists() { Some(p) } else { None }
}

/// Return `.ruv/bin/R` if project symlinks are set up, else `None`.
pub fn project_r() -> Option<PathBuf> {
    let p = PathBuf::from(".ruv/bin/R");
    if p.exists() { Some(p) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_r_version_output() {
        let s = "R version 4.4.2 (2024-10-31) -- \"Pile of Leaves\"\nCopyright (C) 2024 The R Foundation";
        assert_eq!(parse_r_version_output(s).unwrap().to_string(), "4.4.2");
    }

    #[test]
    fn test_parse_r_version_output_short() {
        assert_eq!(
            parse_r_version_output("R version 4.3.0 (2023-04-21)")
                .unwrap()
                .to_string(),
            "4.3.0"
        );
    }

    #[test]
    fn test_parse_r_version_output_invalid() {
        assert!(parse_r_version_output("bash: R: command not found").is_none());
        assert!(parse_r_version_output("").is_none());
    }

    #[test]
    fn test_parse_r_version_two_part() {
        assert_eq!(
            parse_r_version_output("R version 4.5 (2025-01-01)")
                .unwrap()
                .to_string(),
            "4.5"
        );
    }
}
