use crate::Violation;
use std::path::Path;

pub struct CommentStyleRule;

impl CommentStyleRule {
    pub fn lint(path: &Path, _syntax: &syn::File) -> Vec<Violation> {
        let Ok(content) = std::fs::read_to_string(path) else {
            return Vec::new();
        };
        content
            .lines()
            .enumerate()
            .filter_map(|(line_index, line)| Self::violation(path, line_index, line))
            .collect()
    }

    fn violation(path: &Path, line_index: usize, line: &str) -> Option<Violation> {
        let trimmed = line.trim();
        if !Self::is_plain_line_comment(trimmed) {
            return None;
        }
        Some(Violation::new(
            path,
            line_index + 1,
            1,
            "通常の `//` コメントは禁止です。`/* WHY: ... */` 等で理由を書いてください。",
        ))
    }

    fn is_plain_line_comment(trimmed: &str) -> bool {
        trimmed.starts_with("//") && !trimmed.starts_with("///") && !trimmed.starts_with("//!")
    }
}
