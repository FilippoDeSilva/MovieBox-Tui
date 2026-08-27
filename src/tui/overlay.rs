use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
};

use crate::tui::theme::Theme;

const MAX_PICKER_ROWS_CAP: usize = 14;

pub(crate) fn max_picker_rows(area: Rect) -> usize {
    (area.height.saturating_sub(6) as usize / 2).clamp(4, MAX_PICKER_ROWS_CAP)
}

pub use crate::models::{Notification, NotificationKind};

pub struct PickerSpec<'a> {
    pub title: &'a str,
    pub confirm_label: &'a str,
    pub minimum_width: u16,
}

pub fn picker_layout(
    area: Rect,
    items: &[String],
    confirm_label: &str,
    minimum_width: u16,
) -> Rect {
    let visible_rows = items.len().clamp(1, max_picker_rows(area));
    let footer_str = format!("[↑↓] Move  [Enter] {confirm_label}  [Esc] Back");
    let footer_width = crate::tui::text::width(&footer_str);
    let content_width = items
        .iter()
        .map(|item| crate::tui::text::width(item))
        .max()
        .unwrap_or(0)
        .max(footer_width)
        .saturating_add(4);
    centered(
        area,
        content_width as u16,
        visible_rows as u16 + 4,
        minimum_width,
        64,
    )
}

pub fn tv_config_layout(
    area: Rect,
    longest_source_width: usize,
    total_rows: usize,
    input_active: bool,
) -> Rect {
    let content_width = longest_source_width.max(48).max(crate::tui::text::width(
        "[ Add URL ] [ Add file ] [ Reload ] [ Done ]",
    ));
    let popup_width = 68u16
        .max(content_width.saturating_add(6) as u16)
        .min(area.width.saturating_sub(4));
    let popup_height = if input_active {
        7u16
    } else {
        total_rows.min(10).saturating_add(6) as u16
    };
    centered(area, popup_width, popup_height, 36, 74)
}

pub fn addon_manager_layout(area: Rect, addons_count: usize, input_active: bool) -> Rect {
    let popup_width = 76u16.min(area.width.saturating_sub(4)).max(56);
    let popup_height = if input_active {
        7u16
    } else {
        (addons_count as u16)
            .saturating_add(6)
            .min(area.height.saturating_sub(4))
            .max(7)
    };
    centered(area, popup_width, popup_height, 36, 80)
}

pub fn download_confirm_layout(
    area: Rect,
    summary_lines: usize,
    longest_line_width: usize,
) -> Rect {
    let content_width = longest_line_width.max(36);
    centered(
        area,
        content_width.saturating_add(4) as u16,
        summary_lines as u16 + 4,
        36,
        64,
    )
}

pub fn download_confirm_action_row(popup: Rect, summary_lines: usize) -> u16 {
    popup.y + summary_lines as u16 + 1
}

pub fn picker(
    frame: &mut Frame,
    area: Rect,
    items: &[String],
    state: &mut ListState,
    spec: PickerSpec<'_>,
    theme: &Theme,
    basic_terminal: bool,
) {
    let selected = state
        .selected()
        .unwrap_or(0)
        .min(items.len().saturating_sub(1));
    let visible_rows = items.len().clamp(1, max_picker_rows(area));
    let popup = picker_layout(area, items, spec.confirm_label, spec.minimum_width);
    let title = format!(
        "{} · {}/{}",
        spec.title,
        selected.saturating_add(1),
        items.len().max(1)
    );
    let inner = crate::tui::widgets::ModalFrame::new(&title, theme, basic_terminal)
        .render(frame, popup, area);

    let sections = Layout::vertical([Constraint::Min(1), Constraint::Length(2)]).split(inner);
    let max_item_w = sections[0]
        .width
        .saturating_sub(if items.len() > visible_rows { 3 } else { 1 })
        as usize;
    let list_items = items
        .iter()
        .map(|item| {
            let truncated = crate::tui::text::truncate_width(item, max_item_w);
            ListItem::new(truncated).style(theme.text)
        })
        .collect::<Vec<_>>();
    let list = List::new(list_items)
        .highlight_style(selection_style(theme, basic_terminal))
        .highlight_symbol(if basic_terminal { "> " } else { "▌ " });
    frame.render_stateful_widget(list, sections[0], state);

    if items.len() > visible_rows {
        crate::tui::widgets::render_scrollbar(
            frame,
            sections[0],
            items.len(),
            visible_rows,
            selected,
            theme,
            basic_terminal,
        );
    }

    let confirm_label = if sections[1].width < 40 && spec.confirm_label == "Download" {
        "Save"
    } else {
        spec.confirm_label
    };
    let footer = vec![
        key_hint("↑↓", "Move", theme),
        Span::raw("  "),
        key_hint("Enter", confirm_label, theme),
        Span::raw("  "),
        key_hint("Esc", "Back", theme),
    ];
    crate::tui::widgets::render_modal_footer(frame, sections[1], footer, theme);
}

