use kcu_linter::AstLintRunner;
use std::path::{Path, PathBuf};

pub(crate) struct Fixture;

impl Fixture {
    pub(crate) fn parse(code: &str) -> Result<syn::File, syn::Error> {
        syn::parse_file(code)
    }

    pub(crate) fn path() -> &'static Path {
        Path::new("fixture.rs")
    }

    pub(crate) fn root() -> Result<PathBuf, String> {
        AstLintRunner::workspace_root()
    }
}

pub(crate) fn floem_widget_path() -> &'static Path {
    Path::new("crates/katana-chat-ui-floem/src/widget.rs")
}

pub(crate) fn manual_host_path() -> &'static Path {
    Path::new("tools/manual-host-floem/src/main.rs")
}

pub(crate) fn manual_host_src_dirs(root: &Path) -> Vec<PathBuf> {
    vec![
        root.join("tools/manual-host-egui/src"),
        root.join("tools/manual-host-floem/src"),
        root.join("tools/manual-host-gpui/src"),
    ]
}

pub(crate) fn read_workspace_file(root: &Path, path: &str) -> Result<String, String> {
    let absolute_path = root.join(path);
    std::fs::read_to_string(&absolute_path)
        .map_err(|it| format!("{} を読めません: {it}", absolute_path.display()))
}

pub(crate) fn assert_contains(content: &str, expected: &str) {
    assert!(
        content.contains(expected),
        "`{expected}` が見つかりません。API-only 完了扱いを防ぐ contract を維持してください。"
    );
}
