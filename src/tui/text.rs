use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

pub fn width(value: &str) -> usize {
    UnicodeWidthStr::width(value)
}

pub fn grapheme_count(value: &str) -> usize {
    value.graphemes(true).count()
}

pub fn take_graphemes(value: &str, count: usize) -> String {
    value.graphemes(true).take(count).collect()
}

pub fn remove_last_grapheme(value: &mut String) {
    if let Some((index, _)) = value.grapheme_indices(true).next_back() {
        value.truncate(index);
    }
}

pub fn truncate_width(value: &str, max_width: usize) -> String {
    if width(value) <= max_width {
        return value.to_string();
    }
    if max_width <= 3 {
        return ".".repeat(max_width);
    }

    let content_width = max_width - 3;
    let mut output = String::new();
    let mut used = 0;
    for grapheme in value.graphemes(true) {
        let grapheme_width = width(grapheme);
        if used + grapheme_width > content_width {
            break;
        }
        output.push_str(grapheme);
        used += grapheme_width;
    }
    output.push_str("...");
    output
}
