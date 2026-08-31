pub(crate) use crate::tui::widgets::{
    extract_media_tags, resolution_badge_spans, resolution_label,
};
use crate::tui::{state::AppState, theme::Theme};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailsLayoutTier {
    Wide,
    Medium,
    Narrow,
    Tiny,
}

impl DetailsLayoutTier {
    pub fn for_area(area: Rect) -> Self {
        if area.width < 60 || area.height < 22 {
            Self::Tiny
        } else if area.width < 85 {
            Self::Narrow
        } else if area.width < 115 {
            Self::Medium
        } else {
            Self::Wide
        }
    }

    pub(crate) fn header_height(self, area: Rect, details: Option<&serde_json::Value>) -> u16 {
        let (minimum, maximum, synopsis_limit, reserved_width) = match self {
            Self::Wide => (9, 12, 4, 30),
            Self::Medium => (8, 11, 3, 24),
            Self::Narrow => (7, 10, 3, 4),
            Self::Tiny => (5, 8, 2, 4),
        };
        let available_maximum = area.height.saturating_sub(match self {
            Self::Wide => 16,
            Self::Medium => 14,
            Self::Narrow => 12,
            Self::Tiny => 10,
        });
        let maximum = maximum.min(available_maximum.max(minimum));

        let Some(details) = details else {
            return minimum.min(maximum);
        };
        let synopsis = details
            .get("description")
            .and_then(|value| value.as_str())
            .or_else(|| details.get("intro").and_then(|value| value.as_str()))
            .unwrap_or_default();
        let text_width = area.width.saturating_sub(reserved_width).max(20) as usize;
        let title = details
            .get("title")
            .and_then(|value| value.as_str())
            .unwrap_or_default();
        let title_rows = (crate::tui::text::width(title) + 14)
            .div_ceil(text_width)
            .clamp(1, 2);
        let synopsis_rows = crate::tui::text::width(synopsis)
            .div_ceil(text_width)
            .clamp(1, synopsis_limit);
        let metadata_rows = match self {
            Self::Wide | Self::Medium => 5,
            Self::Narrow => 4,
            Self::Tiny => 3,
        };
        let content_rows = metadata_rows + title_rows.saturating_sub(1) + synopsis_rows;
        (content_rows as u16 + 2).clamp(minimum, maximum)
    }
    pub(crate) fn footer_height(self, width: u16) -> u16 {
        if width >= DETAILS_FOOTER_SPLIT_THRESHOLD {
            1
        } else {
            2
        }
    }
}

pub const DETAILS_FOOTER_SPLIT_THRESHOLD: u16 = 80;

pub(crate) fn visible_selector_panes(
    available_panes: &[crate::tui::state::DetailsPane],
    current_pane: crate::tui::state::DetailsPane,
    width: u16,
) -> Vec<crate::tui::state::DetailsPane> {
    if available_panes.is_empty() {
        return Vec::new();
    }
    if width < 85 {
        if available_panes.contains(&current_pane) {
            vec![current_pane]
        } else if let Some(last) = available_panes.last() {
            vec![*last]
        } else {
            Vec::new()
        }
    } else {
        available_panes.to_vec()
    }
}

pub(crate) fn selector_pane_constraints(
    visible_panes: &[crate::tui::state::DetailsPane],
    total_width: u16,
) -> Vec<Constraint> {
    use crate::tui::state::DetailsPane;
    match visible_panes.len() {
        0 => Vec::new(),
        1 => vec![Constraint::Min(20)],
        2 => {
            if visible_panes.contains(&DetailsPane::Languages) {
                if total_width < 100 {
                    vec![Constraint::Percentage(38), Constraint::Percentage(62)]
                } else {
                    vec![Constraint::Length(24), Constraint::Min(30)]
                }
            } else if visible_panes.contains(&DetailsPane::Seasons)
                && visible_panes.contains(&DetailsPane::Episodes)
            {
                if total_width < 100 {
                    vec![Constraint::Percentage(32), Constraint::Percentage(68)]
                } else {
                    vec![Constraint::Length(20), Constraint::Min(35)]
                }
            } else {
                vec![Constraint::Percentage(40), Constraint::Percentage(60)]
            }
        }
        _ => {
            if total_width < 120 {
                vec![
                    Constraint::Percentage(28),
                    Constraint::Percentage(24),
                    Constraint::Percentage(48),
                ]
            } else {
                vec![
                    Constraint::Length(24),
                    Constraint::Length(18),
                    Constraint::Min(35),
                ]
            }
        }
    }
}

fn subject_provider(state: &AppState, subject_id: &str) -> crate::providers::models::ProviderKind {
    state
        .search_results
        .iter()
        .find(|result| result.id == subject_id)
        .map(|result| result.provider)
        .unwrap_or(state.active_provider)
}

#[derive(Debug, Clone, Copy)]
pub struct DetailsScreenLayout {
    pub tier: DetailsLayoutTier,
    pub header_area: Rect,
    pub workflow_area: Rect,
    pub bottom_area: Rect,
    pub footer_area: Rect,
}

pub fn details_screen_layout(
    area: Rect,
    selected_details: Option<&serde_json::Value>,
) -> DetailsScreenLayout {
    let tier = DetailsLayoutTier::for_area(area);
    let header_height = tier.header_height(area, selected_details);
    let footer_height = tier.footer_height(area.width);
    let chunks = Layout::vertical([
        Constraint::Length(header_height),
        Constraint::Length(1),
        Constraint::Min(5),
        Constraint::Length(footer_height),
    ])
    .split(area);

    DetailsScreenLayout {
        tier,
        header_area: chunks[0],
        workflow_area: chunks[1],
        bottom_area: chunks[2],
        footer_area: chunks[3],
    }
}

