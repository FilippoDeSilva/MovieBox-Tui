use crate::tui::{state::AppState, theme::Theme};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, Padding, Paragraph},
};

const QUOTES: &[&str] = &[
    "\"Talk is cheap. Show me the code.\" — Linus Torvalds",
    "\"First, solve the problem. Then, write the code.\" — John Johnson",
    "\"Fix the cause, not the symptom.\" — Steve Maguire",
    "\"Simplicity is the soul of efficiency.\" — Austin Freeman",
    "\"Make it work, make it right, make it fast.\" — Kent Beck",
];

pub fn draw(frame: &mut Frame, area: Rect, state: &mut AppState, theme: &Theme) {
    let panel = centered_rect(68.min(area.width.saturating_sub(4)), 15, area);
    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(" MOVIEBOX ", theme.title),
            Span::styled("· STARTUP ", theme.text_dim),
        ]))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_focus)
        .padding(Padding::new(3, 3, 1, 1));
    let inner = block.inner(panel);
    frame.render_widget(block, panel);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

    let (active_step, status_text) = if state.updater_done {
        (2, "Update installed. Restarting MovieBox…".to_string())
    } else if state.updater_progress.is_some() {
        (
            1,
            state
                .updater_status
                .clone()
                .unwrap_or_else(|| "Downloading the latest release…".to_string()),
        )
    } else {
        (0, "Checking GitHub for a newer release…".to_string())
    };

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("01  CHECK", step_style(active_step, 0, theme)),
            Span::styled("  ─────  ", theme.muted),
            Span::styled("02  INSTALL", step_style(active_step, 1, theme)),
            Span::styled("  ─────  ", theme.muted),
            Span::styled("03  LAUNCH", step_style(active_step, 2, theme)),
        ]))
        .alignment(Alignment::Center),
        rows[0],
    );

    let spinner_frames = ['◐', '◓', '◑', '◒'];
    let spinner = spinner_frames[(state.tick_count as usize / 3) % spinner_frames.len()];
    let status_style = if state.updater_done {
        theme.success.add_modifier(Modifier::BOLD)
    } else {
        theme.text
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(format!("{spinner}  "), theme.accent),
            Span::styled(status_text, status_style),
        ]))
        .alignment(Alignment::Center),
        rows[2],
    );

    if let Some(progress) = state.updater_progress {
        let progress = progress.clamp(0.0, 1.0);
        let label = if state.updater_done {
            "READY".to_string()
        } else {
            format!("{:>3}%", (progress * 100.0).round() as u16)
        };
        frame.render_widget(
            Gauge::default()
                .gauge_style(if state.updater_done {
                    theme.success
                } else {
                    theme.accent
                })
                .label(Span::styled(label, theme.text.add_modifier(Modifier::BOLD)))
                .ratio(progress),
            rows[4],
        );
    } else {
        let pulse =
            ["●  ·  ·", "·  ●  ·", "·  ·  ●", "·  ●  ·"][(state.tick_count as usize / 4) % 4];
        frame.render_widget(
            Paragraph::new(Span::styled(pulse, theme.accent)).alignment(Alignment::Center),
            rows[4],
        );
    }

    let detail = if state.updater_progress.is_some() && !state.updater_done {
        QUOTES[(state.tick_count as usize / 300) % QUOTES.len()]
    } else if state.updater_done {
        "Your new version is ready."
    } else {
        "This only takes a moment. Your library stays untouched."
    };
    frame.render_widget(
        Paragraph::new(Span::styled(detail, theme.text_dim)).alignment(Alignment::Center),
        rows[6],
    );

    let footer = Rect {
        x: panel.x,
        y: panel.y.saturating_add(panel.height),
        width: panel.width,
        height: 1,
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("CURRENT  ", theme.muted),
            Span::styled(
                format!("v{}", env!("CARGO_PKG_VERSION")),
                theme.text_dim.add_modifier(Modifier::BOLD),
            ),
            Span::styled("   •   AUTOMATIC UPDATES ON", theme.muted),
        ]))
        .alignment(Alignment::Center),
        footer,
    );
}

fn step_style(active: usize, step: usize, theme: &Theme) -> ratatui::style::Style {
    if step < active {
        theme.success.add_modifier(Modifier::BOLD)
    } else if step == active {
        theme.accent.add_modifier(Modifier::BOLD)
    } else {
        theme.muted
    }
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height: height.min(area.height),
    }
}
