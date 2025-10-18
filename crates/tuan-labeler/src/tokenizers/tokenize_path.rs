use convert_case::{Case, Casing};
use std::path::Path;

pub fn tokenize_path(file_path: &Path, project_path: &str) -> Vec<String> {
    let path_str = file_path.to_string_lossy();
    let path_str = path_str.strip_prefix(project_path).unwrap_or(&path_str);
    path_str
        .to_case(Case::Lower)
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| s.len() >= 3)
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
}
