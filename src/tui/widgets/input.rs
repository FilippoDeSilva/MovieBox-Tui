use ratatui::{
    Frame,
    layout::Rect,
    style::Modifier,
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};

use crate::tui::{text::TextInputBuffer, theme::Theme};

pub fn render_single_line_input(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    buffer: &TextInputBuffer,
    theme: &Theme,
    basic_terminal: bool,
) {
    let segments = buffer.graphemes();
    let cursor = buffer.cursor();
    let max_width = (area.width as usize).saturating_sub(6);

    let mut start = 0;
    if cursor >= max_width {
        start = cursor - max_width + 1;
    }

    let mut before_cursor: String = segments[start..cursor].concat();
    if start > 0 && before_cursor.chars().count() > 3 {
        before_cursor = format!("...{}", &before_cursor[3..]);
    }

    let cursor_char = if cursor < segments.len() {
        segments[cursor].to_string()
    } else {
        " ".to_string()
    };

    let end = (start + max_width).min(segments.len());
    let after_slice = &segments[cursor.saturating_add(1).min(segments.len())..end];
    let mut after_cursor: String = after_slice.concat();
    if end < segments.len() {
        let len = after_cursor.chars().count();
        if len > 3 {
            let keep: String = after_cursor.chars().take(len - 3).collect();
            after_cursor = format!("{keep}...");
        } else if !after_cursor.is_empty() {
            after_cursor = "...".to_string();
        }
    }

    let prompt_symbol = if basic_terminal { " > " } else { " ❯ " };
    let lines = vec![
        Line::from(vec![Span::raw(" "), Span::styled(label, theme.sapphire)]),
        Line::from(vec![
            Span::styled(prompt_symbol, theme.sapphire),
            Span::styled(before_cursor, theme.text),
            Span::styled(cursor_char, theme.text.add_modifier(Modifier::REVERSED)),
            Span::styled(after_cursor, theme.text),
        ]),
    ];

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn test_render_single_line_input_empty() {
        let backend = TestBackend::new(80, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        let buffer = TextInputBuffer::new();
        let theme = Theme::default();

        terminal
            .draw(|f| {
                render_single_line_input(
                    f,
                    Rect::new(0, 0, 80, 3),
                    "Enter URL:",
                    &buffer,
                    &theme,
                    false,
                );
            })
            .unwrap();
    }

    #[test]
    fn test_render_single_line_input_with_content() {
        let backend = TestBackend::new(40, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut buffer = TextInputBuffer::from_str("https://example.com/playlist.m3u8");
        buffer.move_home();
        let theme = Theme::default();

        terminal
            .draw(|f| {
                render_single_line_input(
                    f,
                    Rect::new(0, 0, 40, 3),
                    "Enter URL:",
                    &buffer,
                    &theme,
                    true,
                );
            })
            .unwrap();
    }
}
