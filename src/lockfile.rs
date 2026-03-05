use std::collections::HashMap;
use crate::index::Package;

pub fn write_lockfile(packages: &[String], index: &HashMap<String, Package>) {
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
