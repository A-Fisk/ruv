use flate2::read::GzDecoder;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::collections::HashMap;
use std::io::Read;
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

pub fn build_urls(packages: &[String], index: &HashMap<String, Package>) -> Vec<String> {
    let arch = get_arch();
    let r_version = get_r_version();

    packages.iter()
        .filter_map(|name| {
            let pkg = index.get(name)?;
            let url = format!(
                "https://cloud.r-project.org/bin/macosx/{}/contrib/{}/{}_{}.tgz",
                arch, r_version, name, pkg.version
            );
            Some(url)
        })
        .collect()
}

pub fn download_and_install(urls: &[String], lib_dir: &str) {
    std::fs::create_dir_all(lib_dir).unwrap();

    let mp = MultiProgress::new();
    let style = ProgressStyle::with_template(
        " {spinner:.green} {wide_msg:<30} [{bar:30.cyan/blue}] {bytes}/{total_bytes}"
    )
    .unwrap()
    .progress_chars("█▓░");

    urls.par_iter().for_each(|url| {
        // extract package name from URL: "ggplot2_3.5.1.tgz" → "ggplot2"
        let filename = url.split('/').last().unwrap_or(url);
        let pkg_name = filename.split('_').next().unwrap_or(filename);

        let pb = mp.add(ProgressBar::new(0));
        pb.set_style(style.clone());
        pb.set_message(pkg_name.to_string());

        let response = reqwest::blocking::get(url).unwrap();
        let total = response.content_length().unwrap_or(0);
        pb.set_length(total);

        // stream response through the progress bar so it updates as bytes arrive
        let mut src = pb.wrap_read(response);
        let mut bytes = Vec::new();
        src.read_to_end(&mut bytes).unwrap();
        pb.finish();

        let decoder = GzDecoder::new(bytes.as_slice());
        let mut archive = tar::Archive::new(decoder);
        archive.unpack(lib_dir).unwrap();
    });
}
