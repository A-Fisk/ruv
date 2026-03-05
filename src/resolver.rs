use std::collections::{HashMap, HashSet, VecDeque};
use crate::index::Package;

pub fn resolve(root: &str, index: &HashMap<String, Package>) -> Vec<String> {
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

pub fn resolve_all(roots: &[String], index: &HashMap<String, Package>) -> Vec<String> {
    let mut all: HashSet<String> = HashSet::new();
    for root in roots {
        all.insert(root.clone());
        for dep in resolve(root, index) {
            all.insert(dep);
        }
    }
    all.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_index() -> HashMap<String, Package> {
        let mut index = HashMap::new();
        index.insert("ggplot2".to_string(), Package {
            version: "3.5.1".to_string(),
            deps: vec!["rlang".to_string(), "scales".to_string()],
        });
        index.insert("rlang".to_string(), Package {
            version: "1.1.4".to_string(),
            deps: vec![],
        });
        index.insert("scales".to_string(), Package {
            version: "1.3.0".to_string(),
            deps: vec!["rlang".to_string()],
        });
        index
    }

    #[test]
    fn test_resolve_transitive_deps() {
        let index = make_index();
        let mut deps = resolve("ggplot2", &index);
        deps.sort();
        assert_eq!(deps, vec!["rlang", "scales"]);
    }

    #[test]
    fn test_resolve_deduplicates() {
        // rlang is a dep of both ggplot2 and scales — should only appear once
        let index = make_index();
        let deps = resolve("ggplot2", &index);
        let rlang_count = deps.iter().filter(|d| *d == "rlang").count();
        assert_eq!(rlang_count, 1);
    }

    #[test]
    fn test_resolve_excludes_root() {
        let index = make_index();
        let deps = resolve("ggplot2", &index);
        assert!(!deps.contains(&"ggplot2".to_string()));
    }

    #[test]
    fn test_resolve_unknown_package_returns_empty() {
        let index = make_index();
        let deps = resolve("nonexistent", &index);
        assert!(deps.is_empty());
    }

    #[test]
    fn test_resolve_all_unions_results() {
        let index = make_index();
        let roots = vec!["ggplot2".to_string(), "scales".to_string()];
        let all = resolve_all(&roots, &index);
        assert!(all.contains(&"ggplot2".to_string()));
        assert!(all.contains(&"scales".to_string()));
        assert!(all.contains(&"rlang".to_string()));
    }
}