pub fn draw(frame: &mut Frame, area: Rect, state: &mut AppState, theme: &Theme) {
    let layout = details_screen_layout(area, state.selected_details.as_ref());
    let tier = layout.tier;
    let header_area = layout.header_area;
    let workflow_area = layout.workflow_area;
    let bottom_area = layout.bottom_area;
    let footer_area = layout.footer_area;
    let details_json = match &state.selected_details {
        Some(d) => d,
        None => {
            if let Some(err) = &state.details_error {
                let box_width = area.width.saturating_sub(4).clamp(30, 60).min(area.width);
                let box_height = area.height.saturating_sub(2).clamp(5, 7).min(area.height);
                let x = area.x + (area.width.saturating_sub(box_width)) / 2;
                let y = area.y + (area.height.saturating_sub(box_height)) / 2;
                let error_area = Rect::new(x, y, box_width, box_height);

                let error_block = Block::default()
                    .borders(Borders::ALL)
                    .border_type(crate::tui::overlay::border_type(state.basic_terminal))
                    .border_style(theme.error)
                    .title(Line::from(vec![Span::styled(
                        " Error Loading Details ",
                        theme.error.add_modifier(Modifier::BOLD),
                    )]));

                let err_msg =
                    crate::tui::text::truncate_width(err, (box_width.saturating_sub(4)) as usize);
                let text = vec![
                    Line::from(""),
                    Line::from(Span::styled(err_msg, theme.text)),
                    Line::from(""),
                    Line::from(vec![
                        crate::tui::overlay::key_hint("r", "Retry fetch", theme),
                        Span::raw("   "),
                        crate::tui::overlay::key_hint("Esc", "Back", theme),
                    ]),
                ];

                let error_p = Paragraph::new(text)
                    .block(error_block)
                    .alignment(Alignment::Center);

                frame.render_widget(error_p, error_area);
                return;
            }

            let spinner = stream_loading_spinner(state.tick_count, state.basic_terminal);

            let vertical_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(45),
                    Constraint::Length(1),
                    Constraint::Percentage(50),
                ])
                .split(area);

            let loading_text = if state.basic_terminal {
                format!("Loading details {spinner}")
            } else {
                format!("{spinner} Loading details...")
            };
            let loading_p = Paragraph::new(loading_text)
                .alignment(ratatui::layout::Alignment::Center)
                .style(theme.lavender);

            frame.render_widget(loading_p, vertical_chunks[1]);
            return;
        }
    };

    let raw_title = details_json
        .get("title")
        .or_else(|| details_json.get("name"))
        .and_then(|t| t.as_str())
        .filter(|s| !s.trim().is_empty())
        .or_else(|| {
            state
                .search_results
                .iter()
                .find(|r| {
                    if let Some(act_id) = state.active_subject_id.as_deref() {
                        r.id == act_id
                    } else {
                        false
                    }
                })
                .map(|r| r.title.as_str())
        })
        .unwrap_or("Unknown Title");
    let title = crate::providers::moviebox::clean_moviebox_title(raw_title);
    let intro = details_json
        .get("description")
        .or_else(|| details_json.get("intro"))
        .or_else(|| details_json.get("synopsis"))
        .or_else(|| details_json.get("overview"))
        .and_then(|d| d.as_str())
        .filter(|s| !s.trim().is_empty())
        .or_else(|| {
            state.search_preview.as_ref().and_then(|p| {
                p.get("description")
                    .or_else(|| p.get("intro"))
                    .or_else(|| p.get("synopsis"))
                    .and_then(|s| s.as_str())
            })
        })
        .unwrap_or("No description available.");
    let year = details_json
        .get("releaseDate")
        .or_else(|| details_json.get("year"))
        .or_else(|| details_json.get("releaseInfo"))
        .and_then(|y| y.as_str())
        .filter(|s| !s.trim().is_empty())
        .or_else(|| {
            state
                .search_results
                .iter()
                .find(|r| {
                    if let Some(act_id) = state.active_subject_id.as_deref() {
                        r.id == act_id
                    } else {
                        false
                    }
                })
                .map(|r| r.release_year.as_str())
        })
        .unwrap_or("N/A");
    let type_val = crate::tui::state::stype(details_json);
    let type_str = if type_val == 2 { "Series" } else { "Movie" };

    let genres = details_json
        .get("genre")
        .or_else(|| details_json.get("genres"))
        .and_then(|g| {
            if let Some(a) = g.as_array() {
                let joined = a
                    .iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                if joined.is_empty() {
                    None
                } else {
                    Some(joined)
                }
            } else if let Some(s) = g.as_str() {
                if s.is_empty() {
                    None
                } else {
                    Some(s.to_string())
                }
            } else {
                None
            }
        })
        .unwrap_or_else(|| "N/A".to_string());
    let duration = details_json
        .get("duration")
        .and_then(|d| d.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("N/A");
    let country = details_json
        .get("countryName")
        .and_then(|c| c.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("N/A");

    let imdb_rating = details_json
        .get("imdbRatingValue")
        .and_then(|r| {
            r.as_f64()
                .map(|rf| rf.to_string())
                .or_else(|| r.as_str().map(|s| s.to_string()))
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "N/A".to_string());
    let tagline = details_json
        .get("tagline")
        .and_then(|value| value.as_str())
        .filter(|value| !value.is_empty());

    let details_block = Block::default()
        .borders(Borders::ALL)
        .border_type(crate::tui::overlay::border_type(state.basic_terminal))
        .border_style(theme.surface1)
        .padding(ratatui::widgets::Padding::new(
            if matches!(tier, DetailsLayoutTier::Wide) {
                2
            } else {
                1
            },
            1,
            0,
            0,
        ));

    let inner_area = details_block.inner(header_area);
    frame.render_widget(details_block.clone(), header_area);

    let show_poster = !matches!(tier, DetailsLayoutTier::Tiny | DetailsLayoutTier::Narrow)
        && inner_area.height >= 7
        && inner_area.width >= 75;
    let poster_width = if show_poster {
        let width_for_height = state
            .poster_image
            .as_ref()
            .zip(state.image_picker.as_ref())
            .map(|(image, picker)| {
                let font = picker.font_size();
                let target_pixel_height =
                    u64::from(inner_area.height) * u64::from(font.height.max(1));
                let target_pixel_width = target_pixel_height * u64::from(image.width())
                    / u64::from(image.height().max(1));
                target_pixel_width.div_ceil(u64::from(font.width.max(1))) as u16
            })
            .unwrap_or_else(|| (inner_area.height as f32 * 1.5).ceil() as u16);
        width_for_height.clamp(10, 26).min(inner_area.width / 3)
    } else {
        0
    };

    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(poster_width),
            Constraint::Length(if show_poster { 2 } else { 0 }),
            Constraint::Min(1),
        ])
        .split(inner_area);

    let poster_area = h_chunks[0];
    let right_area = h_chunks[2];

    if show_poster && state.image_supported {
        if let Some(img) = &state.poster_image {
            if state.poster_protocol.as_ref().map(|(r, _)| *r) != Some(poster_area)
                && let Some(picker) = &mut state.image_picker
            {
                let size = ratatui::layout::Size::new(poster_area.width, poster_area.height);
                if let Ok(proto) =
                    picker.new_protocol((**img).clone(), size, ratatui_image::Resize::Fit(None))
                {
                    state.poster_protocol = Some((poster_area, proto));
                }
            }
            if let Some((_, proto)) = &state.poster_protocol {
                if !state.has_active_modal() {
                    frame.render_widget(ratatui_image::Image::new(proto), poster_area);
                }
            }
        } else {
            let dots = match (state.tick_count / 4) % 4 {
                0 => "",
                1 => ".",
                2 => "..",
                _ => "...",
            };

            let placeholder_block = Block::default()
                .borders(Borders::ALL)
                .border_style(theme.muted);

            let inner = placeholder_block.inner(poster_area);

            let (pad, msg) = if state.is_loading {
                let p = "\n".repeat((inner.height.saturating_sub(1) / 2) as usize);
                (p, format!("Loading{dots}"))
            } else {
                let p = "\n".repeat((inner.height.saturating_sub(1) / 2) as usize);
                (p, title.to_string())
            };

            let placeholder = Paragraph::new(format!("{}{}", pad, msg))
                .style(theme.text_dim)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true })
                .block(placeholder_block);
            frame.render_widget(placeholder, poster_area);
        }
    } else if show_poster {
        let placeholder_block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme.muted);

        let inner = placeholder_block.inner(poster_area);
        let pad_top = "\n".repeat((inner.height.saturating_sub(2) / 2) as usize);
        let lines = format!("{pad_top}No\nPoster");

        let placeholder = Paragraph::new(lines)
            .style(theme.text_dim)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .block(placeholder_block);
        frame.render_widget(placeholder, poster_area);
    }

    let display_title = title.to_string();
    let details_subject_id = state.active_subject_id.as_deref().unwrap_or("");
    let is_favorited = state
        .favorites
        .is_favorite(&crate::models::SubjectIdentity {
            provider: subject_provider(state, details_subject_id).cache_key(),
            subject_id: details_subject_id,
            title: &title,
            stype: type_val,
            release_year: year,
        });

    let mut title_spans = Vec::new();
    if is_favorited {
        title_spans.push(Span::styled(
            if state.basic_terminal { "* " } else { "★ " },
            theme.rating,
        ));
    }
    title_spans.push(Span::styled(
        display_title,
        theme.text.add_modifier(ratatui::style::Modifier::BOLD),
    ));
    title_spans.push(Span::styled("   ", theme.text));

    if imdb_rating != "N/A" {
        if state.basic_terminal {
            title_spans.push(Span::styled(
                format!("[★ {} IMDb]", imdb_rating),
                theme.rating.add_modifier(Modifier::BOLD),
            ));
        } else {
            let badge_bg = theme_color(theme.rating, Color::Rgb(249, 226, 175));
            let badge_fg = if theme.is_light {
                Color::White
            } else {
                theme_color(theme.crust, Color::Rgb(17, 17, 27))
            };
            title_spans.push(Span::styled(
                format!(" ★ {} IMDb ", imdb_rating),
                Style::default()
                    .bg(badge_bg)
                    .fg(badge_fg)
                    .add_modifier(Modifier::BOLD),
            ));
        }
    } else {
        title_spans.push(Span::styled("★ IMDb N/A", theme.text_dim));
    }
    let title_line = Line::from(title_spans);

    let duration_str = if duration.is_empty() || duration == "N/A" {
        "".to_string()
    } else {
        format!(" • {}", duration)
    };

    let mut metadata = vec![type_str.to_string()];
    if year != "N/A" {
        metadata.push(year.to_string());
    }
    if country != "N/A" {
        metadata.push(country.to_string());
    }
    if !duration_str.is_empty() {
        metadata.push(duration.to_string());
    }
    let subject_id = state.active_subject_id.as_deref().unwrap_or("");
    let provider = subject_provider(state, subject_id).cache_key();
    if type_val == 1 {
        if let Some(hist) = state
            .history
            .get_item(provider, subject_id, 0, 0, Some(&title))
        {
            if hist.is_in_progress() {
                let p_bar = hist.progress_bar(8);
                let pct = hist
                    .progress_percentage()
                    .map(|p| format!("{:.0}%", p))
                    .unwrap_or_default();
                let rem = hist
                    .formatted_remaining()
                    .map(|r| format!(" • {r}"))
                    .unwrap_or_default();
                metadata.push(format!("{p_bar} {pct}{rem}"));
            } else if hist.completed {
                metadata.push(if state.basic_terminal {
                    "[Watched]".to_string()
                } else {
                    "[✓ Watched]".to_string()
                });
            }
        }
    }
    metadata.retain(|s| !s.trim().is_empty());
    let meta_line = Line::from(vec![Span::styled(
        metadata.join(" • "),
        metadata_style(theme),
    )]);

    let genre_line = Line::from(vec![Span::styled(
        genres.to_string(),
        metadata_style(theme),
    )]);

    let mut top_meta = vec![
        title_line,
        meta_line,
        genre_line,
        Line::from(vec![Span::styled(
            tagline.unwrap_or_default(),
            theme
                .overlay1
                .add_modifier(ratatui::style::Modifier::ITALIC),
        )]),
        Line::from(vec![Span::styled("Synopsis", theme.title)]),
    ];
    if matches!(tier, DetailsLayoutTier::Tiny) {
        top_meta.truncate(3);
    } else if matches!(tier, DetailsLayoutTier::Narrow) {
        top_meta.truncate(4);
    }
    let rating_text_len = if imdb_rating != "N/A" {
        crate::tui::text::width(&format!("   ★ {} IMDb", imdb_rating)) + 2
    } else {
        crate::tui::text::width("   ★ IMDb N/A")
    };
    let title_width = crate::tui::text::width(&title) + rating_text_len;
    let title_rows = title_width
        .div_ceil(right_area.width.max(1) as usize)
        .clamp(1, 2);
    let metadata_height = (top_meta.len() + title_rows.saturating_sub(1)) as u16;

    let meta_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(metadata_height), Constraint::Min(0)])
        .split(right_area);

    let meta_p = Paragraph::new(top_meta).wrap(Wrap { trim: true });
    frame.render_widget(meta_p, meta_chunks[0]);

    let max_width = meta_chunks[1].width as usize;
    let max_lines = meta_chunks[1].height as usize;
    let mut wrapped = crate::tui::text::wrap_text(intro, max_width);
    if wrapped.len() > max_lines {
        wrapped.truncate(max_lines);
        if let Some(last) = wrapped.last_mut() {
            let ellipsis = if state.basic_terminal { "..." } else { "…" };
            let ellipsis_w = crate::tui::text::width(ellipsis);
            if max_width >= ellipsis_w {
                let trimmed =
                    crate::tui::text::truncate_width(last, max_width.saturating_sub(ellipsis_w));
                let trimmed = trimmed
                    .trim_end_matches(['.', ',', '!', '?', ':', ';'])
                    .trim_end();
                *last = format!("{trimmed}{ellipsis}");
            }
        }
    }
    let syn_lines: Vec<Line> = wrapped
        .into_iter()
        .map(|line| Line::from(vec![Span::styled(line, theme.subtext1)]))
        .collect();
    let intro_p = Paragraph::new(syn_lines);
    frame.render_widget(intro_p, meta_chunks[1]);

    let has_languages = if let Some(dubs) = details_json.get("dubs").and_then(|d| d.as_array()) {
        dubs.len() > 1
    } else {
        false
    };

    let is_series = type_val == 2 && !state.available_seasons.is_empty();
    let streams_count = state
        .selected_resources
        .as_ref()
        .and_then(|resources| resources.get("list"))
        .and_then(|list| list.as_array())
        .map_or(0, Vec::len);

    render_workflow(
        frame,
        workflow_area,
        state,
        details_json,
        has_languages,
        is_series,
        streams_count,
        theme,
    );

    let mut available_selector_panes = Vec::new();
    if has_languages {
        available_selector_panes.push(crate::tui::state::DetailsPane::Languages);
    }
    if is_series {
        available_selector_panes.push(crate::tui::state::DetailsPane::Seasons);
        available_selector_panes.push(crate::tui::state::DetailsPane::Episodes);
    }

    let visible_selector_panes =
        visible_selector_panes(&available_selector_panes, state.details_pane, area.width);

    let selector_height = if visible_selector_panes.is_empty() {
        0
    } else {
        let episode_count = state
            .available_episode_numbers
            .get(state.season_list_state.selected().unwrap_or(0))
            .map_or(0, Vec::len);
        let language_count = details_json
            .get("dubs")
            .and_then(|dubs| dubs.as_array())
            .map_or(0, Vec::len);
        language_count
            .max(state.available_seasons.len())
            .max(episode_count)
            .min((bottom_area.height / 3).clamp(4, 10) as usize) as u16
            + 2
    };

    let lower_chunks = Layout::vertical([Constraint::Length(selector_height), Constraint::Min(3)])
        .split(bottom_area);
    let selector_area = lower_chunks[0];
    let streams_area = lower_chunks[1];

    let selector_chunks = if visible_selector_panes.is_empty() {
        Vec::new()
    } else {
        Layout::horizontal(selector_pane_constraints(
            &visible_selector_panes,
            selector_area.width,
        ))
        .split(selector_area)
        .to_vec()
    };

    let mut lang_area = None;
    let mut seasons_area = None;
    let mut eps_area = None;
    for (pane, pane_area) in visible_selector_panes
        .iter()
        .copied()
        .zip(selector_chunks.iter().copied())
    {
        match pane {
            crate::tui::state::DetailsPane::Languages => lang_area = Some(pane_area),
            crate::tui::state::DetailsPane::Seasons => seasons_area = Some(pane_area),
            crate::tui::state::DetailsPane::Episodes => eps_area = Some(pane_area),
            crate::tui::state::DetailsPane::Streams => {}
        }
    }

    if has_languages {
        use ratatui::widgets::{List, ListItem};
        let mut lang_items = Vec::new();
        if let Some(dubs) = details_json.get("dubs").and_then(|d| d.as_array()) {
            for dub in dubs {
                if let Some(lang) = dub.get("lanName").and_then(|n| n.as_str()) {
                    let name = clean_language_name(lang);
                    lang_items.push(ListItem::new(name).style(theme.text));
                }
            }
        }
        let language_count = lang_items.len();

        let language_focused = state.details_pane == crate::tui::state::DetailsPane::Languages;
        let lang_border = if language_focused {
            focused_border_style(theme)
        } else {
            unfocused_border_style(theme)
        };
        let lang_list = List::new(lang_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(crate::tui::overlay::border_type(state.basic_terminal))
                    .title(pane_title(
                        "Audio",
                        language_count,
                        crate::tui::state::DetailsPane::Languages,
                        language_focused,
                        state,
                        lang_area.map_or(0, |a| a.width),
                    ))
                    .title_style(if language_focused {
                        focused_title_style(theme)
                    } else {
                        unfocused_title_style(theme)
                    })
                    .border_style(lang_border)
                    .padding(ratatui::widgets::Padding::horizontal(1)),
            )
            .highlight_style(selection_style(
                language_focused,
                state.basic_terminal,
                theme,
            ))
            .highlight_symbol(selection_symbol(language_focused, state.basic_terminal));

        if let Some(area) = lang_area {
            frame.render_stateful_widget(lang_list, area, &mut state.language_list_state);
            render_scroll_indicator(
                frame,
                area,
                language_count,
                state.language_list_state.selected().unwrap_or(0),
                theme,
                state.basic_terminal,
            );
        }
    }

    if is_series {
        use ratatui::widgets::{List, ListItem};
        let seasons_items: Vec<ListItem> = state
            .available_seasons
            .iter()
            .map(|s| {
                let se_num = s.get("se").and_then(|v| v.as_i64()).unwrap_or(1);
                ListItem::new(format!("Season {}", se_num)).style(theme.text)
            })
            .collect();

        let seasons_focused = state.details_pane == crate::tui::state::DetailsPane::Seasons;
        let seasons_border = if seasons_focused {
            focused_border_style(theme)
        } else {
            unfocused_border_style(theme)
        };
        let seasons_list = List::new(seasons_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(crate::tui::overlay::border_type(state.basic_terminal))
                    .title(pane_title(
                        "Seasons",
                        state.available_seasons.len(),
                        crate::tui::state::DetailsPane::Seasons,
                        seasons_focused,
                        state,
                        seasons_area.map_or(0, |a| a.width),
                    ))
                    .title_style(if seasons_focused {
                        focused_title_style(theme)
                    } else {
                        unfocused_title_style(theme)
                    })
                    .border_style(seasons_border)
                    .padding(ratatui::widgets::Padding::horizontal(1)),
            )
            .highlight_style(selection_style(
                seasons_focused,
                state.basic_terminal,
                theme,
            ))
            .highlight_symbol(selection_symbol(seasons_focused, state.basic_terminal));

        if let Some(area) = seasons_area {
            frame.render_stateful_widget(seasons_list, area, &mut state.season_list_state);
            render_scroll_indicator(
                frame,
                area,
                state.available_seasons.len(),
                state.season_list_state.selected().unwrap_or(0),
                theme,
                state.basic_terminal,
            );
        }

        let ep_items: Vec<ListItem> = if let Some(ep_numbers) = state
            .available_episode_numbers
            .get(state.season_list_state.selected().unwrap_or(0))
        {
            let season_idx = state.season_list_state.selected().unwrap_or(0);
            let se_num = state
                .available_seasons
                .get(season_idx)
                .and_then(|s| s.get("se"))
                .and_then(|v| v.as_i64())
                .unwrap_or(1) as usize;
            let subject_id = state.active_subject_id.as_deref().unwrap_or("");
            let provider = subject_provider(state, subject_id).cache_key();

            let check_sym = if state.basic_terminal {
                "[x] "
            } else {
                "✓  "
            };
            let play_sym = if state.basic_terminal {
                "[>] "
            } else {
                "▶  "
            };
            let unwatched_sym = if state.basic_terminal { "[ ] " } else { "·  " };
            ep_numbers
                .iter()
                .map(|&ep| {
                    if let Some(hist) =
                        state
                            .history
                            .get_item(provider, subject_id, se_num, ep, Some(&title))
                    {
                        if hist.completed {
                            ListItem::new(format!("{check_sym}EP {ep:02}")).style(theme.text_dim)
                        } else if hist.is_in_progress() {
                            let progress_info =
                                match (hist.progress_percentage(), hist.formatted_remaining()) {
                                    (Some(p), Some(r)) => format!(" ({:.0}% · {r})", p),
                                    (Some(p), None) => format!(" ({:.0}%)", p),
                                    (None, Some(r)) => format!(" ({r})"),
                                    (None, None) => String::new(),
                                };
                            ListItem::new(format!("{play_sym}EP {ep:02}{progress_info}"))
                                .style(theme.accent)
                        } else {
                            ListItem::new(format!("{unwatched_sym}EP {ep:02}")).style(theme.text)
                        }
                    } else if state.history.is_watched(provider, subject_id, se_num, ep) {
                        ListItem::new(format!("{check_sym}EP {ep:02}")).style(theme.text_dim)
                    } else {
                        ListItem::new(format!("{unwatched_sym}EP {ep:02}")).style(theme.text)
                    }
                })
                .collect()
        } else {
            vec![]
        };
        let episode_count = ep_items.len();

        let episodes_focused = state.details_pane == crate::tui::state::DetailsPane::Episodes;
        let eps_border = if episodes_focused {
            focused_border_style(theme)
        } else {
            unfocused_border_style(theme)
        };
        let eps_list = List::new(ep_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(crate::tui::overlay::border_type(state.basic_terminal))
                    .title(pane_title(
                        "Episodes",
                        episode_count,
                        crate::tui::state::DetailsPane::Episodes,
                        episodes_focused,
                        state,
                        eps_area.map_or(0, |a| a.width),
                    ))
                    .title_style(if episodes_focused {
                        focused_title_style(theme)
                    } else {
                        unfocused_title_style(theme)
                    })
                    .border_style(eps_border)
                    .padding(ratatui::widgets::Padding::horizontal(1)),
            )
            .highlight_style(selection_style(
                episodes_focused,
                state.basic_terminal,
                theme,
            ))
            .highlight_symbol(selection_symbol(episodes_focused, state.basic_terminal));

        if let Some(area) = eps_area {
            frame.render_stateful_widget(eps_list, area, &mut state.episode_list_state);
            let episode_count = state
                .available_episode_numbers
                .get(state.season_list_state.selected().unwrap_or(0))
                .map_or(0, Vec::len);
            render_scroll_indicator(
                frame,
                area,
                episode_count,
                state.episode_list_state.selected().unwrap_or(0),
                theme,
                state.basic_terminal,
            );
        }
    }

    let streams_focused = state.details_pane == crate::tui::state::DetailsPane::Streams;
    let streams_border = if streams_focused {
        focused_border_style(theme)
    } else {
        unfocused_border_style(theme)
    };

    let streams_title = if streams_count > 0 {
        let selected = state
            .resource_list_state
            .selected()
            .unwrap_or(0)
            .min(streams_count.saturating_sub(1));
        let marker = if streams_focused {
            focus_title_marker(state.basic_terminal)
        } else {
            ""
        };
        let avail = streams_area.width.saturating_sub(4) as usize;
        let title_full = format!(
            " {marker}Streams · {} available · {}/{} ",
            streams_count,
            selected + 1,
            streams_count
        );
        if avail > 0 && crate::tui::text::width(&title_full) > avail {
            let title_medium = format!(
                " {marker}Streams · {} ({}/{}) ",
                streams_count,
                selected + 1,
                streams_count
            );
            if crate::tui::text::width(&title_medium) <= avail {
                title_medium
            } else {
                format!(" {marker}Streams ({streams_count}) ")
            }
        } else {
            title_full
        }
    } else if streams_focused {
        format!(" {}Streams ", focus_title_marker(state.basic_terminal))
    } else {
        " Streams ".to_string()
    };
    let streams_block = Block::default()
        .borders(Borders::ALL)
        .border_type(crate::tui::overlay::border_type(state.basic_terminal))
        .title(ratatui::text::Line::from(streams_title).alignment(Alignment::Left))
        .title_style(if streams_focused {
            focused_title_style(theme)
        } else {
            unfocused_title_style(theme)
        })
        .border_style(streams_border)
        .padding(ratatui::widgets::Padding::horizontal(1));

    match &state.selected_resources {
        Some(res) => {
            if let Some(list) = res.get("list").and_then(|l| l.as_array()) {
                let mut prev_quality = String::new();
                let selected_idx = state.resource_list_state.selected();
                let mut quality_counts = std::collections::HashMap::new();
                for file in list {
                    let resolution = file
                        .get("resolution")
                        .and_then(|value| value.as_i64())
                        .unwrap_or(0);
                    let label = resolution_label(resolution);
                    *quality_counts.entry(label).or_insert(0usize) += 1;
                }

                let list_items: Vec<ListItem> = list
                    .iter()
                    .enumerate()
                    .map(|(i, file)| {
                        let resolution =
                            file.get("resolution").and_then(|r| r.as_i64()).unwrap_or(0);
                        let quality_label = resolution_label(resolution);

                        let is_first_of_quality = quality_label != prev_quality;
                        prev_quality = quality_label.to_string();

                        let codec = file
                            .get("codecName")
                            .and_then(|c| c.as_str())
                            .unwrap_or("None");

                        let duration = file.get("duration").and_then(|d| d.as_u64()).unwrap_or(0);
                        let duration_str = if duration > 0 {
                            crate::tui::text::format_duration(duration)
                        } else {
                            "--:--".to_string()
                        };

                        let size_formatted = file
                            .get("size")
                            .and_then(|s| s.as_str())
                            .and_then(|s| s.parse::<f64>().ok())
                            .map(crate::tui::text::format_file_size)
                            .unwrap_or_else(|| "--".to_string());

                        let is_selected = Some(i) == selected_idx;
                        let pointer = if is_selected {
                            selection_symbol(streams_focused, state.basic_terminal)
                        } else {
                            "  "
                        };

                        let row_style = if is_selected {
                            selection_style(streams_focused, state.basic_terminal, theme)
                        } else {
                            metadata_style(theme)
                        };
                        let marker_style = if is_selected && streams_focused {
                            with_selection_surface(theme.accent, state.basic_terminal, theme)
                                .add_modifier(Modifier::BOLD)
                        } else if is_selected {
                            theme.accent
                        } else {
                            metadata_style(theme)
                        };
                        let primary_style = if is_selected {
                            with_selection_surface(theme.text, state.basic_terminal, theme)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            theme.text
                        };
                        let secondary_style = if is_selected {
                            with_selection_surface(
                                metadata_style(theme),
                                state.basic_terminal,
                                theme,
                            )
                        } else {
                            metadata_style(theme)
                        };

                        let is_fourk = file.get("_fourk_release").is_some();
                        let is_addon = file.get("_addon_release").is_some();
                        let raw_release_title = file
                            .get("fileName")
                            .or_else(|| file.get("title"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let release_title = crate::tui::text::clean_stream_text(raw_release_title);
                        let raw_upload_by = file
                            .get("uploadBy")
                            .and_then(|u| u.as_str())
                            .unwrap_or("Unknown");
                        let upload_by = crate::tui::text::clean_stream_text(raw_upload_by);
                        let raw_language = file
                            .get("language")
                            .and_then(|value| value.as_str())
                            .unwrap_or("Unknown");
                        let language = crate::tui::text::clean_stream_text(raw_language);
                        let source_count = file
                            .get("sourceCount")
                            .and_then(|value| value.as_u64())
                            .unwrap_or(0);
                        let is_ultra_compact = streams_area.width < 58;
                        let is_compact = streams_area.width < 85;
                        let is_wide = streams_area.width >= 115;
                        let stream_width = streams_area.width.saturating_sub(6) as usize;

                        let mut stream_spans = vec![Span::styled(pointer, marker_style)];

                        stream_spans.extend(resolution_badge_spans(
                            resolution,
                            theme,
                            state.basic_terminal,
                        ));

                        stream_spans.push(Span::styled(
                            format!("{size_formatted:>7}  "),
                            primary_style,
                        ));

                        let codec_str = codec.to_uppercase();
                        let tags = extract_media_tags(raw_release_title, &codec_str);
                        let mut tag_parts = Vec::new();
                        if let Some(hdr) = tags.hdr {
                            tag_parts.push(hdr);
                        }
                        if let Some(codec_tag) = tags.codec {
                            tag_parts.push(codec_tag);
                        } else if codec != "None" && !codec.is_empty() {
                            tag_parts.push(codec_str.as_str());
                        }
                        if let Some(audio) = tags.audio {
                            tag_parts.push(audio);
                        }
                        if let Some(source) = tags.source {
                            tag_parts.push(source);
                        }
                        let sep = if state.basic_terminal { " - " } else { " · " };
                        let tags_or_codec = tag_parts.join(sep);

                        let duration_col = if is_fourk && duration == 0 && source_count > 0 {
                            format!(
                                "{source_count} mirr{}",
                                if source_count == 1 { " " } else { "s" }
                            )
                        } else {
                            duration_str.clone()
                        };

                        if is_ultra_compact {
                            let used_prefix = stream_spans
                                .iter()
                                .map(|s| crate::tui::text::width(s.content.as_ref()))
                                .sum::<usize>();
                            let remaining = stream_width.saturating_sub(used_prefix);
                            if remaining > 0 {
                                let title_trunc =
                                    crate::tui::text::truncate_width(&release_title, remaining);
                                stream_spans.push(Span::styled(title_trunc, primary_style));
                            }
                        } else if is_compact {
                            let codec_display = crate::tui::text::truncate_width(&codec_str, 5);
                            stream_spans.push(Span::styled(
                                format!("{codec_display:<5} "),
                                secondary_style,
                            ));

                            let used_prefix = stream_spans
                                .iter()
                                .map(|s| crate::tui::text::width(s.content.as_ref()))
                                .sum::<usize>();
                            let remaining = stream_width.saturating_sub(used_prefix);
                            if remaining > 0 {
                                let title_trunc =
                                    crate::tui::text::truncate_width(&release_title, remaining);
                                stream_spans.push(Span::styled(title_trunc, primary_style));
                            }
                        } else if is_wide {
                            let tags_display = crate::tui::text::truncate_width(&tags_or_codec, 22);
                            stream_spans.push(Span::styled(
                                format!("{tags_display:<22} "),
                                secondary_style,
                            ));

                            let duration_display =
                                crate::tui::text::truncate_width(&duration_col, 8);
                            stream_spans.push(Span::styled(
                                format!("{duration_display:<8} "),
                                secondary_style,
                            ));

                            let uploader_display =
                                if upload_by != "Unknown" && !upload_by.is_empty() {
                                    format!(
                                        "{:<14}  ",
                                        crate::tui::text::truncate_width(&upload_by, 14)
                                    )
                                } else {
                                    format!("{:<16}", "")
                                };
                            stream_spans.push(Span::styled(uploader_display, secondary_style));

                            let used_prefix = stream_spans
                                .iter()
                                .map(|s| crate::tui::text::width(s.content.as_ref()))
                                .sum::<usize>();
                            let remaining = stream_width.saturating_sub(used_prefix);
                            if remaining > 0 {
                                let title_trunc =
                                    crate::tui::text::truncate_width(&release_title, remaining);
                                stream_spans.push(Span::styled(title_trunc, primary_style));
                            }
                        } else {
                            let tags_display = crate::tui::text::truncate_width(&tags_or_codec, 14);
                            stream_spans.push(Span::styled(
                                format!("{tags_display:<14} "),
                                secondary_style,
                            ));

                            let duration_display =
                                crate::tui::text::truncate_width(&duration_col, 8);
                            stream_spans.push(Span::styled(
                                format!("{duration_display:<8} "),
                                secondary_style,
                            ));

                            let used_prefix_width = stream_spans
                                .iter()
                                .map(|s| crate::tui::text::width(s.content.as_ref()))
                                .sum::<usize>();
                            let remaining = stream_width.saturating_sub(used_prefix_width);

                            if remaining > 0 {
                                if is_addon {
                                    let has_lang = language != "Unknown" && !language.is_empty();
                                    if has_lang && remaining >= 24 {
                                        let lang_str = format!(
                                            "{} ",
                                            crate::tui::text::pad_to_width(&language, 10)
                                        );
                                        let lang_len = crate::tui::text::width(&lang_str);
                                        let title_avail = remaining.saturating_sub(lang_len);
                                        let title_trunc = crate::tui::text::truncate_width(
                                            &release_title,
                                            title_avail,
                                        );
                                        stream_spans.push(Span::styled(lang_str, secondary_style));
                                        stream_spans.push(Span::styled(title_trunc, primary_style));
                                    } else {
                                        let title_trunc = crate::tui::text::truncate_width(
                                            &release_title,
                                            remaining,
                                        );
                                        stream_spans.push(Span::styled(title_trunc, primary_style));
                                    }
                                } else if is_fourk {
                                    let has_lang = language != "Unknown" && !language.is_empty();
                                    if has_lang {
                                        let lang_trunc =
                                            crate::tui::text::truncate_width(&language, remaining);
                                        stream_spans
                                            .push(Span::styled(lang_trunc, secondary_style));
                                    }
                                } else {
                                    let has_uploader =
                                        upload_by != "Unknown" && !upload_by.is_empty();
                                    let has_title =
                                        !release_title.is_empty() && release_title != upload_by;

                                    if has_uploader && has_title && remaining >= 24 {
                                        let uploader_width = crate::tui::text::width(&upload_by);
                                        let title_avail =
                                            remaining.saturating_sub(uploader_width + 2);
                                        if title_avail >= 8 {
                                            let title_trunc = crate::tui::text::truncate_width(
                                                &release_title,
                                                title_avail,
                                            );
                                            stream_spans.push(Span::styled(
                                                format!("{upload_by}  "),
                                                secondary_style,
                                            ));
                                            stream_spans
                                                .push(Span::styled(title_trunc, secondary_style));
                                        } else {
                                            let uploader_trunc = crate::tui::text::truncate_width(
                                                &upload_by, remaining,
                                            );
                                            stream_spans.push(Span::styled(
                                                uploader_trunc,
                                                secondary_style,
                                            ));
                                        }
                                    } else if has_uploader {
                                        let uploader_trunc =
                                            crate::tui::text::truncate_width(&upload_by, remaining);
                                        stream_spans
                                            .push(Span::styled(uploader_trunc, secondary_style));
                                    } else if has_title {
                                        let title_trunc = crate::tui::text::truncate_width(
                                            &release_title,
                                            remaining,
                                        );
                                        stream_spans
                                            .push(Span::styled(title_trunc, secondary_style));
                                    }
                                }
                            }
                        }

                        if is_selected {
                            let used_width = stream_spans
                                .iter()
                                .map(|span| crate::tui::text::width(span.content.as_ref()))
                                .sum::<usize>();
                            if stream_width > used_width {
                                stream_spans.push(Span::styled(
                                    " ".repeat(stream_width.saturating_sub(used_width)),
                                    row_style,
                                ));
                            }
                        }
                        let stream_line = Line::from(stream_spans);
                        let mut lines = vec![];
                        if is_first_of_quality {
                            if i > 0 {
                                lines.push(ratatui::text::Line::from(""));
                            }
                            let option_count =
                                quality_counts.get(quality_label).copied().unwrap_or(1);
                            let header_spans = vec![
                                Span::styled(
                                    quality_label,
                                    theme.highlight.add_modifier(Modifier::BOLD),
                                ),
                                Span::styled(" · ", theme.overlay0),
                                Span::styled(
                                    format!(
                                        "{} option{}",
                                        option_count,
                                        if option_count == 1 { "" } else { "s" }
                                    ),
                                    metadata_style(theme),
                                ),
                            ];
                            lines.push(Line::from(header_spans));
                            if i == 0 {
                                lines.push(Line::from(stream_table_header_spans(
                                    streams_area.width,
                                    theme,
                                )));
                            }
                        }
                        lines.push(stream_line);
                        ListItem::new(lines)
                    })
                    .collect();

                let content_height = list_items.iter().map(ListItem::height).sum();
                let l = List::new(list_items).block(streams_block.clone());

                frame.render_stateful_widget(l, streams_area, &mut state.resource_list_state);
                let rendered_position = selected_idx.map_or(0, |selected| {
                    let mut headings = 0;
                    let mut previous: Option<&'static str> = None;
                    for (i, file) in list.iter().take(selected.saturating_add(1)).enumerate() {
                        let resolution = file
                            .get("resolution")
                            .and_then(|value| value.as_i64())
                            .unwrap_or(0);
                        let label = resolution_label(resolution);
                        if previous != Some(label) {
                            headings += 1;
                            if i == 0 {
                                headings += 1;
                            }
                            previous = Some(label);
                        }
                    }
                    selected + headings
                });
                render_scroll_indicator(
                    frame,
                    streams_area,
                    content_height,
                    rendered_position,
                    theme,
                    state.basic_terminal,
                );
            } else {
                let has_multiple_dubs = state
                    .selected_details
                    .as_ref()
                    .and_then(|d| d.get("dubs"))
                    .and_then(|d| d.as_array())
                    .is_some_and(|a| a.len() > 1);
                let provider_label = state
                    .active_subject_id
                    .as_deref()
                    .map(|id| subject_provider(state, id).label())
                    .unwrap_or_else(|| state.active_provider.label());

                let msg = if has_multiple_dubs && !state.language_chosen {
                    "Choose an audio track to load streams.".to_string()
                } else {
                    format!(
                        "No stream sources found on {provider_label} — press Ctrl+P to try another provider, or r to retry."
                    )
                };

                let inner = streams_block.inner(streams_area);
                let pad = "\n".repeat((inner.height.saturating_sub(1) / 2) as usize);
                let p = Paragraph::new(format!("{}{}", pad, msg))
                    .style(theme.text_dim)
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true })
                    .block(streams_block.clone());
                frame.render_widget(p, streams_area);
            }
        }
        None => {
            let has_multiple_dubs = state
                .selected_details
                .as_ref()
                .and_then(|d| d.get("dubs"))
                .and_then(|d| d.as_array())
                .is_some_and(|a| a.len() > 1);

            let waiting_for_language = has_multiple_dubs && !state.language_chosen;
            let has_error = state.stream_error.is_some();

            let msg = if waiting_for_language {
                "Choose an audio track to load streams.".to_string()
            } else if let Some(error) = &state.stream_error {
                if state.is_addon_mode {
                    error.clone()
                } else if error.contains("No stream sources") || error.contains("not listed") {
                    format!("{error}.\nPress Ctrl+P to try another provider, or r to refresh.")
                } else {
                    format!("{error} — press r to retry or Ctrl+P to switch provider.")
                }
            } else {
                let spinner = stream_loading_spinner(state.tick_count, state.basic_terminal);
                if state.basic_terminal {
                    format!("Loading streams {spinner}")
                } else {
                    format!("{spinner} Loading streams...")
                }
            };

            let style = if has_error {
                theme.error
            } else if waiting_for_language {
                theme.text_dim
            } else {
                theme.lavender
            };

            if !msg.is_empty() {
                let inner = streams_block.inner(streams_area);
                let pad = "\n".repeat((inner.height.saturating_sub(1) / 2) as usize);
                let p = Paragraph::new(format!("{}{}", pad, msg))
                    .style(style)
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true })
                    .block(streams_block.clone());
                frame.render_widget(p, streams_area);
            } else {
                frame.render_widget(streams_block.clone(), streams_area);
            }
        }
    }

    let (mut primary_footer, secondary_footer) = details_footer(state, theme, area.width);
    let footer_p = if area.width >= DETAILS_FOOTER_SPLIT_THRESHOLD {
        primary_footer.extend(secondary_footer);
        Paragraph::new(Line::from(primary_footer))
    } else {
        if let Some(last) = primary_footer.last_mut() {
            *last = Span::raw("");
        }
        Paragraph::new(vec![
            Line::from(primary_footer),
            Line::from(secondary_footer),
        ])
    }
    .alignment(Alignment::Center);
    frame.render_widget(footer_p, footer_area);

    if state.subtitle_popup || state.is_download_subtitle_popup {
        let items = state
            .subtitle_list
            .iter()
            .map(|(name, _)| {
                if name == "None" {
                    "No subtitles".to_string()
                } else {
                    crate::tui::text::sanitize_language_label(name)
                }
            })
            .collect::<Vec<_>>();
        crate::tui::overlay::picker(
            frame,
            area,
            &items,
            &mut state.subtitle_list_state,
            crate::tui::overlay::PickerSpec {
                title: "Subtitles",
                confirm_label: if state.is_download_subtitle_popup {
                    "Download"
                } else {
                    "Use"
                },
                minimum_width: 32,
            },
            theme,
            state.basic_terminal,
        );
    }

    if state.show_season_download_confirm {
        let summary = season_confirm_summary(state);
        let lines: Vec<Line<'_>> = summary
            .iter()
            .map(|text| Line::from(text.clone()))
            .collect();
        crate::tui::overlay::confirmation(
            frame,
            area,
            "Download season",
            &lines,
            state.season_download_confirm_yes_selected,
            theme,
            state.basic_terminal,
        );
    } else if state.show_episode_download_confirm {
        let summary = episode_confirm_summary(state);
        let lines: Vec<Line<'_>> = summary
            .iter()
            .map(|text| Line::from(text.clone()))
            .collect();
        crate::tui::overlay::confirmation(
            frame,
            area,
            if crate::tui::state::stype(details_json) == 2 {
                "Download episode"
            } else {
                "Download movie"
            },
            &lines,
            state.episode_download_confirm_yes_selected,
            theme,
            state.basic_terminal,
        );
    }
}

