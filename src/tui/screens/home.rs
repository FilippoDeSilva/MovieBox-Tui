use crate::tui::{
    state::{AppState, InputMode},
    theme::Theme,
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SearchViewState {
    Empty,
    Editing,
    Loading,
    Results,
    NoResults,
    Error,
}

fn search_view_state(state: &AppState) -> SearchViewState {
    if state.input_mode == InputMode::Editing {
        SearchViewState::Editing
    } else if state.is_loading
        && (!state.search_query.trim().is_empty()
            || state.active_browse_preset.is_some()
            || state.active_addon_catalog.is_some()
            || state.is_homepage_mode)
    {
        SearchViewState::Loading
    } else if state.search_error.is_some() {
        SearchViewState::Error
    } else if !state.search_results.is_empty() {
        SearchViewState::Results
    } else if !state.search_query.trim().is_empty()
        || state.active_browse_preset.is_some()
        || state.active_addon_catalog.is_some()
    {
        SearchViewState::NoResults
    } else {
        SearchViewState::Empty
    }
}

fn centered_width(area: Rect, maximum: u16) -> Rect {
    let width = area.width.min(maximum).max(1);
    Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        width,
        ..area
    }
}

pub(crate) fn slash_command_description(cmd: &str, state: &AppState) -> Option<&'static str> {
    crate::tui::commands::SlashCommand::description_for(cmd, state)
}

pub(crate) fn poster_placeholder_lines(basic: bool, is_loading: bool, tick: u64) -> &'static str {
    if basic {
        if is_loading {
            match (tick / 4) % 3 {
                0 => "[ . . . . . ]",
                1 => "[ o o o o o ]",
                _ => "[ O O O O O ]",
            }
        } else {
            "[ no poster ]"
        }
    } else if is_loading {
        match (tick / 4) % 3 {
            0 => "┌──────┐\n│ ·  · │\n│  ──  │\n│ ·  · │\n└──────┘",
            1 => "┌──────┐\n│ ◦  ◦ │\n│  ──  │\n│ ◦  ◦ │\n└──────┘",
            _ => "┌──────┐\n│ ○  ○ │\n│  ──  │\n│ ○  ○ │\n└──────┘",
        }
    } else {
        "┌──────┐\n│ ▓  ▓ │\n│  ──  │\n│ ▓  ▓ │\n└──────┘"
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HomeLayoutTier {
    Compact,
    Normal,
    Wide,
}

impl HomeLayoutTier {
    pub(crate) fn for_width(width: u16) -> Self {
        if width < 76 {
            Self::Compact
        } else if width < 110 {
            Self::Normal
        } else {
            Self::Wide
        }
    }

    pub(crate) fn is_compact(self) -> bool {
        self == Self::Compact
    }
}

pub(crate) struct LandingRows {
    pub rects: std::rc::Rc<[Rect]>,
    pub logo: usize,
    pub version: usize,
    pub search: usize,
    pub favorites: usize,
    pub mode_row: usize,
    pub util_row: Option<usize>,
    pub logo_width: u16,
}

pub(crate) fn landing_split(
    area: Rect,
    tv_mode: bool,
    basic_terminal: bool,
) -> (HomeLayoutTier, LandingRows) {
    let tier = HomeLayoutTier::for_width(area.width);
    let compact_logo = tier.is_compact() || (tv_mode && area.width < 80);
    let effective_basic = basic_terminal || compact_logo;
    let logo_height = if effective_basic { 2 } else { 6 };
    let logo_width = if effective_basic {
        if tv_mode { 33 } else { 31 }
    } else if tv_mode {
        75
    } else {
        73
    };
    if compact_logo {
        let rects = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(12),
                Constraint::Length(logo_height),
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(area);
        (
            tier,
            LandingRows {
                rects,
                logo: 1,
                version: 2,
                search: 4,
                favorites: 6,
                mode_row: 7,
                util_row: None,
                logo_width,
            },
        )
    } else {
        let rects = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(16),
                Constraint::Length(logo_height),
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(area);
        (
            tier,
            LandingRows {
                rects,
                logo: 1,
                version: 2,
                search: 4,
                favorites: 6,
                mode_row: 7,
                util_row: Some(8),
                logo_width,
            },
        )
    }
}

pub(crate) fn search_deck_width(area: Rect, _state: &AppState, landing: bool) -> u16 {
    let tier = HomeLayoutTier::for_width(area.width);
    if landing {
        let target_width = match tier {
            HomeLayoutTier::Compact => 54,
            HomeLayoutTier::Normal => 68,
            HomeLayoutTier::Wide => 80,
        };
        target_width.min(area.width.saturating_sub(4)).max(24)
    } else {
        let target_width = match tier {
            HomeLayoutTier::Compact => 64,
            HomeLayoutTier::Normal => 84,
            HomeLayoutTier::Wide => 104,
        };
        target_width.min(area.width.saturating_sub(4)).max(30)
    }
}

fn render_search_state(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    theme: &Theme,
    view: SearchViewState,
) {
    if area.height < 3 || area.width < 20 {
        return;
    }

    let mut card_width = area.width.min(64);
    let query = crate::tui::text::truncate_width(
        &state.search_query,
        card_width.saturating_sub(10) as usize,
    );

    let ctrl_p = crate::tui::text::ctrl_key("P");
    let ctrl_u = crate::tui::text::ctrl_key("U");
    let bullet = if state.basic_terminal {
        " - "
    } else {
        "  •  "
    };

    let mut lines: Vec<Line> = Vec::new();

    match view {
        SearchViewState::Loading => {
            let msg = if state.basic_terminal {
                let dots = match (state.tick_count / 4) % 3 {
                    0 => ".",
                    1 => "..",
                    _ => "...",
                };
                if let Some(preset) = state.active_browse_preset {
                    format!("Loading {}{dots}", preset.label())
                } else if let Some(catalog) = &state.active_addon_catalog {
                    format!("Loading {}{dots}", catalog.label)
                } else if state.is_homepage_mode {
                    format!("Loading discover{dots}")
                } else if !state.search_query.trim().is_empty() {
                    format!("Searching for “{query}”{dots}")
                } else {
                    format!("Loading{dots}")
                }
            } else {
                const SPINNER_FRAMES: [&str; 10] =
                    ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                let spinner =
                    SPINNER_FRAMES[(state.tick_count as usize / 2) % SPINNER_FRAMES.len()];
                if let Some(preset) = state.active_browse_preset {
                    format!("{spinner} Loading {}", preset.label())
                } else if let Some(catalog) = &state.active_addon_catalog {
                    format!("{spinner} Loading {}", catalog.label)
                } else if state.is_homepage_mode {
                    format!("{spinner} Loading discover")
                } else if !state.search_query.trim().is_empty() {
                    format!("{spinner} Searching for “{query}”")
                } else {
                    format!("{spinner} Loading")
                }
            };
            lines.push(Line::from(vec![Span::styled(msg, theme.lavender)]));
        }
        SearchViewState::NoResults => {
            let symbol = if state.basic_terminal { "-" } else { "·" };
            let msg = if let Some(preset) = state.active_browse_preset {
                format!("No items found for {}", preset.label())
            } else if let Some(catalog) = &state.active_addon_catalog {
                format!("No items found for {}", catalog.label)
            } else if state.search_query.trim().eq_ignore_ascii_case("/history") {
                "No watch history found".to_string()
            } else if !state.search_query.trim().is_empty() {
                format!("No matches for “{query}”")
            } else {
                "No results found".to_string()
            };
            lines.push(Line::from(vec![
                Span::styled(format!("{symbol} "), theme.text_dim),
                Span::styled(msg, theme.text),
            ]));
            lines.push(Line::from(""));
            if area.width < 76 {
                lines.push(Line::from(vec![
                    Span::styled("[", theme.text_dim),
                    Span::styled(&ctrl_p, theme.shortcut),
                    Span::styled("] ", theme.text_dim),
                    Span::styled("Switch provider", theme.text_dim),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("[", theme.text_dim),
                    Span::styled("/browse", theme.shortcut),
                    Span::styled("] ", theme.text_dim),
                    Span::styled("Browse categories", theme.text_dim),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("[", theme.text_dim),
                    Span::styled(&ctrl_u, theme.shortcut),
                    Span::styled("] ", theme.text_dim),
                    Span::styled("Clear", theme.text_dim),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::styled("[", theme.text_dim),
                    Span::styled(&ctrl_p, theme.shortcut),
                    Span::styled("] ", theme.text_dim),
                    Span::styled("Switch provider", theme.text_dim),
                    Span::styled(bullet, theme.text_dim),
                    Span::styled("[", theme.text_dim),
                    Span::styled("/browse", theme.shortcut),
                    Span::styled("] ", theme.text_dim),
                    Span::styled("Browse categories", theme.text_dim),
                    Span::styled(bullet, theme.text_dim),
                    Span::styled("[", theme.text_dim),
                    Span::styled(&ctrl_u, theme.shortcut),
                    Span::styled("] ", theme.text_dim),
                    Span::styled("Clear", theme.text_dim),
                ]));
            }
        }
        SearchViewState::Error => {
            let symbol = if state.basic_terminal { "!" } else { "×" };
            let err_text = state.search_error.as_deref().unwrap_or_else(|| {
                if !state.status_message.is_empty() {
                    &state.status_message
                } else {
                    "Search request failed"
                }
            });
            let wrap_width = card_width.saturating_sub(6).max(10) as usize;
            let wrapped_err_lines = crate::tui::text::wrap_text(err_text, wrap_width);
            if wrapped_err_lines.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled(format!("{symbol} "), theme.error),
                    Span::styled("Search request failed", theme.error),
                ]));
            } else {
                for (idx, eline) in wrapped_err_lines.into_iter().enumerate() {
                    if idx == 0 {
                        lines.push(Line::from(vec![
                            Span::styled(format!("{symbol} "), theme.error),
                            Span::styled(eline, theme.error),
                        ]));
                    } else {
                        lines.push(Line::from(vec![
                            Span::styled("  ", theme.error),
                            Span::styled(eline, theme.error),
                        ]));
                    }
                }
            }
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("[", theme.text_dim),
                Span::styled("r", theme.shortcut),
                Span::styled("] ", theme.text_dim),
                Span::styled("Retry request", theme.text_dim),
                Span::styled(bullet, theme.text_dim),
                Span::styled("[", theme.text_dim),
                Span::styled(&ctrl_p, theme.shortcut),
                Span::styled("] ", theme.text_dim),
                Span::styled("Switch provider", theme.text_dim),
                Span::styled(bullet, theme.text_dim),
                Span::styled("[", theme.text_dim),
                Span::styled("Esc", theme.shortcut),
                Span::styled("] ", theme.text_dim),
                Span::styled("Back", theme.text_dim),
            ]));
        }
        _ => return,
    };

    let max_line_width = lines.iter().map(|l| l.width()).max().unwrap_or(0) as u16;
    if matches!(view, SearchViewState::Error | SearchViewState::NoResults) {
        card_width = max_line_width.clamp(20, area.width.min(64));
    }

    let card_height = (lines.len() as u16).min(area.height);
    let card = Rect {
        x: area.x + area.width.saturating_sub(card_width) / 2,
        y: area.y + area.height.saturating_sub(card_height) / 2,
        width: card_width,
        height: card_height,
    };

    let alignment = if matches!(view, SearchViewState::Error | SearchViewState::NoResults) {
        Alignment::Left
    } else {
        Alignment::Center
    };

    frame.render_widget(Paragraph::new(lines).alignment(alignment), card);
}
fn render_favorites_landing(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    if area.height < 2 || area.width < 20 {
        return;
    }

    let items: Vec<crate::favorites::FavoriteItem> = state
        .favorites_landing_items()
        .into_iter()
        .cloned()
        .collect();
    if items.is_empty() {
        return;
    }

    let overflow = state.favorites.items.len().saturating_sub(items.len());
    let card_width = if HomeLayoutTier::for_width(area.width).is_compact() {
        area.width.clamp(20, 44)
    } else {
        area.width.clamp(20, 56)
    };
    let row_count = items.len() as u16;
    let overflow_row = u16::from(overflow > 0);
    let content_height = (1 + row_count + overflow_row).min(area.height);

    let card = Rect {
        x: area.x + area.width.saturating_sub(card_width) / 2,
        y: area.y,
        width: card_width,
        height: content_height,
    };

    if state.favorites_focus {
        let bg = theme.surface0.fg.unwrap_or(theme.base);
        frame.render_widget(Block::default().style(Style::default().bg(bg)), card);
    }

    let mut constraints = vec![Constraint::Length(1)];
    constraints.extend(std::iter::repeat_n(Constraint::Length(1), items.len()));
    if overflow > 0 {
        constraints.push(Constraint::Length(1));
    }
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(card);

    let header_style = if state.favorites_focus {
        theme.title
    } else {
        theme.subtext1
    };
    frame.render_widget(
        Paragraph::new("Favorites")
            .style(header_style)
            .alignment(Alignment::Center),
        sections[0],
    );

    let selected = if state.favorites_focus {
        state.favorites_landing_state.selected()
    } else {
        None
    };

    for (i, item) in items.iter().enumerate() {
        let Some(row_area) = sections.get(1 + i) else {
            break;
        };
        let is_selected = selected == Some(i);
        let type_tag = if item.stype == 2 { "Series" } else { "Movie" };
        let prefix = if is_selected {
            if state.basic_terminal { "> " } else { "▌ " }
        } else {
            "  "
        };
        let title_style = if is_selected {
            theme.title.add_modifier(Modifier::BOLD)
        } else {
            theme.text
        };
        let max_title_width = row_area.width.saturating_sub(14) as usize;
        let line = Line::from(vec![
            Span::styled(prefix, theme.accent),
            Span::styled(
                crate::tui::text::truncate_width(&item.title, max_title_width),
                title_style,
            ),
            Span::styled(format!("  {type_tag}"), theme.text_dim),
        ]);
        frame.render_widget(Paragraph::new(line), *row_area);
    }

    if overflow > 0 {
        if let Some(row_area) = sections.last() {
            let pill_text = format!("[ +{overflow} more · /favorites ]");
            let pill_style = if state.basic_terminal {
                theme.sapphire
            } else {
                let bg = theme.surface1.fg.unwrap_or(Color::Rgb(69, 71, 90));
                let fg = theme.sapphire.fg.unwrap_or(Color::Rgb(116, 199, 236));
                Style::default().bg(bg).fg(fg)
            };
            frame.render_widget(
                Paragraph::new(Line::from(vec![Span::styled(pill_text, pill_style)]))
                    .alignment(Alignment::Center),
                *row_area,
            );
        }
    }
}

