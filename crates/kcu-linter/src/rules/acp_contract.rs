use crate::Violation;
use crate::ast::SpanLocator;
use quote::ToTokens;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use syn::visit::Visit;

use super::acp_context::{MethodContract, RequestContextVisitor};

pub struct AcpContractRule;

impl AcpContractRule {
    pub fn lint(path: &Path, syntax: &syn::File) -> Vec<Violation> {
        let methods = AcpContractVisitor::method_contracts(syntax);
        let mut visitor = AcpContractVisitor::new(path.to_path_buf(), methods);
        visitor.visit_file(syntax);
        visitor.violations
    }
}

struct AcpContractVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
    methods: BTreeMap<String, MethodContract>,
}

impl AcpContractVisitor {
    fn new(file: PathBuf, methods: BTreeMap<String, MethodContract>) -> Self {
        Self {
            file,
            violations: Vec::new(),
            methods,
        }
    }

    fn check_impl(&mut self, node: &syn::ItemImpl) {
        if !Self::is_ai_provider_impl(node) {
            return;
        }
        if Self::method_uses_context("execute", &self.methods, &mut BTreeSet::new()) {
            return;
        }
        let (line, column) = SpanLocator::locate(node.impl_token.span);
        self.violations.push(Violation::new(
            self.file.clone(),
            line,
            column,
            "`AiProvider::execute` は `AiRequest.context` をプロンプト構築に使ってください。",
        ));
    }

    fn is_ai_provider_impl(node: &syn::ItemImpl) -> bool {
        let Some((_, path, _)) = &node.trait_ else {
            return false;
        };
        path.segments
            .last()
            .is_some_and(|it| it.ident == "AiProvider")
    }

    fn method_contracts(syntax: &syn::File) -> BTreeMap<String, MethodContract> {
        let mut methods = BTreeMap::new();
        for item in &syntax.items {
            Self::collect_item_methods(item, &mut methods);
        }
        methods
    }

    fn collect_item_methods(item: &syn::Item, methods: &mut BTreeMap<String, MethodContract>) {
        let syn::Item::Impl(item_impl) = item else {
            return;
        };
        for impl_item in &item_impl.items {
            if let syn::ImplItem::Fn(method) = impl_item {
                methods.insert(method.sig.ident.to_string(), Self::inspect_method(method));
            }
        }
    }

    fn inspect_method(method: &syn::ImplItemFn) -> MethodContract {
        let request_name = Self::request_parameter_name(method);
        let token_uses_context = Self::method_tokens_use_context(method, request_name.as_deref());
        let mut visitor = RequestContextVisitor::new(request_name);
        visitor.visit_block(&method.block);
        visitor.contract.uses_context |= token_uses_context;
        visitor.contract
    }

    fn method_tokens_use_context(method: &syn::ImplItemFn, request_name: Option<&str>) -> bool {
        let Some(request_name) = request_name else {
            return false;
        };
        let needle = format!("{request_name} . context");
        method.block.to_token_stream().to_string().contains(&needle)
    }

    fn request_parameter_name(method: &syn::ImplItemFn) -> Option<String> {
        for input in &method.sig.inputs {
            let syn::FnArg::Typed(pat_type) = input else {
                continue;
            };
            if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                return Some(pat_ident.ident.to_string());
            }
        }
        None
    }

    fn method_uses_context(
        name: &str,
        methods: &BTreeMap<String, MethodContract>,
        visited: &mut BTreeSet<String>,
    ) -> bool {
        if !visited.insert(name.to_string()) {
            return false;
        }
        let Some(contract) = methods.get(name) else {
            return false;
        };
        contract.uses_context
            || contract
                .calls
                .iter()
                .any(|it| Self::method_uses_context(it, methods, visited))
            || Self::execute_delegates_to_context_method(name, contract, methods)
    }

    fn execute_delegates_to_context_method(
        name: &str,
        contract: &MethodContract,
        methods: &BTreeMap<String, MethodContract>,
    ) -> bool {
        name == "execute"
            && !contract.calls.is_empty()
            && methods.values().any(|it| it.uses_context)
    }
}

impl<'ast> Visit<'ast> for AcpContractVisitor {
    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        self.check_impl(node);
        syn::visit::visit_item_impl(self, node);
    }
}
