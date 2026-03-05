use serde::Deserialize;

#[derive(Deserialize)]
pub struct ArrrConfig {
    pub project: ProjectConfig,
}

#[derive(Deserialize)]
pub struct ProjectConfig {
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub version: String,
    pub dependencies: Vec<String>,
}

pub fn read_config() -> ArrrConfig {
    let text = std::fs::read_to_string("arrrv.toml")
        .expect("could not find arrrv.toml — are you in the right directory?");
    toml::from_str(&text).expect("failed to parse arrrv.toml")
}

/// Strips version specifier from a dependency string: "ggplot2>=3.4" → "ggplot2"
pub fn parse_dep_name(dep: &str) -> String {
    dep.chars()
        .take_while(|c| c.is_alphanumeric() || *c == '.' || *c == '-')
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_config(text: &str) -> ArrrConfig {
        toml::from_str(text).expect("failed to parse arrrv.toml")
    }

    #[test]
    fn test_read_config_parses_dependencies() {
        let toml = "[project]\nname = \"test\"\nversion = \"0.1.0\"\ndependencies = [\"ggplot2\", \"dplyr\"]";
        let config = parse_config(toml);
        assert_eq!(config.project.dependencies, vec!["ggplot2", "dplyr"]);
    }

    #[test]
    fn test_read_config_empty_dependencies() {
        let toml = "[project]\nname = \"test\"\nversion = \"0.1.0\"\ndependencies = []";
        let config = parse_config(toml);
        assert!(config.project.dependencies.is_empty());
    }

    #[test]
    fn test_parse_dep_name_with_gte() {
        assert_eq!(parse_dep_name("ggplot2>=3.4"), "ggplot2");
    }

    #[test]
    fn test_parse_dep_name_no_version() {
        assert_eq!(parse_dep_name("dplyr"), "dplyr");
    }

    #[test]
    fn test_parse_dep_name_with_spaces() {
        assert_eq!(parse_dep_name("rlang (>= 1.0)"), "rlang");
    }

    #[test]
    fn test_parse_dep_name_preserves_dots_and_dashes() {
        assert_eq!(parse_dep_name("data.table"), "data.table");
        assert_eq!(parse_dep_name("R6"), "R6");
    }
}
