use gpui::{
    App, Application, Bounds, Context, Div, Window, WindowBounds, WindowOptions, div, prelude::*,
    px, rgb, size,
};
use katana_chat_ui::{
    ChatOutputKind, ChatRenderModel, ChatSession, ChatSessionError, DiffCandidateOutput,
    MessageRenderModel, OutputRenderModel,
};

struct ManualGpuiHost {
    model: ChatRenderModel,
    click_count: usize,
}

impl ManualGpuiHost {
    fn new() -> Result<Self, ChatSessionError> {
        let mut session = ChatSession::new();
        session.set_provider_configured("Ollama local");
        session.draft_mut().set_text("GPUI host から確認");
        session.submit_draft()?;
        let assistant_id = session.start_assistant_stream("GPUI host 応答")?;
        session.finish_assistant_message()?;
        session.add_output(assistant_id, Self::sample_output())?;
        Ok(Self {
            model: session.render_model(),
            click_count: 0,
        })
    }

    fn on_sample_click(
        &mut self,
        _event: &gpui::ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.click_count += 1;
        cx.notify();
    }

    fn sample_output() -> ChatOutputKind {
        ChatOutputKind::DiffCandidate(DiffCandidateOutput::new(
            "src/lib.rs",
            "--- a/src/lib.rs\n+++ b/src/lib.rs\n",
        ))
    }
}

impl Render for ManualGpuiHost {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .p_4()
            .bg(rgb(0x202124))
            .text_color(rgb(0xf1f3f4))
            .size_full()
            .gap_4()
            .child(Self::header())
            .child(
                div()
                    .flex()
                    .gap_4()
                    .child(Self::panel("会話表示", self.message_rows()).flex_1())
                    .child(Self::panel("出力物と host 操作", self.output_rows()).flex_1()),
            )
            .child(Self::panel("入力欄", self.composer_rows()))
            .child(
                div()
                    .id("sample-action")
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .bg(rgb(0x3f5f8f))
                    .text_color(rgb(0xffffff))
                    .cursor_pointer()
                    .child(format!(
                        "送信を模擬する / 操作回数: {} 回",
                        self.click_count
                    ))
                    .on_click(cx.listener(Self::on_sample_click)),
            )
    }
}

impl ManualGpuiHost {
    fn header() -> Div {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child("katana-chat-ui 検証 - GPUI host")
            .child("ChatRenderModel を GPUI の host adapter がどう表示するかを確認する画面です。")
            .child("確認対象: 会話、入力欄、生成ファイル候補、差分候補、host action intent")
    }

    fn panel(title: &'static str, rows: Vec<String>) -> Div {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .p_3()
            .rounded_md()
            .border_1()
            .border_color(rgb(0x4b4f56))
            .bg(rgb(0x2b2d31))
            .child(div().text_color(rgb(0xffffff)).child(title))
            .children(rows.into_iter().map(Self::row))
    }

    fn row(text: String) -> Div {
        div()
            .p_2()
            .rounded_md()
            .bg(rgb(0x1f2024))
            .text_color(rgb(0xd7dbe0))
            .child(text)
    }

    fn message_rows(&self) -> Vec<String> {
        self.model.messages.iter().map(Self::message_row).collect()
    }

    fn message_row(message: &MessageRenderModel) -> String {
        format!(
            "{:?} / {:?}: {} blocks, {} attachments",
            message.role,
            message.status,
            message.blocks.len(),
            message.attachments.len()
        )
    }

    fn output_rows(&self) -> Vec<String> {
        self.model.outputs.iter().map(Self::output_row).collect()
    }

    fn output_row(output: &OutputRenderModel) -> String {
        format!(
            "output #{} from message #{} / actions: {}",
            output.id,
            output.source_message_id,
            output.actions.len()
        )
    }

    fn composer_rows(&self) -> Vec<String> {
        vec![
            "ここに host 側の入力欄を置く想定です。".to_string(),
            "GPUI MVP では送信ボタン操作で host action を模擬します。".to_string(),
            "本実装では GPUI text input を次 task で深掘りします。".to_string(),
        ]
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let host = match ManualGpuiHost::new() {
            Ok(host) => host,
            Err(error) => {
                eprintln!("manual-host-gpui failed to build model: {error}");
                return;
            }
        };
        let bounds = Bounds::centered(None, size(px(720.0), px(520.0)), cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            ..Default::default()
        };
        let result = cx.open_window(options, |_, cx| cx.new(|_| host));
        if let Err(error) = result {
            eprintln!("manual-host-gpui failed to start: {error}");
            return;
        }
        cx.activate(true);
    });
}
