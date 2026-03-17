use std::collections::HashSet;
use std::io::Read;
use std::path::{Path, PathBuf};

use indicatif::{ProgressBar, ProgressStyle};

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

// ── Discovery ────────────────────────────────────────────────────────────────

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

    // macOS: Homebrew
    #[cfg(target_os = "macos")]
    {
        let bin = PathBuf::from("/opt/homebrew/bin");
        if bin.join("R").exists() {
            bin_dirs.push(bin);
        }
    }

    // Common PATH locations
    for dir in &["/usr/local/bin", "/usr/bin"] {
        let bin = PathBuf::from(dir);
        if bin.join("R").exists() {
            bin_dirs.push(bin);
        }
    }

    // Linux: Posit/rig managed installations at /opt/R/{version}/bin/
    #[cfg(target_os = "linux")]
    {
        let opt_r = Path::new("/opt/R");
        if let Ok(entries) = std::fs::read_dir(opt_r) {
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

    // Managed by ruv: ~/.local/share/ruv/r/{version}/bin/
    // Trust the directory name for the version — probing fails when R can't find
    // its own framework outside the standard /Library/Frameworks location.
    if let Some(data_dir) = dirs::data_local_dir() {
        let managed = data_dir.join("ruv").join("r");
        if let Ok(entries) = std::fs::read_dir(&managed) {
            for entry in entries.flatten() {
                let bin = entry.path().join("bin");
                if !bin.join("R").exists() {
                    continue;
                }
                let dir_name = entry.file_name().to_string_lossy().to_string();
                let Some(version) = RVersion::parse(&dir_name) else {
                    continue;
                };
                if seen.insert(version.to_string()) {
                    installations.push(RInstallation {
                        version,
                        bin_dir: bin,
                    });
                }
            }
        }
    }

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

// ── Selection ─────────────────────────────────────────────────────────────────

/// Prefix match: `"4.3"` matches `4.3.0`, `4.3.1`, `4.3.2` but not `4.4.0`.
fn bare_version_matches(installed: &RVersion, spec: &RVersion) -> bool {
    spec.parts()
        .iter()
        .enumerate()
        .all(|(i, part)| installed.parts().get(i).copied().unwrap_or(0) == *part)
}

type ConstraintFn = Box<dyn Fn(&RVersion) -> bool>;

/// Build a predicate from a constraint string (operator or bare version).
fn make_constraint(constraint: &str) -> Result<ConstraintFn, String> {
    if let Some(req) = VersionReq::parse(constraint) {
        Ok(Box::new(move |v| req.matches(v)))
    } else if let Some(spec) = RVersion::parse(constraint) {
        Ok(Box::new(move |v| bare_version_matches(v, &spec)))
    } else {
        Err(format!(
            "could not parse r-version constraint: {}",
            constraint
        ))
    }
}

/// Find the best *already installed* R satisfying `constraint`.
///
/// Constraint formats:
/// - `"4.3"` (bare) — prefix match: any R 4.3.x
/// - `">=4.3"`, `"==4.3.2"`, etc. — standard version operator
pub fn select_r(constraint: &str) -> Result<RInstallation, String> {
    let matches_constraint = make_constraint(constraint)?;

    let installations = find_r_installations();

    let found: Vec<String> = installations
        .iter()
        .map(|i| i.version.to_string())
        .collect();

    // Sorted descending, so first match is the highest satisfying version
    installations
        .into_iter()
        .find(|i| matches_constraint(&i.version))
        .ok_or_else(|| {
            if found.is_empty() {
                "no R installations found on this system".to_string()
            } else {
                format!(
                    "r-version = \"{}\" not satisfied by any installed R (found: {})",
                    constraint,
                    found.join(", ")
                )
            }
        })
}

// ── Download & install ────────────────────────────────────────────────────────

fn r_managed_dir() -> PathBuf {
    dirs::data_local_dir()
        .expect("could not find local data directory")
        .join("ruv")
        .join("r")
}

#[cfg(target_os = "macos")]
fn cran_arch() -> &'static str {
    match std::env::consts::ARCH {
        "aarch64" => "big-sur-arm64",
        _ => "big-sur-x86_64",
    }
}

#[cfg(target_os = "macos")]
fn cran_arch_suffix() -> &'static str {
    match std::env::consts::ARCH {
        "aarch64" => "arm64",
        _ => "x86_64",
    }
}

