use std::ops::Range;

pub(super) fn slice(value: &str, range: Range<usize>) -> Option<String> {
    let units = value.encode_utf16().collect::<Vec<_>>();
    String::from_utf16(units.get(range)?).ok()
}

pub(super) fn clamp(value: &str, range: Range<usize>) -> Range<usize> {
    let length = value.encode_utf16().count();
    range.start.min(length)..range.end.min(length)
}

pub(super) fn byte_range(value: &str, range: Range<usize>) -> Range<usize> {
    byte(value, range.start)..byte(value, range.end)
}

pub(super) fn checked_byte_range(value: &str, range: Range<usize>) -> Option<Range<usize>> {
    if range.start > range.end || range.end > value.encode_utf16().count() {
        return None;
    }
    Some(byte_range(value, range))
}

pub(super) fn composition_caret_byte(value: &str, selected: Option<Range<usize>>) -> usize {
    let Some(selected) = selected else {
        return value.len();
    };
    checked_byte_range(value, selected).map_or(value.len(), |range| range.end)
}

fn byte(value: &str, target: usize) -> usize {
    let mut units = 0;
    for (offset, character) in value.char_indices() {
        if units >= target {
            return offset;
        }
        units += character.len_utf16();
    }
    value.len()
}