#[cfg(test)]
fn search_content(
    state: &AppState,
    view: SearchViewState,
    show_cursor: bool,
    width: u16,
    real_cursor: bool,
) -> String {
    let prefix = if state.basic_terminal { "> " } else { "❯ " };
    let editing = view == SearchViewState::Editing;
    let cursor_width = usize::from(editing && !real_cursor);
    let available = width
        .saturating_sub(4)
        .saturating_sub(crate::tui::text::width(prefix) as u16)
        .saturating_sub(cursor_width as u16) as usize;
    let has_status = state.status_timer > 0 && !state.status_message.is_empty();

    if state.search_query.is_empty() {
        let content = if has_status && !editing {
            crate::tui::text::truncate_width(&state.status_message, available)
        } else if state.is_tv_mode {
            "Search live channels…".to_string()
        } else if state.is_addon_mode {
            "Search movies and series via addons…".to_string()
        } else {
            "Search movies and series…".to_string()
        };
        if editing {
            let cursor = if !real_cursor {
                if show_cursor { "█ " } else { "  " }
            } else {
                ""
            };
            format!("{prefix}{cursor}{content}")
        } else {
            format!("{prefix}{content}")
        }
    } else if editing && !real_cursor {
        let segments = state.search_query.graphemes();
        let cursor = state.search_query.cursor();
        let cursor_char = if show_cursor {
            "█"
        } else if cursor < segments.len() {
            segments[cursor]
        } else {
            " "
        };
        let before: String = segments.iter().take(cursor).copied().collect();
        let after: String = if cursor < segments.len() {
            segments.iter().skip(cursor + 1).copied().collect()
        } else {
            String::new()
        };
        let full = format!("{before}{cursor_char}{after}");
        let truncated = crate::tui::text::truncate_width(&full, available);
        format!("{prefix}{truncated}")
    } else {
        let content = crate::tui::text::truncate_width(state.search_query.as_str(), available);
        format!("{prefix}{content}")
    }
}