/// Fetch available R versions for this platform.
fn fetch_available_r_versions() -> Result<Vec<RVersion>, String> {
    #[cfg(target_os = "macos")]
    return fetch_available_r_versions_macos();
    #[cfg(target_os = "linux")]
    return fetch_available_r_versions_linux();
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    Err("automatic R installation is not supported on this platform".to_string())
}

#[cfg(target_os = "macos")]
fn fetch_available_r_versions_macos() -> Result<Vec<RVersion>, String> {
    let url = format!(
        "https://cran.r-project.org/bin/macosx/{}/base/",
        cran_arch()
    );
    let body = reqwest::blocking::get(&url)
        .map_err(|e| format!("failed to fetch CRAN version list: {}", e))?
        .text()
        .map_err(|e| format!("failed to read CRAN version list: {}", e))?;
    let versions = parse_cran_pkg_listing(&body, cran_arch_suffix());
    if versions.is_empty() {
        Err("no R versions found in CRAN listing".to_string())
    } else {
        Ok(versions)
    }
}

/// Read /etc/os-release and return the Posit CDN distro string, e.g. "ubuntu-2204".
#[cfg(target_os = "linux")]
fn linux_posit_distro() -> String {
    let content = std::fs::read_to_string("/etc/os-release").unwrap_or_default();
    let mut id = String::new();
    let mut version_id = String::new();
    for line in content.lines() {
        if let Some(v) = line.strip_prefix("ID=") {
            id = v.trim_matches('"').to_lowercase();
        } else if let Some(v) = line.strip_prefix("VERSION_ID=") {
            version_id = v.trim_matches('"').replace('.', "");
        }
    }
    match id.as_str() {
        "ubuntu" | "debian" if !version_id.is_empty() => format!("{}-{}", id, version_id),
        _ => {
            eprintln!(
                "warning: unrecognised Linux distro ({} {}), defaulting to ubuntu-2204 for Posit CDN",
                id, version_id
            );
            "ubuntu-2204".to_string()
        }
    }
}

#[cfg(target_os = "linux")]
fn linux_posit_arch() -> &'static str {
    match std::env::consts::ARCH {
        "aarch64" => "arm64",
        _ => "amd64",
    }
}

#[cfg(target_os = "linux")]
fn fetch_available_r_versions_linux() -> Result<Vec<RVersion>, String> {
    let distro = linux_posit_distro();
    let arch = linux_posit_arch();
    let url = format!("https://cdn.posit.co/r/{}/pkgs/", distro);
    let body = reqwest::blocking::get(&url)
        .map_err(|e| format!("failed to fetch Posit R version list: {}", e))?
        .text()
        .map_err(|e| format!("failed to read Posit R version list: {}", e))?;
    let versions = parse_posit_deb_listing(&body, arch);
    if versions.is_empty() {
        Err(format!(
            "no R versions found for {} on Posit CDN ({})",
            distro, url
        ))
    } else {
        Ok(versions)
    }
}

/// Parse Posit CDN directory listing HTML for .deb filenames, e.g. `r-4.3.2_1_amd64.deb`.
#[cfg(target_os = "linux")]
fn parse_posit_deb_listing(html: &str, arch: &str) -> Vec<RVersion> {
    let suffix = format!("_1_{}.deb", arch);
    let mut versions = Vec::new();
    for part in html.split("r-") {
        let Some(end) = part.find(suffix.as_str()) else {
            continue;
        };
        let version_str = &part[..end];
        if let Some(v) = RVersion::parse(version_str) {
            versions.push(v);
        }
    }
    versions.sort_by(|a, b| b.cmp(a));
    versions.dedup_by(|a, b| a == b);
    versions
}

/// Parse CRAN directory listing HTML for .pkg filenames, e.g. `R-4.3.2-arm64.pkg`.
fn parse_cran_pkg_listing(html: &str, arch_suffix: &str) -> Vec<RVersion> {
    let suffix = format!("-{}.pkg", arch_suffix);
    let mut versions = Vec::new();
    for part in html.split("R-") {
        let Some(end) = part.find(suffix.as_str()) else {
            continue;
        };
        let version_str = &part[..end];
        if let Some(v) = RVersion::parse(version_str) {
            versions.push(v);
        }
    }
    versions.sort_by(|a, b| b.cmp(a)); // descending
    versions.dedup_by(|a, b| a == b);
    versions
}

