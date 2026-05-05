use crate::Violation;
use std::path::Path;

const MAX_FILE_LINES: usize = 200;

pub struct FileLengthRule;

impl FileLengthRule {
    pub fn lint(path: &Path, _syntax: &syn::File) -> Vec<Violation> {
        let Ok(content) = std::fs::read_to_string(path) else {
            return vec![Violation::new(
                path,
                1,
                1,
                "ファイル行数を確認するための読み取りに失敗しました。",
            )];
        };

        let line_count = content.lines().count();
        if line_count <= MAX_FILE_LINES {
            return Vec::new();
        }

        vec![Violation::new(
            path,
            1,
            1,
            format!(
                "{MAX_FILE_LINES} 行を超えています。現在は {line_count} 行です。責務単位で分割してください。"
            ),
        )]
    }
}
