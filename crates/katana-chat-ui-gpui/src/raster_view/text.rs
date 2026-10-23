pub(super) fn byte_range_to_utf16(
    value: &str,
    range: std::ops::Range<usize>,
) -> std::ops::Range<usize> {
    byte_to_utf16(value, range.start)..byte_to_utf16(value, range.end)
}

fn byte_to_utf16(value: &str, byte: usize) -> usize {
    value
        .get(..byte)
        .map_or(0, |prefix| prefix.encode_utf16().count())
}
