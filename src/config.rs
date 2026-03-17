use crate::version::Dep;
use serde::Deserialize;
use std::io::ErrorKind;
use std::path::Path;

pub const CONFIG_FILE: &str = "ruv.toml";

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
    #[serde(rename = "r-version")]
    pub r_version: Option<String>,
    pub dependencies: Vec<String>,
}

pub fn read_config() -> Result<ArrrConfig, String> {
    let text = std::fs::read_to_string(CONFIG_FILE).map_err(|e| {
        if e.kind() == ErrorKind::NotFound {
            format!(
                "could not find {} in this directory — run `ruv init` first",
                CONFIG_FILE
            )
        } else {
            format!("failed to read {}: {}", CONFIG_FILE, e)
        }
    })?;
    toml::from_str(&text).map_err(|e| format!("failed to parse {}: {}", CONFIG_FILE, e))
}

pub fn init_config(project_name: &str) -> Result<(), String> {
    if Path::new(CONFIG_FILE).exists() {
        return Err(format!("{} already exists in this directory", CONFIG_FILE));
    }

    let text = default_config_toml(project_name);
    std::fs::write(CONFIG_FILE, text).map_err(|e| format!("failed to write {}: {}", CONFIG_FILE, e))
}

pub enum AddDependencyResult {
    Added,
    AlreadyPresent,
}

pub fn add_dependency(dep: &str) -> Result<AddDependencyResult, String> {
    let text = std::fs::read_to_string(CONFIG_FILE).map_err(|e| {
        if e.kind() == ErrorKind::NotFound {
            format!(
                "could not find {} in this directory — run `ruv init` first",
                CONFIG_FILE
            )
        } else {
            format!("failed to read {}: {}", CONFIG_FILE, e)
        }
    })?;
    let (updated, added) = add_dependency_to_toml_text(&text, dep)?;
    std::fs::write(CONFIG_FILE, updated)
        .map_err(|e| format!("failed to write {}: {}", CONFIG_FILE, e))?;
    if added {
        Ok(AddDependencyResult::Added)
    } else {
        Ok(AddDependencyResult::AlreadyPresent)
    }
}

fn default_config_toml(project_name: &str) -> String {
    format!(
        "[project]\nname = \"{}\"\nversion = \"0.1.0-alpha.1\"\nr-version = \">=4.3\"\ndependencies = []\n",
        project_name
    )
}

fn add_dependency_to_toml_text(text: &str, dep: &str) -> Result<(String, bool), String> {
    let root: toml::Value =
        toml::from_str(text).map_err(|e| format!("failed to parse {}: {}", CONFIG_FILE, e))?;
    let deps_array = root
        .get("project")
        .and_then(|v| v.get("dependencies"))
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            format!(
                "invalid {}: missing [project].dependencies array",
                CONFIG_FILE
            )
        })?;
    let existing: Vec<String> = deps_array
        .iter()
        .map(|v| {
            v.as_str().map(|s| s.to_string()).ok_or_else(|| {
                format!(
                    "invalid {}: dependencies entries must be strings",
                    CONFIG_FILE
                )
            })
        })
        .collect::<Result<_, _>>()?;
    let new_name = parse_dep_name(dep);
    let already_present = existing
        .iter()
        .any(|existing| parse_dep_name(existing) == new_name);
    if already_present {
        return Ok((text.to_string(), false));
    }

    let (body_start, body_end) = find_project_body_range(text)?;
    let dep_field = find_dependencies_field(text, body_start, body_end);
    let updated = if let Some(field) = dep_field {
        rewrite_existing_dependencies(text, field, &existing, dep)?
    } else {
        insert_new_dependencies_field(text, body_start, body_end, dep)
    };
    Ok((updated, true))
}

#[derive(Clone, Copy)]
struct DependenciesField {
    open_bracket: usize,
    close_bracket: usize,
    key_indent_start: usize,
    key_indent_end: usize,
}

fn find_project_body_range(text: &str) -> Result<(usize, usize), String> {
    let spans = line_spans(text);
    let mut project_header_end = None;
    let mut project_body_end = text.len();

    for (idx, (start, end)) in spans.iter().enumerate() {
        let trimmed = text[*start..*end].trim();
        if project_header_end.is_none() {
            if trimmed == "[project]" {
                project_header_end = Some(*end);
            }
            continue;
        }
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            project_body_end = *start;
            break;
        }
        if idx + 1 == spans.len() {
            project_body_end = text.len();
        }
    }

    match project_header_end {
        Some(body_start) => Ok((body_start, project_body_end)),
        None => Err(format!("invalid {}: missing [project] table", CONFIG_FILE)),
    }
}

