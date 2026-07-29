use crate::tui::{
    state::{AppState, InputMode},
    theme::Theme,
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Cell, Paragraph, Row, Table},
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
    } else if state.is_loading {
        SearchViewState::Loading
    } else if state
        .status_message
        .to_ascii_lowercase()
        .contains("search failed")
    {
        SearchViewState::Error
    } else if !state.search_results.is_empty() {
        SearchViewState::Results
    } else if !state.search_query.trim().is_empty()
        && state
            .status_message
            .to_ascii_lowercase()
            .starts_with("no matches")
    {
        SearchViewState::NoResults
    } else {
        SearchViewState::Empty
    }
}

fn search_spinner(state: &AppState) -> char {
    if state.basic_terminal {
        let frames = ['-', '\\', '|', '/'];
        frames[(state.tick_count as usize) % frames.len()]
    } else {
        let frames = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        frames[(state.tick_count as usize) % frames.len()]
    }
}

fn search_hint(view: SearchViewState, width: u16, theme: &Theme) -> Line<'static> {
    let text = match view {
        SearchViewState::Editing if width >= 82 => "[↑↓] Suggestions  [Enter] Search  [Esc] Cancel",
        SearchViewState::Editing if width >= 54 => "[↑↓] Suggest  [Enter] Search  [Esc] Cancel",
        SearchViewState::Editing => "[Enter] Search  [Esc] Cancel",
        SearchViewState::Error if width >= 62 => "[Enter] Retry  [Type] Edit  [Esc] Clear",
        SearchViewState::Error => "[Enter] Retry  [Esc] Clear",
        SearchViewState::Results if width >= 62 => "[Type] Edit  [↑↓] Browse  [Enter] Open",
        SearchViewState::Results => "[↑↓] Browse  [Enter] Open",
        SearchViewState::NoResults if width >= 62 => "[Type] Edit  [Enter] Retry  [Esc] Clear",
        SearchViewState::NoResults => "[Type] Edit  [Esc] Clear",
        SearchViewState::Loading => "",
        SearchViewState::Empty => "",
    };

    let mut spans = Vec::new();
    let mut remaining = text;
    while let Some(open) = remaining.find('[') {
        if open > 0 {
            spans.push(Span::styled(remaining[..open].to_string(), theme.text_dim));
        }
        let Some(close) = remaining[open..].find(']') else {
            spans.push(Span::styled(remaining[open..].to_string(), theme.text_dim));
            remaining = "";
            break;
        };
        let close = open + close;
        spans.push(Span::styled("[", theme.text_dim));
        spans.push(Span::styled(
            remaining[open + 1..close].to_string(),
            theme.shortcut,
        ));
        spans.push(Span::styled("]", theme.text_dim));
        remaining = &remaining[close + 1..];
    }
    if !remaining.is_empty() {
        spans.push(Span::styled(remaining.to_string(), theme.text_dim));
    }
    Line::from(spans).centered()
}

fn centered_width(area: Rect, maximum: u16) -> Rect {
    let width = area.width.min(maximum).max(1);
    Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        width,
        ..area
    }
}

