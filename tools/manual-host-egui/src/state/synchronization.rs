use katana_chat_ui::ChatUiSurface;

pub(super) fn preserve_pending_draft(
    incoming: &mut ChatUiSurface,
    acknowledged: u64,
    latest_draft: u64,
    current_text: &str,
) {
    if acknowledged < latest_draft {
        incoming.composer.text = current_text.to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::preserve_pending_draft;
    use katana_chat_ui::{ChatSession, ChatUiSurface};

    #[test]
    fn stale_snapshot_keeps_local_japanese_draft() {
        let mut incoming = surface("old draft");
        preserve_pending_draft(&mut incoming, 3, 4, "入力中の文章");
        assert_eq!(incoming.composer.text, "入力中の文章");
    }

    #[test]
    fn acknowledged_snapshot_uses_server_draft() {
        let mut incoming = surface("");
        preserve_pending_draft(&mut incoming, 5, 4, "送信した文章");
        assert!(incoming.composer.text.is_empty());
    }

    fn surface(text: &str) -> ChatUiSurface {
        let mut session = ChatSession::new();
        session.draft_mut().set_text(text);
        ChatUiSurface::from_render_model(&session.render_model())
    }
}
