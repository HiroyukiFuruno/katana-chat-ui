use std::ops::Range;

use super::utf16;

pub(crate) struct CompositionState {
    replacement: Range<usize>,
    marked: Range<usize>,
    selected: Range<usize>,
    text: String,
}

impl CompositionState {
    pub(crate) fn new(
        draft: &str,
        requested: Option<Range<usize>>,
        previous: Option<&Self>,
        selection: Range<usize>,
        text: &str,
        selected: Option<Range<usize>>,
    ) -> Option<Self> {
        let replacement = Self::replacement_range(draft, requested, previous, selection)?;
        let marked = Self::virtual_marked_range(&replacement, text);
        let selected = Self::virtual_selected_range(text, selected, marked.start, marked.end);
        Some(Self {
            replacement,
            marked,
            selected,
            text: text.to_string(),
        })
    }

    pub(crate) fn document_text(&self, draft: &str) -> Option<String> {
        let range = utf16::checked_byte_range(draft, self.replacement.clone())?;
        Some(format!(
            "{}{}{}",
            &draft[..range.start],
            self.text,
            &draft[range.end..]
        ))
    }

    pub(crate) fn marked_range(&self) -> Range<usize> {
        self.marked.clone()
    }

    pub(crate) fn selected_range(&self) -> Range<usize> {
        self.selected.clone()
    }

    pub(crate) fn commit_range(
        &self,
        draft: &str,
        requested: Option<Range<usize>>,
    ) -> Option<Range<usize>> {
        requested.map_or_else(
            || Some(self.replacement.clone()),
            |range| self.base_range_for_virtual(draft, range),
        )
    }

    fn replacement_range(
        draft: &str,
        requested: Option<Range<usize>>,
        previous: Option<&Self>,
        selection: Range<usize>,
    ) -> Option<Range<usize>> {
        match (requested, previous) {
            (Some(range), Some(state)) => state.base_range_for_virtual(draft, range),
            (Some(range), None) => utf16::checked_byte_range(draft, range.clone()).map(|_| range),
            (None, Some(state)) => Some(state.replacement.clone()),
            (None, None) => utf16::checked_byte_range(draft, selection.clone()).map(|_| selection),
        }
    }

    fn base_range_for_virtual(&self, draft: &str, range: Range<usize>) -> Option<Range<usize>> {
        utf16::checked_byte_range(&self.document_text(draft)?, range.clone())?;
        let start = self.base_endpoint(range.start, true);
        let end = self.base_endpoint(range.end, false);
        (start <= end).then_some(start..end)
    }

    fn base_endpoint(&self, index: usize, is_start: bool) -> usize {
        if index <= self.marked.start {
            return index;
        }
        if index >= self.marked.end {
            return self.shift_after_mark(index);
        }
        if is_start {
            self.replacement.start
        } else {
            self.replacement.end
        }
    }

    fn shift_after_mark(&self, index: usize) -> usize {
        let marked_len = self.marked.end.saturating_sub(self.marked.start);
        let replacement_len = self.replacement.end.saturating_sub(self.replacement.start);
        if replacement_len >= marked_len {
            index.saturating_add(replacement_len.saturating_sub(marked_len))
        } else {
            index.saturating_sub(marked_len.saturating_sub(replacement_len))
        }
    }

    fn virtual_marked_range(replacement: &Range<usize>, text: &str) -> Range<usize> {
        let end = replacement
            .start
            .saturating_add(text.encode_utf16().count());
        replacement.start..end
    }

    fn virtual_selected_range(
        text: &str,
        selected: Option<Range<usize>>,
        start: usize,
        end: usize,
    ) -> Range<usize> {
        let length = text.encode_utf16().count();
        let range = selected.unwrap_or(length..length);
        let range = utf16::clamp(text, range);
        start.saturating_add(range.start)..start.saturating_add(range.end).min(end)
    }
}
