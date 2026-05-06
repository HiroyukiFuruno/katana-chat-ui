use crate::Violation;
use crate::ast::TestFileMatcher;
use std::path::Path;

const MAX_MIXED_FILE_LINES: usize = 250;

pub struct TypeSeparationRule;

impl TypeSeparationRule {
    pub fn lint(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        if TestFileMatcher::is_test_file(path) || Self::is_allowed_file(path) {
            return Vec::new();
        }
        let line_count = Self::line_count(path);
        if line_count <= MAX_MIXED_FILE_LINES || !Self::mixes_public_type_and_logic(syntax) {
            return Vec::new();
        }
        vec![Violation::new(
            path,
            1,
            1,
            "公開型と実装ロジックが同居しています。types.rs / state.rs 等へ責務分離してください。",
        )]
    }

    fn line_count(path: &Path) -> usize {
        match std::fs::read_to_string(path) {
            Ok(content) => content.lines().count(),
            Err(_) => 0,
        }
    }

    fn mixes_public_type_and_logic(syntax: &syn::File) -> bool {
        let has_type = syntax.items.iter().any(Self::is_public_type);
        let has_logic = syntax.items.iter().any(Self::is_impl_with_function);
        has_type && has_logic
    }

    fn is_public_type(item: &syn::Item) -> bool {
        matches!(item, syn::Item::Struct(it) if matches!(it.vis, syn::Visibility::Public(_)))
            || matches!(item, syn::Item::Enum(it) if matches!(it.vis, syn::Visibility::Public(_)))
    }

    fn is_impl_with_function(item: &syn::Item) -> bool {
        matches!(item, syn::Item::Impl(it) if it.items.iter().any(|item| matches!(item, syn::ImplItem::Fn(_))))
    }

    fn is_allowed_file(path: &Path) -> bool {
        let path_text = path.to_string_lossy().replace('\\', "/");
        path_text.ends_with("types.rs")
            || path_text.ends_with("state.rs")
            || path_text.ends_with("lib.rs")
            || path_text.ends_with("main.rs")
            || path_text.contains("/types/")
            || path_text.contains("/state/")
    }
}