fn render_search_bar(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    theme: &Theme,
    view: SearchViewState,
    show_cursor: bool,
    centered: bool,
) {
    let result_status = if view == SearchViewState::Results && !state.search_results.is_empty() {
        let total = state.search_results.len();
        let selected_idx = state
            .search_list_state
            .selected()
            .unwrap_or(0)
            .min(total.saturating_sub(1));
        let selected_num = selected_idx + 1;
        let visible_items = state
            .last_result_metrics
            .map(|m| m.visible_items)
            .unwrap_or(8)
            .max(1);
        let total_pages = (total.saturating_sub(1) / visible_items) + 1;
        let page = (selected_idx / visible_items) + 1;

        if total_pages > 1 {
            if area.width < 58 {
                Some(format!(
                    "{}/{} • p{}/{}",
                    selected_num, total, page, total_pages
                ))
            } else {
                Some(format!(
                    "Item {} of {} • Page {}/{}",
                    selected_num, total, page, total_pages
                ))
            }
        } else if total == 1 {
            Some("1 result".to_string())
        } else if area.width < 45 {
            Some(format!("{}/{}", selected_num, total))
        } else {
            Some(format!("Item {} of {}", selected_num, total))
        }
    } else {
        None
    };

    let status_width = result_status
        .as_deref()
        .map(crate::tui::text::width)
        .unwrap_or(0) as u16;
    let content_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(status_width.saturating_add(u16::from(status_width > 0) * 2)),
        ])
        .split(area);

    let editing = view == SearchViewState::Editing;
    let real_cursor = editing && show_cursor && !state.basic_terminal;
    let has_status = state.status_timer > 0
        && !state.status_message.is_empty()
        && state.search_query.is_empty()
        && !editing;

    let prefix = if state.basic_terminal { "> " } else { "❯ " };
    let prefix_width = crate::tui::text::width(prefix) as u16;

    let search_line = if state.search_query.is_empty() {
        let placeholder_text = if has_status {
            state.status_message.as_str()
        } else if state.is_tv_mode {
            "Search live channels…"
        } else if state.is_addon_mode {
            "Search movies and series via addons…"
        } else {
            "Search movies and series…"
        };

        if editing {
            let cursor_str = if show_cursor { "█" } else { " " };
            Line::from(vec![
                Span::styled(prefix, theme.accent),
                Span::styled(cursor_str, theme.accent),
                Span::raw(" "),
                Span::styled(placeholder_text, theme.text_dim),
            ])
        } else if has_status {
            Line::from(vec![
                Span::styled(prefix, theme.accent),
                Span::styled(placeholder_text, theme.accent),
            ])
        } else {
            Line::from(vec![
                Span::styled(prefix, theme.text_dim),
                Span::styled(placeholder_text, theme.text_dim),
            ])
        }
    } else if editing && !real_cursor {
        let segments = state.search_query.graphemes();
        let cursor = state.search_query.cursor();
        let cursor_char = if show_cursor {
            "█"
        } else if cursor < segments.len() {
            segments[cursor]
        } else {
            " "
        };
        let before: String = segments.iter().take(cursor).copied().collect();
        let after: String = if cursor < segments.len() {
            segments.iter().skip(cursor + 1).copied().collect()
        } else {
            String::new()
        };
        Line::from(vec![
            Span::styled(prefix, theme.accent),
            Span::styled(before, theme.text),
            Span::styled(cursor_char, theme.accent),
            Span::styled(after, theme.text),
        ])
    } else {
        Line::from(vec![
            Span::styled(prefix, if editing { theme.accent } else { theme.text }),
            Span::styled(state.search_query.as_str(), theme.text),
        ])
    };

    let mut paragraph = Paragraph::new(search_line);
    if centered {
        paragraph = paragraph.alignment(Alignment::Center);
    }
    frame.render_widget(paragraph, content_row[0]);

    if real_cursor {
        let (cursor_x, cursor_y) = if state.search_query.is_empty() {
            let placeholder_width = crate::tui::text::width(if state.is_tv_mode {
                "Search live channels…"
            } else if state.is_addon_mode {
                "Search movies and series via addons…"
            } else {
                "Search movies and series…"
            }) as u16;
            let total_len = prefix_width + 2 + placeholder_width;
            let line_offset = if centered {
                content_row[0].width.saturating_sub(total_len) / 2
            } else {
                0
            };
            let cx = (content_row[0].x + line_offset + prefix_width)
                .min(content_row[0].right().saturating_sub(1));
            (cx, content_row[0].y)
        } else {
            let segments = state.search_query.graphemes();
            let cursor = state.search_query.cursor();
            let before_cursor: String = segments.into_iter().take(cursor).collect();
            let before_cursor_width = crate::tui::text::width(&before_cursor) as u16;
            let total_text_width = crate::tui::text::width(state.search_query.as_str()) as u16;
            let shown = total_text_width.min(content_row[0].width.saturating_sub(6));
            let line_offset = if centered {
                (content_row[0].width.saturating_sub(prefix_width + shown)) / 2
            } else {
                0
            };
            let cx = (content_row[0].x + line_offset + prefix_width + before_cursor_width)
                .min(content_row[0].right().saturating_sub(1));
            (cx, content_row[0].y)
        };
        frame.set_cursor_position((cursor_x, cursor_y));
    }

    if let Some(status) = result_status {
        frame.render_widget(
            Paragraph::new(status)
                .style(theme.accent)
                .alignment(Alignment::Right),
            content_row[1],
        );
    }
}

