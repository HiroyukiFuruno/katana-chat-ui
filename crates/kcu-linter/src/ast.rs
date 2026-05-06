use syn::visit::Visit;

pub struct SpanLocator;

impl SpanLocator {
    pub fn locate(span: proc_macro2::Span) -> (usize, usize) {
        let start = span.start();
        (start.line, start.column + 1)
    }

    pub fn line_span(span: proc_macro2::Span) -> (usize, usize) {
        (span.start().line, span.end().line)
    }
}

pub struct AttributeMatcher;

impl AttributeMatcher {
    pub fn has_cfg_test(attrs: &[syn::Attribute]) -> bool {
        attrs.iter().any(Self::is_cfg_test)
    }

    fn is_cfg_test(attr: &syn::Attribute) -> bool {
        if !attr.path().is_ident("cfg") {
            return attr.path().is_ident("test");
        }
        let Ok(syn::Meta::Path(path)) = attr.parse_args::<syn::Meta>() else {
            return false;
        };
        path.is_ident("test")
    }
}

pub struct TestFileMatcher;

impl TestFileMatcher {
    pub fn is_test_file(path: &std::path::Path) -> bool {
        let path_text = path.to_string_lossy().replace('\\', "/");
        path_text.contains("/tests/") || path_text.ends_with("tests.rs")
    }
}

pub struct NumericLiteralMatcher;

impl NumericLiteralMatcher {
    pub fn is_allowed(value: f64) -> bool {
        [0.0, 1.0, 2.0, 100.0, -1.0]
            .iter()
            .any(|it| (value - it).abs() < f64::EPSILON)
    }
}

pub struct MethodCallFinder;

impl MethodCallFinder {
    pub fn contains(expr: &syn::Expr, method: &str) -> bool {
        let mut visitor = MethodCallVisitor {
            method,
            found: false,
        };
        visitor.visit_expr(expr);
        visitor.found
    }
}

struct MethodCallVisitor<'a> {
    method: &'a str,
    found: bool,
}

impl<'ast> Visit<'ast> for MethodCallVisitor<'_> {
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if node.method == self.method {
            self.found = true;
            return;
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}
