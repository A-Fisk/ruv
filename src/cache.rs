use std::path::{Path, PathBuf};

pub fn cache_dir() -> PathBuf {
    dirs::cache_dir()
        .expect("could not find cache directory")
        .join("arrrv")
}

pub fn package_cache_path(name: &str, version: &str) -> PathBuf {
    cache_dir().join("packages").join(format!("{}_{}", name, version))
}

pub fn is_cached(name: &str, version: &str) -> bool {
    package_cache_path(name, version).exists()
}

/// Hard-links a cached package directory into the project library.
/// Creates .arrrv/library/{name}/ with hard-links to every file in the cache.
pub fn hard_link_into_library(name: &str, version: &str, lib_dir: &Path) {
    let src = package_cache_path(name, version);
    let dst = lib_dir.join(name);
    hard_link_dir(&src, &dst).expect("failed to hard-link package into library");
}

fn hard_link_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let dst_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            hard_link_dir(&entry.path(), &dst_path)?;
        } else {
            std::fs::hard_link(entry.path(), dst_path)?;
        }
    }
    Ok(())
}