fn line_spans(text: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut start = 0usize;
    for (i, ch) in text.char_indices() {
        if ch == '\n' {
            spans.push((start, i + 1));
            start = i + 1;
        }
    }
    if start < text.len() {
        spans.push((start, text.len()));
    }
    if spans.is_empty() {
        spans.push((0, 0));
    }
    spans
}

fn find_dependencies_field(
    text: &str,
    body_start: usize,
    body_end: usize,
) -> Option<DependenciesField> {
    for (line_start, line_end) in line_spans(text) {
        if line_start < body_start || line_start >= body_end {
            continue;
        }
        let line = &text[line_start..line_end];
        let line_no_comment = line.split('#').next().unwrap_or(line);
        let trimmed = line_no_comment.trim_start();
        if !trimmed.starts_with("dependencies") {
            continue;
        }

        let ws_len = line_no_comment.len() - trimmed.len();
        let after_key = &trimmed["dependencies".len()..];
        if !(after_key.is_empty()
            || after_key.starts_with(char::is_whitespace)
            || after_key.starts_with('='))
        {
            continue;
        }

        let Some(eq_rel) = line_no_comment.find('=') else {
            continue;
        };
        let eq_abs = line_start + eq_rel;
        let rest = &text[eq_abs + 1..body_end];
        let Some(bracket_rel) = rest.find('[') else {
            continue;
        };
        let open = eq_abs + 1 + bracket_rel;
        let Some(close) = find_matching_bracket(text, open, body_end) else {
            continue;
        };
        return Some(DependenciesField {
            open_bracket: open,
            close_bracket: close,
            key_indent_start: line_start,
            key_indent_end: line_start + ws_len,
        });
    }
    None
}

fn find_matching_bracket(text: &str, open: usize, limit: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (off, ch) in text[open..limit].char_indices() {
        let abs = open + off;
        if in_string {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '[' => depth += 1,
            ']' => {
                if depth == 0 {
                    return None;
                }
                depth -= 1;
                if depth == 0 {
                    return Some(abs);
                }
            }
            _ => {}
        }
    }
    None
}

fn rewrite_existing_dependencies(
    text: &str,
    field: DependenciesField,
    existing: &[String],
    dep: &str,
) -> Result<String, String> {
    let key_indent = &text[field.key_indent_start..field.key_indent_end];
    let current_array = &text[field.open_bracket..=field.close_bracket];
    let multiline = true;
    let item_indent = if current_array.contains('\n') {
        infer_item_indent(text, field.open_bracket, field.close_bracket)
            .unwrap_or_else(|| format!("{}    ", key_indent))
    } else {
        format!("{}    ", key_indent)
    };

    let mut all = existing.to_vec();
    all.push(dep.to_string());
    let replacement = render_dependencies_array(&all, multiline, key_indent, &item_indent);
    let mut out = String::with_capacity(text.len() + dep.len() + 16);
    out.push_str(&text[..field.open_bracket]);
    out.push_str(&replacement);
    out.push_str(&text[field.close_bracket + 1..]);
    Ok(out)
}

fn infer_item_indent(text: &str, open: usize, close: usize) -> Option<String> {
    let inside = &text[open + 1..close];
    for line in inside.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let indent_len = line.len() - line.trim_start().len();
        return Some(line[..indent_len].to_string());
    }
    None
}

fn render_dependencies_array(
    deps: &[String],
    multiline: bool,
    key_indent: &str,
    item_indent: &str,
) -> String {
    if multiline {
        let mut out = String::from("[\n");
        for dep in deps {
            out.push_str(item_indent);
            out.push('"');
            out.push_str(dep);
            out.push_str("\",\n");
        }
        out.push_str(key_indent);
        out.push(']');
        out
    } else {
        let joined = deps
            .iter()
            .map(|d| format!("\"{}\"", d))
            .collect::<Vec<_>>()
            .join(", ");
        format!("[{}]", joined)
    }
}

fn insert_new_dependencies_field(
    text: &str,
    body_start: usize,
    body_end: usize,
    dep: &str,
) -> String {
    let key_indent = infer_project_key_indent(text, body_start, body_end);
    let item_indent = format!("{}    ", key_indent);
    let dep_block = format!(
        "{}dependencies = [\n{}\"{}\",\n{}]\n",
        key_indent, item_indent, dep, key_indent
    );
    let mut out = String::with_capacity(text.len() + dep_block.len() + 1);
    out.push_str(&text[..body_end]);
    if body_end > body_start && !text[..body_end].ends_with('\n') {
        out.push('\n');
    }
    if body_end > body_start && !text[..body_end].ends_with("\n\n") {
        out.push('\n');
    }
    out.push_str(&dep_block);
    out.push_str(&text[body_end..]);
    out
}

