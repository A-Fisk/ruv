use flate2::read::GzDecoder;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use crate::cache::{cache_dir, hard_link_into_library, is_cached, package_cache_path};
use crate::index::Package;

pub fn get_arch() -> &'static str {
    match std::env::consts::ARCH {
        "aarch64" => "big-sur-arm64",
        "x86_64"  => "big-sur-x86_64",
        other     => panic!("Unsupported architecture: {}", other),
    }
}

pub fn get_r_version() -> String {
    let output = std::process::Command::new("Rscript")
        .arg("-e")
        .arg("cat(R.Version()$major, R.Version()$minor, sep='.')")
        .output()
        .expect("Failed to run Rscript — is R installed?");

    let full = String::from_utf8(output.stdout).unwrap();
    full.split('.')
        .take(2)
        .collect::<Vec<_>>()
        .join(".")
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

pub fn download_and_install(packages: &[(String, String, String)], lib_dir: &str) {
    let lib_path = Path::new(lib_dir);

    // wipe the project library so we get a clean sync rather than accumulating stale files
    if lib_path.exists() {
        std::fs::remove_dir_all(lib_path).unwrap();
    }
    std::fs::create_dir_all(lib_path).unwrap();

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

    let overall = mp.add(ProgressBar::new(packages.len() as u64));
    overall.set_style(overall_style);
    overall.set_message("Installing packages");

    packages.par_iter().for_each(|(name, version, url)| {
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

    overall.finish_with_message("done");
}
