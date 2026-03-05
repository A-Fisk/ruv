use clap::{Parser, Subcommand};
use flate2::read::GzDecoder;
use serde::Deserialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::io::Read;

// ── CLI ──────────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "arrrv", about = "A fast R package manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install an R package and its dependencies
    Install {
        /// Name of the package to install
        package: String,
    },
    /// Sync project library from arrrv.toml
    Sync,
    /// Add a package to arrrv.toml and sync
    Add {
        /// Name of the package to add
        package: String,
    },
}

// ── PROJECT CONFIG ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct ArrrConfig {
    project: ProjectConfig,
}

#[derive(Deserialize)]
struct ProjectConfig {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    version: String,
    dependencies: Vec<String>,
}

fn read_config() -> ArrrConfig {
    let text = std::fs::read_to_string("arrrv.toml")
        .expect("could not find arrrv.toml — are you in the right directory?");
    toml::from_str(&text).expect("failed to parse arrrv.toml")
}

// strips version specifier: "ggplot2>=3.4" → "ggplot2"
fn parse_dep_name(dep: &str) -> String {
    dep.chars()
        .take_while(|c| c.is_alphanumeric() || *c == '.' || *c == '-')
        .collect()
}

// ── CRAN PACKAGE INDEX ────────────────────────────────────────────────────────

struct Package {
    version: String,
    deps: Vec<String>,
}

fn parse_packages(text: &str) -> HashMap<String, Package> {
    let mut index = HashMap::new();

    for block in text.split("\n\n") {
        // join continuation lines back onto the previous line
        let joined = block
            .lines()
            .fold(String::new(), |mut acc, line| {
                if line.starts_with(' ') {
                    acc.push(' ');
                    acc.push_str(line.trim());
                } else {
                    if !acc.is_empty() { acc.push('\n'); }
                    acc.push_str(line)
                }
                acc
            });

        let mut name = None;
        let mut version = None;
        let mut deps: Vec<String> = Vec::new();

        for line in joined.lines() {
            if let Some((key, val)) = line.split_once(": ") {
                match key {
                    "Package" => name = Some(val.to_string()),
                    "Version" => version = Some(val.to_string()),
                    "Imports" | "Depends" => {
                        for dep in val.split(',') {
                            let dep_name = dep
                                .trim()
                                .split_once(' ')
                                .map(|(n, _)| n)
                                .unwrap_or(dep.trim())
                                .to_string();
                            let base_packages = [
                                "R", "base", "utils", "stats", "graphics", "grDevices",
                                "methods", "datasets", "tools", "grid", "compiler",
                            ];
                            if !base_packages.contains(&dep_name.as_str()) && !dep_name.is_empty() {
                                deps.push(dep_name);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if let (Some(name), Some(version)) = (name, version) {
            index.insert(name, Package { version, deps });
        }
    }

    index
}

fn fetch_cran_index() -> HashMap<String, Package> {
    println!("fetching CRAN package index...");
    let response = reqwest::blocking::get("https://cloud.r-project.org/src/contrib/PACKAGES.gz").unwrap();
    let bytes = response.bytes().unwrap();
    let mut decoder = GzDecoder::new(&bytes[..]);
    let mut text = String::new();
    decoder.read_to_string(&mut text).unwrap();
    parse_packages(&text)
}

// ── RESOLVER ──────────────────────────────────────────────────────────────────

fn resolve(root: &str, index: &HashMap<String, Package>) -> Vec<String> {
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<String> = VecDeque::new();

    queue.push_back(root.to_string());

    while let Some(name) = queue.pop_front() {
        if visited.contains(&name) {
            continue;
        }
        visited.insert(name.clone());

        if let Some(pkg) = index.get(&name) {
            for dep in &pkg.deps {
                if !visited.contains(dep) {
                    queue.push_back(dep.clone());
                }
            }
        }
    }

    visited.remove(root);
    visited.into_iter().collect()
}

fn resolve_all(roots: &[String], index: &HashMap<String, Package>) -> Vec<String> {
    let mut all: HashSet<String> = HashSet::new();
    for root in roots {
        all.insert(root.clone());
        for dep in resolve(root, index) {
            all.insert(dep);
        }
    }
    all.into_iter().collect()
}

// ── INSTALLER ─────────────────────────────────────────────────────────────────

fn get_arch() -> &'static str {
    match std::env::consts::ARCH {
        "aarch64" => "big-sur-arm64",
        "x86_64"  => "big-sur-x86_64",
        other     => panic!("Unsupported architecture: {}", other),
    }
}

fn get_r_version() -> String {
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

fn build_urls(packages: &[String], index: &HashMap<String, Package>) -> Vec<String> {
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

fn download_and_install(urls: &[String], lib_dir: &str) {
    std::fs::create_dir_all(lib_dir).unwrap();

    for url in urls {
        println!("downloading {}", url);
        let response = reqwest::blocking::get(url).unwrap();
        let bytes = response.bytes().unwrap();
        let decoder = GzDecoder::new(&bytes[..]);
        let mut archive = tar::Archive::new(decoder);
        archive.unpack(lib_dir).unwrap();
    }
}

// ── LOCKFILE ──────────────────────────────────────────────────────────────────

fn write_lockfile(packages: &[String], index: &HashMap<String, Package>) {
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
    std::fs::write("arrrv.lock", out).unwrap();
    println!("wrote arrrv.lock");
}

// ── MAIN ──────────────────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Install { package } => {
            let index = fetch_cran_index();
            println!("resolving dependencies for {}...", package);
            let deps = resolve(&package, &index);
            println!("installing {} packages...", deps.len());
            let urls = build_urls(&deps, &index);
            download_and_install(&urls, "./arrrv_lib");
            println!("done! run with: R_LIBS=./arrrv_lib Rscript -e \"library({})\"", package);
        }

        Commands::Sync => {
            let config = read_config();
            let roots: Vec<String> = config.project.dependencies
                .iter()
                .map(|d| parse_dep_name(d))
                .collect();
            println!("resolving {} direct dependencies...", roots.len());
            let index = fetch_cran_index();
            let all = resolve_all(&roots, &index);
            println!("installing {} packages total...", all.len());
            let urls = build_urls(&all, &index);
            download_and_install(&urls, "./arrrv_lib");
            write_lockfile(&all, &index);
            println!("done! run with: R_LIBS=./arrrv_lib Rscript -e \"library(...)\"");
        }

        Commands::Add { package } => {
            println!("add \"{}\" to your arrrv.toml dependencies, then run arrrv sync", package);
            println!("  dependencies = [\"{}\"]", package);
        }
    }
}
