use crate::Violation;
use crate::ast::{AttributeMatcher, SpanLocator};
use std::path::{Path, PathBuf};
use syn::visit::Visit;

pub struct PublicFreeFunctionRule;

impl PublicFreeFunctionRule {
    pub fn lint(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        let mut visitor = PublicFreeFunctionVisitor::new(path.to_path_buf());
        visitor.visit_file(syntax);
        visitor.violations
    }
}

struct PublicFreeFunctionVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
    in_test_context: bool,
}

impl PublicFreeFunctionVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
            in_test_context: false,
        }
    }

    fn check_function(&mut self, node: &syn::ItemFn) {
        if self.in_test_context || node.sig.ident == "main" {
            return;
        }
        if matches!(node.vis, syn::Visibility::Inherited) {
            return;
        }
        let (line, column) = SpanLocator::locate(node.sig.ident.span());
        self.violations.push(Violation::new(
            self.file.clone(),
            line,
            column,
            format!(
                "公開された単独関数 `{}` は禁止です。struct と impl に寄せてください。",
                node.sig.ident
            ),
        ));
    }
}

impl<'ast> Visit<'ast> for PublicFreeFunctionVisitor {
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        let previous = self.in_test_context;
        self.in_test_context = previous || AttributeMatcher::has_cfg_test(&node.attrs);
        syn::visit::visit_item_mod(self, node);
        self.in_test_context = previous;
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.check_function(node);
        syn::visit::visit_item_fn(self, node);
    }
}