pub(crate) fn season_confirm_summary(state: &AppState) -> Vec<String> {
    let title = state
        .selected_details
        .as_ref()
        .and_then(|d| d.get("title"))
        .and_then(|t| t.as_str())
        .unwrap_or("Series");
    let season_idx = state.selected_season;
    let eps_count = if season_idx > 0 && season_idx <= state.available_episode_numbers.len() {
        state.available_episode_numbers[season_idx - 1].len()
    } else {
        0
    };
    let mut summary = vec![format!(
        "{title} • Season {season_idx} ({eps_count} Episodes)"
    )];
    if let Some(stream) = selected_stream_summary(state) {
        summary.push(format!("Quality: {stream}"));
    }
    let dest = state
        .download_dir
        .as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| {
            crate::service::resolve_download_dir(None)
                .to_string_lossy()
                .to_string()
        });
    summary.push(format!("Save to: {dest}"));
    summary
}

pub(crate) fn episode_confirm_summary(state: &AppState) -> Vec<String> {
    let title = state
        .selected_details
        .as_ref()
        .and_then(|d| d.get("title"))
        .and_then(|t| t.as_str())
        .unwrap_or("Media");
    let year = state
        .selected_details
        .as_ref()
        .and_then(|d| d.get("releaseDate").or_else(|| d.get("year")))
        .and_then(|y| y.as_str())
        .unwrap_or("");
    let season_idx = state.selected_season;
    let ep_idx = state.selected_episode;
    let type_val = state
        .selected_details
        .as_ref()
        .map(crate::tui::state::stype)
        .unwrap_or(1);
    let mut summary = if type_val == 2 {
        vec![format!("{title} • Season {season_idx} Episode {ep_idx}")]
    } else if !year.is_empty() && year != "N/A" {
        vec![format!("{title} ({year}) • Movie")]
    } else {
        vec![format!("{title} • Movie")]
    };
    if let Some(stream) = selected_stream_summary(state) {
        summary.push(format!("Quality: {stream}"));
    }
    let dest = state
        .download_dir
        .as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| {
            crate::service::resolve_download_dir(None)
                .to_string_lossy()
                .to_string()
        });
    summary.push(format!("Save to: {dest}"));
    summary
}