pub fn confirmation(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    summary: &[Line<'_>],
    confirm_selected: bool,
    theme: &Theme,
    basic_terminal: bool,
) {
    let content_width = summary.iter().map(Line::width).max().unwrap_or(0).max(36);
    let popup = centered(
        area,
        content_width.saturating_add(4) as u16,
        summary.len() as u16 + 4,
        36,
        64,
    );
    let inner = crate::tui::widgets::ModalFrame::new(title, theme, basic_terminal)
        .render(frame, popup, area);
    let sections = Layout::vertical([
        Constraint::Length(summary.len() as u16),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(inner);
    frame.render_widget(
        Paragraph::new(summary.to_vec()).alignment(Alignment::Center),
        sections[0],
    );

    let selected_style = selection_style(theme, basic_terminal);
    let actions = Line::from(vec![
        Span::styled(
            " Download ",
            if confirm_selected {
                selected_style
            } else {
                theme.text_dim
            },
        ),
        Span::raw("    "),
        Span::styled(
            " Cancel ",
            if confirm_selected {
                theme.text_dim
            } else {
                selected_style
            },
        ),
    ]);
    frame.render_widget(
        Paragraph::new(actions).alignment(Alignment::Center),
        sections[1],
    );
    let footer = if sections[2].width < 44 {
        Line::from(vec![
            key_hint("←→", "Choose", theme),
            Span::raw(" "),
            key_hint("Enter", "OK", theme),
            Span::raw(" "),
            key_hint("Esc", "Back", theme),
        ])
    } else {
        Line::from(vec![
            key_hint("←→", "Choose", theme),
            Span::raw("  "),
            key_hint("Enter", "Confirm", theme),
            Span::raw("  "),
            key_hint("Esc", "Back", theme),
        ])
    };
    frame.render_widget(
        Paragraph::new(footer).alignment(Alignment::Center),
        sections[2],
    );
}

pub fn notifications(
    frame: &mut Frame,
    area: Rect,
    notifications: &std::collections::VecDeque<Notification>,
    theme: &Theme,
    basic_terminal: bool,
    download_active: bool,
) {
    let bottom_offset = if download_active { 5 } else { 2 };
    let mut y = area.bottom().saturating_sub(bottom_offset);

    for notification in notifications.iter().rev().take(3) {
        let (badge, badge_style) = notification_style(notification.kind, theme, basic_terminal);
        let has_message =
            !notification.message.is_empty() && notification.message != notification.title;

        let max_card_width = (area.width.saturating_sub(4) as usize).min(72);
        let title_w = crate::tui::text::width(&notification.title).saturating_add(6);
        let badge_w = badge.len().saturating_add(6);
        let raw_msg_w = if has_message {
            crate::tui::text::width(&notification.message).saturating_add(6)
        } else {
            0
        };

        let target_card_width = title_w
            .max(badge_w)
            .max(raw_msg_w)
            .clamp(20, max_card_width.max(20)) as u16;

        let inner_width = (target_card_width.saturating_sub(4) as usize).max(1);

        let msg_lines: Vec<String> = if has_message {
            crate::tui::text::wrap_text(&notification.message, inner_width)
                .into_iter()
                .take(4)
                .collect()
        } else {
            Vec::new()
        };

        let height = 2 + 1 + msg_lines.len() as u16;

        if target_card_width < 10 || y < area.y.saturating_add(height) {
            break;
        }

        y = y.saturating_sub(height);

        let toast_area = Rect::new(
            area.right()
                .saturating_sub(target_card_width)
                .saturating_sub(2),
            y,
            target_card_width,
            height,
        );

        crate::tui::clear_area(frame, toast_area, theme);

        let mut lines = Vec::new();
        lines.push(Line::from(vec![Span::styled(
            crate::tui::text::truncate_width(&notification.title, inner_width),
            theme.text.add_modifier(Modifier::BOLD),
        )]));

        for line in &msg_lines {
            lines.push(Line::from(vec![Span::styled(
                crate::tui::text::truncate_width(line, inner_width),
                theme.subtext1,
            )]));
        }

        let total_duration = match notification.kind {
            NotificationKind::Info => std::time::Duration::from_secs(4),
            NotificationKind::Success => std::time::Duration::from_secs(5),
            NotificationKind::Warning => std::time::Duration::from_secs(7),
            NotificationKind::Error => std::time::Duration::from_secs(10),
        };
        let remaining = notification
            .expires_at
            .saturating_duration_since(std::time::Instant::now());
        let ratio = (remaining.as_secs_f64() / total_duration.as_secs_f64()).clamp(0.0, 1.0);
        let bar_width = inner_width.clamp(3, 16);
        let filled = ((bar_width as f64) * ratio).round() as usize;
        let countdown_bar = if basic_terminal {
            format!(
                "[{}{}]",
                "=".repeat(filled),
                "-".repeat(bar_width.saturating_sub(filled))
            )
        } else {
            format!(
                "{}{}",
                "━".repeat(filled),
                "─".repeat(bar_width.saturating_sub(filled))
            )
        };

        let block = Block::default()
            .title(Line::from(vec![Span::styled(
                format!(" {badge} "),
                badge_style.add_modifier(Modifier::BOLD),
            )]))
            .title_bottom(
                Line::from(vec![Span::styled(
                    format!(" {countdown_bar} "),
                    badge_style.add_modifier(Modifier::DIM),
                )])
                .alignment(Alignment::Right),
            )
            .borders(Borders::ALL)
            .border_type(border_type(basic_terminal))
            .border_style(badge_style)
            .padding(ratatui::widgets::Padding::horizontal(1));

        frame.render_widget(Paragraph::new(lines).block(block), toast_area);

        y = y.saturating_sub(1);
    }
}

pub fn centered(
    area: Rect,
    desired_width: u16,
    desired_height: u16,
    minimum_width: u16,
    maximum_width: u16,
) -> Rect {
    let available_width = area.width.saturating_sub(2).max(1);
    let available_height = area.height.saturating_sub(2).max(1);
    let width = desired_width
        .max(minimum_width.min(available_width))
        .min(maximum_width)
        .min(available_width);
    let height = desired_height.min(available_height).max(1);
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    )
}

pub fn clear_modal_area(frame: &mut Frame, _bounds: Rect, popup: Rect, theme: &Theme) {
    crate::tui::clear_area(frame, popup, theme);
}

pub fn border_type(basic_terminal: bool) -> BorderType {
    if basic_terminal {
        BorderType::Plain
    } else {
        BorderType::Rounded
    }
}

pub(crate) fn key_hint(key: &str, action: &str, theme: &Theme) -> Span<'static> {
    Span::styled(format!("[{key}] {action}"), theme.text_dim)
}

pub(crate) fn selection_style(theme: &Theme, basic_terminal: bool) -> Style {
    let style = theme.text.add_modifier(Modifier::BOLD);
    if basic_terminal {
        style.add_modifier(Modifier::UNDERLINED)
    } else {
        style.bg(theme.surface0.fg.unwrap_or(theme.base))
    }
}

fn notification_style(
    kind: NotificationKind,
    theme: &Theme,
    basic_terminal: bool,
) -> (&'static str, Style) {
    match kind {
        NotificationKind::Info => (
            if basic_terminal { "i INFO" } else { "ℹ INFO" },
            theme.sapphire,
        ),
        NotificationKind::Success => (
            if basic_terminal {
                "+ SUCCESS"
            } else {
                "✔ SUCCESS"
            },
            theme.success,
        ),
        NotificationKind::Warning => (
            if basic_terminal {
                "! WARNING"
            } else {
                "⚠ WARNING"
            },
            theme.rating,
        ),
        NotificationKind::Error => (
            if basic_terminal {
                "x ERROR"
            } else {
                "✖ ERROR"
            },
            theme.error,
        ),
    }
}

pub fn notification_rects(
    area: Rect,
    notifications: &std::collections::VecDeque<Notification>,
    basic_terminal: bool,
    download_active: bool,
) -> Vec<(usize, Rect)> {
    let mut rects = Vec::new();
    let bottom_offset = if download_active { 5 } else { 2 };
    let mut y = area.bottom().saturating_sub(bottom_offset);
    let theme_placeholder = Theme::default();

    for (rev_idx, notification) in notifications.iter().rev().take(3).enumerate() {
        let (badge, _) = notification_style(notification.kind, &theme_placeholder, basic_terminal);
        let has_message =
            !notification.message.is_empty() && notification.message != notification.title;

        let max_card_width = (area.width.saturating_sub(4) as usize).min(72);
        let title_w = crate::tui::text::width(&notification.title).saturating_add(6);
        let badge_w = badge.len().saturating_add(6);
        let raw_msg_w = if has_message {
            crate::tui::text::width(&notification.message).saturating_add(6)
        } else {
            0
        };

        let target_card_width = title_w
            .max(badge_w)
            .max(raw_msg_w)
            .clamp(20, max_card_width.max(20)) as u16;

        let inner_width = (target_card_width.saturating_sub(4) as usize).max(1);

        let msg_lines: Vec<String> = if has_message {
            crate::tui::text::wrap_text(&notification.message, inner_width)
                .into_iter()
                .take(4)
                .collect()
        } else {
            Vec::new()
        };

        let height = 2 + 1 + msg_lines.len() as u16;

        if target_card_width < 10 || y < area.y.saturating_add(height) {
            break;
        }

        y = y.saturating_sub(height);

        let toast_area = Rect::new(
            area.right()
                .saturating_sub(target_card_width)
                .saturating_sub(2),
            y,
            target_card_width,
            height,
        );

        let original_idx = notifications.len().saturating_sub(1 + rev_idx);
        rects.push((original_idx, toast_area));

        y = y.saturating_sub(1);
    }
    rects
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpdateModalLayout {
    pub popup_area: Rect,
    pub display_count: usize,
    pub has_more: bool,
    pub button_row_y: u16,
    pub update_btn_end_x: u16,
    pub open_btn_end_x: u16,
    pub open_button_midpoint_x: u16,
}

pub fn update_modal_layout(area: Rect, notes: &str) -> UpdateModalLayout {
    let note_lines_count = notes
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .count();

    let min_w: u16 = 58;
    let max_w: u16 = 72;
    let desired_w = max_w
        .min(area.width.saturating_sub(4))
        .max(min_w.min(area.width));

    let header_rows: u16 = 5;
    let footer_rows: u16 = 3;
    let available_height = area.height.saturating_sub(4);
    let available_note_rows =
        (available_height.saturating_sub(header_rows + footer_rows) as usize).clamp(2, 10);

    let display_count = note_lines_count.min(available_note_rows);
    let has_more = note_lines_count > display_count;
    let total_rows =
        header_rows + (display_count as u16) + (if has_more { 1 } else { 0 }) + footer_rows;
    let desired_h = total_rows.clamp(10, available_height.max(10));

    const UPDATE_SEGMENT: u16 = 18;
    const OPEN_SEGMENT: u16 = 26;
    const DISMISS_SEGMENT: u16 = 12;
    let footer_width = UPDATE_SEGMENT + OPEN_SEGMENT + DISMISS_SEGMENT;

    let popup_area = centered(area, desired_w, desired_h, min_w.min(area.width), max_w);
    let button_row_y = popup_area.y + popup_area.height.saturating_sub(2);
    let inner_width = popup_area.width.saturating_sub(2);
    let footer_start = popup_area.x + 1 + inner_width.saturating_sub(footer_width) / 2;
    let update_btn_end_x = footer_start + UPDATE_SEGMENT;
    let open_btn_end_x = update_btn_end_x + OPEN_SEGMENT;
    let open_button_midpoint_x = update_btn_end_x + OPEN_SEGMENT / 2;

    UpdateModalLayout {
        popup_area,
        display_count,
        has_more,
        button_row_y,
        update_btn_end_x,
        open_btn_end_x,
        open_button_midpoint_x,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_modal_mouse_hitbox_matches_rendered_geometry() {
        let area = Rect::new(0, 0, 80, 24);
        let notes = "Line 1\nLine 2\nLine 3\nLine 4";
        let layout = update_modal_layout(area, notes);

        assert_eq!(layout.popup_area.width, 72);
        assert_eq!(layout.display_count, 4);
        assert!(!layout.has_more);
        assert_eq!(layout.popup_area.height, 12);
        assert_eq!(layout.popup_area.x, (80 - 72) / 2);
        assert_eq!(layout.popup_area.y, (24 - 12) / 2);
        assert_eq!(layout.button_row_y, layout.popup_area.y + 10);
        let footer_start = layout.popup_area.x + 1 + (70 - 56) / 2;
        assert_eq!(layout.update_btn_end_x, footer_start + 18);
        assert_eq!(layout.open_btn_end_x, layout.update_btn_end_x + 26);
        assert_eq!(layout.open_button_midpoint_x, layout.update_btn_end_x + 13);
    }

    #[test]
    fn test_update_modal_zones_cover_visible_labels_only() {
        let area = Rect::new(0, 0, 80, 24);
        let layout = update_modal_layout(area, "notes");
        let row = layout.button_row_y;

        assert!(layout.popup_area.contains(ratatui::layout::Position::new(
            layout.update_btn_end_x - 2,
            row
        )));
        assert!(layout.popup_area.contains(ratatui::layout::Position::new(
            layout.open_btn_end_x - 2,
            row
        )));

        let gap_before_open = layout.update_btn_end_x;
        assert!(gap_before_open < layout.open_btn_end_x);
        let dismiss_center = layout.open_btn_end_x + 6;
        assert!(dismiss_center > layout.open_btn_end_x);
    }

    #[test]
    fn test_update_modal_open_release_click() {
        let area = Rect::new(0, 0, 80, 24);
        let notes = "### Highlights\n- Feature A\n- Feature B";
        let layout = update_modal_layout(area, notes);

        let click_x = layout.popup_area.x + 5;
        let click_y = layout.button_row_y;

        assert!(
            layout
                .popup_area
                .contains(ratatui::layout::Position::new(click_x, click_y))
        );
        assert_eq!(click_y, layout.button_row_y);
        assert!(click_x < layout.open_button_midpoint_x);
    }

    #[test]
    fn test_update_modal_dismiss_click() {
        let area = Rect::new(0, 0, 80, 24);
        let notes = "Feature A";
        let layout = update_modal_layout(area, notes);

        let dismiss_x = layout.open_btn_end_x + 6;
        let dismiss_y = layout.button_row_y;

        assert!(
            layout
                .popup_area
                .contains(ratatui::layout::Position::new(dismiss_x, dismiss_y))
        );
        assert_eq!(dismiss_y, layout.button_row_y);
        assert!(dismiss_x >= layout.open_button_midpoint_x);

        let outside_x = layout.popup_area.x.saturating_sub(2);
        let outside_y = layout.popup_area.y.saturating_sub(2);
        assert!(
            !layout
                .popup_area
                .contains(ratatui::layout::Position::new(outside_x, outside_y))
        );
    }

    #[test]
    fn test_update_modal_geometry_bounds_various_screens() {
        let notes = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9\nLine 10\nLine 11";

        let compact = update_modal_layout(Rect::new(0, 0, 40, 15), notes);
        assert!(compact.popup_area.width <= 38);
        assert!(compact.popup_area.height <= 13);
        assert!(compact.has_more);

        let large = update_modal_layout(Rect::new(0, 0, 160, 50), notes);
        assert_eq!(large.popup_area.width, 72);
        assert_eq!(large.display_count, 10);
        assert!(large.has_more);
    }

    #[test]
    fn test_download_confirm_action_row_matches_rendered_button_section() {
        let popup = Rect::new(10, 10, 40, 10);
        let summary_lines = 3;
        let action_row = download_confirm_action_row(popup, summary_lines);

        // Borders = 1 (top border at popup.y), Summary = 3 lines (popup.y + 1 .. popup.y + 4)
        // Action row = popup.y + 1 + 3 = popup.y + 4
        assert_eq!(action_row, popup.y + 4);
        assert!(popup.contains(ratatui::layout::Position::new(popup.x + 2, action_row)));
    }

    #[test]
    fn test_download_confirm_zones_do_not_overlap() {
        let area = Rect::new(0, 0, 80, 24);
        let summary_lines = 4;
        let longest = 30;
        let popup = download_confirm_layout(area, summary_lines, longest);
        let action_row = download_confirm_action_row(popup, summary_lines);

        assert!(popup.contains(ratatui::layout::Position::new(popup.x + 1, action_row)));
        assert!(action_row < popup.bottom() - 1); // Action row is inside inner area before footer/border
    }
}