/// Pick the best version from `available` that satisfies `constraint`.
fn resolve_version_to_download(
    constraint: &str,
    available: &[RVersion],
) -> Result<RVersion, String> {
    let matches_constraint = make_constraint(constraint)?;
    available
        .iter()
        .find(|v| matches_constraint(v))
        .cloned()
        .ok_or_else(|| {
            format!(
                "no R version on CRAN satisfies {} — check https://cran.r-project.org",
                constraint
            )
        })
}

/// Download the R .pkg for `version` to a temp file, with a progress bar.
#[cfg(target_os = "macos")]
fn download_r_pkg(version: &RVersion) -> Result<PathBuf, String> {
    let url = format!(
        "https://cran.r-project.org/bin/macosx/{}/base/R-{}-{}.pkg",
        cran_arch(),
        version,
        cran_arch_suffix()
    );

    let response = reqwest::blocking::get(&url)
        .map_err(|e| format!("failed to download R {}: {}", version, e))?;

    if !response.status().is_success() {
        return Err(format!(
            "failed to download R {} (HTTP {})\n       URL: {}",
            version,
            response.status(),
            url
        ));
    }

    let total = response.content_length().unwrap_or(0);
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::with_template(
            "  Downloading R {msg} [{wide_bar:.green/dim}] {bytes:>10}/{total_bytes:<10}",
        )
        .unwrap()
        .progress_chars("━━╌"),
    );
    pb.set_message(version.to_string());

    let mut src = pb.wrap_read(response);
    let mut bytes = Vec::new();
    src.read_to_end(&mut bytes)
        .map_err(|e| format!("failed to read R download: {}", e))?;
    pb.finish_and_clear();

    let tmp = std::env::temp_dir().join(format!("ruv-R-{}-{}.pkg", version, cran_arch_suffix()));
    std::fs::write(&tmp, &bytes).map_err(|e| format!("failed to write R pkg to temp: {}", e))?;
    Ok(tmp)
}

/// Extract the R .pkg into `dest_dir` using `pkgutil --expand` + `cpio`.
/// Only available on macOS.
#[cfg(target_os = "macos")]
fn extract_r_pkg(pkg_path: &Path, dest_dir: &Path) -> Result<(), String> {
    let expand_dir = std::env::temp_dir().join(format!("ruv-r-expand-{}", std::process::id()));

    // Step 1: expand the XAR archive
    let status = std::process::Command::new("pkgutil")
        .args([
            "--expand",
            &pkg_path.to_string_lossy(),
            &expand_dir.to_string_lossy(),
        ])
        .status()
        .map_err(|e| format!("failed to run pkgutil: {}", e))?;
    if !status.success() {
        return Err("pkgutil --expand failed".to_string());
    }

    // Step 2: find the R framework payload (R-fw.pkg/Payload)
    let payload = find_fw_payload(&expand_dir)?;

    // Step 3: decompress cpio payload into dest_dir
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!(
            "gzip -dc '{}' | cpio -id 2>/dev/null",
            payload.to_string_lossy()
        ))
        .current_dir(dest_dir)
        .status()
        .map_err(|e| format!("failed to extract R framework: {}", e))?;
    if !status.success() {
        return Err("cpio extraction failed".to_string());
    }

    // Clean up temp expand dir
    let _ = std::fs::remove_dir_all(&expand_dir);
    Ok(())
}