fn selected_stream_summary(state: &AppState) -> Option<String> {
    let resource = state
        .selected_resources
        .as_ref()?
        .get("list")?
        .as_array()?
        .get(state.resource_list_state.selected().unwrap_or(0))?;
    let resolution = resource
        .get("resolution")
        .and_then(|value| value.as_i64())
        .filter(|value| *value > 0)
        .map(|value| format!("{value}p"));
    let codec = resource
        .get("codecName")
        .and_then(|value| value.as_str())
        .filter(|value| !value.is_empty())
        .map(str::to_uppercase);
    let size = resource
        .get("size")
        .and_then(|value| value.as_str())
        .and_then(|value| value.parse::<f64>().ok())
        .map(crate::tui::text::format_file_size);
    let fields = [size, resolution, codec]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    (!fields.is_empty()).then(|| fields.join(" · "))
}

fn clean_language_name(value: &str) -> String {
    let mut name = if value.to_ascii_lowercase().starts_with("original") {
        "Original".to_string()
    } else {
        value
            .replace("dub", "")
            .replace("Dub", "")
            .trim()
            .to_string()
    };
    if name.eq_ignore_ascii_case("ptbr") {
        name = "Portuguese (BR)".to_string();
    } else if name.eq_ignore_ascii_case("esla") {
        name = "Spanish (LA)".to_string();
    }
    name
}

