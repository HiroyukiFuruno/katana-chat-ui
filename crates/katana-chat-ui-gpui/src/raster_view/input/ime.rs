use std::ops::Range;

use gpui::{Bounds, Context, EntityInputHandler, Pixels, UTF16Selection, Window};
use katana_chat_ui::kuc::{ChatUiRuntimeCompositionPhase, ChatUiRuntimeInput};

use super::{CompositionState, GpuiKucView, utf16};

impl EntityInputHandler for GpuiKucView {
    fn text_for_range(
        &mut self,
        range: Range<usize>,
        _: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        if let Some(state) = self.composition.as_ref() {
            return state
                .document_text(&self.draft)
                .and_then(|text| utf16::slice(&text, range));
        }
        utf16::slice(&self.draft, range)
    }

    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self
                .composition
                .as_ref()
                .map_or_else(|| self.selection.clone(), CompositionState::selected_range),
            reversed: false,
        })
    }

    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.composition
            .as_ref()
            .map(CompositionState::marked_range)
    }

    fn unmark_text(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.composition = None;
        self.dispatch_input(ChatUiRuntimeInput::ImeCancel, window, cx);
    }

    fn replace_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = if let Some(state) = self.composition.as_ref() {
            state.commit_range(&self.draft, range)
        } else {
            Some(range.unwrap_or_else(|| self.selection.clone()))
        };
        let Some(range) = range.and_then(|range| utf16::checked_byte_range(&self.draft, range))
        else {
            return;
        };
        self.composition = None;
        self.dispatch_input(
            ChatUiRuntimeInput::ImeCommitRange {
                start_byte: range.start,
                end_byte: range.end,
                text: text.to_string(),
            },
            window,
            cx,
        );
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        selected: Option<Range<usize>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if text.is_empty() {
            // AppKit は cancel を空の marked text として通知することがある。
            // 0..0 の marked range を残すと NSTextInputClient が marked 状態を報告し続ける。
            self.composition = None;
            self.dispatch_input(ChatUiRuntimeInput::ImeCancel, window, cx);
            return;
        }
        let Some(state) = CompositionState::new(
            &self.draft,
            range,
            self.composition.as_ref(),
            self.selection.clone(),
            text,
            selected.clone(),
        ) else {
            return;
        };
        self.composition = Some(state);
        self.dispatch_input(
            ChatUiRuntimeInput::ImeComposition {
                phase: ChatUiRuntimeCompositionPhase::Update,
                preedit: text.to_string(),
                caret: utf16::composition_caret_byte(text, selected),
            },
            window,
            cx,
        );
    }

    fn bounds_for_range(
        &mut self,
        range: Range<usize>,
        _: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let frame = self.frame.as_ref()?;
        let bounds = self.paint_bounds?;
        let range = utf16::byte_range(&self.draft, range);
        let rect = frame
            .composer
            .layout
            .bounds_for_byte_range(range.start, range.end)?;
        Some(Bounds::from_corners(
            gpui::point(
                bounds.left() + gpui::px(rect.x as f32),
                bounds.top() + gpui::px(rect.y as f32),
            ),
            gpui::point(
                bounds.left() + gpui::px((rect.x + rect.width as i32) as f32),
                bounds.top() + gpui::px((rect.y + rect.height as i32) as f32),
            ),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: gpui::Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        let frame = self.frame.as_ref()?;
        let point = self.local_point(point)?;
        let local_x = f32::from(point.x).round() as i32;
        let local_y = f32::from(point.y).round() as i32;
        let byte = frame
            .composer
            .layout
            .hit_test(local_x, local_y)
            .caret_position();
        Some(self.draft.get(..byte)?.encode_utf16().count())
    }
}
