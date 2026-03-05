use flate2::read::GzDecoder;
use rayon::prelude::*;
use std::collections::HashMap;
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

    urls.par_iter().for_each(|url| {
        println!("downloading {}", url);
        let response = reqwest::blocking::get(url).unwrap();
        let bytes = response.bytes().unwrap();
        let decoder = GzDecoder::new(&bytes[..]);
        let mut archive = tar::Archive::new(decoder);
        archive.unpack(lib_dir).unwrap();
    });
}
