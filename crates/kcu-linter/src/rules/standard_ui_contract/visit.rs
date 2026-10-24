use super::StandardUiContractVisitor;
use syn::visit::Visit;

impl<'ast> Visit<'ast> for StandardUiContractVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.check_function(&node.sig);
        self.check_manual_ident(&node.sig.ident);
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.check_function(&node.sig);
        self.check_manual_ident(&node.sig.ident);
        syn::visit::visit_impl_item_fn(self, node);
    }

    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        self.check_chat_surface(node);
        self.check_manual_ident(&node.ident);
        syn::visit::visit_item_struct(self, node);
    }

    fn visit_lit_str(&mut self, node: &'ast syn::LitStr) {
        self.check_manual_literal(node);
        self.check_standard_widget_literal(node);
        syn::visit::visit_lit_str(self, node);
    }

    fn visit_ident(&mut self, node: &'ast syn::Ident) {
        self.check_standard_widget_ident(node);
        syn::visit::visit_ident(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        self.check_tool_send_icon_override(node);
        self.check_debug_popup_fixed_width(node);
        syn::visit::visit_expr_method_call(self, node);
    }
}