/// Download the R .deb for `version` to a temp file, with a progress bar.
#[cfg(target_os = "linux")]
fn download_r_deb(version: &RVersion) -> Result<PathBuf, String> {
    let distro = linux_posit_distro();
    let arch = linux_posit_arch();
    let url = format!(
        "https://cdn.posit.co/r/{}/pkgs/r-{}_1_{}.deb",
        distro, version, arch
    );

    let response = reqwest::blocking::get(&url)
        .map_err(|e| format!("failed to download R {}: {}", version, e))?;

    if !response.status().is_success() {
        return Err(format!(
            "failed to download R {} (HTTP {})\n       URL: {}",
            version,
            response.status(),
            url
        ));
    }

    let total = response.content_length().unwrap_or(0);
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::with_template(
            "  Downloading R {msg} [{wide_bar:.green/dim}] {bytes:>10}/{total_bytes:<10}",
        )
        .unwrap()
        .progress_chars("━━╌"),
    );
    pb.set_message(version.to_string());

    let mut src = pb.wrap_read(response);
    let mut bytes = Vec::new();
    src.read_to_end(&mut bytes)
        .map_err(|e| format!("failed to read R download: {}", e))?;
    pb.finish_and_clear();

    let tmp = std::env::temp_dir().join(format!("ruv-R-{}-{}.deb", version, arch));
    std::fs::write(&tmp, &bytes).map_err(|e| format!("failed to write R deb to temp: {}", e))?;
    Ok(tmp)
}

/// Extract a Posit .deb into `dest_dir` using `ar` + `tar`.
/// Requires `binutils` (provides `ar`) — present on all standard Linux distros.
#[cfg(target_os = "linux")]
fn extract_r_deb(deb_path: &Path, dest_dir: &Path) -> Result<(), String> {
    let work_dir = std::env::temp_dir().join(format!("ruv-deb-{}", std::process::id()));
    std::fs::create_dir_all(&work_dir).map_err(|e| format!("failed to create temp dir: {}", e))?;

    // Step 1: ar x file.deb — extracts debian-binary, control.tar.*, data.tar.*
    let status = std::process::Command::new("ar")
        .args(["x", &deb_path.to_string_lossy()])
        .current_dir(&work_dir)
        .status()
        .map_err(|e| format!("failed to run ar (is binutils installed?): {}", e))?;
    if !status.success() {
        return Err("ar extraction of .deb failed".to_string());
    }

    // Step 2: find data.tar.* (xz, gz, or zst depending on distro)
    let data_tar = find_deb_data_tar(&work_dir)?;

    // Step 3: extract data tarball into dest_dir
    let status = std::process::Command::new("tar")
        .args([
            "xf",
            &data_tar.to_string_lossy(),
            "-C",
            &dest_dir.to_string_lossy(),
        ])
        .status()
        .map_err(|e| format!("failed to run tar: {}", e))?;
    if !status.success() {
        return Err("tar extraction of R deb data failed".to_string());
    }

    let _ = std::fs::remove_dir_all(&work_dir);
    Ok(())
}

/// Find data.tar.* inside an extracted .deb working directory.
#[cfg(target_os = "linux")]
fn find_deb_data_tar(work_dir: &Path) -> Result<PathBuf, String> {
    for name in &["data.tar.xz", "data.tar.gz", "data.tar.zst"] {
        let p = work_dir.join(name);
        if p.exists() {
            return Ok(p);
        }
    }
    Err(format!(
        "could not find data.tar.* in extracted .deb at {}",
        work_dir.display()
    ))
}

/// Download and extract R for the current platform into `install_dir`.
fn download_and_extract_r(version: &RVersion, install_dir: &Path) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let pkg_path = download_r_pkg(version)?;
        println!("  Installing R {}...", version);
        extract_r_pkg(&pkg_path, install_dir)?;
        let _ = std::fs::remove_file(&pkg_path);
        Ok(())
    }
    #[cfg(target_os = "linux")]
    {
        let deb_path = download_r_deb(version)?;
        println!("  Installing R {}...", version);
        extract_r_deb(&deb_path, install_dir)?;
        let _ = std::fs::remove_file(&deb_path);
        Ok(())
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = (version, install_dir);
        Err("automatic R installation is not supported on this platform".to_string())
    }
}

/// Find the R framework Payload file inside an expanded .pkg directory.
/// Looks for a subdirectory matching `*-fw.pkg` or `R-fw.pkg`.
#[cfg(target_os = "macos")]
fn find_fw_payload(expand_dir: &Path) -> Result<PathBuf, String> {
    let entries = std::fs::read_dir(expand_dir)
        .map_err(|e| format!("failed to read expanded pkg dir: {}", e))?;

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_lowercase();
        if name.ends_with("-fw.pkg") || name == "r-fw.pkg" {
            let payload = entry.path().join("Payload");
            if payload.exists() {
                return Ok(payload);
            }
        }
    }
    Err(format!(
        "could not find R framework payload in expanded pkg at {}",
        expand_dir.display()
    ))
}