pub fn draw(frame: &mut Frame, area: Rect, state: &mut AppState, theme: &Theme) {
    let show_cursor = (state.tick_count % 16) < 8;
    let view = search_view_state(state);
    let search_bar_area;

    if view == SearchViewState::Empty
        || (view == SearchViewState::Editing && state.search_results.is_empty())
    {
        let basic_terminal = state.basic_terminal;
        let (tier, rows) = landing_split(area, state.is_tv_mode, basic_terminal);
        let vertical_chunks = rows.rects.clone();

        let logo_text = if basic_terminal || tier.is_compact() {
            if state.is_tv_mode {
                "█▀▄▀█ █▀█ █ █ █ █▀▀ █▀▄ █▀█ ▀▄▀\n█ ▀ █ █▄█ ▀▄▀ █ ██▄ █▄▀ █▄█ █ █TV".to_string()
            } else {
                "█▀▄▀█ █▀█ █ █ █ █▀▀ █▀▄ █▀█ ▀▄▀\n█ ▀ █ █▄█ ▀▄▀ █ ██▄ █▄▀ █▄█ █ █".to_string()
            }
        } else if state.is_tv_mode {
            r"███╗   ███╗  ██████╗  ██╗   ██╗ ██╗ ███████╗ ██████╗   ██████╗  ██╗  ██╗
████╗ ████║ ██╔═══██╗ ██║   ██║ ██║ ██╔════╝ ██╔══██╗ ██╔═══██╗ ╚██╗██╔╝
██╔████╔██║ ██║   ██║ ██║   ██║ ██║ █████╗   ██████╔╝ ██║   ██║  ╚███╔╝ 
██║╚██╔╝██║ ██║   ██║ ╚██╗ ██╔╝ ██║ ██╔══╝   ██╔══██╗ ██║   ██║  ██╔██╗ TV
██║ ╚═╝ ██║ ╚██████╔╝  ╚████╔╝  ██║ ███████╗ ██████╔╝ ╚██████╔╝ ██╔╝ ██╗
╚═╝     ╚═╝  ╚═════╝    ╚═══╝   ╚═╝ ╚══════╝ ╚═════╝   ╚═════╝  ╚═╝  ╚═╝"
                .to_string()
        } else {
            r"███╗   ███╗  ██████╗  ██╗   ██╗ ██╗ ███████╗ ██████╗   ██████╗  ██╗  ██╗
████╗ ████║ ██╔═══██╗ ██║   ██║ ██║ ██╔════╝ ██╔══██╗ ██╔═══██╗ ╚██╗██╔╝
██╔████╔██║ ██║   ██║ ██║   ██║ ██║ █████╗   ██████╔╝ ██║   ██║  ╚███╔╝ 
██║╚██╔╝██║ ██║   ██║ ╚██╗ ██╔╝ ██║ ██╔══╝   ██╔══██╗ ██║   ██║  ██╔██╗ 
██║ ╚═╝ ██║ ╚██████╔╝  ╚████╔╝  ██║ ███████╗ ██████╔╝ ╚██████╔╝ ██╔╝ ██╗
╚═╝     ╚═╝  ╚═════╝    ╚═══╝   ╚═╝ ╚══════╝ ╚═════╝   ╚═════╝  ╚═╝  ╚═╝"
                .to_string()
        };

        let logo_width: u16 = rows.logo_width;

        let pad = area.width.saturating_sub(logo_width) / 2;
        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(pad),
                Constraint::Length(logo_width),
                Constraint::Min(0),
            ])
            .split(vertical_chunks[rows.logo]);

        let version_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(pad),
                Constraint::Length(logo_width),
                Constraint::Min(0),
            ])
            .split(vertical_chunks[rows.version]);

        let title_art = Paragraph::new(logo_text)
            .alignment(Alignment::Left)
            .style(theme.title);
        frame.render_widget(title_art, horizontal_chunks[1]);

        let version = Paragraph::new(format!("v{}", env!("CARGO_PKG_VERSION")))
            .alignment(Alignment::Right)
            .style(theme.text_dim);
        frame.render_widget(version, version_chunks[1]);

        let search_width = search_deck_width(area, state, true);
        search_bar_area = centered_width(vertical_chunks[rows.search], search_width);

        if !state.tv_config_popup {
            render_search_bar(
                frame,
                search_bar_area,
                state,
                theme,
                view,
                show_cursor,
                true,
            );
        }

        if state.favorites_landing_visible() {
            render_favorites_landing(frame, vertical_chunks[rows.favorites], state, theme);
        }

        let compact_tabs = area.width < 76;
        let ultra_compact_tabs = area.width < 58;

        let ctrl_s = if ultra_compact_tabs {
            "S".to_string()
        } else {
            crate::tui::text::ctrl_key("S")
        };
        let ctrl_t = if ultra_compact_tabs {
            "T".to_string()
        } else {
            crate::tui::text::ctrl_key("T")
        };
        let ctrl_a = if ultra_compact_tabs {
            "A".to_string()
        } else {
            crate::tui::text::ctrl_key("A")
        };
        let ctrl_p = if ultra_compact_tabs {
            "P".to_string()
        } else {
            crate::tui::text::ctrl_key("P")
        };

        let stream_label = if compact_tabs { "Stream" } else { "Streaming" };
        let addon_label = if compact_tabs { "Addon" } else { "Addons" };

        let current_mode = state.mode();
        let mut mode_tabs: Vec<Vec<Span>> = Vec::new();

        if state.streaming_enabled {
            let mut spans = vec![];
            if current_mode == crate::tui::state::AppMode::Streaming {
                spans.push(Span::styled("[", theme.text_dim));
                spans.push(Span::styled(&ctrl_p, theme.shortcut));
                spans.push(Span::styled("] ", theme.text_dim));
                spans.push(Span::styled(
                    state.active_provider.label(),
                    theme.highlight.add_modifier(Modifier::BOLD),
                ));
            } else {
                spans.push(Span::styled("[", theme.text_dim));
                spans.push(Span::styled(&ctrl_s, theme.shortcut));
                spans.push(Span::styled("] ", theme.text_dim));
                spans.push(Span::styled(stream_label, theme.text_dim));
            }
            mode_tabs.push(spans);
        }

        if state.tv_enabled {
            let mut spans = vec![];
            if current_mode == crate::tui::state::AppMode::Tv {
                spans.push(Span::styled("[ ", theme.text_dim));
                spans.push(Span::styled(
                    "TV",
                    theme.highlight.add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::styled(" ]", theme.text_dim));
            } else {
                spans.push(Span::styled("[", theme.text_dim));
                spans.push(Span::styled(&ctrl_t, theme.shortcut));
                spans.push(Span::styled("] ", theme.text_dim));
                spans.push(Span::styled("TV", theme.text_dim));
            }
            mode_tabs.push(spans);
        }

        if state.addons_enabled {
            let mut spans = vec![];
            if current_mode == crate::tui::state::AppMode::Addon {
                spans.push(Span::styled("[ ", theme.text_dim));
                spans.push(Span::styled(
                    if compact_tabs { "Addon" } else { "Addons" },
                    theme.highlight.add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::styled(" ]", theme.text_dim));
            } else {
                spans.push(Span::styled("[", theme.text_dim));
                spans.push(Span::styled(&ctrl_a, theme.shortcut));
                spans.push(Span::styled("] ", theme.text_dim));
                spans.push(Span::styled(addon_label, theme.text_dim));
            }
            mode_tabs.push(spans);
        }

        let mut mode_spans = vec![];
        for (i, tab) in mode_tabs.into_iter().enumerate() {
            if i > 0 {
                let separator = if compact_tabs {
                    " • "
                } else {
                    if tier.is_compact() { "   " } else { "     " }
                };
                mode_spans.push(Span::raw(separator));
            }
            mode_spans.extend(tab);
        }

        let util_spans = vec![
            Span::styled("[", theme.text_dim),
            Span::styled("?", theme.shortcut),
            Span::styled("] ", theme.text_dim),
            Span::styled("Help", theme.text_dim),
            Span::raw("     "),
            Span::styled("[", theme.text_dim),
            Span::styled("q", theme.shortcut),
            Span::styled("] ", theme.text_dim),
            Span::styled("Quit", theme.text_dim),
        ];

        match rows.util_row {
            Some(util_idx) => {
                frame.render_widget(
                    Paragraph::new(Line::from(mode_spans)).alignment(Alignment::Center),
                    vertical_chunks[rows.mode_row],
                );
                frame.render_widget(
                    Paragraph::new(Line::from(util_spans)).alignment(Alignment::Center),
                    vertical_chunks[util_idx],
                );
            }
            None => {
                let row_rect = vertical_chunks[rows.mode_row];
                let util_width = 19u16;
                let sub_chunks = Layout::horizontal([
                    Constraint::Min(1),
                    Constraint::Length(util_width.min(row_rect.width)),
                ])
                .split(row_rect);
                frame.render_widget(
                    Paragraph::new(Line::from(mode_spans)).alignment(Alignment::Left),
                    sub_chunks[0],
                );
                frame.render_widget(
                    Paragraph::new(Line::from(util_spans)).alignment(Alignment::Right),
                    sub_chunks[1],
                );
            }
        }
    } else {
        if state.is_loading && state.search_results.is_empty() {
            render_search_state(frame, area, state, theme, SearchViewState::Loading);
            return;
        }

        let has_results = !state.search_results.is_empty();
        let suggestion_height =
            if state.input_mode == InputMode::Editing && !state.search_suggestions.is_empty() {
                state.search_suggestions.len().min(6) as u16 + 3
            } else {
                0
            };
        let chunks = if has_results {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(0),
                ])
                .split(area)
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(suggestion_height),
                    Constraint::Length(0),
                    Constraint::Min(0),
                ])
                .split(area)
        };

        let results_chunk = if has_results { chunks[2] } else { chunks[4] };
        search_bar_area = Rect {
            x: chunks[0].x + 2,
            width: chunks[0].width.saturating_sub(4),
            ..chunks[0]
        };
        render_search_bar(
            frame,
            search_bar_area,
            state,
            theme,
            view,
            show_cursor,
            false,
        );

        let list_block = Block::default();
        if !state.search_results.is_empty() {
            let poster_width = if state.image_supported {
                state
                    .image_picker
                    .as_ref()
                    .map(|picker| {
                        let font = picker.font_size();
                        let pixel_height =
                            u64::from(state.poster_rows.max(3)) * u64::from(font.height.max(1));
                        let pixel_width = pixel_height * 2 / 3;
                        u16::try_from(pixel_width.div_ceil(u64::from(font.width.max(1))))
                            .unwrap_or(u16::MAX)
                            .max(6)
                    })
                    .unwrap_or_else(|| state.poster_rows.saturating_mul(4).div_ceil(3).max(6))
            } else {
                12
            };
            let initial_metrics = state.result_metrics(results_chunk.height, results_chunk.width);
            let has_scrollbar = state.search_results.len() > initial_metrics.visible_items;
            let results_area = if has_scrollbar {
                Rect {
                    width: results_chunk.width.saturating_sub(1),
                    ..results_chunk
                }
            } else {
                results_chunk
            };
            crate::tui::clear_area(frame, results_chunk, theme);
            let selected_idx = state.search_list_state.selected();

            let metrics = state.result_metrics(results_area.height, results_area.width);
            let poster_width = poster_width
                .min(metrics.col_width.saturating_sub(18).max(6))
                .max(6);
            state.last_result_metrics = Some(metrics);
            let row_height = metrics.row_height;
            let rows = state
                .search_results
                .iter()
                .map(|_| Row::new(vec![Cell::from("")]).height(row_height));

            let table = Table::new(rows, [Constraint::Percentage(100)]).block(list_block);

            frame.render_stateful_widget(table, results_area, &mut state.search_list_state);

            let inner_area = results_area;
            let is_editing = state.input_mode == InputMode::Editing;

            for slot in 0..metrics.visible_items {
                let i = state.result_scroll + slot;
                let Some(res) = state.search_results.get(i) else {
                    break;
                };
                let visible_index = slot / metrics.columns as usize;
                let column = (slot % metrics.columns as usize) as u16;
                let current_y =
                    inner_area.y + (visible_index as u16 * row_height).min(inner_area.height);

                let item_area = Rect {
                    x: inner_area.x + column * metrics.col_width,
                    y: current_y,
                    width: if column + 1 == metrics.columns {
                        inner_area.width.saturating_sub(column * metrics.col_width)
                    } else {
                        metrics.col_width
                    },
                    height: metrics.poster_rows_eff,
                };

                let is_selected = Some(i) == selected_idx;
                if is_selected && !is_editing {
                    let selected_bg = theme.surface0.fg.unwrap_or(theme.base);
                    frame.render_widget(
                        Block::default().style(Style::default().bg(selected_bg)),
                        item_area,
                    );
                }

                let layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Length(2),
                        Constraint::Length(poster_width),
                        Constraint::Length(1),
                        Constraint::Min(0),
                    ])
                    .split(item_area);

                let highlight_area = layout[0];
                let poster_area = layout[1];
                let text_area = layout[3];

                if is_selected {
                    let (indicator_sym, indicator_style) = if is_editing {
                        (
                            if state.basic_terminal { "- " } else { "· " },
                            theme.text_dim,
                        )
                    } else {
                        (if state.basic_terminal { "> " } else { "▌ " }, theme.accent)
                    };
                    let indicator = Paragraph::new(ratatui::text::Line::from(vec![
                        ratatui::text::Span::styled(indicator_sym, indicator_style),
                    ]));

                    let v_layout = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(item_area.height.saturating_sub(1) / 2),
                            Constraint::Length(1),
                            Constraint::Min(0),
                        ])
                        .split(highlight_area);

                    frame.render_widget(indicator, v_layout[1]);
                }

                if state.image_supported {
                    if let Some(img) = state.search_posters.peek(&res.id) {
                        let target_dims = (poster_area.width, state.poster_rows);
                        let needs_protocol =
                            state.search_poster_protocols.peek(&res.id).map(|(d, _)| *d)
                                != Some(target_dims);
                        if needs_protocol {
                            if let Some(picker) = &mut state.image_picker {
                                let size = ratatui::layout::Size::new(target_dims.0, target_dims.1);
                                if let Ok(proto) = picker.new_protocol(
                                    (**img).clone(),
                                    size,
                                    ratatui_image::Resize::Fit(None),
                                ) {
                                    state
                                        .search_poster_protocols
                                        .put(res.id.clone(), (target_dims, proto));
                                }
                            }
                        }
                        if let Some((_, proto)) = state.search_poster_protocols.peek(&res.id) {
                            if !state.has_active_modal() {
                                let img_height = poster_area.height.min(state.poster_rows);
                                let img_y_offset = item_area.height.saturating_sub(img_height) / 2;
                                let p_area = Rect {
                                    y: poster_area.y + img_y_offset,
                                    height: img_height,
                                    ..poster_area
                                };
                                frame.render_widget(ratatui_image::Image::new(proto), p_area);
                            }
                        }
                    } else {
                        let is_in_flight = state.in_flight_posters.contains(&res.id);
                        let placeholder = Paragraph::new(poster_placeholder_lines(
                            state.basic_terminal,
                            is_in_flight,
                            state.tick_count,
                        ))
                        .style(if is_in_flight {
                            theme.lavender
                        } else {
                            theme.text_dim
                        })
                        .alignment(Alignment::Center);
                        frame.render_widget(placeholder, poster_area);
                    }
                } else {
                    let placeholder_height = item_area.height.min(2);
                    let v_center = item_area.height.saturating_sub(placeholder_height) / 2;
                    let p_area = Rect {
                        x: poster_area.x,
                        y: poster_area.y + v_center,
                        width: 12,
                        height: placeholder_height,
                    };
                    let placeholder = Paragraph::new("No\nPoster")
                        .style(theme.text_dim)
                        .alignment(Alignment::Center);
                    frame.render_widget(placeholder, p_area);
                }

                let text_top_padding = text_area.height.saturating_sub(2) / 2;
                let text_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(text_top_padding),
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Min(0),
                    ])
                    .split(text_area);

                let title_style = if is_selected {
                    if is_editing {
                        theme.text
                    } else {
                        theme.title.add_modifier(Modifier::BOLD)
                    }
                } else {
                    theme.text
                };
                let is_favorited = res.stype != 3
                    && state
                        .favorites
                        .is_favorite(&crate::models::SubjectIdentity {
                            provider: res.provider.cache_key(),
                            subject_id: &res.id,
                            title: &res.title,
                            stype: res.stype,
                            release_year: &res.release_year,
                        });
                let title_reserved = if is_favorited { 6 } else { 4 };
                let max_title_width = text_area.width.saturating_sub(title_reserved) as usize;
                let display_title = crate::tui::text::truncate_width(&res.title, max_title_width);

                let mut type_tag = if state.is_tv_mode || res.stype == 3 {
                    "TV Channel".to_string()
                } else if res.stype == 1 {
                    "Movie".to_string()
                } else if res.stype == 2 {
                    "Series".to_string()
                } else {
                    "".to_string()
                };

                let is_history = state.search_query.trim().to_lowercase() == "/history";
                if !is_history && type_tag.is_empty() {
                    type_tag = "Unknown".to_string();
                }

                let mut title_spans = vec![ratatui::text::Span::raw(" ")];
                if is_favorited {
                    title_spans.push(ratatui::text::Span::styled(
                        if state.basic_terminal { "* " } else { "★ " },
                        theme.rating,
                    ));
                }
                title_spans.push(ratatui::text::Span::styled(display_title, title_style));
                let title_line = ratatui::text::Line::from(title_spans);
                if text_layout[1].height > 0 {
                    frame.render_widget(Paragraph::new(title_line), text_layout[1]);
                }

                let mut info_spans = vec![];

                if is_history {
                    if !type_tag.is_empty() {
                        info_spans.push(ratatui::text::Span::styled(&type_tag, theme.text));
                        info_spans.push(ratatui::text::Span::styled(" • ", theme.text_dim));
                    }
                    if res.season > 0 {
                        info_spans.push(ratatui::text::Span::styled(
                            format!("S{:02}E{:02}", res.season, res.episode),
                            theme.text,
                        ));
                        info_spans.push(ratatui::text::Span::styled(" • ", theme.text_dim));
                    }

                    if let Some(hist) = state.history.get_item(
                        res.provider.cache_key(),
                        &res.id,
                        res.season,
                        res.episode,
                        Some(&res.title),
                    ) {
                        if hist.is_in_progress() {
                            let (filled, empty) = hist.progress_bar_parts(8);
                            info_spans.push(ratatui::text::Span::styled(
                                filled,
                                theme.accent.add_modifier(ratatui::style::Modifier::BOLD),
                            ));
                            info_spans.push(ratatui::text::Span::styled(empty, theme.text_dim));

                            let pct = hist
                                .progress_percentage()
                                .map(|p| format!(" {:.0}%", p))
                                .unwrap_or_default();
                            info_spans.push(ratatui::text::Span::styled(pct, theme.text));

                            if let Some(r) = hist.formatted_remaining() {
                                info_spans.push(ratatui::text::Span::styled(
                                    format!(" ({r})"),
                                    theme.text_dim,
                                ));
                            }
                            info_spans.push(ratatui::text::Span::styled(" • ", theme.text_dim));
                            info_spans.push(ratatui::text::Span::styled(
                                format!("Watched {}", hist.formatted_relative_time()),
                                theme.text_dim,
                            ));
                            info_spans.push(ratatui::text::Span::styled(" • ", theme.text_dim));
                        } else if hist.completed {
                            info_spans.push(ratatui::text::Span::styled(
                                if state.basic_terminal {
                                    "[Completed]"
                                } else {
                                    "[✓ Completed]"
                                },
                                theme.text_dim,
                            ));
                            info_spans.push(ratatui::text::Span::styled(" • ", theme.text_dim));
                            info_spans.push(ratatui::text::Span::styled(
                                format!("Watched {}", hist.formatted_relative_time()),
                                theme.text_dim,
                            ));
                            info_spans.push(ratatui::text::Span::styled(" • ", theme.text_dim));
                        }
                    }

                    info_spans.push(crate::tui::widgets::badge::provider_badge_span(
                        res.provider,
                        theme,
                        state.basic_terminal,
                    ));
                } else {
                    if let Some(resolution) =
                        crate::tui::widgets::badge::extract_resolution(&res.title, None)
                    {
                        info_spans.extend(crate::tui::widgets::badge::resolution_badge_spans(
                            resolution,
                            theme,
                            state.basic_terminal,
                        ));
                    }

                    macro_rules! push_year {
                        () => {
                            if res.release_year != "Unknown" && !res.release_year.is_empty() {
                                info_spans
                                    .push(ratatui::text::Span::styled(&res.release_year, theme.text));
                                info_spans.push(ratatui::text::Span::styled(" • ", theme.text_dim));
                            }
                        };
                    }

                    if is_selected {
                        if let Some(meta) = &state.search_preview {
                            let rating = meta
                                .get("imdbRating")
                                .or_else(|| meta.get("imdbRatingValue"))
                                .and_then(|v| v.as_str());
                            if let Some(r) = rating {
                                let star = if state.basic_terminal { "* " } else { "★ " };
                                info_spans.push(ratatui::text::Span::styled(star, theme.rating));
                                info_spans.push(ratatui::text::Span::styled(r, theme.text));
                                info_spans.push(ratatui::text::Span::styled(" • ", theme.text_dim));
                            }
                            push_year!();

                            let mut g_names = vec![];
                            if let Some(genres) = meta.get("genres").and_then(|g| g.as_array()) {
                                g_names = genres
                                    .iter()
                                    .filter_map(|g| {
                                        g.get("name")
                                            .and_then(|n| n.as_str())
                                            .map(|s| s.to_string())
                                    })
                                    .collect();
                            }
                            if !g_names.is_empty() {
                                info_spans.push(ratatui::text::Span::styled(
                                    g_names.join(" • "),
                                    theme.text,
                                ));
                                info_spans.push(ratatui::text::Span::styled(" • ", theme.text_dim));
                            }
                            info_spans.push(ratatui::text::Span::styled(&type_tag, theme.text));
                        } else if state.preview_loading {
                            push_year!();
                            info_spans.push(ratatui::text::Span::styled(&type_tag, theme.text));
                            info_spans.push(ratatui::text::Span::styled(" • ", theme.text_dim));
                            let dots = match (state.tick_count / 4) % 4 {
                                0 => "",
                                1 => ".",
                                2 => "..",
                                _ => "...",
                            };
                            info_spans.push(ratatui::text::Span::styled(
                                format!("Loading{dots}"),
                                theme.text_dim,
                            ));
                        } else {
                            push_year!();
                            info_spans.push(ratatui::text::Span::styled(&type_tag, theme.text));
                        }
                    } else {
                        push_year!();
                        info_spans.push(ratatui::text::Span::styled(&type_tag, theme.text));
                    }

                    if !info_spans.is_empty() {
                        info_spans.push(ratatui::text::Span::styled(" • ", theme.text_dim));
                    }
                    info_spans.push(crate::tui::widgets::badge::provider_badge_span(
                        res.provider,
                        theme,
                        state.basic_terminal,
                    ));
                }

                if text_layout[2].height > 0 && !info_spans.is_empty() {
                    let mut padded = vec![ratatui::text::Span::raw(" ")];
                    padded.extend(info_spans);
                    frame.render_widget(
                        Paragraph::new(ratatui::text::Line::from(padded)),
                        text_layout[2],
                    );
                }
            }

            crate::tui::widgets::render_scrollbar(
                frame,
                results_chunk,
                state.search_results.len(),
                metrics.visible_items,
                state.result_scroll,
                theme,
                state.basic_terminal,
            );
        } else {
            render_search_state(frame, chunks[4], state, theme, view);
        }
    }

    render_search_suggestions(frame, area, search_bar_area, state, theme, view);
    if state.tv_config_popup {
        let rows = state.tv_manager_rows();
        let total_rows = rows.len();
        let longest_source_width = state
            .tv_playlists
            .iter()
            .map(|source| crate::tui::text::width(source))
            .max()
            .unwrap_or(28);
        let popup_area = crate::tui::overlay::tv_config_layout(
            area,
            longest_source_width,
            total_rows,
            state.tv_input_active,
        );
        let title = format!(
            "TV Playlists · {}/{}",
            state.tv_manager_selected.saturating_add(1),
            total_rows.max(1)
        );
        let inner_area = crate::tui::widgets::ModalFrame::new(&title, theme, state.basic_terminal)
            .render(frame, popup_area, area);

        let sections = ratatui::layout::Layout::vertical([
            ratatui::layout::Constraint::Min(1),
            ratatui::layout::Constraint::Length(2),
        ])
        .split(inner_area);

        if state.tv_input_active {
            let label = if state.tv_input_is_file {
                "Enter playlist file path:"
            } else {
                "Enter playlist URL:"
            };
            crate::tui::widgets::render_single_line_input(
                frame,
                sections[0],
                label,
                &state.tv_input_buffer,
                theme,
                state.basic_terminal,
            );
        } else {
            let items: Vec<ratatui::widgets::ListItem> = rows
                .iter()
                .map(|row| {
                    use crate::tui::state::TvManagerRow;
                    match row {
                        TvManagerRow::Header(label) => {
                            ratatui::widgets::ListItem::new(ratatui::text::Line::from(vec![
                                ratatui::text::Span::raw(" "),
                                ratatui::text::Span::styled(label.to_string(), theme.muted),
                            ]))
                        }
                        TvManagerRow::Playlist(index) => {
                            let source =
                                state.tv_playlists.get(*index).cloned().unwrap_or_default();
                            let url_budget = (inner_area.width as usize).saturating_sub(6).max(12);
                            let display_source =
                                crate::tui::text::truncate_middle_width(&source, url_budget);
                            ratatui::widgets::ListItem::new(ratatui::text::Line::from(vec![
                                ratatui::text::Span::raw(" "),
                                ratatui::text::Span::styled(
                                    format!("{} {}", index + 1, display_source),
                                    theme.text,
                                ),
                            ]))
                        }
                        TvManagerRow::AddUrl => {
                            ratatui::widgets::ListItem::new(ratatui::text::Line::from(vec![
                                ratatui::text::Span::raw(" "),
                                ratatui::text::Span::styled("[ Add URL ]", theme.sapphire),
                            ]))
                        }
                        TvManagerRow::AddFile => {
                            ratatui::widgets::ListItem::new(ratatui::text::Line::from(vec![
                                ratatui::text::Span::raw(" "),
                                ratatui::text::Span::styled("[ Add file ]", theme.sapphire),
                            ]))
                        }
                        TvManagerRow::Reload => {
                            ratatui::widgets::ListItem::new(ratatui::text::Line::from(vec![
                                ratatui::text::Span::raw(" "),
                                ratatui::text::Span::styled("[ Reload ]", theme.rating),
                            ]))
                        }
                        TvManagerRow::Done => {
                            ratatui::widgets::ListItem::new(ratatui::text::Line::from(vec![
                                ratatui::text::Span::raw(" "),
                                ratatui::text::Span::styled("[ Done ]", theme.success),
                            ]))
                        }
                    }
                })
                .collect();

            let list = ratatui::widgets::List::new(items)
                .highlight_style(crate::tui::overlay::selection_style(
                    theme,
                    state.basic_terminal,
                ))
                .highlight_symbol(if state.basic_terminal { "> " } else { "▌ " });

            let mut list_state = ratatui::widgets::ListState::default();
            list_state.select(Some(state.tv_manager_selected));
            frame.render_stateful_widget(list, sections[0], &mut list_state);
        }

        let footer = if state.tv_input_active {
            vec![
                crate::tui::overlay::key_hint("Enter", "Add", theme),
                ratatui::text::Span::raw("  "),
                crate::tui::overlay::key_hint("Esc", "Cancel", theme),
            ]
        } else {
            vec![
                crate::tui::overlay::key_hint("↑↓", "Move", theme),
                ratatui::text::Span::raw("  "),
                crate::tui::overlay::key_hint("Enter", "Select", theme),
                ratatui::text::Span::raw("  "),
                crate::tui::overlay::key_hint("d", "Remove", theme),
                ratatui::text::Span::raw("  "),
                crate::tui::overlay::key_hint("Esc", "Close", theme),
            ]
        };
        crate::tui::widgets::render_modal_footer(frame, sections[1], footer, theme);
    }

    if state.addon_manager_popup {
        let addons_count = state.installed_addons.len();
        let total_rows = state.addon_manager_rows().len();
        let popup_area =
            crate::tui::overlay::addon_manager_layout(area, addons_count, state.addon_input_active);
        let title = format!(
            "Addons Manager · {}/{}",
            state.addon_manager_selected.saturating_add(1),
            total_rows.max(1)
        );
        let inner_area = crate::tui::widgets::ModalFrame::new(&title, theme, state.basic_terminal)
            .render(frame, popup_area, area);

        if state.addon_input_active {
            let sections = ratatui::layout::Layout::vertical([
                ratatui::layout::Constraint::Min(1),
                ratatui::layout::Constraint::Length(2),
            ])
            .split(inner_area);

            crate::tui::widgets::render_single_line_input(
                frame,
                sections[0],
                "Enter Addon Manifest URL:",
                &state.addon_input_buffer,
                theme,
                state.basic_terminal,
            );

            let footer = vec![
                crate::tui::overlay::key_hint("Enter", "Add", theme),
                ratatui::text::Span::raw("  "),
                crate::tui::overlay::key_hint("Esc", "Cancel", theme),
            ];
            crate::tui::widgets::render_modal_footer(frame, sections[1], footer, theme);
        } else {
            let sections = ratatui::layout::Layout::vertical([
                ratatui::layout::Constraint::Min(1),
                ratatui::layout::Constraint::Length(1),
                ratatui::layout::Constraint::Length(2),
            ])
            .split(inner_area);

            let is_add_selected = state.addon_manager_selected == state.installed_addons.len() + 1;

            let mut items = vec![ratatui::widgets::ListItem::new(ratatui::text::Line::from(
                vec![
                    ratatui::text::Span::raw("   "),
                    ratatui::text::Span::styled("Installed Addons", theme.muted),
                ],
            ))];

            for (idx, a) in state.installed_addons.iter().enumerate() {
                let row_idx = idx + 1;
                let is_selected = state.addon_manager_selected == row_idx;
                let prefix = if is_selected {
                    ratatui::text::Span::styled(
                        if state.basic_terminal { "> " } else { "▌ " },
                        theme.sapphire,
                    )
                } else {
                    ratatui::text::Span::raw("  ")
                };
                let check = if a.enabled {
                    ratatui::text::Span::styled("[x] ", theme.success)
                } else {
                    ratatui::text::Span::styled("[ ] ", theme.text_dim)
                };
                let name_budget = (inner_area.width as usize)
                    .saturating_sub(3 + 4 + 12)
                    .max(10);
                let addon_label = format!("{} v{} ", a.name, a.version.as_deref().unwrap_or("1.0"));
                let name = ratatui::text::Span::styled(
                    crate::tui::text::truncate_width(&addon_label, name_budget),
                    if is_selected {
                        theme.text.add_modifier(ratatui::style::Modifier::BOLD)
                    } else {
                        theme.text
                    },
                );
                let mut badges = Vec::new();
                if a.is_core() {
                    badges.push(ratatui::text::Span::styled("[Core] ", theme.lavender));
                }
                if a.provides_meta {
                    badges.push(ratatui::text::Span::styled("[Meta] ", theme.sapphire));
                }
                if a.provides_stream {
                    badges.push(ratatui::text::Span::styled("[Streams] ", theme.rating));
                }
                if a.provides_catalog {
                    badges.push(ratatui::text::Span::styled("[Catalog]", theme.teal));
                }
                let mut spans = vec![ratatui::text::Span::raw(" "), prefix, check, name];
                spans.extend(badges);
                items.push(ratatui::widgets::ListItem::new(ratatui::text::Line::from(
                    spans,
                )));
            }

            let list = ratatui::widgets::List::new(items);
            frame.render_widget(list, sections[0]);

            let add_prefix = if is_add_selected {
                if state.basic_terminal { "> " } else { "▌ " }
            } else {
                "  "
            };
            let add_button = if is_add_selected {
                ratatui::text::Span::styled(
                    format!("{add_prefix}[ Add Manifest URL ]"),
                    theme.sapphire.add_modifier(ratatui::style::Modifier::BOLD),
                )
            } else {
                ratatui::text::Span::styled(
                    format!("{add_prefix}[ Add Manifest URL ]"),
                    theme.sapphire,
                )
            };

            let button_line =
                ratatui::text::Line::from(vec![ratatui::text::Span::raw(" "), add_button]);
            frame.render_widget(ratatui::widgets::Paragraph::new(button_line), sections[1]);

            let footer = vec![
                crate::tui::overlay::key_hint("↑↓←→", "Move", theme),
                ratatui::text::Span::raw("  "),
                crate::tui::overlay::key_hint("Enter/Space", "Toggle/Select", theme),
                ratatui::text::Span::raw("  "),
                crate::tui::overlay::key_hint("d", "Remove", theme),
                ratatui::text::Span::raw("  "),
                crate::tui::overlay::key_hint("Esc", "Close", theme),
            ];
            crate::tui::widgets::render_modal_footer(frame, sections[2], footer, theme);
        }
    }

    if state.show_browse_popup {
        let raw_labels: Vec<String> = if state.mode() == crate::tui::state::AppMode::Addon {
            crate::providers::addons::models::curated_catalog_presets(&state.installed_addons)
                .into_iter()
                .map(|target| target.label)
                .collect()
        } else {
            crate::tui::state::BrowsePreset::ALL
                .iter()
                .map(|preset| preset.label().to_string())
                .collect()
        };

        let raw_items: Vec<String> = raw_labels
            .iter()
            .map(|label| {
                let (_, spacing) = crate::tui::overlay::browse_category_badge(label, theme);
                let badge_str = if label.to_ascii_lowercase().contains("movie")
                    || label.to_ascii_lowercase().contains("top rated (all-time)")
                    || label.to_ascii_lowercase().contains("top rated (recent")
                {
                    "[MOVIES]"
                } else if label.to_ascii_lowercase().contains("series")
                    || label.to_ascii_lowercase().contains("airing")
                    || label.to_ascii_lowercase().contains("show")
                    || label.to_ascii_lowercase().contains("tv")
                {
                    "[SERIES]"
                } else {
                    "[DISCOVER]"
                };
                format!("{badge_str}{spacing}{label}")
            })
            .collect();

        let lines: Vec<Line> = raw_labels
            .iter()
            .map(|label| {
                let (badge, spacing) = crate::tui::overlay::browse_category_badge(label, theme);
                Line::from(vec![
                    badge,
                    Span::raw(spacing),
                    Span::styled(label.to_string(), theme.text),
                ])
            })
            .collect();

        crate::tui::overlay::picker_with_lines(
            frame,
            area,
            &lines,
            &raw_items,
            &mut state.browse_list_state,
            crate::tui::overlay::PickerSpec {
                title: "Browse",
                confirm_label: "Open",
                minimum_width: 36,
            },
            theme,
            state.basic_terminal,
        );
    }

    if state.player_picker_popup {
        let items = state
            .available_players
            .iter()
            .map(|k| k.label().to_string())
            .collect::<Vec<_>>();
        crate::tui::overlay::picker(
            frame,
            area,
            &items,
            &mut state.player_picker_state,
            crate::tui::overlay::PickerSpec {
                title: "Open with",
                confirm_label: "Open",
                minimum_width: 24,
            },
            theme,
            state.basic_terminal,
        );
    }
}
fn suggestion_source_badge<'a>(
    suggestion: &str,
    state: &AppState,
    theme: &'a Theme,
    basic_terminal: bool,
) -> Option<Span<'a>> {
    let (tag, style) = if suggestion.starts_with('/') {
        if suggestion.eq_ignore_ascii_case("/history") {
            ("[HISTORY]", theme.sapphire)
        } else if suggestion.eq_ignore_ascii_case("/favorites") {
            ("[FAVORITES]", theme.rating)
        } else {
            ("[CMD]", theme.teal)
        }
    } else if state.is_tv_mode {
        ("[TV]", theme.lavender)
    } else {
        return None;
    };

    if basic_terminal {
        Some(Span::styled(
            format!("{tag} "),
            style.add_modifier(Modifier::BOLD),
        ))
    } else {
        Some(Span::styled(format!("{tag} "), style))
    }
}

