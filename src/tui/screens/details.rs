use crate::tui::{state::AppState, theme::Theme};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Wrap,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DetailsLayoutTier {
    Wide,
    Medium,
    Narrow,
    Tiny,
}

impl DetailsLayoutTier {
    fn for_area(area: Rect) -> Self {
        if area.width < 60 || area.height < 24 {
            Self::Tiny
        } else if area.width < 80 {
            Self::Narrow
        } else if area.width < 120 {
            Self::Medium
        } else {
            Self::Wide
        }
    }

    fn header_height(self, area: Rect) -> u16 {
        match self {
            Self::Wide => area.height.saturating_sub(18).clamp(10, 12),
            Self::Medium => area.height.saturating_sub(17).clamp(9, 11),
            Self::Narrow => area.height.saturating_sub(16).clamp(7, 9),
            Self::Tiny => area.height.saturating_sub(12).clamp(4, 6),
        }
    }

    fn footer_height(self) -> u16 {
        if matches!(self, Self::Wide) { 1 } else { 2 }
    }
}

pub fn draw(frame: &mut Frame, area: Rect, state: &mut AppState, theme: &Theme) {
    let tier = DetailsLayoutTier::for_area(area);
    let header_height = tier.header_height(area);
    let footer_height = tier.footer_height();
    let chunks = Layout::vertical([
        Constraint::Length(header_height),
        Constraint::Length(1),
        Constraint::Min(5),
        Constraint::Length(footer_height),
    ])
    .split(area);
    let workflow_area = chunks[1];
    let bottom_area = chunks[2];

    let details_json = match &state.selected_details {
        Some(d) => d,
        None => {
            let spinner = if state.basic_terminal {
                let frames = ['-', '\\', '|', '/'];
                frames[(state.tick_count as usize) % frames.len()]
            } else {
                let frames = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
                frames[(state.tick_count as usize) % frames.len()]
            };

            let vertical_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(45),
                    Constraint::Length(1),
                    Constraint::Percentage(50),
                ])
                .split(area);

            let loading_p = Paragraph::new(format!("{} Loading details...", spinner))
                .alignment(ratatui::layout::Alignment::Center)
                .style(theme.text_dim);

            frame.render_widget(loading_p, vertical_chunks[1]);
            return;
        }
    };

    let raw_title = details_json
        .get("title")
        .and_then(|t| t.as_str())
        .unwrap_or("Unknown Title");
    let title = crate::tui::app::clean_moviebox_title(raw_title);
    let intro = details_json
        .get("description")
        .and_then(|d| d.as_str())
        .or_else(|| details_json.get("intro").and_then(|i| i.as_str()))
        .unwrap_or("No description available.");
    let year = details_json
        .get("releaseDate")
        .and_then(|y| y.as_str())
        .or_else(|| details_json.get("year").and_then(|y| y.as_str()))
        .unwrap_or("N/A");
    let type_val = details_json
        .get("subjectType")
        .and_then(|s| s.as_i64())
        .or_else(|| details_json.get("stype").and_then(|s| s.as_i64()))
        .unwrap_or(1);
    let type_str = if type_val == 2 { "Series" } else { "Movie" };

    let genres = details_json
        .get("genre")
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
        .border_type(if state.basic_terminal {
            ratatui::widgets::BorderType::Plain
        } else {
            ratatui::widgets::BorderType::Rounded
        })
        .border_style(theme.border)
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

    let inner_area = details_block.inner(chunks[0]);
    frame.render_widget(details_block.clone(), chunks[0]);

    let show_poster = !matches!(tier, DetailsLayoutTier::Tiny)
        && inner_area.height >= 6
        && inner_area.width >= 60;
    let poster_width = if show_poster && state.image_supported {
        ((inner_area.height as f32 * 0.78).ceil() as u16).clamp(8, 18)
    } else if show_poster {
        16.min(inner_area.width / 4)
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

    let poster_height = ((poster_width as f32) / 1.33).ceil() as u16;
    let poster_area = ratatui::layout::Rect {
        height: h_chunks[0].height.min(poster_height),
        ..h_chunks[0]
    };
    let right_area = h_chunks[2];

    if show_poster && state.image_supported {
        if let Some(img) = &state.poster_image {
            if state.poster_protocol.as_ref().map(|(r, _)| *r) != Some(poster_area)
                && let Some(picker) = &mut state.image_picker
            {
                let size = ratatui::layout::Size::new(poster_area.width, poster_area.height);
                if let Ok(proto) =
                    picker.new_protocol(img.clone(), size, ratatui_image::Resize::Fit(None))
                {
                    state.poster_protocol = Some((poster_area, proto));
                }
            }
            if let Some((_, proto)) = &state.poster_protocol {
                if !state.show_help {
                    frame.render_widget(ratatui_image::Image::new(proto), poster_area);
                }
            }
        } else {
            let current_spinner = if state.basic_terminal {
                let frames = ['-', '\\', '|', '/'];
                frames[(state.tick_count as usize) % frames.len()]
            } else {
                let frames = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
                frames[(state.tick_count as usize) % frames.len()]
            };

            let placeholder_block = Block::default()
                .borders(Borders::ALL)
                .border_style(theme.muted);

            let inner = placeholder_block.inner(poster_area);

            let (pad, msg) = if state.is_loading {
                let p = "\n".repeat((inner.height.saturating_sub(1) / 2) as usize);
                (p, format!("{}\nLoading Art", current_spinner))
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
        let lines = if inner.height >= 5 {
            let pad_top = "\n".repeat((inner.height.saturating_sub(5) / 2) as usize);
            format!("{pad_top}Poster preview\nunsupported\n\nUse a graphics-\ncapable terminal")
        } else {
            "Poster\nunsupported".to_string()
        };

        let placeholder = Paragraph::new(lines)
            .style(theme.text_dim)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .block(placeholder_block);
        frame.render_widget(placeholder, poster_area);
    }

    let title_line = Line::from(vec![
        Span::styled(
            title.to_string(),
            theme.text.add_modifier(ratatui::style::Modifier::BOLD),
        ),
        Span::styled("   ", theme.text),
        Span::styled(format!("★ IMDb {}", imdb_rating), theme.rating),
    ]);

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
    let meta_line = Line::from(vec![Span::styled(metadata.join(" • "), theme.text)]);

    let genre_line = Line::from(vec![Span::styled(genres.to_string(), theme.text_dim)]);

    let mut top_meta = vec![
        title_line,
        meta_line,
        genre_line,
        Line::from(vec![Span::styled(
            tagline.unwrap_or_default(),
            theme
                .text_dim
                .add_modifier(ratatui::style::Modifier::ITALIC),
        )]),
        Line::from(vec![Span::styled("Synopsis", theme.text)]),
    ];
    if matches!(tier, DetailsLayoutTier::Tiny) {
        top_meta.truncate(3);
    } else if matches!(tier, DetailsLayoutTier::Narrow) {
        top_meta.truncate(4);
    }

    let meta_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(top_meta.len() as u16),
            Constraint::Min(0),
        ])
        .split(right_area);

    let meta_p = Paragraph::new(top_meta).wrap(Wrap { trim: true });
    frame.render_widget(meta_p, meta_chunks[0]);

    let synopsis_capacity =
        (meta_chunks[1].width as usize).saturating_mul(meta_chunks[1].height as usize);
    let synopsis = truncate_with_ellipsis(intro, synopsis_capacity);
    let syn_lines = vec![Line::from(vec![Span::styled(
        synopsis,
        theme.text_dim.add_modifier(ratatui::style::Modifier::DIM),
    )])];
    let intro_p = Paragraph::new(syn_lines).wrap(Wrap { trim: true });
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
        if matches!(tier, DetailsLayoutTier::Narrow | DetailsLayoutTier::Tiny) {
            available_selector_panes
                .iter()
                .copied()
                .filter(|pane| *pane == state.details_pane)
                .collect::<Vec<_>>()
        } else {
            available_selector_panes
        };

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
            .min(4) as u16
            + 2
    };

    let lower_chunks = Layout::vertical([Constraint::Length(selector_height), Constraint::Min(3)])
        .split(bottom_area);
    let selector_area = lower_chunks[0];
    let streams_area = lower_chunks[1];

    let selector_chunks = if visible_selector_panes.is_empty() {
        Vec::new()
    } else {
        Layout::horizontal(vec![
            Constraint::Ratio(
                1,
                visible_selector_panes.len() as u32
            );
            visible_selector_panes.len()
        ])
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
                    let mut name = if lang.to_lowercase().starts_with("original") {
                        "Original".to_string()
                    } else {
                        lang.replace("dub", "")
                            .replace("Dub", "")
                            .trim()
                            .to_string()
                    };
                    if name.to_lowercase() == "ptbr" {
                        name = "Portuguese (BR)".to_string();
                    }
                    lang_items.push(ListItem::new(name).style(theme.text));
                }
            }
        }
        let language_count = lang_items.len();

        let language_focused = state.details_pane == crate::tui::state::DetailsPane::Languages;
        let lang_border = if language_focused {
            theme.border_focus
        } else {
            theme.border
        };
        let lang_list = List::new(lang_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(if state.basic_terminal {
                        ratatui::widgets::BorderType::Plain
                    } else {
                        ratatui::widgets::BorderType::Rounded
                    })
                    .title(pane_title(
                        "Audio",
                        language_count,
                        crate::tui::state::DetailsPane::Languages,
                        language_focused,
                        state,
                    ))
                    .title_style(if language_focused {
                        theme.accent
                    } else {
                        theme.text_dim
                    })
                    .border_style(lang_border)
                    .padding(ratatui::widgets::Padding::horizontal(1)),
            )
            .highlight_style(selection_style(language_focused, theme))
            .highlight_symbol(selection_symbol(language_focused, state.basic_terminal));

        if let Some(area) = lang_area {
            frame.render_stateful_widget(lang_list, area, &mut state.language_list_state);
            render_scroll_indicator(
                frame,
                area,
                language_count,
                state.language_list_state.selected().unwrap_or(0),
                theme,
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
            theme.border_focus
        } else {
            theme.border
        };
        let seasons_list = List::new(seasons_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(if state.basic_terminal {
                        ratatui::widgets::BorderType::Plain
                    } else {
                        ratatui::widgets::BorderType::Rounded
                    })
                    .title(pane_title(
                        "Seasons",
                        state.available_seasons.len(),
                        crate::tui::state::DetailsPane::Seasons,
                        seasons_focused,
                        state,
                    ))
                    .title_style(if seasons_focused {
                        theme.accent
                    } else {
                        theme.text_dim
                    })
                    .border_style(seasons_border)
                    .padding(ratatui::widgets::Padding::horizontal(1)),
            )
            .highlight_style(selection_style(seasons_focused, theme))
            .highlight_symbol(selection_symbol(seasons_focused, state.basic_terminal));

        if let Some(area) = seasons_area {
            frame.render_stateful_widget(seasons_list, area, &mut state.season_list_state);
            render_scroll_indicator(
                frame,
                area,
                state.available_seasons.len(),
                state.season_list_state.selected().unwrap_or(0),
                theme,
            );
        }

        let ep_items: Vec<ListItem> = if let Some(ep_numbers) = state
            .available_episode_numbers
            .get(state.season_list_state.selected().unwrap_or(0))
        {
            ep_numbers
                .iter()
                .map(|&ep| ListItem::new(format!("Episode {}", ep)).style(theme.text))
                .collect()
        } else {
            vec![]
        };
        let episode_count = ep_items.len();

        let episodes_focused = state.details_pane == crate::tui::state::DetailsPane::Episodes;
        let eps_border = if episodes_focused {
            theme.border_focus
        } else {
            theme.border
        };
        let eps_list = List::new(ep_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(if state.basic_terminal {
                        ratatui::widgets::BorderType::Plain
                    } else {
                        ratatui::widgets::BorderType::Rounded
                    })
                    .title(pane_title(
                        "Episodes",
                        episode_count,
                        crate::tui::state::DetailsPane::Episodes,
                        episodes_focused,
                        state,
                    ))
                    .title_style(if episodes_focused {
                        theme.accent
                    } else {
                        theme.text_dim
                    })
                    .border_style(eps_border)
                    .padding(ratatui::widgets::Padding::horizontal(1)),
            )
            .highlight_style(selection_style(episodes_focused, theme))
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
            );
        }
    }

    let streams_border = if state.details_pane == crate::tui::state::DetailsPane::Streams {
        theme.border_focus
    } else {
        theme.border
    };

    let streams_title = if streams_count > 0 {
        let selected = state
            .resource_list_state
            .selected()
            .unwrap_or(0)
            .min(streams_count.saturating_sub(1));
        format!(
            " Streams · {} available · {}/{} ",
            streams_count,
            selected + 1,
            streams_count
        )
    } else {
        " Streams ".to_string()
    };

    let streams_block = Block::default()
        .borders(Borders::ALL)
        .border_type(if state.basic_terminal {
            ratatui::widgets::BorderType::Plain
        } else {
            ratatui::widgets::BorderType::Rounded
        })
        .title(ratatui::text::Line::from(streams_title).alignment(Alignment::Left))
        .title_style(
            if state.details_pane == crate::tui::state::DetailsPane::Streams {
                theme.accent
            } else {
                theme.text_dim
            },
        )
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
                    *quality_counts.entry(resolution).or_insert(0usize) += 1;
                }

                let list_items: Vec<ListItem> = list
                    .iter()
                    .enumerate()
                    .map(|(i, file)| {
                        let resolution =
                            file.get("resolution").and_then(|r| r.as_i64()).unwrap_or(0);
                        let quality_str = format!("{}p", resolution);

                        let is_first_of_quality = quality_str != prev_quality;
                        prev_quality = quality_str.clone();

                        let codec = file
                            .get("codecName")
                            .and_then(|c| c.as_str())
                            .unwrap_or("None");
                        let upload_by = file
                            .get("uploadBy")
                            .and_then(|u| u.as_str())
                            .unwrap_or("Unknown");
                        let size_str = file.get("size").and_then(|s| s.as_str()).unwrap_or("0");

                        let duration = file.get("duration").and_then(|d| d.as_u64()).unwrap_or(0);
                        let duration_str = if duration > 0 {
                            let hours = duration / 3600;
                            let mins = (duration % 3600) / 60;
                            let secs = duration % 60;
                            if hours > 0 {
                                format!("{:02}:{:02}:{:02}", hours, mins, secs)
                            } else {
                                format!("{:02}:{:02}", mins, secs)
                            }
                        } else {
                            "--:--".to_string()
                        };

                        let size_formatted = if let Ok(bytes) = size_str.parse::<f64>() {
                            let mb = bytes / 1024.0 / 1024.0;
                            if mb > 1024.0 {
                                format!("{:.1}GB", mb / 1024.0)
                            } else {
                                format!("{:.0}MB", mb)
                            }
                        } else {
                            "Unknown".to_string()
                        };

                        let is_selected = Some(i) == selected_idx;
                        let pointer = if is_selected {
                            if state.basic_terminal { "> " } else { "▌ " }
                        } else {
                            "  "
                        };

                        let stream_style = if is_selected {
                            selection_style(
                                state.details_pane
                                    == crate::tui::state::DetailsPane::Streams,
                                theme,
                            )
                        } else {
                            theme.text_dim
                        };

                        let is_fourk = file.get("_fourk_release").is_some();
                        let language = file
                            .get("language")
                            .and_then(|value| value.as_str())
                            .unwrap_or("Unknown");
                        let source_count = file
                            .get("sourceCount")
                            .and_then(|value| value.as_u64())
                            .unwrap_or(0);
                        let stream_width = streams_area.width.saturating_sub(6) as usize;
                        let codec = codec.to_uppercase();
                        let metadata = if is_fourk && stream_width >= 58 {
                            format!(
                                "{size_formatted:<9}{codec:<8}{language:<16}{source_count} mirror{}",
                                if source_count == 1 { "" } else { "s" }
                            )
                        } else if is_fourk && stream_width >= 38 {
                            format!("{size_formatted:<9}{codec:<8}{language}")
                        } else if is_fourk {
                            format!("{size_formatted:<9}{codec}")
                        } else if stream_width >= 64 {
                            let fixed_width = 9 + 8 + 12;
                            let uploader = crate::tui::text::truncate_width(
                                upload_by,
                                stream_width.saturating_sub(fixed_width).max(4),
                            );
                            format!("{size_formatted:<9}{codec:<8}{duration_str:<12}{uploader}")
                        } else if stream_width >= 38 {
                            format!("{size_formatted:<9}{codec:<8}{duration_str}")
                        } else {
                            format!("{size_formatted:<9}{codec}")
                        };
                        let metadata = if is_selected {
                            format!("{metadata:<width$}", width = stream_width.saturating_sub(2))
                        } else {
                            metadata
                        };
                        let stream_line = ratatui::text::Line::from(vec![
                            ratatui::text::Span::styled(pointer, stream_style),
                            ratatui::text::Span::styled(metadata, stream_style),
                        ]);

                        let mut lines = vec![];
                        if is_first_of_quality {
                            if i > 0 {
                                lines.push(ratatui::text::Line::from(""));
                            }
                            let option_count =
                                quality_counts.get(&resolution).copied().unwrap_or(1);
                            lines.push(ratatui::text::Line::from(
                                ratatui::text::Span::styled(
                                    format!(
                                        "{} · {} option{}",
                                        quality_str,
                                        option_count,
                                        if option_count == 1 { "" } else { "s" }
                                    ),
                                    theme.accent,
                                ),
                            ));
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
                    let mut previous = None;
                    for file in list.iter().take(selected.saturating_add(1)) {
                        let resolution = file
                            .get("resolution")
                            .and_then(|value| value.as_i64())
                            .unwrap_or(0);
                        if previous != Some(resolution) {
                            headings += 1;
                            previous = Some(resolution);
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
                );
            } else {
                let has_multiple_dubs = state
                    .selected_details
                    .as_ref()
                    .and_then(|d| d.get("dubs"))
                    .and_then(|d| d.as_array())
                    .is_some_and(|a| a.len() > 1);
                let msg = if has_multiple_dubs && !state.language_chosen {
                    "Choose an audio track to load streams."
                } else {
                    "No streams found — press r to retry."
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

            let is_busy = state.is_loading || state.is_fetching_streams;

            let msg = if is_busy {
                let spinner = if state.basic_terminal {
                    let frames = ['-', '\\', '|', '/'];
                    frames[(state.tick_count as usize) % frames.len()]
                } else {
                    let frames = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
                    frames[(state.tick_count as usize) % frames.len()]
                };
                format!("{} Loading streams...", spinner)
            } else if has_multiple_dubs && !state.language_chosen {
                "Choose an audio track to load streams.".to_string()
            } else if state.status_message.to_lowercase().contains("no streams")
                || state.status_message.to_lowercase().contains("error")
            {
                state.status_message.clone()
            } else {
                "No streams found — press r to retry.".to_string()
            };

            let style = if is_busy || (has_multiple_dubs && !state.language_chosen) {
                theme.text_dim
            } else {
                theme.error
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
    if !state.selected_resources.is_some() {
        frame.render_widget(streams_block, streams_area);
    }
    if state.subtitle_popup || state.is_download_subtitle_popup {
        let popup_width = 50;
        let popup_height = std::cmp::min(15, state.subtitle_list.len() as u16 + 2);

        let area = frame.area();
        let popup_area = ratatui::layout::Rect {
            x: area.width.saturating_sub(popup_width) / 2,
            y: area.height.saturating_sub(popup_height) / 2,
            width: popup_width,
            height: popup_height,
        };

        crate::tui::clear_area(frame, popup_area, theme);

        let items: Vec<ratatui::widgets::ListItem> = state
            .subtitle_list
            .iter()
            .map(|(name, _)| ratatui::widgets::ListItem::new(name.clone()))
            .collect();

        let title = if state.is_download_subtitle_popup {
            " Select Subtitle to Download "
        } else {
            " Select Subtitle "
        };

        let list = ratatui::widgets::List::new(items)
            .block(
                ratatui::widgets::Block::default()
                    .title(title)
                    .title_style(theme.title)
                    .borders(ratatui::widgets::Borders::ALL)
                    .border_type(if state.basic_terminal {
                        ratatui::widgets::BorderType::Plain
                    } else {
                        ratatui::widgets::BorderType::Rounded
                    })
                    .border_style(theme.border),
            )
            .highlight_style(theme.highlight)
            .highlight_symbol(if state.basic_terminal { "> " } else { "▌ " });

        frame.render_stateful_widget(list, popup_area, &mut state.subtitle_list_state);
    }

    if state.player_picker_popup {
        let popup_width = 24;
        let popup_height = std::cmp::min(15, state.available_players.len() as u16 + 2);

        let area = frame.area();
        let popup_area = ratatui::layout::Rect {
            x: area.width.saturating_sub(popup_width) / 2,
            y: area.height.saturating_sub(popup_height) / 2,
            width: popup_width,
            height: popup_height,
        };

        crate::tui::clear_area(frame, popup_area, theme);

        let items: Vec<ratatui::widgets::ListItem> = state
            .available_players
            .iter()
            .map(|k| {
                let text = match k {
                    crate::tui::state::PlayerKind::Mpv => "mpv",
                    crate::tui::state::PlayerKind::Iina => "IINA",
                    crate::tui::state::PlayerKind::Vlc => "VLC",
                };
                ratatui::widgets::ListItem::new(text)
            })
            .collect();

        let list = ratatui::widgets::List::new(items)
            .block(
                ratatui::widgets::Block::default()
                    .title(" Open With ")
                    .title_style(theme.title)
                    .borders(ratatui::widgets::Borders::ALL)
                    .border_type(if state.basic_terminal {
                        ratatui::widgets::BorderType::Plain
                    } else {
                        ratatui::widgets::BorderType::Rounded
                    })
                    .border_style(theme.border),
            )
            .highlight_style(theme.highlight)
            .highlight_symbol(if state.basic_terminal { "> " } else { "▌ " });

        frame.render_stateful_widget(list, popup_area, &mut state.player_picker_state);
    }

    let (mut primary_footer, secondary_footer) = details_footer(state, theme, area.width);
    let footer_p = if matches!(tier, DetailsLayoutTier::Wide) {
        primary_footer.extend(secondary_footer);
        Paragraph::new(Line::from(primary_footer))
    } else {
        Paragraph::new(vec![
            Line::from(primary_footer),
            Line::from(secondary_footer),
        ])
    }
    .alignment(Alignment::Center);
    frame.render_widget(footer_p, chunks[3]);

    if state.show_season_download_confirm {
        let popup_width = 50;
        let popup_height = 7;
        let popup_area = Rect::new(
            (area.width.saturating_sub(popup_width)) / 2,
            (area.height.saturating_sub(popup_height)) / 2,
            popup_width,
            popup_height,
        );
        crate::tui::clear_area(frame, popup_area, theme);

        let season_idx = state.selected_season;
        let eps_count = if season_idx > 0 && season_idx <= state.available_episode_numbers.len() {
            state.available_episode_numbers[season_idx - 1].len()
        } else {
            0
        };

        let msg = format!(
            "Download all {} episodes in Season {}?",
            eps_count, season_idx
        );
        let yes_text = if state.season_download_confirm_yes_selected {
            "  < Yes >  "
        } else {
            "    Yes    "
        };
        let no_text = if !state.season_download_confirm_yes_selected {
            "  < No >  "
        } else {
            "    No    "
        };

        let confirm_block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme.border_focus)
            .title(" Season Download ")
            .title_alignment(Alignment::Center);

        let lines = vec![
            Line::from(""),
            Line::from(msg),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    yes_text,
                    if state.season_download_confirm_yes_selected {
                        theme.highlight
                    } else {
                        theme.text_dim
                    },
                ),
                Span::raw("    "),
                Span::styled(
                    no_text,
                    if !state.season_download_confirm_yes_selected {
                        theme.highlight
                    } else {
                        theme.text_dim
                    },
                ),
            ]),
        ];

        let p = Paragraph::new(lines)
            .block(confirm_block)
            .alignment(Alignment::Center)
            .style(theme.text);

        frame.render_widget(p, popup_area);
    } else if state.show_episode_download_confirm {
        let popup_width = 50;
        let popup_height = 7;
        let popup_area = Rect::new(
            (area.width.saturating_sub(popup_width)) / 2,
            (area.height.saturating_sub(popup_height)) / 2,
            popup_width,
            popup_height,
        );
        crate::tui::clear_area(frame, popup_area, theme);

        let season_idx = state.selected_season;
        let ep_idx = state.selected_episode;

        let msg = if type_val == 2 {
            format!("Download Episode {} of Season {}?", ep_idx, season_idx)
        } else {
            "Download this Movie?".to_string()
        };

        let yes_text = if state.episode_download_confirm_yes_selected {
            "  < Yes >  "
        } else {
            "    Yes    "
        };
        let no_text = if !state.episode_download_confirm_yes_selected {
            "  < No >  "
        } else {
            "    No    "
        };

        let confirm_block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme.border_focus)
            .title(" Download ")
            .title_alignment(Alignment::Center);

        let lines = vec![
            Line::from(""),
            Line::from(msg),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    yes_text,
                    if state.episode_download_confirm_yes_selected {
                        theme.highlight
                    } else {
                        theme.text_dim
                    },
                ),
                Span::raw("    "),
                Span::styled(
                    no_text,
                    if !state.episode_download_confirm_yes_selected {
                        theme.highlight
                    } else {
                        theme.text_dim
                    },
                ),
            ]),
        ];

        let p = Paragraph::new(lines)
            .block(confirm_block)
            .alignment(Alignment::Center)
            .style(theme.text);

        frame.render_widget(p, popup_area);
    }
}

