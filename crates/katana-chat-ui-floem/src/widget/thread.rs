use super::{
    markdown::FloemMarkdownView,
    styles,
    thinking::FloemThinkingView,
    thinking_indicator::FloemThinkingIndicator,
    thread_layout::{MessageBubbleLayout, ThreadMessagePresenter},
};
use floem::{AnyView, peniko::Color, prelude::*};
use katana_chat_ui::{ChatUiMessageAlignment, ChatUiMessageSurface, ChatUiSurface};

const MESSAGE_ROW_PADDING_X: f64 = 5.0;
const MESSAGE_CONTENT_GAP: f64 = 10.0;

pub struct FloemThreadView;

impl FloemThreadView {
    pub fn render(surface: RwSignal<ChatUiSurface>) -> impl IntoView {
        scroll(message_stack(surface)).style(thread_scroll_style)
    }
}

fn message_stack(surface: RwSignal<ChatUiSurface>) -> impl IntoView {
    h_stack((message_column(surface),)).style(message_stack_style)
}

fn message_column(surface: RwSignal<ChatUiSurface>) -> impl IntoView {
    dyn_stack(
        move || surface.get().message_list.messages,
        ThreadMessagePresenter::message_key,
        message_row,
    )
    .style(message_column_style)
}

fn message_column_style(style: floem::style::Style) -> floem::style::Style {
    style
        .gap(styles::MESSAGE_GAP)
        .width_full()
        .max_width(styles::CHAT_BODY_MAX_WIDTH)
        .min_width(0.0)
        .flex_col()
}

fn message_stack_style(style: floem::style::Style) -> floem::style::Style {
    style
        .width_full()
        .min_width(0.0)
        .items_center()
        .justify_center()
}

fn thread_scroll_style(style: floem::style::Style) -> floem::style::Style {
    style
        .width_full()
        .min_width(0.0)
        .min_height(0.0)
        .height_full()
        .flex_grow(1.0)
        .flex_shrink(1.0)
        .padding(styles::THREAD_PADDING)
        .items_center()
        .justify_start()
        .background(Color::TRANSPARENT)
}

fn message_row(message: ChatUiMessageSurface) -> impl IntoView {
    let trailing = message.alignment == ChatUiMessageAlignment::Trailing;
    h_stack((message_bubble(message),)).style(move |style| {
        style
            .width_full()
            .min_width(0.0)
            .padding_horiz(MESSAGE_ROW_PADDING_X)
            .apply_if(trailing, |style| style.justify_end())
            .apply_if(!trailing, |style| style.justify_start())
    })
}

fn message_bubble(message: ChatUiMessageSurface) -> AnyView {
    let trailing = message.alignment == ChatUiMessageAlignment::Trailing;
    let body = ThreadMessagePresenter::bubble_body(&message);
    if body.trim().is_empty() && message.thinking.is_none() {
        return empty().into_any();
    }
    if ThreadMessagePresenter::is_waiting_indicator(&message) {
        return FloemThinkingIndicator::render(body, trailing).into_any();
    }
    let layout = ThreadMessagePresenter::bubble_layout(&message);
    let Some(thinking) = message.thinking.clone() else {
        return message_body_bubble(message, body, layout).into_any();
    };
    if body.trim().is_empty() {
        return FloemThinkingView::render(thinking);
    }
    v_stack((
        FloemThinkingView::render(thinking),
        message_body_bubble(message, body, layout),
    ))
    .style(|style| style.width_full().min_width(0.0).gap(MESSAGE_CONTENT_GAP))
    .into_any()
}

fn message_body_bubble(
    message: ChatUiMessageSurface,
    body: String,
    layout: MessageBubbleLayout,
) -> impl IntoView {
    container(message_content(message, body, layout))
        .style(move |style| bubble_style(style, layout))
}

fn message_content(
    message: ChatUiMessageSurface,
    body: String,
    layout: MessageBubbleLayout,
) -> AnyView {
    let markdown = match layout {
        MessageBubbleLayout::AgentFixed => FloemMarkdownView::render(message.blocks, body),
        MessageBubbleLayout::ContentSized => {
            FloemMarkdownView::render_compact(message.blocks, body)
        }
    };
    v_stack((markdown,))
        .style(move |style| content_style(style, layout))
        .into_any()
}

fn content_style(style: floem::style::Style, layout: MessageBubbleLayout) -> floem::style::Style {
    style
        .min_width(0.0)
        .gap(MESSAGE_CONTENT_GAP)
        .apply_if(layout == MessageBubbleLayout::AgentFixed, |style| {
            style.width_full()
        })
        .apply_if(layout == MessageBubbleLayout::ContentSized, |style| {
            style.items_center()
        })
}

fn bubble_style(style: floem::style::Style, layout: MessageBubbleLayout) -> floem::style::Style {
    let trailing = layout == MessageBubbleLayout::ContentSized;
    style
        .min_width(0.0)
        .flex_shrink(1.0)
        .padding_horiz(styles::BUBBLE_PADDING_X)
        .padding_vert(styles::BUBBLE_PADDING_Y)
        .border_radius(styles::BUBBLE_RADIUS)
        .border(1.0)
        .border_color(styles::COLOR_BORDER)
        .background(if trailing {
            styles::COLOR_USER
        } else {
            styles::COLOR_ASSISTANT
        })
        .color(styles::COLOR_TEXT)
        .apply_if(layout == MessageBubbleLayout::AgentFixed, |style| {
            style
                .width_pct(styles::AGENT_BUBBLE_WIDTH_PERCENT)
                .max_width(styles::AGENT_BUBBLE_MAX_WIDTH)
        })
        .apply_if(layout == MessageBubbleLayout::ContentSized, |style| {
            style
                .max_width(styles::USER_BUBBLE_MAX_WIDTH)
                .items_center()
        })
}
