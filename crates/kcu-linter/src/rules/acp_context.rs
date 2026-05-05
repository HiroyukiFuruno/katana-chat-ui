use std::collections::BTreeSet;
use syn::visit::Visit;

#[derive(Default)]
pub(super) struct MethodContract {
    pub(super) uses_context: bool,
    pub(super) calls: BTreeSet<String>,
}

pub(super) struct RequestContextVisitor {
    request_name: Option<String>,
    pub(super) contract: MethodContract,
}

impl RequestContextVisitor {
    pub(super) fn new(request_name: Option<String>) -> Self {
        Self {
            request_name,
            contract: MethodContract::default(),
        }
    }

    fn is_request_path(&self, expr: &syn::Expr) -> bool {
        let (Some(request_name), syn::Expr::Path(path)) = (&self.request_name, expr) else {
            return false;
        };
        path.path
            .get_ident()
            .is_some_and(|it| it == request_name.as_str())
    }

    fn self_call_with_request(&self, call: &syn::ExprCall) -> Option<String> {
        let syn::Expr::Path(func_path) = &*call.func else {
            return None;
        };
        if !Self::is_self_path(&func_path.path) {
            return None;
        }
        if !call.args.iter().any(|it| self.is_request_path(it)) {
            return None;
        }
        func_path
            .path
            .segments
            .last()
            .map(|it| it.ident.to_string())
    }

    fn is_self_path(path: &syn::Path) -> bool {
        path.segments.first().is_some_and(|it| it.ident == "Self")
    }
}

impl<'ast> Visit<'ast> for RequestContextVisitor {
    fn visit_expr_field(&mut self, node: &'ast syn::ExprField) {
        let is_context = matches!(&node.member, syn::Member::Named(ident) if ident == "context");
        if self.is_request_path(&node.base) && is_context {
            self.contract.uses_context = true;
        }
        syn::visit::visit_expr_field(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let Some(method) = self.self_call_with_request(node) {
            self.contract.calls.insert(method);
        }
        syn::visit::visit_expr_call(self, node);
    }
}
