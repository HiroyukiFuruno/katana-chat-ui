use crate::Violation;
use crate::ast::SpanLocator;
use quote::ToTokens;
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;
use syn::visit::Visit;

pub struct ProhibitedTypeRule;

impl ProhibitedTypeRule {
    pub fn lint(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        let mut visitor = ProhibitedTypeVisitor::new(path.to_path_buf());
        visitor.visit_file(syntax);
        visitor.violations
    }
}

struct ProhibitedTypeVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
}

impl ProhibitedTypeVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
        }
    }

    fn check_type(&mut self, node: &syn::Type) {
        let rendered = node.to_token_stream().to_string();
        if !Self::is_prohibited(&rendered) {
            return;
        }
        let (line, column) = SpanLocator::locate(node.span());
        self.violations.push(Violation::new(
            self.file.clone(),
            line,
            column,
            format!("禁止型 `{rendered}` を使わず、専用の型を定義してください。"),
        ));
    }

    fn is_prohibited(rendered: &str) -> bool {
        let normalized = rendered.replace(' ', "");
        normalized == "Box<dynstd::any::Any>"
            || normalized == "HashMap<String,serde_json::Value>"
            || normalized == "std::sync::RwLock"
    }
}

impl<'ast> Visit<'ast> for ProhibitedTypeVisitor {
    fn visit_type(&mut self, node: &'ast syn::Type) {
        self.check_type(node);
        syn::visit::visit_type(self, node);
    }
}
