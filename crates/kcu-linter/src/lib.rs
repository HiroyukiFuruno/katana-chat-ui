pub mod ast;
pub mod rules;

use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub message: String,
}

impl Violation {
    pub fn new(
        file: impl Into<PathBuf>,
        line: usize,
        column: usize,
        message: impl Into<String>,
    ) -> Self {
        Self {
            file: file.into(),
            line,
            column,
            message: message.into(),
        }
    }
}

pub type Rule = fn(&Path, &syn::File) -> Vec<Violation>;

pub struct AstLintRunner;

impl AstLintRunner {
    pub fn run(rule_name: &str, hint: &str, target_dirs: &[PathBuf], rule: Rule) {
        let violations = Self::collect_violations(target_dirs, rule);
        Self::panic_on_violations(rule_name, hint, &violations);
    }

    pub fn workspace_root() -> Result<PathBuf, String> {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        match manifest_dir.parent().and_then(|it| it.parent()) {
            Some(root) => Ok(root.to_path_buf()),
            None => Err("workspace root was not found".to_string()),
        }
    }

    pub fn target_src_dirs(root: &Path) -> Vec<PathBuf> {
        vec![
            root.join("crates/katana-acp-client/src"),
            root.join("crates/katana-chat-ui/src"),
            root.join("crates/kcu-linter/src"),
        ]
    }

    fn collect_violations(target_dirs: &[PathBuf], rule: Rule) -> Vec<Violation> {
        let mut violations = Vec::new();
        for target_dir in target_dirs {
            for file in Self::collect_rust_files(target_dir) {
                violations.extend(Self::lint_file(&file, rule));
            }
        }
        violations
    }

    fn collect_rust_files(root: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let walker = WalkBuilder::new(root).standard_filters(true).build();
        for entry in walker.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|it| it == "rs") {
                files.push(path.to_path_buf());
            }
        }
        files.sort();
        files
    }

    fn lint_file(file: &Path, rule: Rule) -> Vec<Violation> {
        match Self::parse_file(file) {
            Ok(syntax) => rule(file, &syntax),
            Err(violation) => vec![violation],
        }
    }

    fn parse_file(file: &Path) -> Result<syn::File, Violation> {
        let source = std::fs::read_to_string(file).map_err(|err| {
            Violation::new(file, 1, 1, format!("Rust ファイルを読めません: {err}"))
        })?;
        syn::parse_file(&source).map_err(|err| {
            let (line, column) = ast::SpanLocator::locate(err.span());
            Violation::new(
                file,
                line,
                column,
                format!("Rust 構文を解析できません: {err}"),
            )
        })
    }

    fn panic_on_violations(rule_name: &str, hint: &str, violations: &[Violation]) {
        if violations.is_empty() {
            return;
        }
        assert!(
            violations.is_empty(),
            "{}",
            Self::format_report(rule_name, hint, violations)
        );
    }

    fn format_report(rule_name: &str, hint: &str, violations: &[Violation]) -> String {
        let mut report = format!("\n[AST Lint Error] {rule_name}\n");
        for violation in violations {
            report.push_str(&format!(
                "  {}:{}:{} - {}\n",
                violation.file.display(),
                violation.line,
                violation.column,
                violation.message
            ));
        }
        report.push_str(&format!("\n修正方針: {hint}\n"));
        report
    }
}