pub fn search_suggestions_bounds(area: Rect, search_bar_area: Rect, count: usize) -> (Rect, Rect) {
    if count == 0 || search_bar_area.width == 0 {
        return (Rect::default(), Rect::default());
    }

    let start_y = search_bar_area.bottom();
    let max_h = area.bottom().saturating_sub(start_y);
    let container_h = ((count as u16).saturating_add(2)).min(max_h);

    let max_w = area
        .right()
        .saturating_sub(search_bar_area.x)
        .saturating_sub(1);
    let container_w = search_bar_area.width.max(48).min(max_w);

    let container_area = Rect {
        x: search_bar_area.x,
        y: start_y,
        width: container_w,
        height: container_h,
    };

    let inner_area = Rect {
        x: container_area.x.saturating_add(1),
        y: container_area.y.saturating_add(1),
        width: container_area.width.saturating_sub(2),
        height: container_area.height.saturating_sub(2),
    };

    (container_area, inner_area)
}

fn render_search_suggestions(
    frame: &mut Frame,
    area: Rect,
    search_bar_area: Rect,
    state: &AppState,
    theme: &Theme,
    _view: SearchViewState,
) {
    if state.input_mode != InputMode::Editing
        || state.search_suggestions.is_empty()
        || search_bar_area.width == 0
    {
        return;
    }

    let visible_count = state.search_suggestions.len().min(6);
    let selected_index = state.suggest_index.unwrap_or(0);
    let suggestion_offset = selected_index
        .saturating_add(1)
        .saturating_sub(visible_count)
        .min(state.search_suggestions.len().saturating_sub(visible_count));

    let visible_slice: Vec<(usize, &String)> = state
        .search_suggestions
        .iter()
        .enumerate()
        .skip(suggestion_offset)
        .take(visible_count)
        .collect();

    if visible_slice.is_empty() {
        return;
    }

    let (container_area, inner_area) =
        search_suggestions_bounds(area, search_bar_area, visible_slice.len());

    if container_area.width == 0 || container_area.height <= 2 || inner_area.height == 0 {
        return;
    }

    crate::tui::clear_area(frame, container_area, theme);
    let container_block = Block::default()
        .borders(Borders::ALL)
        .border_type(crate::tui::overlay::border_type(state.basic_terminal))
        .border_style(theme.border_focus)
        .style(Style::default().bg(theme.surface0.fg.unwrap_or(theme.base)));
    frame.render_widget(container_block, container_area);

    for (row_idx, &(orig_idx, suggestion)) in visible_slice.iter().enumerate() {
        let current_y = inner_area.y + row_idx as u16;
        if current_y >= inner_area.bottom() {
            break;
        }

        let is_selected = Some(orig_idx) == state.suggest_index;

        let indicator_symbol = if is_selected {
            if state.basic_terminal { "> " } else { "▌ " }
        } else {
            "  "
        };

        let indicator_style = if is_selected {
            theme.accent.add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let is_slash_cmd = suggestion.starts_with('/');
        let display_name = if is_slash_cmd {
            suggestion.strip_prefix('/').unwrap_or(suggestion)
        } else {
            suggestion.as_str()
        };

        let desc = slash_command_description(suggestion, state);

        let row_bg = if is_selected {
            theme.surface1.fg.unwrap_or(theme.base)
        } else {
            theme.surface0.fg.unwrap_or(theme.base)
        };
        let row_style = Style::default().bg(row_bg);

        let text_style = if is_selected {
            theme.highlight.add_modifier(Modifier::BOLD)
        } else {
            theme.text_dim
        };

        let desc_style = if is_selected {
            theme.subtext1.add_modifier(Modifier::BOLD)
        } else {
            theme.overlay1
        };

        let badge_span = suggestion_source_badge(suggestion, state, theme, state.basic_terminal);
        let badge_width = badge_span
            .as_ref()
            .map_or(0, |b| crate::tui::text::width(&b.content));

        let mut spans = vec![Span::styled(indicator_symbol, indicator_style)];
        if let Some(badge) = badge_span {
            spans.push(badge);
        }
        spans.push(Span::styled(display_name, text_style));

        if let Some(description) = desc {
            let name_len = crate::tui::text::width(display_name) + badge_width;
            let pad = 28usize.saturating_sub(name_len).max(2);
            spans.push(Span::raw(" ".repeat(pad)));
            let indicator_width = crate::tui::text::width(indicator_symbol);
            let desc_budget =
                (inner_area.width as usize).saturating_sub(indicator_width + name_len + pad);
            if desc_budget > 0 {
                spans.push(Span::styled(
                    crate::tui::text::truncate_width(description, desc_budget),
                    desc_style,
                ));
            }
        }

        let row_area = Rect {
            x: inner_area.x,
            y: current_y,
            width: inner_area.width,
            height: 1,
        };

        frame.render_widget(Paragraph::new(Line::from(spans)).style(row_style), row_area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn test_suggestion_source_badge_types() {
        let theme = Theme::mocha();
        let state = AppState::default();

        let badge_cmd = suggestion_source_badge("/help", &state, &theme, false);
        assert!(badge_cmd.is_some());
        assert!(badge_cmd.unwrap().content.contains("[CMD]"));

        let badge_hist = suggestion_source_badge("/history", &state, &theme, false);
        assert!(badge_hist.is_some());
        assert!(badge_hist.unwrap().content.contains("[HISTORY]"));

        let badge_fav = suggestion_source_badge("/favorites", &state, &theme, false);
        assert!(badge_fav.is_some());
        assert!(badge_fav.unwrap().content.contains("[FAVORITES]"));

        let badge_sug = suggestion_source_badge("Breaking Bad", &state, &theme, false);
        assert!(badge_sug.is_none());

        let tv_state = AppState {
            is_tv_mode: true,
            ..Default::default()
        };
        let badge_tv = suggestion_source_badge("CNN", &tv_state, &theme, false);
        assert!(badge_tv.is_some());
        assert!(badge_tv.unwrap().content.contains("[TV]"));
    }

    #[test]
    fn test_render_search_suggestions_dropdown() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let state = AppState {
            input_mode: InputMode::Editing,
            search_suggestions: vec![
                "/help".to_string(),
                "/history".to_string(),
                "Inception".to_string(),
            ],
            suggest_index: Some(0),
            basic_terminal: false,
            ..Default::default()
        };
        let theme = Theme::mocha();

        terminal
            .draw(|frame| {
                let area = Rect::new(0, 0, 80, 24);
                let search_bar = Rect::new(10, 2, 60, 3);
                render_search_suggestions(
                    frame,
                    area,
                    search_bar,
                    &state,
                    &theme,
                    SearchViewState::Empty,
                );
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let mut rendered = String::new();
        for y in 0..24 {
            for x in 0..80 {
                rendered.push_str(buffer[(x, y)].symbol());
            }
            rendered.push('\n');
        }
        assert!(rendered.contains("help"));
        assert!(rendered.contains("history"));
        assert!(rendered.contains("Inception"));
        assert!(rendered.contains("[CMD]"));
        assert!(rendered.contains("[HISTORY]"));
        assert!(!rendered.contains("[SUGGEST]"));
        assert!(rendered.contains('▌'));
        assert!(!rendered.contains('├'));
        assert!(!rendered.contains('└'));
    }

    #[test]
    fn test_search_suggestions_bounds() {
        let area = Rect::new(0, 0, 80, 24);
        let search_bar = Rect::new(10, 2, 60, 3);
        let (container, inner) = search_suggestions_bounds(area, search_bar, 3);
        assert_eq!(container.x, 10);
        assert_eq!(container.y, 5);
        assert_eq!(container.width, 60);
        assert_eq!(container.height, 5);
        assert_eq!(inner.x, 11);
        assert_eq!(inner.y, 6);
        assert_eq!(inner.width, 58);
        assert_eq!(inner.height, 3);
    }

    #[test]
    fn test_search_deck_width_stability() {
        let mut state = AppState::default();
        let compact_area = Rect::new(0, 0, 70, 24);
        let normal_area = Rect::new(0, 0, 90, 24);
        let wide_area = Rect::new(0, 0, 120, 24);

        // Landing widths
        let w_compact_empty = search_deck_width(compact_area, &state, true);
        let w_normal_empty = search_deck_width(normal_area, &state, true);
        let w_wide_empty = search_deck_width(wide_area, &state, true);
        assert_eq!(w_compact_empty, 54);
        assert_eq!(w_normal_empty, 68);
        assert_eq!(w_wide_empty, 80);

        // Changing query does NOT jump landing search deck width
        state.search_query = "Inception 2010 1080p".into();
        assert_eq!(search_deck_width(compact_area, &state, true), 54);
        assert_eq!(search_deck_width(normal_area, &state, true), 68);
        assert_eq!(search_deck_width(wide_area, &state, true), 80);
    }

    #[test]
    fn test_search_content_ghost_placeholder() {
        let state_rich = AppState {
            input_mode: InputMode::Editing,
            basic_terminal: false,
            ..Default::default()
        };
        let content_rich = search_content(&state_rich, SearchViewState::Editing, true, 80, false);
        assert!(content_rich.contains("❯ █ Search movies and series…"));

        let state_basic = AppState {
            input_mode: InputMode::Editing,
            basic_terminal: true,
            ..Default::default()
        };
        let content_basic = search_content(&state_basic, SearchViewState::Editing, true, 80, false);
        assert!(content_basic.contains("> █ Search movies and series…"));
    }

    #[test]
    fn test_favorites_landing_rendering() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut state = AppState {
            favorites_focus: true,
            ..Default::default()
        };
        for i in 0..10 {
            state.favorites.items.push(crate::favorites::FavoriteItem {
                provider: "moviebox".to_string(),
                subject_id: format!("fav-{i}"),
                title: format!("Favorite Movie {i}"),
                cover_url: None,
                stype: 1,
                release_year: "2024".to_string(),
                added_at: 0,
            });
        }
        let theme = Theme::mocha();

        terminal
            .draw(|frame| {
                let area = Rect::new(0, 0, 80, 24);
                render_favorites_landing(frame, area, &state, &theme);
            })
            .unwrap();
    }

    #[test]
    fn test_search_results_card_and_scrollbar_draw() {
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut state = AppState {
            input_mode: InputMode::Normal,
            search_query: "Matrix".into(),
            search_results: vec![
                crate::models::SearchResult {
                    id: "1".into(),
                    title: "The Matrix 1080p".into(),
                    stype: 1,
                    release_year: "1999".into(),
                    cover_url: None,
                    season: 0,
                    episode: 0,
                    provider: crate::providers::models::ProviderKind::MovieBox,
                },
                crate::models::SearchResult {
                    id: "2".into(),
                    title: "The Matrix Reloaded 4K".into(),
                    stype: 1,
                    release_year: "2003".into(),
                    cover_url: None,
                    season: 0,
                    episode: 0,
                    provider: crate::providers::models::ProviderKind::FourKHdHub,
                },
            ],
            ..Default::default()
        };
        state.search_list_state.select(Some(0));
        let theme = Theme::mocha();

        // Normal mode drawing (active highlight)
        terminal
            .draw(|frame| {
                let area = Rect::new(0, 0, 100, 30);
                draw(frame, area, &mut state, &theme);
            })
            .unwrap();

        // Editing mode drawing (dimmed selection)
        state.input_mode = InputMode::Editing;
        terminal
            .draw(|frame| {
                let area = Rect::new(0, 0, 100, 30);
                draw(frame, area, &mut state, &theme);
            })
            .unwrap();
    }
}