fn pane_title(
    label: &str,
    count: usize,
    pane: crate::tui::state::DetailsPane,
    focused: bool,
    state: &AppState,
    max_width: u16,
) -> Line<'static> {
    let marker = if focused {
        focus_title_marker(state.basic_terminal)
    } else {
        ""
    };
    let mut panes = Vec::new();
    let has_languages = state
        .selected_details
        .as_ref()
        .and_then(|details| details.get("dubs"))
        .and_then(|dubs| dubs.as_array())
        .is_some_and(|dubs| dubs.len() > 1);
    if has_languages {
        panes.push(crate::tui::state::DetailsPane::Languages);
    }
    if !state.available_seasons.is_empty() {
        panes.push(crate::tui::state::DetailsPane::Seasons);
        panes.push(crate::tui::state::DetailsPane::Episodes);
    }
    panes.push(crate::tui::state::DetailsPane::Streams);

    let position_str = if focused {
        if let Some(position) = panes.iter().position(|candidate| *candidate == pane) {
            format!("  {}/{}", position + 1, panes.len())
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let title_full = format!(" {marker}{label} · {count}{position_str} ");
    let avail = max_width.saturating_sub(2) as usize;
    let title = if avail > 0 && crate::tui::text::width(&title_full) > avail {
        let title_compact = format!(" {marker}{label} · {count} ");
        if crate::tui::text::width(&title_compact) <= avail {
            title_compact
        } else {
            format!(" {marker}{label} ({count}) ")
        }
    } else {
        title_full
    };

    Line::from(title)
}

fn theme_color(style: Style, fallback: ratatui::style::Color) -> ratatui::style::Color {
    style.fg.unwrap_or(fallback)
}

fn focused_border_style(theme: &Theme) -> Style {
    theme.lavender
}

fn unfocused_border_style(theme: &Theme) -> Style {
    theme.surface1
}

fn focused_title_style(theme: &Theme) -> Style {
    theme.title
}

fn unfocused_title_style(theme: &Theme) -> Style {
    theme.subtext1
}

fn metadata_style(theme: &Theme) -> Style {
    theme.subtext1
}

const BRAILLE_SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub(crate) fn stream_loading_spinner(tick_count: u64, basic_terminal: bool) -> &'static str {
    if basic_terminal {
        match (tick_count / 4) % 4 {
            0 => "..",
            1 => "...",
            2 => "....",
            _ => "..",
        }
    } else {
        BRAILLE_SPINNER[(tick_count as usize) % BRAILLE_SPINNER.len()]
    }
}