fn search_deck_width(area: Rect, state: &AppState, landing: bool) -> u16 {
    let query_width = if state.search_query.is_empty() {
        crate::tui::text::width("Search movies and series…") as u16
    } else {
        crate::tui::text::width(&state.search_query) as u16
    };
    let minimum = if landing { 38 } else { 48 };
    let maximum = if landing && area.width >= 120 {
        88
    } else if landing {
        72
    } else {
        104
    }
    .min(area.width.saturating_sub(4));

    query_width
        .saturating_add(10)
        .max(minimum.min(maximum))
        .min(maximum)
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

    let card_width = area.width.min(64);
    let card = Rect {
        x: area.x + area.width.saturating_sub(card_width) / 2,
        y: area.y + area.height.saturating_sub(3) / 2,
        width: card_width,
        height: 3,
    };
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(card);

    let pulse = match (state.tick_count / 4) % 4 {
        0 => "·",
        1 | 3 => "◦",
        _ => "○",
    };
    let query = crate::tui::text::truncate_width(
        &state.search_query,
        card_width.saturating_sub(10) as usize,
    );

    let (symbol, title, title_style, detail) = match view {
        SearchViewState::Loading => (
            search_spinner(state).to_string(),
            "Searching",
            theme.lavender,
            format!("Looking for “{query}”"),
        ),
        SearchViewState::NoResults => (
            if state.basic_terminal { "-" } else { pulse }.to_string(),
            "No matches",
            if (state.tick_count / 4) % 2 == 0 {
                theme.lavender
            } else {
                theme.subtext1
            },
            format!("Nothing found for “{query}”"),
        ),
        SearchViewState::Error => (
            if state.basic_terminal { "!" } else { "×" }.to_string(),
            "Search failed",
            theme.error,
            crate::tui::text::truncate_width(
                &state.status_message,
                card_width.saturating_sub(4) as usize,
            ),
        ),
        _ => return,
    };

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(format!("{symbol} "), title_style),
            Span::styled(title, title_style.add_modifier(Modifier::BOLD)),
        ]))
        .alignment(Alignment::Center),
        rows[0],
    );
    frame.render_widget(
        Paragraph::new(detail)
            .style(theme.text_dim)
            .alignment(Alignment::Center),
        rows[1],
    );

    let guidance = match view {
        SearchViewState::Loading => "Please wait",
        SearchViewState::NoResults => "Type to edit  ·  Enter to retry",
        SearchViewState::Error => "Enter to retry  ·  Type to edit",
        _ => "",
    };
    frame.render_widget(
        Paragraph::new(guidance)
            .style(theme.overlay0)
            .alignment(Alignment::Center),
        rows[2],
    );
}

fn search_content(
    state: &AppState,
    view: SearchViewState,
    show_cursor: bool,
    width: u16,
) -> String {
    let prefix = if state.basic_terminal { "> " } else { "❯ " };
    let cursor_width = usize::from(view == SearchViewState::Editing);
    let available = width
        .saturating_sub(4)
        .saturating_sub(crate::tui::text::width(prefix) as u16)
        .saturating_sub(cursor_width as u16) as usize;
    let content = if state.search_query.is_empty() {
        "Search movies and series…".to_string()
    } else {
        crate::tui::text::truncate_width(&state.search_query, available)
    };
    let cursor = if view == SearchViewState::Editing {
        if show_cursor { "█" } else { " " }
    } else {
        ""
    };
    format!("{prefix}{content}{cursor}")
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
    let rule_style = if view == SearchViewState::Editing {
        theme.border_focus
    } else if view == SearchViewState::Error {
        theme.error
    } else {
        theme.border
    };
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);
    let mut paragraph = Paragraph::new(search_content(state, view, show_cursor, area.width)).style(
        if view == SearchViewState::Editing {
            theme.text
        } else if state.search_query.is_empty() {
            theme.text_dim
        } else {
            theme.text
        },
    );
    if centered {
        paragraph = paragraph.alignment(Alignment::Center);
    }
    frame.render_widget(paragraph, rows[0]);

    let status = match view {
        SearchViewState::Results => format!(" {} results ", state.search_results.len()),
        _ => String::new(),
    };
    let status_width = crate::tui::text::width(&status) as u16;
    let rule_width = area.width.saturating_sub(status_width);
    let rule = if state.basic_terminal { "-" } else { "─" };
    let rule_text = rule.repeat(rule_width as usize);
    let status_style = if view == SearchViewState::Results {
        theme.accent
    } else {
        theme.text_dim
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(rule_text, rule_style),
            Span::styled(status, status_style),
        ])),
        rows[1],
    );
}

