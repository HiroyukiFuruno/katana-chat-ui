use crate::Violation;
use crate::ast::SpanLocator;
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;
use syn::visit::Visit;

pub struct ProhibitedAttributeRule;

impl ProhibitedAttributeRule {
    pub fn lint(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        let mut visitor = ProhibitedAttributeVisitor::new(path.to_path_buf());
        visitor.check_attrs(&syntax.attrs);
        visitor.visit_file(syntax);
        visitor.violations
    }
}

struct ProhibitedAttributeVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
}

impl ProhibitedAttributeVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
        }
    }

    fn check_attrs(&mut self, attrs: &[syn::Attribute]) {
        for attr in attrs {
            self.check_attr(attr);
        }
    }

    fn check_attr(&mut self, attr: &syn::Attribute) {
        if !attr.path().is_ident("allow") {
            return;
        }
        let (line, column) = SpanLocator::locate(attr.span());
        self.violations.push(Violation::new(
            self.file.clone(),
            line,
            column,
            "`#[allow(...)]` は禁止です。検査を緩めず、原因側を修正してください。",
        ));
    }
}

impl<'ast> Visit<'ast> for ProhibitedAttributeVisitor {
    fn visit_attribute(&mut self, node: &'ast syn::Attribute) {
        self.check_attr(node);
        syn::visit::visit_attribute(self, node);
    }
}