fn with_selection_surface(style: Style, basic_terminal: bool, theme: &Theme) -> Style {
    if basic_terminal {
        style
    } else {
        style.bg(theme_color(theme.surface0, theme.base))
    }
}

fn selection_style(focused: bool, basic_terminal: bool, theme: &Theme) -> Style {
    if focused {
        let style =
            with_selection_surface(theme.text, basic_terminal, theme).add_modifier(Modifier::BOLD);
        if basic_terminal {
            style.add_modifier(Modifier::UNDERLINED)
        } else {
            style
        }
    } else {
        theme.text.add_modifier(Modifier::BOLD)
    }
}

fn focus_title_marker(basic_terminal: bool) -> &'static str {
    if basic_terminal { "> " } else { "● " }
}

fn selection_symbol(focused: bool, basic_terminal: bool) -> &'static str {
    if focused {
        if basic_terminal { "> " } else { "▌ " }
    } else if basic_terminal {
        "* "
    } else {
        "· "
    }
}

#[allow(clippy::too_many_arguments)]
fn render_workflow(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    details: &serde_json::Value,
    has_languages: bool,
    is_series: bool,
    streams_count: usize,
    theme: &Theme,
) {
    let compact = area.width < 100;
    let mut steps = Vec::new();

    if has_languages {
        let active_idx = details
            .get("dubs")
            .and_then(|dubs| dubs.as_array())
            .and_then(|dubs| {
                dubs.iter().position(|dub| {
                    dub.get("subjectId")
                        .and_then(crate::tui::state::subject_id)
                        .as_deref()
                        == state.active_subject_id.as_deref()
                })
            })
            .or_else(|| state.language_list_state.selected())
            .unwrap_or(0);

        let language = details
            .get("dubs")
            .and_then(|dubs| dubs.as_array())
            .and_then(|dubs| dubs.get(active_idx))
            .and_then(|dub| dub.get("lanName"))
            .and_then(|name| name.as_str())
            .map(clean_language_name)
            .unwrap_or_else(|| "Choose".to_string());
        steps.push((
            crate::tui::state::DetailsPane::Languages,
            format!("Audio: {language}"),
        ));
    }
    if is_series {
        steps.push((
            crate::tui::state::DetailsPane::Seasons,
            if compact {
                format!("S{}", state.selected_season)
            } else {
                format!("Season {}", state.selected_season)
            },
        ));

        let ep_label = if compact {
            format!("E{}", state.selected_episode)
        } else {
            format!("Episode {}", state.selected_episode)
        };

        steps.push((crate::tui::state::DetailsPane::Episodes, ep_label));
    }
    steps.push((
        crate::tui::state::DetailsPane::Streams,
        format!("Streams: {streams_count}"),
    ));

    if area.width < 60 {
        let position = steps
            .iter()
            .position(|(pane, _)| *pane == state.details_pane)
            .unwrap_or(0);
        let label = steps
            .get(position)
            .map(|(_, label)| label.as_str())
            .unwrap_or("Streams");
        let text = format!("{label}  ·  {}/{}", position + 1, steps.len());
        frame.render_widget(
            Paragraph::new(text)
                .style(focused_title_style(theme))
                .alignment(Alignment::Center),
            area,
        );
        return;
    }

    let mut spans = Vec::new();
    let active_index = steps
        .iter()
        .position(|(pane, _)| *pane == state.details_pane)
        .unwrap_or(0);
    for (index, (pane, label)) in steps.iter().enumerate() {
        if index > 0 {
            spans.push(Span::styled(
                if state.basic_terminal {
                    " > "
                } else {
                    "  ›  "
                },
                theme.overlay0,
            ));
        }
        if *pane == state.details_pane {
            spans.push(Span::styled(
                focus_title_marker(state.basic_terminal),
                theme.lavender.add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(label.clone(), focused_title_style(theme)));
        } else {
            spans.push(Span::styled(
                label.clone(),
                if index < active_index {
                    theme.text
                } else {
                    theme.overlay0
                },
            ));
        }
    }
    frame.render_widget(
        Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
        area,
    );
}

fn stream_table_header_spans(width: u16, theme: &Theme) -> Vec<Span<'static>> {
    let header_style = theme.overlay0.add_modifier(Modifier::BOLD);
    if width < 58 {
        vec![Span::styled("  RES     SIZE     RELEASE", header_style)]
    } else if width < 85 {
        vec![Span::styled(
            "  RES     SIZE     CODEC   RELEASE",
            header_style,
        )]
    } else if width < 115 {
        vec![Span::styled(
            "  RES     SIZE     MEDIA TAGS     DURATION   RELEASE",
            header_style,
        )]
    } else {
        vec![Span::styled(
            "  RES     SIZE     MEDIA TAGS             DURATION   UPLOADER        RELEASE",
            header_style,
        )]
    }
}

