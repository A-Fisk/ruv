use crate::cache::{cache_dir, hard_link_into_library, is_cached, package_cache_path};
use crate::index::Package;
use flate2::read::GzDecoder;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::sync::OnceLock;

/// Returns the RSPM platform path segment and file extension for the current OS/arch.
/// macOS: ("macosx/big-sur-arm64", "tgz")
/// Linux: ("linux/ubuntu-jammy", "tar.gz")  — distro read from /etc/os-release
pub fn get_platform() -> (&'static str, &'static str) {
    static PLATFORM: OnceLock<(String, String)> = OnceLock::new();
    let (path, ext) = PLATFORM.get_or_init(|| {
        #[cfg(target_os = "macos")]
        {
            let arch = match std::env::consts::ARCH {
                "aarch64" => "macosx/big-sur-arm64",
                "x86_64" => "macosx/big-sur-x86_64",
                other => panic!("Unsupported macOS architecture: {}", other),
            };
            (arch.to_string(), "tgz".to_string())
        }
        #[cfg(target_os = "linux")]
        {
            let distro = linux_rspm_distro();
            (format!("linux/{}", distro), "tar.gz".to_string())
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            panic!("Unsupported OS for RSPM package downloads");
        }
    });
    (path.as_str(), ext.as_str())
}

/// Parse /etc/os-release content and return the RSPM distro string.
/// Separated from linux_rspm_distro() to enable testing without file I/O.
fn parse_os_release(content: &str) -> String {
    let mut id = String::new();
    let mut version_id = String::new();
    let mut version_codename = String::new();
    let mut ubuntu_codename = String::new();

    for line in content.lines() {
        if let Some(v) = line.strip_prefix("ID=") {
            id = v.trim_matches('"').to_lowercase();
        } else if let Some(v) = line.strip_prefix("VERSION_ID=") {
            version_id = v.trim_matches('"').to_string();
        } else if let Some(v) = line.strip_prefix("VERSION_CODENAME=") {
            version_codename = v.trim_matches('"').to_string();
        } else if let Some(v) = line.strip_prefix("UBUNTU_CODENAME=") {
            ubuntu_codename = v.trim_matches('"').to_string();
        }
    }

    match id.as_str() {
        "ubuntu" => {
            let codename = if !version_codename.is_empty() {
                version_codename
            } else if !ubuntu_codename.is_empty() {
                ubuntu_codename
            } else {
                // fallback by version number
                if version_id.starts_with("20.") {
                    "focal".to_string()
                } else if version_id.starts_with("22.") {
                    "jammy".to_string()
                } else if version_id.starts_with("24.") {
                    "noble".to_string()
                } else {
                    eprintln!(
                        "warning: unknown Ubuntu version '{}', defaulting to ubuntu-jammy",
                        version_id
                    );
                    "jammy".to_string()
                }
            };
            format!("ubuntu-{}", codename)
        }
        "debian" => match version_id.as_str() {
            "11" => "debian-bullseye".to_string(),
            "12" => "debian-bookworm".to_string(),
            _ => {
                eprintln!(
                    "warning: unknown Debian version '{}', defaulting to ubuntu-jammy",
                    version_id
                );
                "ubuntu-jammy".to_string()
            }
        },
        "rhel" | "centos" | "rocky" | "almalinux" => {
            let major = version_id.split('.').next().unwrap_or("8");
            match major {
                "9" => "rhel9".to_string(),
                _ => "rhel8".to_string(),
            }
        }
        id if id == "sles" || id == "sle_hpc" || id.starts_with("opensuse") => {
            eprintln!("error: SLES/openSUSE is not supported by RSPM binary packages.");
            eprintln!(
                "       For HPC clusters, use `ruv sync --offline` with a pre-populated library."
            );
            eprintln!("       To populate the library on a supported system, run `ruv sync` there first,");
            eprintln!("       then copy the .ruv/library directory to the HPC node.");
            std::process::exit(1);
        }
        _ => {
            eprintln!(
                "warning: unknown Linux distribution '{}', defaulting to ubuntu-jammy",
                id
            );
            "ubuntu-jammy".to_string()
        }
    }
}

fn linux_rspm_distro() -> &'static str {
    static DISTRO: OnceLock<String> = OnceLock::new();
    DISTRO.get_or_init(|| {
        let content = std::fs::read_to_string("/etc/os-release").unwrap_or_default();
        parse_os_release(&content)
    })
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
fn make_url(
    name: &str,
    version: &str,
    platform: &str,
    ext: &str,
    r_version: &str,
    registry: &str,
) -> String {
    format!(
        "{}/bin/{}/contrib/{}/{}_{}.{}",
        registry, platform, r_version, name, version, ext
    )
}