fn truncate_with_ellipsis(value: &str, capacity: usize) -> String {
    if capacity == 0 {
        return String::new();
    }
    if crate::tui::text::width(value) <= capacity {
        return value.to_string();
    }
    if capacity == 1 {
        return "…".to_string();
    }
    format!(
        "{}…",
        crate::tui::text::truncate_width(value, capacity.saturating_sub(1))
    )
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
    }
    name
}

fn pane_title(
    label: &str,
    count: usize,
    pane: crate::tui::state::DetailsPane,
    focused: bool,
    state: &AppState,
) -> Line<'static> {
    let marker = if focused {
        if state.basic_terminal { "> " } else { "◆ " }
    } else {
        ""
    };
    let mut title = format!(" {marker}{label} · {count}");
    if focused {
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
        if let Some(position) = panes.iter().position(|candidate| *candidate == pane) {
            title.push_str(&format!("  {}/{}", position + 1, panes.len()));
        }
    }
    title.push(' ');
    Line::from(title)
}

fn selection_style(focused: bool, theme: &Theme) -> ratatui::style::Style {
    if focused {
        theme
            .highlight
            .add_modifier(Modifier::BOLD | Modifier::REVERSED)
    } else {
        theme.text.add_modifier(Modifier::BOLD)
    }
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
        let language = details
            .get("dubs")
            .and_then(|dubs| dubs.as_array())
            .and_then(|dubs| dubs.get(state.language_list_state.selected().unwrap_or(0)))
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
        steps.push((
            crate::tui::state::DetailsPane::Episodes,
            if compact {
                format!("E{}", state.selected_episode)
            } else {
                format!("Episode {}", state.selected_episode)
            },
        ));
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
                .style(theme.accent)
                .alignment(Alignment::Center),
            area,
        );
        return;
    }

    let mut spans = Vec::new();
    for (index, (pane, label)) in steps.iter().enumerate() {
        if index > 0 {
            spans.push(Span::styled(
                if state.basic_terminal {
                    " > "
                } else {
                    "  ›  "
                },
                theme.muted,
            ));
        }
        spans.push(Span::styled(
            label.clone(),
            if *pane == state.details_pane {
                theme.accent
            } else {
                theme.text_dim
            },
        ));
    }
    frame.render_widget(
        Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
        area,
    );
}