pub fn draw(frame: &mut Frame, area: Rect, state: &mut AppState, theme: &Theme) {
    let show_cursor = (state.tick_count % 16) < 8;
    let view = search_view_state(state);
    let mut search_bar_area = Rect::default();
    let mut suggestion_area = Rect::default();

    if view == SearchViewState::Empty
        || (view == SearchViewState::Editing && state.search_results.is_empty())
    {
        if state.tick_count < 1 {
            return;
        }

        let is_narrow = area.width < 100 || area.height < 28 || state.basic_terminal;
        let is_wide = area.width >= 120 && area.height >= 32 && !state.basic_terminal;
        let logo_height = if is_narrow {
            2
        } else if is_wide {
            6
        } else {
            4
        };

        let logo_text = if is_narrow {
            if state.is_tv_mode {
                "█▀▄▀█ █▀█ █ █ █ █▀▀ █▀▄ █▀█ ▀▄▀\n█ ▀ █ █▄█ ▀▄▀ █ ██▄ █▄▀ █▄█ █ █TV".to_string()
            } else {
                "█▀▄▀█ █▀█ █ █ █ █▀▀ █▀▄ █▀█ ▀▄▀\n█ ▀ █ █▄█ ▀▄▀ █ ██▄ █▄▀ █▄█ █ █".to_string()
            }
        } else if is_wide {
            if state.is_tv_mode {
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
            }
        } else {
            if state.is_tv_mode {
                r"  __  __  ___  __   __ ___  ___  ___   ___  __  __ 
 |  \/  |/ _ \ \ \ / /|_ _|| __|| _ ) / _ \ \ \/ / 
 | |\/| | (_) | \ V /  | | | _| | _ \| (_) | >  <  TV
 |_|  |_|\___/   \_/  |___||___||___/ \___/ /_/\_\ "
                    .to_string()
            } else {
                r"  __  __  ___  __   __ ___  ___  ___   ___  __  __ 
 |  \/  |/ _ \ \ \ / /|_ _|| __|| _ ) / _ \ \ \/ / 
 | |\/| | (_) | \ V /  | | | _| | _ \| (_) | >  <  
 |_|  |_|\___/   \_/  |___||___||___/ \___/ /_/\_\ "
                    .to_string()
            }
        };

        let logo_width: u16 = if is_narrow {
            if state.is_tv_mode { 33 } else { 31 }
        } else if is_wide {
            if state.is_tv_mode { 75 } else { 73 }
        } else {
            if state.is_tv_mode { 57 } else { 55 }
        };
        let suggestions_open =
            state.input_mode == InputMode::Editing && !state.search_suggestions.is_empty();
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(18),
                Constraint::Length(logo_height),
                Constraint::Length(1),
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(area);

        let pad = area.width.saturating_sub(logo_width) / 2;
        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(pad),
                Constraint::Length(logo_width),
                Constraint::Min(0),
            ])
            .split(vertical_chunks[1]);

        let version_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(pad),
                Constraint::Length(logo_width),
                Constraint::Min(0),
            ])
            .split(vertical_chunks[2]);

        let logo_style = if state.basic_terminal || state.tick_count >= 8 {
            theme.title
        } else {
            let t = state.tick_count as f32 / 8.0;
            let (start, end) = logo_fade_colors(theme);
            let r = (start.0 + (end.0 - start.0) * t) as u8;
            let g = (start.1 + (end.1 - start.1) * t) as u8;
            let b = (start.2 + (end.2 - start.2) * t) as u8;
            ratatui::style::Style::default().fg(ratatui::style::Color::Rgb(r, g, b))
        };

        if is_wide && !state.basic_terminal && state.tick_count < 15 {
            let rows: Vec<&str> = logo_text.split('\n').collect();
            for (i, row) in rows.iter().enumerate() {
                let row_tick_start = 1 + i as u64;
                if state.tick_count >= row_tick_start {
                    let row_t = ((state.tick_count - row_tick_start) as f32 / 7.0).clamp(0.0, 1.0);
                    let (start, end) = logo_fade_colors(theme);
                    let r = (start.0 + (end.0 - start.0) * row_t) as u8;
                    let g = (start.1 + (end.1 - start.1) * row_t) as u8;
                    let b = (start.2 + (end.2 - start.2) * row_t) as u8;
                    let row_style =
                        ratatui::style::Style::default().fg(ratatui::style::Color::Rgb(r, g, b));

                    let row_area = Rect {
                        x: horizontal_chunks[1].x,
                        y: horizontal_chunks[1].y + i as u16,
                        width: horizontal_chunks[1].width,
                        height: 1,
                    };
                    frame.render_widget(Paragraph::new(*row).style(row_style), row_area);
                }
            }
        } else {
            let title_art = Paragraph::new(logo_text)
                .alignment(Alignment::Left)
                .style(logo_style);
            frame.render_widget(title_art, horizontal_chunks[1]);
        }

        let version_style = if state.tick_count < 6 {
            theme.surface1
        } else {
            theme.text_dim
        };
        let version = Paragraph::new(format!("v{}", env!("CARGO_PKG_VERSION")))
            .alignment(Alignment::Right)
            .style(version_style);
        frame.render_widget(version, version_chunks[1]);

        if state.tick_count >= 3 {
            let search_width = search_deck_width(area, state, true);
            search_bar_area = centered_width(vertical_chunks[4], search_width);
            suggestion_area = Rect {
                x: search_bar_area.x,
                y: search_bar_area.bottom(),
                width: search_bar_area.width,
                height: area.bottom().saturating_sub(search_bar_area.bottom()),
            };

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

            let context = Line::from(vec![
                Span::styled(state.active_provider.label().to_string(), theme.accent),
                Span::styled(" • ", theme.muted),
                Span::styled(
                    if state.is_tv_mode { "TV" } else { "Streaming" },
                    theme.text_dim,
                ),
            ]);
            let context_area = if suggestions_open {
                Rect::default()
            } else if view == SearchViewState::Empty {
                vertical_chunks[5]
            } else {
                frame.render_widget(
                    Paragraph::new(search_hint(view, search_bar_area.width, theme))
                        .alignment(Alignment::Center),
                    vertical_chunks[5],
                );
                vertical_chunks[6]
            };
            if context_area.width > 0 {
                frame.render_widget(
                    Paragraph::new(context).alignment(Alignment::Center),
                    context_area,
                );
            }

            let footer = Line::from(vec![
                Span::styled("[", theme.text_dim),
                Span::styled("?", theme.shortcut),
                Span::styled("] ", theme.text_dim),
                Span::styled("Help", theme.text_dim),
                Span::raw("    "),
                Span::styled("[", theme.text_dim),
                Span::styled("q", theme.shortcut),
                Span::styled("] ", theme.text_dim),
                Span::styled("Quit", theme.text_dim),
            ]);
            frame.render_widget(
                Paragraph::new(footer).alignment(Alignment::Center),
                vertical_chunks[8],
            );
        }
    } else {
        let suggestion_height =
            if state.input_mode == InputMode::Editing && !state.search_suggestions.is_empty() {
                state.search_suggestions.len().min(6) as u16 + 2
            } else {
                0
            };
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(1),
                Constraint::Length(suggestion_height),
                Constraint::Length(0),
                Constraint::Min(0),
            ])
            .split(area);

        let search_width = search_deck_width(area, state, false);
        search_bar_area = centered_width(chunks[0], search_width);
        suggestion_area = chunks[2];
        render_search_bar(
            frame,
            search_bar_area,
            state,
            theme,
            view,
            show_cursor,
            false,
        );
        let suggestions_open =
            state.input_mode == InputMode::Editing && !state.search_suggestions.is_empty();
        if !suggestions_open {
            frame.render_widget(
                Paragraph::new(search_hint(view, search_bar_area.width, theme))
                    .alignment(Alignment::Center),
                chunks[1],
            );
        }

        let list_block = Block::default();
        if state.is_loading && state.search_results.is_empty() {
            render_search_state(frame, chunks[4], state, theme, SearchViewState::Loading);
        } else if !state.search_results.is_empty() {
            let poster_width = if state.image_supported {
                (state.poster_rows.saturating_mul(2) / 3).max(5)
            } else {
                12
            };
            let content_width = state
                .search_results
                .iter()
                .map(|result| crate::tui::text::width(&result.title) as u16)
                .max()
                .unwrap_or(0)
                .saturating_add(poster_width)
                .saturating_add(18)
                .clamp(48, 104);
            let results_area = centered_width(chunks[4], content_width);
            let selected_idx = state.search_list_state.selected();
            let offset = state.search_list_state.offset();

            let row_height = state.poster_rows.max(3);
            state.visible_items = (results_area.height as usize) / (row_height as usize);
            let rows = state
                .search_results
                .iter()
                .map(|_| Row::new(vec![Cell::from("")]).height(row_height));

            let table = Table::new(rows, [Constraint::Percentage(100)]).block(list_block);

            frame.render_stateful_widget(table, results_area, &mut state.search_list_state);

            let inner_area = results_area;

            let mut current_y = inner_area.y;

            for (i, res) in state.search_results.iter().enumerate().skip(offset) {
                if current_y >= inner_area.y + inner_area.height {
                    break;
                }

                let item_area = Rect {
                    x: inner_area.x,
                    y: current_y,
                    width: inner_area.width,
                    height: state
                        .poster_rows
                        .min(inner_area.y + inner_area.height.saturating_sub(current_y)),
                };

                if item_area.height == 0 {
                    break;
                }

                let is_selected = Some(i) == selected_idx;
                if is_selected {
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
                    let indicator = Paragraph::new(ratatui::text::Line::from(vec![
                        ratatui::text::Span::styled(
                            if state.basic_terminal { "> " } else { "▌ " },
                            theme.accent,
                        ),
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
                            let p_area = Rect {
                                height: poster_area.height.min(state.poster_rows),
                                ..poster_area
                            };
                            frame.render_widget(ratatui_image::Image::new(proto), p_area);
                        }
                    } else {
                        let placeholder = Paragraph::new("Poster\nunavailable")
                            .style(theme.text_dim)
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
                    let placeholder = Paragraph::new("Poster\nunsupported")
                        .style(theme.text_dim)
                        .alignment(Alignment::Center);
                    frame.render_widget(placeholder, p_area);
                }

                let text_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Min(0),
                    ])
                    .split(text_area);

                let title_style = if is_selected { theme.title } else { theme.text };
                let max_title_width = text_area.width.saturating_sub(4) as usize;
                let display_title = crate::tui::text::truncate_width(&res.title, max_title_width);

                let type_tag = if state.is_tv_mode || res.stype == 3 {
                    "TV Channel"
                } else if res.stype == 1 {
                    "Movie"
                } else if res.stype == 2 {
                    "Series"
                } else {
                    "Unknown"
                };

                let title_line = ratatui::text::Line::from(vec![ratatui::text::Span::styled(
                    display_title,
                    title_style,
                )]);
                if text_layout[0].height > 0 {
                    frame.render_widget(Paragraph::new(title_line), text_layout[0]);
                }

                let mut info_spans = vec![];

                if is_selected {
                    if state.preview_loading || state.is_loading {
                        info_spans.push(ratatui::text::Span::styled(&res.release_year, theme.text));
                        info_spans.push(ratatui::text::Span::styled(" • ", theme.text_dim));
                        info_spans.push(ratatui::text::Span::styled(type_tag, theme.text));
                        info_spans.push(ratatui::text::Span::styled(" • ", theme.text_dim));
                        info_spans.push(ratatui::text::Span::styled("Loading...", theme.text_dim));
                    } else if let Some(meta) = &state.search_preview {
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
                        info_spans.push(ratatui::text::Span::styled(&res.release_year, theme.text));
                        info_spans.push(ratatui::text::Span::styled(" • ", theme.text_dim));

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
                            info_spans
                                .push(ratatui::text::Span::styled(g_names.join(" • "), theme.text));
                            info_spans.push(ratatui::text::Span::styled(" • ", theme.text_dim));
                        }
                        info_spans.push(ratatui::text::Span::styled(type_tag, theme.text));
                    } else {
                        info_spans.push(ratatui::text::Span::styled(&res.release_year, theme.text));
                        info_spans.push(ratatui::text::Span::styled(" • ", theme.text_dim));
                        info_spans.push(ratatui::text::Span::styled(type_tag, theme.text));
                    }
                } else {
                    info_spans.push(ratatui::text::Span::styled(&res.release_year, theme.text));
                    info_spans.push(ratatui::text::Span::styled(" • ", theme.text_dim));
                    info_spans.push(ratatui::text::Span::styled(type_tag, theme.text));
                }

                if text_layout[1].height > 0 && !info_spans.is_empty() {
                    frame.render_widget(
                        Paragraph::new(ratatui::text::Line::from(info_spans)),
                        text_layout[1],
                    );
                }

                current_y += row_height;
            }

            let content_len = state.search_results.len();
            if content_len > state.visible_items {
                let scrollbar = ratatui::widgets::Scrollbar::default()
                    .orientation(ratatui::widgets::ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("▲"))
                    .end_symbol(Some("▼"))
                    .track_symbol(Some("│"))
                    .thumb_symbol(if state.basic_terminal { "|" } else { "█" });

                let mut scrollbar_state = ratatui::widgets::ScrollbarState::default()
                    .content_length(content_len.saturating_sub(state.visible_items))
                    .position(offset);

                let sb_area = results_area;

                frame.render_stateful_widget(scrollbar, sb_area, &mut scrollbar_state);
            }
        } else {
            render_search_state(frame, chunks[4], state, theme, view);
        }
    }

    if state.input_mode == InputMode::Editing
        && !state.search_suggestions.is_empty()
        && search_bar_area.width > 0
    {
        let search_area = search_bar_area;
        let visible_count = state.search_suggestions.len().min(6);
        let dropdown_height = visible_count as u16 + 2;
        let selected_index = state.suggest_index.unwrap_or(0);
        let suggestion_offset = selected_index
            .saturating_add(1)
            .saturating_sub(visible_count)
            .min(state.search_suggestions.len().saturating_sub(visible_count));
        let dropdown_width = search_area.width;
        let dropdown_x = search_area.x;
        let dropdown_area = if suggestion_area.height >= dropdown_height {
            Rect {
                x: dropdown_x,
                y: suggestion_area.y,
                width: dropdown_width,
                height: dropdown_height,
            }
        } else {
            Rect {
                x: dropdown_x,
                y: search_area.y + search_area.height,
                width: dropdown_width,
                height: dropdown_height,
            }
        };

        if dropdown_area.y + dropdown_area.height <= area.y + area.height {
            let surface = theme.surface0.fg.unwrap_or(theme.base);
            let selected_surface = theme.surface1.fg.unwrap_or(surface);
            frame.render_widget(
                Block::default().style(Style::default().bg(surface)),
                dropdown_area,
            );
            let dropdown_rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(visible_count as u16),
                    Constraint::Length(1),
                ])
                .split(dropdown_area);
            let items: Vec<ratatui::widgets::ListItem> = state
                .search_suggestions
                .iter()
                .enumerate()
                .skip(suggestion_offset)
                .take(visible_count)
                .map(|(i, s)| {
                    let selected = Some(i) == state.suggest_index;
                    let marker = if selected {
                        if state.basic_terminal { "> " } else { "▌ " }
                    } else {
                        "  "
                    };
                    let text = format!("{marker}{s}");
                    let style = if selected {
                        theme.highlight
                    } else {
                        theme.text
                    };
                    ratatui::widgets::ListItem::new(
                        ratatui::text::Line::from(ratatui::text::Span::styled(text, style))
                            .alignment(ratatui::layout::Alignment::Left),
                    )
                    .style(if selected {
                        theme.lavender.bg(selected_surface)
                    } else {
                        theme.text.bg(surface)
                    })
                })
                .collect();
            let position = state
                .suggest_index
                .map(|index| format!("{}/{}", index + 1, state.search_suggestions.len()))
                .unwrap_or_else(|| state.search_suggestions.len().to_string());
            let heading = Line::from(vec![
                Span::styled(" Suggestions", theme.title),
                Span::styled(" · ", theme.overlay0),
                Span::styled(position, theme.subtext1),
            ]);
            frame.render_widget(
                Paragraph::new(heading).style(Style::default().bg(surface)),
                dropdown_rows[0],
            );
            let list = ratatui::widgets::List::new(items)
                .highlight_style(
                    theme
                        .lavender
                        .bg(selected_surface)
                        .add_modifier(Modifier::BOLD),
                )
                .style(Style::default().bg(surface));
            frame.render_widget(list, dropdown_rows[1]);

            let footer = if dropdown_area.width >= 50 {
                Line::from(vec![
                    Span::styled(" [", theme.text_dim),
                    Span::styled("↑↓", theme.shortcut),
                    Span::styled("] Move   [", theme.text_dim),
                    Span::styled("Enter", theme.shortcut),
                    Span::styled("] Use   [", theme.text_dim),
                    Span::styled("Esc", theme.shortcut),
                    Span::styled("] Close", theme.text_dim),
                ])
            } else {
                Line::from(vec![
                    Span::styled(" [", theme.text_dim),
                    Span::styled("↑↓", theme.shortcut),
                    Span::styled("] Move  [", theme.text_dim),
                    Span::styled("Enter", theme.shortcut),
                    Span::styled("] Use  [", theme.text_dim),
                    Span::styled("Esc", theme.shortcut),
                    Span::styled("]", theme.text_dim),
                ])
            }
            .centered();
            frame.render_widget(
                Paragraph::new(footer).style(Style::default().bg(surface)),
                dropdown_rows[2],
            );
            if state.search_suggestions.len() > visible_count {
                let mut scrollbar_state = ratatui::widgets::ScrollbarState::default()
                    .content_length(state.search_suggestions.len())
                    .position(selected_index);
                let scrollbar = ratatui::widgets::Scrollbar::default()
                    .orientation(ratatui::widgets::ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .end_symbol(None)
                    .track_symbol(Some("│"))
                    .thumb_symbol(if state.basic_terminal { "|" } else { "█" });
                let scrollbar_area = Rect {
                    x: dropdown_rows[1].x,
                    y: dropdown_rows[1].y,
                    width: dropdown_rows[1].width,
                    height: dropdown_rows[1].height,
                };
                frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
            }
        }
    }
    if state.tv_config_popup {
        let content_height = state.tv_wizard_options.len() as u16;
        let content_width = state
            .tv_wizard_options
            .iter()
            .map(|option| crate::tui::text::width(option))
            .max()
            .unwrap_or(32)
            .max(crate::tui::text::width(
                "[Space] Toggle  [Enter] Confirm  [Esc] Back",
            ));
        let popup_area = crate::tui::overlay::centered(
            area,
            content_width.saturating_add(6) as u16,
            content_height.min(8) + 3,
            36,
            64,
        );
        crate::tui::overlay::clear_modal_area(frame, area, popup_area, theme);
        let popup_block = ratatui::widgets::Block::default()
            .title(if state.tv_wizard_step == 0 {
                " TV Setup: Select Grouping "
            } else {
                " TV Setup: Select Items "
            })
            .title_alignment(ratatui::layout::Alignment::Center)
            .borders(ratatui::widgets::Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(theme.border_focus)
            .style(ratatui::style::Style::default());

        let inner_area = popup_block.inner(popup_area);
        frame.render_widget(popup_block, popup_area);

        let items: Vec<ratatui::widgets::ListItem> = state
            .tv_wizard_options
            .iter()
            .map(|opt| {
                let is_checked = state.tv_wizard_selections.contains(opt);

                let checkbox = if state.tv_wizard_step == 1 {
                    if is_checked { "[x] " } else { "[ ] " }
                } else {
                    ""
                };

                let line = ratatui::text::Line::from(vec![ratatui::text::Span::styled(
                    format!("{}{}", checkbox, opt),
                    theme.text,
                )]);
                ratatui::widgets::ListItem::new(line)
            })
            .collect();

        let list = ratatui::widgets::List::new(items)
            .highlight_style(theme.highlight.add_modifier(ratatui::style::Modifier::BOLD))
            .highlight_symbol(if state.basic_terminal { "> " } else { "▌ " });

        let mut list_area = inner_area;
        list_area.height = list_area.height.saturating_sub(1);

        let mut list_state = ratatui::widgets::ListState::default();
        list_state.select(Some(state.tv_wizard_selected_idx));

        frame.render_stateful_widget(list, list_area, &mut list_state);

        let scrollbar =
            ratatui::widgets::Scrollbar::new(ratatui::widgets::ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("▲"))
                .end_symbol(Some("▼"))
                .track_symbol(Some("│"))
                .thumb_symbol("█");

        let mut scrollbar_state = ratatui::widgets::ScrollbarState::new(
            state
                .tv_wizard_options
                .len()
                .saturating_sub(list_area.height as usize),
        )
        .position(list_state.offset());

        frame.render_stateful_widget(
            scrollbar,
            list_area.inner(ratatui::layout::Margin {
                vertical: 0,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );

        let hint_area = ratatui::layout::Rect {
            x: inner_area.x,
            y: inner_area.y + inner_area.height.saturating_sub(1),
            width: inner_area.width,
            height: 1,
        };
        let hint = if state.tv_wizard_step == 0 {
            " [Enter] Select   [Esc] Cancel "
        } else {
            " [Space] Toggle   [Enter] Confirm   [Esc] Back "
        };
        frame.render_widget(
            ratatui::widgets::Paragraph::new(hint)
                .alignment(ratatui::layout::Alignment::Center)
                .style(theme.text_dim),
            hint_area,
        );
    }

    if state.player_picker_popup {
        let items = state
            .available_players
            .iter()
            .map(|k| {
                match k {
                    crate::tui::state::PlayerKind::Mpv => "mpv",
                    crate::tui::state::PlayerKind::Iina => "IINA",
                    crate::tui::state::PlayerKind::Vlc => "VLC",
                }
                .to_string()
            })
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

fn logo_fade_colors(theme: &Theme) -> ((f32, f32, f32), (f32, f32, f32)) {
    if theme.is_light {
        ((172.0, 176.0, 190.0), (136.0, 57.0, 239.0))
    } else {
        ((73.0, 76.0, 94.0), (203.0, 166.0, 247.0))
    }
}
