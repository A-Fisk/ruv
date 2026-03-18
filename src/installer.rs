use crate::cache::{cache_dir, hard_link_into_library, is_cached, package_cache_path};
use crate::index::Package;
use flate2::read::GzDecoder;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::sync::OnceLock;

pub fn get_arch() -> &'static str {
    match std::env::consts::ARCH {
        "aarch64" => "big-sur-arm64",
        "x86_64" => "big-sur-x86_64",
        other => panic!("Unsupported architecture: {}", other),
    }
}

/// Parses the content of /etc/os-release and returns the RSPM distro path component.
/// Extracted for testability — production callers use `linux_rspm_distro()`.
pub(crate) fn parse_rspm_distro(os_release: &str) -> Result<String, String> {
    let mut id: Option<String> = None;
    let mut version_id: Option<String> = None;

    for line in os_release.lines() {
        if let Some(val) = line.strip_prefix("ID=") {
            id = Some(val.trim_matches('"').to_lowercase());
        } else if let Some(val) = line.strip_prefix("VERSION_ID=") {
            version_id = Some(val.trim_matches('"').to_string());
        }
    }

    let id = id.ok_or_else(|| {
        "could not determine Linux distribution from /etc/os-release (no ID= field)".to_string()
    })?;
    let version_id = version_id.unwrap_or_default();
    let major = version_id.split('.').next().unwrap_or("");

    match (id.as_str(), major) {
        ("rhel" | "rocky" | "almalinux" | "centos", "8") => Ok("rhel8".to_string()),
        ("rhel" | "rocky" | "almalinux" | "centos", "9") => Ok("rhel9".to_string()),
        ("ubuntu", _) => match version_id.as_str() {
            "20.04" => Ok("ubuntu-focal".to_string()),
            "22.04" => Ok("ubuntu-jammy".to_string()),
            "24.04" => Ok("ubuntu-noble".to_string()),
            _ => Ok("ubuntu-jammy".to_string()),
        },
        ("sles" | "sle_hpc" | "opensuse-leap" | "opensuse-tumbleweed", _) => Err(format!(
            "SLES/openSUSE ({} {}) has no RSPM binary support — \
             install from source or switch to a RHEL- or Ubuntu-based distribution",
            id, version_id
        )),
        _ => Err(format!(
            "unsupported Linux distribution '{}' — \
             RSPM binaries are available for RHEL/Rocky/Alma/CentOS (8, 9) and Ubuntu (20.04, 22.04, 24.04)",
            id
        )),
    }
}

/// Reads /etc/os-release and returns the RSPM distro path component for this system
/// (e.g. `"rhel8"`, `"rhel9"`, `"ubuntu-jammy"`).
///
/// Returns an `Err` for distributions where RSPM publishes no pre-built binaries.
#[allow(dead_code)]
pub fn linux_rspm_distro() -> Result<String, String> {
    let content = std::fs::read_to_string("/etc/os-release")
        .map_err(|e| format!("failed to read /etc/os-release: {}", e))?;
    parse_rspm_distro(&content)
}

pub fn get_r_version() -> &'static str {
    static R_VERSION: OnceLock<String> = OnceLock::new();
    R_VERSION.get_or_init(|| {
        // R --version prints e.g. "R version 4.5.2 (2025-10-31) -- ..."
        // This is fast — no interpreter session is started
        let output = std::process::Command::new("R")
            .arg("--version")
            .output()
            .expect("Failed to run R — is R installed?");

        let stdout = String::from_utf8(output.stdout).unwrap();
        let version_str = stdout
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(2))
            .expect("Could not parse R version from `R --version`");

        version_str.split('.').take(2).collect::<Vec<_>>().join(".")
    })
}

/// Constructs a binary download URL from an RSPM registry base URL.
/// registry is e.g. "https://packagemanager.posit.co/cran/2024-06-05"
fn make_url(name: &str, version: &str, arch: &str, r_version: &str, registry: &str) -> String {
    format!(
        "{}/bin/macosx/{}/contrib/{}/{}_{}.tgz",
        registry, arch, r_version, name, version
    )
}

/// Returns (name, version, url) tuples from lockfile (name, version, registry) triples.
pub fn build_urls_from_pairs(
    packages: &[(String, String, String)],
) -> Vec<(String, String, String)> {
    let arch = get_arch();
    let r_version = get_r_version();
    packages
        .iter()
        .map(|(name, version, registry)| {
            (
                name.clone(),
                version.clone(),
                make_url(name, version, arch, r_version, registry),
            )
        })
        .collect()
}

const RSPM_LATEST: &str = "https://packagemanager.posit.co/cran/latest";

/// Returns (name, version, url) tuples for each package, looking up versions in the CRAN index.
/// Uses RSPM latest for installs that don't come from a lockfile.
pub fn build_urls(
    packages: &[String],
    index: &HashMap<String, Package>,
) -> Vec<(String, String, String)> {
    let arch = get_arch();
    let r_version = get_r_version();

    packages
        .iter()
        .filter_map(|name| {
            let pkg = index.get(name)?;
            let url = make_url(name, &pkg.version, arch, r_version, RSPM_LATEST);
            Some((name.clone(), pkg.version.clone(), url))
        })
        .collect()
}

