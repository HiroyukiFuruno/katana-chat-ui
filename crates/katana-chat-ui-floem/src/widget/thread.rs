use super::{
    markdown::FloemMarkdownView, output_cards::FloemOutputCardsView, styles,
    thinking::FloemThinkingView, thinking_indicator::FloemThinkingIndicator,
};
use floem::{AnyView, peniko::Color, prelude::*};
use katana_chat_ui::{ChatUiMessageAlignment, ChatUiMessageSurface, ChatUiSurface, MessageStatus};

const MESSAGE_ROW_PADDING_X: f64 = 16.0;
const MESSAGE_CONTENT_GAP: f64 = 10.0;

pub struct FloemThreadView;

impl FloemThreadView {
    pub fn render(surface: RwSignal<ChatUiSurface>) -> impl IntoView {
        scroll(
            dyn_stack(
                move || surface.get().message_list.messages,
                ThreadMessagePresenter::message_key,
                message_row,
            )
            .style(|style| {
                style
                    .gap(styles::MESSAGE_GAP)
                    .width_full()
                    .min_width(0.0)
                    .flex_col()
            }),
        )
        .style(|style| {
            style
                .width_full()
                .min_width(0.0)
                .min_height(0.0)
                .height_full()
                .flex_grow(1.0)
                .flex_shrink(1.0)
                .padding(styles::THREAD_PADDING)
                .items_start()
                .justify_start()
                .background(Color::TRANSPARENT)
        })
    }
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
    if is_waiting_indicator(&message) {
        return FloemThinkingIndicator::render(body, trailing).into_any();
    }
    container(message_content(message, body))
        .style(move |style| bubble_style(style, trailing))
        .into_any()
}

fn message_content(message: ChatUiMessageSurface, body: String) -> AnyView {
    let markdown = FloemMarkdownView::render(message.blocks, body);
    let outputs = FloemOutputCardsView::render(message.outputs);
    let Some(thinking) = message.thinking else {
        return v_stack((markdown, outputs))
            .style(|style| style.width_full().min_width(0.0).gap(MESSAGE_CONTENT_GAP))
            .into_any();
    };
    v_stack((FloemThinkingView::render(thinking), markdown, outputs))
        .style(|style| style.width_full().min_width(0.0).gap(MESSAGE_CONTENT_GAP))
        .into_any()
}

fn bubble_style(style: floem::style::Style, trailing: bool) -> floem::style::Style {
    style
        .width_pct(styles::BUBBLE_WIDTH_PERCENT)
        .max_width(styles::BUBBLE_MAX_WIDTH)
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
}

fn is_waiting_indicator(message: &ChatUiMessageSurface) -> bool {
    matches!(
        message.status,
        MessageStatus::Sending | MessageStatus::Streaming
    ) && ThreadMessagePresenter::visible_body(message).trim() == message.status_label
}

pub(super) struct ThreadMessagePresenter;

impl ThreadMessagePresenter {
    pub(super) fn message_key(message: &ChatUiMessageSurface) -> (u64, &'static str, usize, usize) {
        (
            message.id,
            Self::status_key(&message.status),
            message.body.len(),
            message.outputs.len(),
        )
    }

    pub(super) fn visible_body(message: &ChatUiMessageSurface) -> String {
        if !message.body.is_empty() {
            return message.body.clone();
        }
        match &message.status {
            MessageStatus::Sending | MessageStatus::Streaming => message.status_label.clone(),
            MessageStatus::Error(error) => error.clone(),
            MessageStatus::Complete => String::new(),
        }
    }

    pub(super) fn bubble_body(message: &ChatUiMessageSurface) -> String {
        Self::visible_body(message)
    }

    #[cfg(test)]
    pub(super) fn bubble_width_percent() -> f64 {
        styles::BUBBLE_WIDTH_PERCENT
    }

    #[cfg(test)]
    pub(super) fn is_waiting_indicator(message: &ChatUiMessageSurface) -> bool {
        is_waiting_indicator(message)
    }

    fn status_key(status: &MessageStatus) -> &'static str {
        match status {
            MessageStatus::Sending => "sending",
            MessageStatus::Streaming => "streaming",
            MessageStatus::Complete => "complete",
            MessageStatus::Error(_) => "error",
        }
    }
}
