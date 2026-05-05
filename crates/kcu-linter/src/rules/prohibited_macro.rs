use crate::Violation;
use crate::ast::SpanLocator;
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;
use syn::visit::Visit;

const PROHIBITED_MACROS: [&str; 3] = ["todo", "unimplemented", "dbg"];

pub struct ProhibitedMacroRule;

impl ProhibitedMacroRule {
    pub fn lint(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        let mut visitor = ProhibitedMacroVisitor::new(path.to_path_buf());
        visitor.visit_file(syntax);
        visitor.violations
    }
}

struct ProhibitedMacroVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
}

impl ProhibitedMacroVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
        }
    }

    fn check_macro(&mut self, node: &syn::Macro) {
        let Some(segment) = node.path.segments.last() else {
            return;
        };
        if !PROHIBITED_MACROS.contains(&segment.ident.to_string().as_str()) {
            return;
        }
        let (line, column) = SpanLocator::locate(node.path.span());
        self.violations.push(Violation::new(
            self.file.clone(),
            line,
            column,
            format!(
                "`{}!()` は禁止です。実装または通常のログ処理に置き換えてください。",
                segment.ident
            ),
        ));
    }
}

impl<'ast> Visit<'ast> for ProhibitedMacroVisitor {
    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        self.check_macro(node);
        syn::visit::visit_macro(self, node);
    }
}