fn footer_group(
    key: &'static str,
    action: &str,
    prominent: bool,
    theme: &Theme,
) -> Vec<Span<'static>> {
    vec![
        Span::styled("[", theme.overlay0),
        Span::styled(key, theme.shortcut),
        Span::styled("] ", theme.overlay0),
        Span::styled(
            action.to_string(),
            if prominent {
                theme.text
            } else {
                theme.subtext1
            },
        ),
        Span::raw("   "),
    ]
}

fn details_footer(
    state: &AppState,
    theme: &Theme,
    width: u16,
) -> (Vec<Span<'static>>, Vec<Span<'static>>) {
    let compact = width < DETAILS_FOOTER_SPLIT_THRESHOLD;
    let is_streams = state.details_pane == crate::tui::state::DetailsPane::Streams;
    let is_languages = state.details_pane == crate::tui::state::DetailsPane::Languages;
    let is_seasons = state.details_pane == crate::tui::state::DetailsPane::Seasons;
    let is_episodes = state.details_pane == crate::tui::state::DetailsPane::Episodes;

    let is_favorited = if let Some(details) = &state.selected_details {
        let details_subject_id = state.active_subject_id.as_deref().unwrap_or("");
        let title = details.get("title").and_then(|t| t.as_str()).unwrap_or("");
        let type_val = crate::tui::state::stype(details);
        let year = details
            .get("releaseDate")
            .or_else(|| details.get("year"))
            .or_else(|| details.get("releaseInfo"))
            .and_then(|y| y.as_str())
            .unwrap_or("N/A");
        state
            .favorites
            .is_favorite(&crate::models::SubjectIdentity {
                provider: subject_provider(state, details_subject_id).cache_key(),
                subject_id: details_subject_id,
                title,
                stype: type_val,
                release_year: year,
            })
    } else {
        false
    };
    let fav_label = if is_favorited {
        "Unfavorite"
    } else {
        "Favorite"
    };

    let mut primary = Vec::new();
    let mut secondary = Vec::new();

    if is_streams {
        primary.extend(footer_group("Enter", "Play", true, theme));
        primary.extend(footer_group(
            "d",
            if compact { "Save" } else { "Download" },
            false,
            theme,
        ));
        secondary.extend(footer_group("f", fav_label, false, theme));
        if !state.subtitle_list.is_empty() {
            secondary.extend(footer_group(
                "s",
                if compact { "Subs" } else { "Subtitles" },
                false,
                theme,
            ));
        }
        secondary.extend(footer_group("Esc", "Back", false, theme));
    } else if is_languages {
        primary.extend(footer_group("Enter", "Select", true, theme));
        primary.extend(footer_group("f", fav_label, false, theme));
        secondary.extend(footer_group("Tab", "Streams", false, theme));
        secondary.extend(footer_group("Esc", "Back", false, theme));
    } else if is_seasons {
        primary.extend(footer_group("Enter", "Select", true, theme));
        primary.extend(footer_group(
            "d",
            if compact {
                "Download"
            } else {
                "Download Season"
            },
            false,
            theme,
        ));
        primary.extend(footer_group("f", fav_label, false, theme));
        secondary.extend(footer_group("Tab", "Streams", false, theme));
        secondary.extend(footer_group("Esc", "Back", false, theme));
    } else if is_episodes {
        primary.extend(footer_group("Enter", "Select", true, theme));
        primary.extend(footer_group(
            "d",
            if compact {
                "Download"
            } else {
                "Download Episode"
            },
            false,
            theme,
        ));
        primary.extend(footer_group("f", fav_label, false, theme));
        secondary.extend(footer_group("Tab", "Streams", false, theme));
        secondary.extend(footer_group("Esc", "Back", false, theme));
    } else {
        primary.extend(footer_group("Enter", "Select", true, theme));
        primary.extend(footer_group("f", fav_label, false, theme));
        secondary.extend(footer_group("Tab", "Streams", false, theme));
        secondary.extend(footer_group("Esc", "Back", false, theme));
    }
    if let Some(last) = secondary.last_mut() {
        *last = Span::raw("");
    }
    (primary, secondary)
}