fn footer_group(key: &'static str, action: &'static str, theme: &Theme) -> Vec<Span<'static>> {
    vec![
        Span::styled("[", theme.text_dim),
        Span::styled(key, theme.shortcut),
        Span::styled("] ", theme.text_dim),
        Span::styled(action, theme.text),
        Span::raw("   "),
    ]
}

fn details_footer(
    state: &AppState,
    theme: &Theme,
    width: u16,
) -> (Vec<Span<'static>>, Vec<Span<'static>>) {
    let compact = width < 80;
    let very_compact = width < 45;
    let mut primary = footer_group("Tab", if compact { "Pane" } else { "Next pane" }, theme);
    primary.extend(footer_group("↑↓", "Move", theme));
    if !very_compact {
        primary.extend(footer_group(
            "Enter",
            if state.details_pane == crate::tui::state::DetailsPane::Streams {
                "Play"
            } else {
                "Select"
            },
            theme,
        ));
    }

    let mut secondary = Vec::new();
    if very_compact {
        secondary.extend(footer_group(
            "Enter",
            if state.details_pane == crate::tui::state::DetailsPane::Streams {
                "Play"
            } else {
                "Select"
            },
            theme,
        ));
    } else {
        if state.details_pane == crate::tui::state::DetailsPane::Streams {
            secondary.extend(footer_group(
                "o",
                if compact { "Open" } else { "Open with" },
                theme,
            ));
        }
        if !matches!(
            state.details_pane,
            crate::tui::state::DetailsPane::Languages
        ) {
            secondary.extend(footer_group(
                "d",
                if compact { "Save" } else { "Download" },
                theme,
            ));
        }
        if !very_compact {
            secondary.extend(footer_group(
                "r",
                if compact { "Retry" } else { "Refresh" },
                theme,
            ));
        }
    }
    secondary.extend(footer_group("Esc", "Back", theme));

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
) {
    let viewport_length = area.height.saturating_sub(2) as usize;
    if content_length <= viewport_length || viewport_length == 0 {
        return;
    }

    let mut state = ScrollbarState::default()
        .content_length(content_length)
        .viewport_content_length(viewport_length)
        .position(position);
    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .thumb_style(theme.accent)
        .track_style(theme.muted)
        .begin_symbol(Some("▲"))
        .end_symbol(Some("▼"));
    frame.render_stateful_widget(
        scrollbar,
        area.inner(ratatui::layout::Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut state,
    );
}
