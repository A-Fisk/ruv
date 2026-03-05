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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::MetadataExt;

    #[test]
    fn test_package_cache_path_format() {
        let path = package_cache_path("ggplot2", "3.5.1");
        let path_str = path.to_string_lossy();
        assert!(path_str.ends_with("arrrv/packages/ggplot2_3.5.1"));
    }

    #[test]
    fn test_hard_link_dir_copies_files() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");

        // create a source directory with a file and a subdirectory
        fs::create_dir_all(src.join("subdir")).unwrap();
        fs::write(src.join("file.txt"), b"hello").unwrap();
        fs::write(src.join("subdir/nested.txt"), b"world").unwrap();

        hard_link_dir(&src, &dst).unwrap();

        assert!(dst.join("file.txt").exists());
        assert!(dst.join("subdir/nested.txt").exists());
    }

    #[test]
    fn test_hard_link_dir_creates_true_hard_links() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");

        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("file.txt"), b"hello").unwrap();

        hard_link_dir(&src, &dst).unwrap();

        // hard-linked files share the same inode
        let src_inode = fs::metadata(src.join("file.txt")).unwrap().ino();
        let dst_inode = fs::metadata(dst.join("file.txt")).unwrap().ino();
        assert_eq!(src_inode, dst_inode);
    }

    #[test]
    fn test_is_cached_returns_false_when_missing() {
        assert!(!is_cached("nonexistent-pkg", "0.0.0"));
    }
}
