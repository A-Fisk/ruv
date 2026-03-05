use flate2::read::GzDecoder;
use std::io::Read;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::collections::HashSet;


// create custom data type
struct Package {
    version: String,
    deps: Vec<String>,
}

// function to be used later
fn parse_packages(text: &str) -> HashMap<String, Package> {

    // create index we will use later
    let mut index = HashMap::new();

    for block in text.split("\n\n") {
        // join continuation lines together
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

        // parse key: value pairs
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

fn resolve(root: &str, index: &HashMap<String, Package>) -> Vec<String> {
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<String> = VecDeque::new();

    queue.push_back(root.to_string());

    while let Some(name) = queue.pop_front() { 

        //ignore if already found
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

    // remove root package itself
    visited.remove(root);
    visited.into_iter().collect()
}

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
    // full looks like "4.4.2" — we only want "4.4"
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

        // decompress gzip
        let decoder = GzDecoder::new(&bytes[..]);

        // untar into lib_dir
        let mut archive = tar::Archive::new(decoder);
        archive.unpack(lib_dir).unwrap();
    }
}

fn main() {
    // define url to get from
    let url = "https://cloud.r-project.org/src/contrib/PACKAGES.gz";

    // connect to url
    let response = reqwest::blocking::get(url).unwrap();

    // get the information
    let bytes = response.bytes().unwrap();

    // decompress to a string
    let mut decoder = GzDecoder::new(&bytes[..]);
    let mut text = String::new();
    decoder.read_to_string(&mut text).unwrap();

    // print the first few packages
    let index = parse_packages(&text);
    
    let deps = resolve("ggplot2", &index);
    let urls = build_urls(&deps, &index);
    download_and_install(&urls, "./arrrv_lib");
    println!("done! run with: R_LIBS=./arrrv_lib Rscript -e \"library(ggplot2)\"")
}