/// Returns (name, version, url) tuples from lockfile (name, version, registry) triples.
pub fn build_urls_from_pairs(
    packages: &[(String, String, String)],
) -> Vec<(String, String, String)> {
    let (platform, ext) = get_platform();
    let r_version = get_r_version();
    packages
        .iter()
        .map(|(name, version, registry)| {
            (
                name.clone(),
                version.clone(),
                make_url(name, version, platform, ext, r_version, registry),
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
    let (platform, ext) = get_platform();
    let r_version = get_r_version();

    packages
        .iter()
        .filter_map(|name| {
            let pkg = index.get(name)?;
            let url = make_url(name, &pkg.version, platform, ext, r_version, RSPM_LATEST);
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
        assert!(url.contains("ggplot2_3.5.1"), "url should contain pkg_ver: {}", url);
        assert!(
            url.starts_with("https://packagemanager.posit.co/cran/latest"),
            "url should start with RSPM base: {}",
            url
        );
        assert!(url.contains("/contrib/"), "url should contain /contrib/: {}", url);
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

    #[test]
    fn test_parse_os_release_ubuntu_codename() {
        let content = "ID=ubuntu\nVERSION_ID=\"22.04\"\nVERSION_CODENAME=jammy\n";
        assert_eq!(parse_os_release(content), "ubuntu-jammy");
    }

    #[test]
    fn test_parse_os_release_ubuntu_ubuntu_codename_field() {
        // Some Ubuntu variants use UBUNTU_CODENAME instead of VERSION_CODENAME
        let content = "ID=ubuntu\nVERSION_ID=\"22.04\"\nUBUNTU_CODENAME=jammy\n";
        assert_eq!(parse_os_release(content), "ubuntu-jammy");
    }

    #[test]
    fn test_parse_os_release_ubuntu_focal() {
        let content = "ID=ubuntu\nVERSION_ID=\"20.04\"\n";
        assert_eq!(parse_os_release(content), "ubuntu-focal");
    }

    #[test]
    fn test_parse_os_release_ubuntu_noble() {
        let content = "ID=ubuntu\nVERSION_ID=\"24.04\"\nVERSION_CODENAME=noble\n";
        assert_eq!(parse_os_release(content), "ubuntu-noble");
    }

    #[test]
    fn test_parse_os_release_debian_11() {
        let content = "ID=debian\nVERSION_ID=\"11\"\n";
        assert_eq!(parse_os_release(content), "debian-bullseye");
    }

    #[test]
    fn test_parse_os_release_debian_12() {
        let content = "ID=debian\nVERSION_ID=\"12\"\n";
        assert_eq!(parse_os_release(content), "debian-bookworm");
    }

    #[test]
    fn test_parse_os_release_rhel8() {
        let content = "ID=\"rhel\"\nVERSION_ID=\"8.9\"\n";
        assert_eq!(parse_os_release(content), "rhel8");
    }

    #[test]
    fn test_parse_os_release_rhel9() {
        let content = "ID=\"rhel\"\nVERSION_ID=\"9.2\"\n";
        assert_eq!(parse_os_release(content), "rhel9");
    }

    #[test]
    fn test_parse_os_release_rocky_linux() {
        let content = "ID=\"rocky\"\nVERSION_ID=\"8.7\"\n";
        assert_eq!(parse_os_release(content), "rhel8");
    }

    #[test]
    fn test_parse_os_release_almalinux_9() {
        let content = "ID=\"almalinux\"\nVERSION_ID=\"9.1\"\n";
        assert_eq!(parse_os_release(content), "rhel9");
    }

    #[test]
    fn test_parse_os_release_centos() {
        let content = "ID=\"centos\"\nVERSION_ID=\"8\"\n";
        assert_eq!(parse_os_release(content), "rhel8");
    }

    #[test]
    fn test_parse_os_release_unknown_defaults_to_ubuntu_jammy() {
        let content = "ID=arch\nVERSION_ID=\"2024.01.01\"\n";
        assert_eq!(parse_os_release(content), "ubuntu-jammy");
    }

    #[test]
    fn test_parse_os_release_quoted_id() {
        // Some distros quote the ID field
        let content = "ID=\"ubuntu\"\nVERSION_ID=\"22.04\"\nVERSION_CODENAME=jammy\n";
        assert_eq!(parse_os_release(content), "ubuntu-jammy");
    }
}
