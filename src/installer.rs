use flate2::read::GzDecoder;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::sync::OnceLock;
use crate::cache::{cache_dir, hard_link_into_library, is_cached, package_cache_path};
use crate::index::Package;

pub fn get_arch() -> &'static str {
    match std::env::consts::ARCH {
        "aarch64" => "big-sur-arm64",
        "x86_64"  => "big-sur-x86_64",
        other     => panic!("Unsupported architecture: {}", other),
    }
}

pub fn get_r_version() -> &'static str {
    static R_VERSION: OnceLock<String> = OnceLock::new();
    R_VERSION.get_or_init(|| {
        // R RHOME just prints the R home path — no interpreter startup
        let output = std::process::Command::new("R")
            .arg("RHOME")
            .output()
            .expect("Failed to run R — is R installed?");

        let r_home = String::from_utf8(output.stdout).unwrap();
        let version_file = std::path::Path::new(r_home.trim()).join("VERSION");
        let full = std::fs::read_to_string(&version_file)
            .expect("Could not read R VERSION file");

        full.trim().split('.')
            .take(2)
            .collect::<Vec<_>>()
            .join(".")
    })
}

/// Returns (name, version, url) tuples for each package
pub fn build_urls(packages: &[String], index: &HashMap<String, Package>) -> Vec<(String, String, String)> {
    let arch = get_arch();
    let r_version = get_r_version();

    packages.iter()
        .filter_map(|name| {
            let pkg = index.get(name)?;
            let url = format!(
                "https://cloud.r-project.org/bin/macosx/{}/contrib/{}/{}_{}.tgz",
                arch, r_version, name, pkg.version
            );
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
pub fn download_and_install(packages: &[(String, String, String)], lib_dir: &str) -> (usize, usize) {
    let lib_path = Path::new(lib_dir);
    std::fs::create_dir_all(lib_path).unwrap();

    // diff current library state against the requested package list
    let installed = read_installed(lib_path);
    let to_install: Vec<_> = packages.iter()
        .filter(|(name, version, _)| installed.get(name).map(|v| v != version).unwrap_or(true))
        .collect();
    let to_remove: Vec<_> = installed.keys()
        .filter(|name| !packages.iter().any(|(n, _, _)| n == *name))
        .cloned()
        .collect();

    let audited = packages.len() - to_install.len();

    // remove packages that are no longer needed
    for name in &to_remove {
        let _ = std::fs::remove_dir_all(lib_path.join(name));
    }

    if to_install.is_empty() {
        return (audited, 0);
    }

    let mp = MultiProgress::new();

    let overall_style = ProgressStyle::with_template(
        "  {msg:<32} [{bar:40.green/dim}] {pos}/{len}"
    )
    .unwrap()
    .progress_chars("━━╌");

    let pkg_style = ProgressStyle::with_template(
        "  {spinner:.green} {msg:<30} [{bar:40.green/dim}] {bytes:>8} / {total_bytes}"
    )
    .unwrap()
    .progress_chars("━━╌");

    let overall = mp.add(ProgressBar::new(to_install.len() as u64));
    overall.set_style(overall_style);
    overall.set_message("Installing packages");

    to_install.par_iter().for_each(|(name, version, url)| {
        // cache hit — hard-link directly into project library, no download needed
        if is_cached(name, version) {
            hard_link_into_library(name, version, lib_path);
            overall.inc(1);
            return;
        }

        let pb = mp.add(ProgressBar::new(0));
        pb.set_style(pkg_style.clone());
        pb.set_message(name.clone());

        let response = reqwest::blocking::get(url).unwrap();
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
        archive.unpack(&packages_dir).unwrap();
        std::fs::rename(packages_dir.join(name), package_cache_path(name, version)).unwrap();

        // hard-link from cache into project library
        hard_link_into_library(name, version, lib_path);

        overall.inc(1);
    });

    overall.finish_and_clear();

    (audited, to_install.len())
}
