use crate::Violation;
use crate::ast::SpanLocator;
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;
use syn::visit::Visit;

const MAX_FUNCTION_LINES: usize = 30;

pub struct FunctionLengthRule;

impl FunctionLengthRule {
    pub fn lint(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        let mut visitor = FunctionLengthVisitor::new(path.to_path_buf());
        visitor.visit_file(syntax);
        visitor.violations
    }
}

struct FunctionLengthVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
}

impl FunctionLengthVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
        }
    }

    fn check_block(&mut self, name: &syn::Ident, block: &syn::Block) {
        let (start, end) = SpanLocator::line_span(block.span());
        let lines = end.saturating_sub(start) + 1;
        if lines <= MAX_FUNCTION_LINES {
            return;
        }
        let (line, column) = SpanLocator::locate(name.span());
        self.violations.push(Violation::new(
            self.file.clone(),
            line,
            column,
            format!(
                "関数 `{name}` が {MAX_FUNCTION_LINES} 行を超えています。現在は {lines} 行です。"
            ),
        ));
    }
}

impl<'ast> Visit<'ast> for FunctionLengthVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.check_block(&node.sig.ident, &node.block);
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.check_block(&node.sig.ident, &node.block);
        syn::visit::visit_impl_item_fn(self, node);
    }
}