fn infer_project_key_indent(text: &str, body_start: usize, body_end: usize) -> String {
    for (line_start, line_end) in line_spans(text) {
        if line_start < body_start || line_start >= body_end {
            continue;
        }
        let line = &text[line_start..line_end];
        if line.trim().is_empty() {
            continue;
        }
        let indent_len = line.len() - line.trim_start().len();
        return line[..indent_len].to_string();
    }
    String::new()
}

/// Parse a dependency string from ruv.toml into a `Dep`.
/// Handles formats: "ggplot2", "ggplot2>=3.4", "rlang (>= 1.0)"
pub fn parse_dep(s: &str) -> Dep {
    let name: String = s
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '.' || *c == '-')
        .collect();
    // remainder after the name: strip whitespace and optional surrounding parens
    let rest = s[name.len()..]
        .trim()
        .trim_matches(|c| c == '(' || c == ')');
    let req = crate::version::VersionReq::parse(rest);
    Dep::new(name, req)
}

/// Strips version specifier from a dependency string: "ggplot2>=3.4" → "ggplot2"
/// Kept for callers that only need the name.
pub fn parse_dep_name(dep: &str) -> String {
    parse_dep(dep).name
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_config(text: &str) -> ArrrConfig {
        toml::from_str(text).expect("failed to parse ruv.toml")
    }

    #[test]
    fn test_default_config_toml() {
        let toml = default_config_toml("my-project");
        assert!(toml.contains("name = \"my-project\""));
        assert!(toml.contains("dependencies = []"));
    }

    #[test]
    fn test_add_dependency_to_toml_text_adds_new_dep() {
        let text = "[project]\nname = \"x\"\nversion = \"0.1.0\"\ndependencies = []\n";
        let (updated, added) = add_dependency_to_toml_text(text, "ggplot2").unwrap();
        assert!(added);
        assert!(updated.contains("dependencies = [\n    \"ggplot2\",\n]"));
    }

    #[test]
    fn test_add_dependency_to_toml_text_no_duplicate_by_name() {
        let text =
            "[project]\nname = \"x\"\nversion = \"0.1.0\"\ndependencies = [\"ggplot2>=3.4\"]\n";
        let (updated, added) = add_dependency_to_toml_text(text, "ggplot2").unwrap();
        assert!(!added);
        assert!(updated.contains("ggplot2>=3.4"));
    }

    #[test]
    fn test_add_dependency_converts_inline_array_to_multiline() {
        let text = "[project]\nname = \"x\"\nversion = \"0.1.0\"\ndependencies = [\"ggplot2\"]\n";
        let (updated, added) = add_dependency_to_toml_text(text, "dplyr").unwrap();
        assert!(added);
        assert!(updated.contains("dependencies = [\n    \"ggplot2\",\n    \"dplyr\",\n]"));
    }

    #[test]
    fn test_add_dependency_preserves_project_key_order() {
        let text = "[project]\nname = \"myproject\"\nversion = \"0.1.0\"\ndescription = \"Example\"\n\ndependencies = [\n    \"ggplot2\",\n]\n";
        let (updated, added) = add_dependency_to_toml_text(text, "dplyr").unwrap();
        assert!(added);
        let name_pos = updated.find("name = \"myproject\"").unwrap();
        let version_pos = updated.find("version = \"0.1.0\"").unwrap();
        let description_pos = updated.find("description = \"Example\"").unwrap();
        let deps_pos = updated.find("dependencies = [").unwrap();
        assert!(name_pos < version_pos);
        assert!(version_pos < description_pos);
        assert!(description_pos < deps_pos);
        assert!(updated.contains("    \"ggplot2\","));
        assert!(updated.contains("    \"dplyr\","));
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

    #[test]
    fn test_parse_dep_no_constraint() {
        use crate::version::Op;
        let d = parse_dep("dplyr");
        assert_eq!(d.name, "dplyr");
        assert!(d.req.is_none());

        let d2 = parse_dep("ggplot2>=3.4");
        assert_eq!(d2.name, "ggplot2");
        let req = d2.req.unwrap();
        assert!(matches!(req.op, Op::Gte));
        assert_eq!(req.version, crate::version::RVersion::parse("3.4").unwrap());
    }

    #[test]
    fn test_parse_dep_space_constraint() {
        use crate::version::Op;
        let d = parse_dep("rlang (>= 1.0)");
        assert_eq!(d.name, "rlang");
        let req = d.req.unwrap();
        assert!(matches!(req.op, Op::Gte));
        assert_eq!(req.version, crate::version::RVersion::parse("1.0").unwrap());
    }
}
