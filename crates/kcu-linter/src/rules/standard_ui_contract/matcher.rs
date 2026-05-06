use quote::ToTokens;
use std::path::Path;

const FLOEM_RENDERER_PATH: &str = "crates/katana-chat-ui-floem/src/";
const MANUAL_HOST_PATH: &str = "tools/manual-host-";
const REQUIRED_CHAT_SURFACE_FIELDS: [&str; 5] = [
    "messages",
    "composer",
    "usage",
    "vendor_controls",
    "settings_icon",
];
const STANDARD_UI_TYPES: [&str; 5] = [
    "ChatUiSurface",
    "ChatUiMessageSurface",
    "ChatUiOutputHandoffSurface",
    "VendorControlRenderModel",
    "VendorControlItem",
];
const MANUAL_COMPLETION_SUBJECTS: [&str; 5] = [
    "standard ui",
    "standard_ui",
    "standardui",
    "chatui",
    "標準ui",
];
const MANUAL_COMPLETION_RESULTS: [&str; 5] = ["complete", "completed", "ready", "done", "完了"];
const PROHIBITED_STANDARD_WIDGET_IDENTIFIERS: [&str; 7] = [
    "toolbar_settings_button",
    "settings_icon",
    "settings_label",
    "debug_hover",
    "debug_popup",
    "output_json",
    "CONTENT_WIDTH",
];
const PROHIBITED_STANDARD_WIDGET_LITERALS: [&str; 2] = ["debug", "output JSON"];

pub(super) struct StandardUiContractMatcher {
    path_text: String,
}

impl StandardUiContractMatcher {
    pub(super) fn new(file: &Path) -> Self {
        Self {
            path_text: file.to_string_lossy().replace('\\', "/"),
        }
    }

    pub(super) fn is_floem_renderer(&self) -> bool {
        self.path_text.contains(FLOEM_RENDERER_PATH)
    }

    pub(super) fn is_manual_host(&self) -> bool {
        self.path_text.contains(MANUAL_HOST_PATH)
    }

    pub(super) fn is_standard_widget_file(&self) -> bool {
        self.path_text
            .contains("crates/katana-chat-ui-floem/src/widget")
    }

    pub(super) fn is_tool_source(&self) -> bool {
        self.path_text.contains("tools/")
    }

    pub(super) fn returns_string(output: &syn::ReturnType) -> bool {
        let syn::ReturnType::Type(_, ty) = output else {
            return false;
        };
        ty.to_token_stream().to_string() == "String"
    }

    pub(super) fn is_standard_ui_text_collapse(signature: &syn::Signature) -> bool {
        Self::name_implies_text_collapse(&signature.ident.to_string())
            && Self::inputs_include_standard_ui_type(&signature.inputs)
    }

    pub(super) fn missing_chat_surface_fields(node: &syn::ItemStruct) -> Vec<&'static str> {
        let field_names = Self::chat_surface_field_names(node);
        REQUIRED_CHAT_SURFACE_FIELDS
            .iter()
            .copied()
            .filter(|it| !field_names.iter().any(|field_name| field_name == it))
            .collect()
    }

    pub(super) fn claims_manual_completion(text: &str) -> bool {
        let normalized = text.to_lowercase();
        Self::contains_any(&normalized, &MANUAL_COMPLETION_SUBJECTS)
            && Self::contains_any(&normalized, &MANUAL_COMPLETION_RESULTS)
    }

    pub(super) fn is_prohibited_standard_widget_identifier(name: &str) -> bool {
        PROHIBITED_STANDARD_WIDGET_IDENTIFIERS.contains(&name)
    }

    pub(super) fn is_prohibited_standard_widget_literal(text: &str) -> bool {
        PROHIBITED_STANDARD_WIDGET_LITERALS.contains(&text)
    }

    pub(super) fn is_send_icon_override_arg(expr: &syn::Expr) -> bool {
        let syn::Expr::Call(call) = expr else {
            return false;
        };
        Self::is_svg_icon_new_call(&call.func)
            && call.args.first().is_some_and(Self::is_send_literal)
    }

    pub(super) fn is_debug_popup_width_arg(expr: &syn::Expr) -> bool {
        let syn::Expr::Path(path) = expr else {
            return false;
        };
        path.path
            .get_ident()
            .is_some_and(|it| it.to_string().starts_with("DEBUG_POPUP_WIDTH"))
    }

    fn name_implies_text_collapse(name: &str) -> bool {
        name.ends_with("_text") || name.ends_with("_row")
    }

    fn inputs_include_standard_ui_type(
        inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    ) -> bool {
        inputs
            .iter()
            .map(ToTokens::to_token_stream)
            .map(|it| it.to_string())
            .any(|it| {
                STANDARD_UI_TYPES
                    .iter()
                    .any(|type_name| it.contains(type_name))
            })
    }

    fn chat_surface_field_names(node: &syn::ItemStruct) -> Vec<String> {
        let syn::Fields::Named(fields) = &node.fields else {
            return Vec::new();
        };
        fields
            .named
            .iter()
            .filter_map(|it| it.ident.as_ref())
            .map(ToString::to_string)
            .collect()
    }

    fn contains_any(text: &str, needles: &[&str]) -> bool {
        needles.iter().any(|it| text.contains(it))
    }

    fn is_svg_icon_new_call(expr: &syn::Expr) -> bool {
        let syn::Expr::Path(path) = expr else {
            return false;
        };
        let segments = &path.path.segments;
        let Some(last) = segments.last() else {
            return false;
        };
        let previous = segments.iter().rev().nth(1);
        last.ident == "new" && previous.is_some_and(|it| it.ident == "SvgIcon")
    }

    fn is_send_literal(expr: &syn::Expr) -> bool {
        match expr {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(value),
                ..
            }) => value.value() == "send",
            syn::Expr::Paren(paren) => Self::is_send_literal(&paren.expr),
            syn::Expr::Reference(reference) => Self::is_send_literal(&reference.expr),
            _ => false,
        }
    }
}