/// Recursively walk `dir` to find a `bin/R` file, returning the `bin/` directory.
fn find_r_bin_in_dir(dir: &Path) -> Option<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return None;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().map(|n| n == "bin").unwrap_or(false)
                && path.join("R").exists()
                && path.join("Rscript").exists()
            {
                return Some(path);
            }
            if let Some(found) = find_r_bin_in_dir(&path) {
                return Some(found);
            }
        }
    }
    None
}

/// Download and install the best R version satisfying `constraint`.
/// Installs to `~/.local/share/ruv/r/{version}/` and returns the installation.
pub fn auto_install_r(constraint: &str) -> Result<RInstallation, String> {
    println!("  Fetching available R versions from CRAN...");
    let available = fetch_available_r_versions()?;

    let version = resolve_version_to_download(constraint, &available)?;

    let install_dir = r_managed_dir().join(version.to_string());

    // Already downloaded and extracted previously
    if install_dir.join("bin").join("R").exists() {
        let bin_dir = install_dir.join("bin");
        return Ok(RInstallation { version, bin_dir });
    }

    std::fs::create_dir_all(&install_dir)
        .map_err(|e| format!("failed to create R install dir: {}", e))?;

    download_and_extract_r(&version, &install_dir)?;

    // Find the R binary inside the extracted framework and create a stable bin/ symlink
    let r_bin_dir = find_r_bin_in_dir(&install_dir).ok_or_else(|| {
        format!(
            "could not find R binary after extraction in {}",
            install_dir.display()
        )
    })?;

    let stable_bin = install_dir.join("bin");
    if !stable_bin.exists() {
        make_symlink(&r_bin_dir, &stable_bin)?;
    }

    Ok(RInstallation {
        version,
        bin_dir: stable_bin,
    })
}

// ── Symlinks ──────────────────────────────────────────────────────────────────

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

const RPROFILE_MARKER: &str = "# <ruv-managed>";
const RPROFILE_BLOCK: &str = r#"# <ruv-managed> — block below is generated by ruv, do not edit manually
local({
  # Add project library to search path (enables RStudio and ruv run to find packages)
  lib <- file.path(getwd(), ".ruv", "library")
  if (dir.exists(lib)) .libPaths(c(lib, .libPaths()))
})
# </ruv-managed>
"#;

/// Write (or update) the project-root `.Rprofile` with the ruv-managed block.
/// If `.Rprofile` already exists the block is appended unless already present.
pub fn setup_r_profile() -> Result<(), String> {
    let profile_path = Path::new(".Rprofile");
    let existing = if profile_path.exists() {
        std::fs::read_to_string(profile_path)
            .map_err(|e| format!("failed to read .Rprofile: {}", e))?
    } else {
        String::new()
    };

    if existing.contains(RPROFILE_MARKER) {
        return Ok(());
    }

    let separator = if existing.is_empty() || existing.ends_with('\n') {
        ""
    } else {
        "\n"
    };
    let updated = format!("{}{}{}", existing, separator, RPROFILE_BLOCK);
    std::fs::write(profile_path, updated).map_err(|e| format!("failed to write .Rprofile: {}", e))
}