/// Reads installed packages from a library dir by parsing each DESCRIPTION file.
/// Returns a map of package name → installed version.
fn read_installed(lib_dir: &Path) -> HashMap<String, String> {
    let mut installed = HashMap::new();
    let Ok(entries) = std::fs::read_dir(lib_dir) else {
        return installed;
    };
    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(entry.path().join("DESCRIPTION")) else {
            continue;
        };
        let mut name = None;
        let mut version = None;
        for line in text.lines() {
            if let Some(v) = line.strip_prefix("Package: ") {
                name = Some(v.trim().to_string());
            } else if let Some(v) = line.strip_prefix("Version: ") {
                version = Some(v.trim().to_string());
            }
            if name.is_some() && version.is_some() {
                break;
            }
        }
        if let (Some(n), Some(v)) = (name, version) {
            installed.insert(n, v);
        }
    }
    installed
}

/// Installs packages into lib_dir. Returns (audited, installed) counts:
/// - audited: packages already present at the correct version (skipped)
/// - installed: packages newly downloaded or hard-linked from cache
pub fn download_and_install(
    packages: &[(String, String, String)],
    lib_dir: &str,
    verbose: bool,
) -> (usize, usize) {
    let lib_path = Path::new(lib_dir);
    std::fs::create_dir_all(lib_path).unwrap();

    // diff current library state against the requested package list
    let installed = read_installed(lib_path);
    let to_install: Vec<_> = packages
        .iter()
        .filter(|(name, version, _)| installed.get(name).map(|v| v != version).unwrap_or(true))
        .collect();
    let to_remove: Vec<_> = installed
        .keys()
        .filter(|name| !packages.iter().any(|(n, _, _)| n == *name))
        .cloned()
        .collect();

    let audited = packages.len() - to_install.len();

    if verbose {
        println!("  already installed: {}", audited);
        println!("  to install:        {}", to_install.len());
        println!("  to remove:         {}", to_remove.len());
    }

    // remove packages that are no longer needed
    for name in &to_remove {
        if verbose {
            println!("  removing {}", name);
        }
        let _ = std::fs::remove_dir_all(lib_path.join(name));
    }

    if to_install.is_empty() {
        return (audited, 0);
    }

    let mp = MultiProgress::new();

    let overall_style =
        ProgressStyle::with_template("  {msg:<32.32} [{wide_bar:.green/dim}] {pos}/{len:>3}")
            .unwrap()
            .progress_chars("━━╌");

    let pkg_style = ProgressStyle::with_template(
        "  {spinner:.green} {msg:<30.30} [{wide_bar:.green/dim}] {bytes:>10}/{total_bytes:<10}",
    )
    .unwrap()
    .progress_chars("━━╌");

    let overall = mp.add(ProgressBar::new(to_install.len() as u64));
    overall.set_style(overall_style);
    overall.set_message("Installing packages");

    to_install.par_iter().for_each(|(name, version, url)| {
        // cache hit — hard-link directly into project library, no download needed
        if is_cached(name, version) {
            if verbose {
                println!("  {} {} (from cache)", name, version);
            }
            hard_link_into_library(name, version, lib_path);
            overall.inc(1);
            return;
        }
        if verbose {
            println!("  {} {} (downloading {})", name, version, url);
        }

        let pb = mp.add(ProgressBar::new(0));
        pb.set_style(pkg_style.clone());
        pb.set_message(name.clone());

        let response = reqwest::blocking::get(url).unwrap_or_else(|e| {
            pb.finish_and_clear();
            eprintln!("\nerror: failed to download {} {}: {}", name, version, e);
            std::process::exit(1);
        });
        if !response.status().is_success() {
            pb.finish_and_clear();
            eprintln!(
                "\nerror: binary not available for {} {} (HTTP {})\n       \
                 The package may not have a pre-built binary for your R version at this snapshot.\n       \
                 URL: {}",
                name,
                version,
                response.status(),
                url
            );
            std::process::exit(1);
        }
        let total = response.content_length().unwrap_or(0);
        pb.set_length(total);

        // stream response through the progress bar so it updates as bytes arrive
        let mut src = pb.wrap_read(response);
        let mut bytes = Vec::new();
        src.read_to_end(&mut bytes).unwrap();
        pb.finish_and_clear();

        // extract to cache: unpacks {name}/ into packages dir, then rename to {name}_{version}/
        let packages_dir = cache_dir().join("packages");
        std::fs::create_dir_all(&packages_dir).unwrap();
        let decoder = GzDecoder::new(bytes.as_slice());
        let mut archive = tar::Archive::new(decoder);
        archive.unpack(&packages_dir).unwrap_or_else(|e| {
            eprintln!(
                "\nerror: failed to extract {} {}: {}\n       \
                 The downloaded file may not be a valid binary package.",
                name, version, e
            );
            std::process::exit(1);
        });
        std::fs::rename(packages_dir.join(name), package_cache_path(name, version)).unwrap();

        // hard-link from cache into project library
        hard_link_into_library(name, version, lib_path);

        overall.inc(1);
    });

    overall.finish_and_clear();

    (audited, to_install.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::Package;

    // --- linux_rspm_distro / parse_rspm_distro tests ---

    fn os_release(id: &str, version_id: &str) -> String {
        format!("ID={}\nVERSION_ID={}\n", id, version_id)
    }

    #[test]
    fn test_rhel8_maps_to_rhel8() {
        assert_eq!(parse_rspm_distro(&os_release("rhel", "8.9")).unwrap(), "rhel8");
    }

    #[test]
    fn test_rhel9_maps_to_rhel9() {
        assert_eq!(parse_rspm_distro(&os_release("rhel", "9.3")).unwrap(), "rhel9");
    }

    #[test]
    fn test_rocky8_maps_to_rhel8() {
        assert_eq!(parse_rspm_distro(&os_release("rocky", "8.9")).unwrap(), "rhel8");
    }

    #[test]
    fn test_rocky9_maps_to_rhel9() {
        assert_eq!(parse_rspm_distro(&os_release("rocky", "9.3")).unwrap(), "rhel9");
    }

    #[test]
    fn test_almalinux8_maps_to_rhel8() {
        assert_eq!(parse_rspm_distro(&os_release("almalinux", "8.9")).unwrap(), "rhel8");
    }

    #[test]
    fn test_almalinux9_maps_to_rhel9() {
        assert_eq!(parse_rspm_distro(&os_release("almalinux", "9.3")).unwrap(), "rhel9");
    }

    #[test]
    fn test_centos8_maps_to_rhel8() {
        assert_eq!(parse_rspm_distro(&os_release("centos", "8")).unwrap(), "rhel8");
    }

    #[test]
    fn test_centos9_maps_to_rhel9() {
        assert_eq!(parse_rspm_distro(&os_release("centos", "9")).unwrap(), "rhel9");
    }

    #[test]
    fn test_ubuntu_jammy_maps_correctly() {
        assert_eq!(
            parse_rspm_distro(&os_release("ubuntu", "22.04")).unwrap(),
            "ubuntu-jammy"
        );
    }

    #[test]
    fn test_ubuntu_focal_maps_correctly() {
        assert_eq!(
            parse_rspm_distro(&os_release("ubuntu", "20.04")).unwrap(),
            "ubuntu-focal"
        );
    }

    #[test]
    fn test_ubuntu_noble_maps_correctly() {
        assert_eq!(
            parse_rspm_distro(&os_release("ubuntu", "24.04")).unwrap(),
            "ubuntu-noble"
        );
    }

    #[test]
    fn test_sles_returns_error() {
        let result = parse_rspm_distro(&os_release("sles", "15.5"));
        assert!(result.is_err(), "SLES should return an error");
        assert!(result.unwrap_err().contains("SLES"));
    }

    #[test]
    fn test_opensuse_leap_returns_error() {
        let result = parse_rspm_distro(&os_release("opensuse-leap", "15.5"));
        assert!(result.is_err(), "openSUSE Leap should return an error");
    }

    #[test]
    fn test_unsupported_distro_returns_error() {
        let result = parse_rspm_distro(&os_release("arch", "rolling"));
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("arch"), "error should name the distro: {}", msg);
    }

    #[test]
    fn test_quoted_values_are_parsed() {
        let content = "ID=\"rocky\"\nVERSION_ID=\"9.3\"\n";
        assert_eq!(parse_rspm_distro(content).unwrap(), "rhel9");
    }

    #[test]
    fn test_missing_id_returns_error() {
        let result = parse_rspm_distro("VERSION_ID=9.3\n");
        assert!(result.is_err());
    }

    fn make_index(entries: &[(&str, &str)]) -> HashMap<String, Package> {
        entries
            .iter()
            .map(|(name, version)| {
                (
                    name.to_string(),
                    Package {
                        version: version.to_string(),
                        deps: vec![], // no deps needed for URL-building tests
                    },
                )
            })
            .collect()
    }

    #[test]
    fn test_build_urls_format() {
        let index = make_index(&[("ggplot2", "3.5.1")]);
        let urls = build_urls(&["ggplot2".to_string()], &index);
        assert_eq!(urls.len(), 1);
        let (name, version, url) = &urls[0];
        assert_eq!(name, "ggplot2");
        assert_eq!(version, "3.5.1");
        assert!(url.contains("ggplot2_3.5.1.tgz"));
        assert!(url.starts_with("https://packagemanager.posit.co/cran/latest/bin/macosx/"));
        assert!(url.contains("/contrib/"));
    }

    #[test]
    fn test_build_urls_drops_missing_packages() {
        let index = make_index(&[("ggplot2", "3.5.1")]);
        let urls = build_urls(&["ggplot2".to_string(), "not-in-index".to_string()], &index);
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0].0, "ggplot2");
    }

    #[test]
    fn test_build_urls_empty_input() {
        let index = make_index(&[("ggplot2", "3.5.1")]);
        let urls = build_urls(&[], &index);
        assert!(urls.is_empty());
    }
}
