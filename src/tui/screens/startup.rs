use crate::tui::{state::AppState, theme::Theme};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Paragraph},
};

const QUOTES: &[&str] = &[
    "\"Talk is cheap. Show me the code.\" - Linus Torvalds",
    "\"First, solve the problem. Then, write the code.\" - John Johnson",
    "\"Experience is the name everyone gives to their mistakes.\" - Oscar Wilde",
    "\"Code is like humor. When you have to explain it, it’s bad.\" - Cory House",
    "\"Fix the cause, not the symptom.\" - Steve Maguire",
    "\"Simplicity is the soul of efficiency.\" - Austin Freeman",
    "\"Before software can be reusable it first has to be usable.\" - Ralph Johnson",
    "\"Make it work, make it right, make it fast.\" - Kent Beck",
    "\"There is no elevator to success, you have to take the stairs.\" - Zig Ziglar",
];

pub fn draw(frame: &mut Frame, area: Rect, state: &mut AppState, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(35),
            Constraint::Length(8),
            Constraint::Percentage(35),
        ])
        .split(area);

    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
        ])
        .split(chunks[1]);

    let spinner_frames = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let spinner = spinner_frames[(state.tick_count as usize / 5) % spinner_frames.len()];

    let quote = QUOTES[(state.tick_count as usize / 300) % QUOTES.len()];

    let status_line = if state.updater_done {
        Line::from(vec![Span::styled(
            "Update complete! Restarting...",
            theme.accent,
        )])
    } else if let Some(prog) = state.updater_progress {
        let p = (prog * 100.0) as u16;
        let filled = (p / 5) as usize;
        let empty = 20 - filled;
        let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));
        Line::from(vec![
            Span::styled(format!("{} Downloading Update... ", spinner), theme.text),
            Span::styled(format!("[{}] {}%", bar, p), theme.rating),
        ])
    } else {
        Line::from(vec![Span::styled(
            format!("{} Checking for updates...", spinner),
            theme.text_dim,
        )])
    };

    let title = Paragraph::new(Line::from(vec![Span::styled(
        "MovieBox TUI",
        theme.header.add_modifier(ratatui::style::Modifier::BOLD),
    )]))
    .alignment(Alignment::Center);

    let status = Paragraph::new(status_line).alignment(Alignment::Center);

    let quote_para = Paragraph::new(Line::from(vec![Span::styled(quote, theme.text_dim)]))
        .alignment(Alignment::Center);

    frame.render_widget(title, inner_chunks[0]);
    frame.render_widget(status, inner_chunks[1]);
    if state.updater_progress.is_some() && !state.updater_done {
        frame.render_widget(quote_para, inner_chunks[2]);
    }
}