/// Create `{project_name}.Rproj` if it doesn't already exist.
/// Opening this in RStudio sets the working directory and auto-sources `.Rprofile`,
/// so packages in `.ruv/library` are available immediately.
pub fn setup_rproj(project_name: &str) -> Result<(), String> {
    let rproj_path = format!("{}.Rproj", project_name);
    if Path::new(&rproj_path).exists() {
        return Ok(());
    }
    let content = "\
Version: 1.0

RestoreWorkspace: No
SaveWorkspace: No
AlwaysSaveHistory: Default

EnableCodeIndexing: Yes
UseSpacesForTab: Yes
NumSpacesForTab: 2
Encoding: UTF-8

RnwWeave: Sweave
LaTeX: pdfLaTeX
";
    std::fs::write(&rproj_path, content)
        .map_err(|e| format!("failed to write {}: {}", rproj_path, e))
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

// ── Tests ─────────────────────────────────────────────────────────────────────

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

    #[test]
    fn test_bare_version_matches_patch() {
        let spec = RVersion::parse("4.3").unwrap();
        assert!(bare_version_matches(
            &RVersion::parse("4.3.0").unwrap(),
            &spec
        ));
        assert!(bare_version_matches(
            &RVersion::parse("4.3.1").unwrap(),
            &spec
        ));
        assert!(bare_version_matches(
            &RVersion::parse("4.3.2").unwrap(),
            &spec
        ));
        assert!(!bare_version_matches(
            &RVersion::parse("4.4.0").unwrap(),
            &spec
        ));
        assert!(!bare_version_matches(
            &RVersion::parse("4.5.0").unwrap(),
            &spec
        ));
        assert!(!bare_version_matches(
            &RVersion::parse("3.3.0").unwrap(),
            &spec
        ));
    }

    #[test]
    fn test_bare_version_exact_match() {
        let spec = RVersion::parse("4.3.2").unwrap();
        assert!(bare_version_matches(
            &RVersion::parse("4.3.2").unwrap(),
            &spec
        ));
        assert!(!bare_version_matches(
            &RVersion::parse("4.3.1").unwrap(),
            &spec
        ));
        assert!(!bare_version_matches(
            &RVersion::parse("4.3.3").unwrap(),
            &spec
        ));
    }

    #[test]
    fn test_parse_cran_pkg_listing_arm64() {
        let html = r#"
            <a href="R-4.3.2-arm64.pkg">R-4.3.2-arm64.pkg</a>
            <a href="R-4.4.0-arm64.pkg">R-4.4.0-arm64.pkg</a>
            <a href="R-4.5.1-arm64.pkg">R-4.5.1-arm64.pkg</a>
            <a href="R-latest-arm64.pkg">R-latest-arm64.pkg</a>
        "#;
        let versions = parse_cran_pkg_listing(html, "arm64");
        // latest should be excluded (not a valid version), sorted descending
        let strs: Vec<String> = versions.iter().map(|v| v.to_string()).collect();
        assert_eq!(strs, vec!["4.5.1", "4.4.0", "4.3.2"]);
    }

    #[test]
    fn test_resolve_version_to_download_bare() {
        let available: Vec<RVersion> = ["4.5.1", "4.4.2", "4.3.3", "4.3.2"]
            .iter()
            .map(|s| RVersion::parse(s).unwrap())
            .collect();
        let v = resolve_version_to_download("4.3", &available).unwrap();
        assert_eq!(v.to_string(), "4.3.3"); // highest 4.3.x
    }

    #[test]
    fn test_resolve_version_to_download_gte() {
        let available: Vec<RVersion> = ["4.5.1", "4.4.2", "4.3.3"]
            .iter()
            .map(|s| RVersion::parse(s).unwrap())
            .collect();
        let v = resolve_version_to_download(">=4.4", &available).unwrap();
        assert_eq!(v.to_string(), "4.5.1"); // highest satisfying >=4.4
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_parse_posit_deb_listing_amd64() {
        let html = r#"
            <a href="r-4.3.2_1_amd64.deb">r-4.3.2_1_amd64.deb</a>
            <a href="r-4.4.0_1_amd64.deb">r-4.4.0_1_amd64.deb</a>
            <a href="r-4.5.1_1_amd64.deb">r-4.5.1_1_amd64.deb</a>
        "#;
        let versions = parse_posit_deb_listing(html, "amd64");
        let strs: Vec<String> = versions.iter().map(|v| v.to_string()).collect();
        assert_eq!(strs, vec!["4.5.1", "4.4.0", "4.3.2"]);
    }

    #[test]
    fn test_resolve_version_to_download_exact() {
        let available: Vec<RVersion> = ["4.5.1", "4.4.2", "4.3.3"]
            .iter()
            .map(|s| RVersion::parse(s).unwrap())
            .collect();
        let v = resolve_version_to_download("==4.4.2", &available).unwrap();
        assert_eq!(v.to_string(), "4.4.2");
    }
}
