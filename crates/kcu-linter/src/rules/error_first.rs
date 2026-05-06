use crate::Violation;
use crate::ast::SpanLocator;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

pub struct ErrorFirstRule;

impl ErrorFirstRule {
    pub fn lint(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        let mut visitor = ErrorFirstVisitor::new(path.to_path_buf());
        visitor.visit_file(syntax);
        visitor.violations
    }
}

struct ErrorFirstVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
}

impl ErrorFirstVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
        }
    }

    fn check_if_let_ok(&mut self, node: &syn::ExprIf) {
        let syn::Expr::Let(let_expr) = &*node.cond else {
            return;
        };
        if !Self::is_ok_pattern(&let_expr.pat) {
            return;
        }
        let (line, column) = SpanLocator::locate(let_expr.let_token.span);
        self.violations.push(Violation::new(
            self.file.clone(),
            line,
            column,
            "`if let Ok(...)` で成功パスを包まず、`?` または早期 return を使ってください。",
        ));
    }

    fn is_ok_pattern(pattern: &syn::Pat) -> bool {
        match pattern {
            syn::Pat::TupleStruct(tuple) => tuple
                .path
                .segments
                .last()
                .is_some_and(|it| it.ident == "Ok"),
            syn::Pat::Path(path) => path.path.segments.last().is_some_and(|it| it.ident == "Ok"),
            _ => false,
        }
    }
}

impl<'ast> Visit<'ast> for ErrorFirstVisitor {
    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        self.check_if_let_ok(node);
        syn::visit::visit_expr_if(self, node);
    }
}