fn render_scroll_indicator(
    frame: &mut Frame,
    area: Rect,
    content_length: usize,
    position: usize,
    theme: &Theme,
    basic_terminal: bool,
) {
    let scroll_area = area.inner(ratatui::layout::Margin {
        vertical: 1,
        horizontal: 0,
    });
    crate::tui::widgets::render_scrollbar(
        frame,
        scroll_area,
        content_length,
        scroll_area.height as usize,
        position,
        theme,
        basic_terminal,
    );
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::widgets::{MediaTags, render_media_tag_spans};
    #[test]
    fn test_stream_loading_spinner_frames() {
        assert_eq!(stream_loading_spinner(0, false), "⠋");
        assert_eq!(stream_loading_spinner(1, false), "⠙");
        assert_eq!(stream_loading_spinner(9, false), "⠏");
        assert_eq!(stream_loading_spinner(10, false), "⠋");
        assert_eq!(stream_loading_spinner(0, true), "..");
    }

    #[test]
    fn test_resolution_badge_spans() {
        let theme = Theme::mocha();
        let spans_4k = resolution_badge_spans(2160, &theme, false);
        assert_eq!(spans_4k[0].content, "  4K   ");

        let spans_1080 = resolution_badge_spans(1080, &theme, false);
        assert_eq!(spans_1080[0].content, " 1080p ");

        let spans_720 = resolution_badge_spans(720, &theme, false);
        assert_eq!(spans_720[0].content, " 720p  ");

        let spans_sd = resolution_badge_spans(480, &theme, false);
        assert_eq!(spans_sd[0].content, " 480p  ");

        let basic_4k = resolution_badge_spans(2160, &theme, true);
        assert_eq!(basic_4k[0].content.trim(), "[4K]");
    }

    #[test]
    fn test_stream_list_tabular_alignment_and_headers() {
        let backend = ratatui::backend::TestBackend::new(120, 30);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        let mut state = AppState {
            selected_details: Some(serde_json::json!({
                "title": "Dune: Part Two",
                "subjectType": 1
            })),
            selected_resources: Some(serde_json::json!({
                "list": [
                    {
                        "resolution": 1080,
                        "size": "1073741824",
                        "codecName": "hevc",
                        "duration": 6533,
                        "uploadBy": "Pahe.in",
                        "fileName": "Dune.Part.Two.2024.1080p.WEBRip.x265"
                    },
                    {
                        "resolution": 1080,
                        "size": "250000000",
                        "codecName": "h264",
                        "duration": 6533,
                        "uploadBy": "GalaxyRG",
                        "fileName": "Dune.Part.Two.2024.1080p.WEBRip.x264"
                    },
                    {
                        "resolution": 720,
                        "size": "500000000",
                        "codecName": "hevc",
                        "duration": 6533,
                        "uploadBy": "PSA",
                        "fileName": "Dune.Part.Two.2024.720p.WEBRip.x265"
                    }
                ]
            })),
            details_pane: crate::tui::state::DetailsPane::Streams,
            ..Default::default()
        };
        let theme = Theme::mocha();

        terminal
            .draw(|frame| {
                draw(frame, frame.area(), &mut state, &theme);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = buffer
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(content.contains("1080p · 2 options"));
        assert!(content.contains("720p · 1 option"));
        assert!(content.contains("1080p"));
        assert!(content.contains("1.0GB"));
        assert!(content.contains("HEVC"));
        assert!(content.contains("1:48:53"));
        assert!(content.contains("Pahe.in"));
    }

    #[test]
    fn test_extract_media_tags() {
        let tags = extract_media_tags(
            "Dune.Part.Two.2024.2160p.UHD.BluRay.x265.TrueHD.7.1.Atmos.DV.HDR-FLUX",
            "HEVC",
        );
        assert_eq!(tags.hdr, Some("DV"));
        assert_eq!(tags.audio, Some("ATMOS"));
        assert_eq!(tags.codec, Some("HEVC"));
        assert_eq!(tags.source, Some("BluRay"));

        let tags_web = extract_media_tags(
            "Movie.Title.2023.1080p.HDR10Plus.WEB-DL.DDP5.1.H.264",
            "H264",
        );
        assert_eq!(tags_web.hdr, Some("HDR10+"));
        assert_eq!(tags_web.audio, Some("5.1"));
        assert_eq!(tags_web.codec, Some("H.264"));
        assert_eq!(tags_web.source, Some("WEB-DL"));
    }

    #[test]
    fn test_render_media_tag_spans() {
        let theme = Theme::mocha();
        let tags = MediaTags {
            hdr: Some("HDR"),
            audio: Some("5.1"),
            codec: Some("HEVC"),
            source: Some("REMUX"),
        };
        let spans = render_media_tag_spans(&tags, &theme, false);
        assert!(!spans.is_empty());
        let basic_spans = render_media_tag_spans(&tags, &theme, true);
        assert_eq!(basic_spans[0].content, "HDR");
    }

    #[test]
    fn test_details_error_state_rendering() {
        let backend = ratatui::backend::TestBackend::new(80, 24);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        let mut state = AppState {
            details_error: Some("Failed to fetch details: network timeout".to_string()),
            ..Default::default()
        };
        let theme = Theme::mocha();

        terminal
            .draw(|frame| {
                draw(frame, frame.area(), &mut state, &theme);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = buffer
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(content.contains("Error Loading Details"));
        assert!(content.contains("Retry fetch"));
        assert!(content.contains("Back"));
    }

    #[test]
    fn test_synopsis_wrap_clamping() {
        let backend = ratatui::backend::TestBackend::new(80, 24);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        let mut state = AppState {
            selected_details: Some(serde_json::json!({
                "title": "Test Movie",
                "description": "A very long synopsis that will definitely exceed the single line boundary and should be clamped cleanly with an ellipsis without overflowing the paragraph bounds."
            })),
            ..Default::default()
        };
        let theme = Theme::mocha();

        terminal
            .draw(|frame| {
                draw(frame, frame.area(), &mut state, &theme);
            })
            .unwrap();
    }

    #[test]
    fn test_details_footer_contextual_hints() {
        let theme = Theme::mocha();
        let mut state = AppState::default();

        state.details_pane = crate::tui::state::DetailsPane::Streams;
        state.subtitle_list = vec![("English".to_string(), "http://sub".to_string())];
        let (primary, secondary) = details_footer(&state, &theme, 100);
        let mut all_spans = primary;
        all_spans.extend(secondary);
        let footer_text: String = all_spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(footer_text.contains("[Enter] Play"));
        assert!(!footer_text.contains("[o] Open With"));
        assert!(footer_text.contains("[d] Download"));
        assert!(footer_text.contains("[f] Favorite"));
        assert!(footer_text.contains("[s] Subtitles"));
        assert!(footer_text.contains("[Esc] Back"));

        state.details_pane = crate::tui::state::DetailsPane::Seasons;
        let (primary, secondary) = details_footer(&state, &theme, 100);
        let mut all_spans = primary;
        all_spans.extend(secondary);
        let footer_text: String = all_spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(footer_text.contains("[Enter] Select"));
        assert!(footer_text.contains("[d] Download Season"));
        assert!(footer_text.contains("[f] Favorite"));
        assert!(footer_text.contains("[Tab] Streams"));
        assert!(footer_text.contains("[Esc] Back"));

        state.details_pane = crate::tui::state::DetailsPane::Episodes;
        let (primary, secondary) = details_footer(&state, &theme, 100);
        let mut all_spans = primary;
        all_spans.extend(secondary);
        let footer_text: String = all_spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(footer_text.contains("[Enter] Select"));
        assert!(footer_text.contains("[d] Download Episode"));
        assert!(footer_text.contains("[f] Favorite"));
        assert!(footer_text.contains("[Tab] Streams"));
        assert!(footer_text.contains("[Esc] Back"));
    }

    #[test]
    fn test_episode_list_zero_padding_and_alignment() {
        let backend = ratatui::backend::TestBackend::new(100, 30);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        let mut state = AppState {
            active_subject_id: Some("test_series".to_string()),
            selected_details: Some(serde_json::json!({
                "id": "test_series",
                "title": "Test Series",
                "subjectType": 2
            })),
            available_seasons: vec![serde_json::json!({ "se": 1, "maxEp": 12 })],
            available_episode_numbers: vec![vec![1, 2, 10]],
            details_pane: crate::tui::state::DetailsPane::Episodes,
            ..Default::default()
        };
        let theme = Theme::mocha();

        terminal
            .draw(|frame| {
                draw(frame, frame.area(), &mut state, &theme);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = buffer
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(content.contains("EP 01"));
        assert!(content.contains("EP 02"));
        assert!(content.contains("EP 10"));
    }

    #[test]
    fn test_responsive_selector_panes_single_pane_on_narrow_screen() {
        use crate::tui::state::DetailsPane;
        let all_panes = vec![
            DetailsPane::Languages,
            DetailsPane::Seasons,
            DetailsPane::Episodes,
        ];

        // Width < 85 should only return the active pane
        let narrow = visible_selector_panes(&all_panes, DetailsPane::Languages, 50);
        assert_eq!(narrow, vec![DetailsPane::Languages]);

        let narrow_season = visible_selector_panes(&all_panes, DetailsPane::Seasons, 60);
        assert_eq!(narrow_season, vec![DetailsPane::Seasons]);

        // Width >= 85 should return all 3 panes
        let wide = visible_selector_panes(&all_panes, DetailsPane::Languages, 100);
        assert_eq!(wide.len(), 3);
    }

    #[test]
    fn test_pane_title_compact_width_does_not_overflow() {
        use crate::tui::state::DetailsPane;
        let state = AppState {
            selected_details: Some(serde_json::json!({
                "dubs": [
                    { "lanName": "Original" },
                    { "lanName": "Hindi" }
                ]
            })),
            ..Default::default()
        };

        let title_wide = pane_title("Audio", 2, DetailsPane::Languages, true, &state, 40);
        assert!(title_wide.to_string().contains("1/"));

        let title_narrow = pane_title("Audio", 2, DetailsPane::Languages, true, &state, 12);
        assert!(!title_narrow.to_string().contains("1/"));
        assert!(title_narrow.to_string().contains("Audio"));
    }

    #[test]
    fn test_details_screen_renders_in_narrow_terminal_without_clipping() {
        let backend = ratatui::backend::TestBackend::new(50, 35);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        let mut state = AppState {
            active_subject_id: Some("breaking_bad".to_string()),
            selected_details: Some(serde_json::json!({
                "id": "breaking_bad",
                "title": "Breaking Bad",
                "subjectType": 2,
                "year": "2008",
                "genres": ["Crime", "Drama", "Thriller"],
                "description": "A chemistry teacher diagnosed with inoperable lung cancer turns to manufacturing and selling methamphetamine.",
                "dubs": [
                    { "lanName": "Original", "subjectId": "1" },
                    { "lanName": "Hindi", "subjectId": "2" },
                    { "lanName": "Spanish (LA)", "subjectId": "3" },
                    { "lanName": "Portuguese (Brazil)", "subjectId": "4" }
                ]
            })),
            available_seasons: vec![serde_json::json!({ "se": 1, "maxEp": 7 })],
            available_episode_numbers: vec![vec![1, 2, 3]],
            details_pane: crate::tui::state::DetailsPane::Languages,
            ..Default::default()
        };
        let theme = Theme::mocha();

        terminal
            .draw(|frame| {
                draw(frame, frame.area(), &mut state, &theme);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = buffer
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(content.contains("Breaking Bad"));
        assert!(content.contains("Portuguese (Brazil)"));
    }
}
