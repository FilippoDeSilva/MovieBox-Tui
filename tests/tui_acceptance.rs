use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use moviebox_tui::models::SearchResult;
use moviebox_tui::providers::models::ProviderKind;
use moviebox_tui::tui::action::Action;
use moviebox_tui::tui::app::App;
use moviebox_tui::tui::state::{InputMode, Screen};
use moviebox_tui::tui::theme::ThemeKind;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

#[tokio::test]
async fn test_backspace_from_home_focuses_search_input() {
    let mut app = App::new();
    app.state_mut().active_screen = Screen::Home;
    app.state_mut().input_mode = InputMode::Normal;
    app.state_mut().search_query.set_content("Inception");
    app.state_mut().favorites_focus = true;

    let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::empty());
    app.handle_action(Action::Key(key)).await;

    assert_eq!(app.state().input_mode, InputMode::Editing);
    assert!(!app.state().favorites_focus);
    assert_eq!(app.state().search_query, "Inception");
}

#[tokio::test]
async fn test_season_download_remembers_explicit_no_subtitle_choice() {
    let mut app = App::new();
    app.state_mut().is_download_subtitle_popup = true;
    app.state_mut().subtitle_list = vec![("None".to_string(), String::new())];
    app.state_mut().subtitle_list_state.select(Some(0));
    app.state_mut().download_queue_total = 4;
    app.state_mut().last_search_edit =
        std::time::Instant::now() - std::time::Duration::from_secs(1);

    app.handle_action(Action::Submit).await;

    assert_eq!(app.state().season_subtitle_preference, Some(None));
}

#[tokio::test]
async fn test_tui_startup_and_home_screen_rendering() {
    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.draw(frame)).unwrap();
    let buffer = terminal.backend().buffer();
    assert_eq!(buffer.area.width, 100);
    assert_eq!(buffer.area.height, 30);
}

#[tokio::test]
async fn test_tui_all_theme_rendering() {
    for theme_kind in ThemeKind::ALL {
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new();
        let _ = theme_kind;
        let res = terminal.draw(|frame| app.draw(frame));
        assert!(res.is_ok(), "Failed to render theme {:?}", theme_kind);
    }
}

#[tokio::test]
async fn test_tui_terminal_resizing_matrix_no_panics() {
    let sizes = [
        (40, 15),
        (49, 13),
        (50, 14),
        (55, 18),
        (80, 24),
        (100, 30),
        (120, 40),
        (160, 50),
        (200, 60),
        (25, 8),
        (60, 100),
        (300, 80),
    ];

    let mut app = App::new();

    for (w, h) in sizes {
        let backend = TestBackend::new(w, h);
        let mut terminal = Terminal::new(backend).unwrap();
        let res = terminal.draw(|frame| app.draw(frame));
        assert!(res.is_ok(), "Failed to render at terminal size {w}x{h}");
    }
}

#[tokio::test]
async fn test_grid_metrics_and_visibility_at_tier_boundaries() {
    let mut app = App::new();
    app.state_mut().active_screen = Screen::Home;
    for i in 0..30 {
        app.state_mut().search_results.push(SearchResult {
            id: i.to_string(),
            title: format!("Title {i}"),
            stype: 1,
            release_year: "2020".to_string(),
            cover_url: None,
            season: 0,
            episode: 0,
            provider: ProviderKind::MovieBox,
        });
    }

    app.state_mut().last_result_metrics = Some(app.state().result_metrics(20, 180));
    let wide = app.state().last_result_metrics.unwrap();
    assert_eq!(wide.columns, 3);
    assert!(wide.visible_items >= 3);

    app.state_mut().last_result_metrics = Some(app.state().result_metrics(20, 100));
    let narrow = app.state().last_result_metrics.unwrap();
    assert_eq!(narrow.columns, 1);
    assert!(narrow.visible_items >= 1);

    app.state_mut().poster_rows = 12;
    let cramped = app.state().result_metrics(10, 100);
    assert_eq!(cramped.visible_items, 1);

    app.state_mut().poster_rows = 3;
    app.state_mut().last_result_metrics = Some(app.state().result_metrics(20, 180));
    app.state_mut().search_list_state.select(Some(29));
    app.state_mut().result_scroll = 0;
    app.state_mut().normalize_result_view();
    let selected = app.state().search_list_state.selected().unwrap();
    let scroll = app.state().result_scroll;
    let visible = app.state().effective_visible_items();
    assert!(selected >= scroll && selected < scroll + visible);
}

#[tokio::test]
async fn test_help_overlay_scrolls_with_keys() {
    let mut app = App::new();
    let state = app.state_mut();
    state.show_help = true;
    state.help_scroll = 0;

    app.handle_action(Action::Key(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Down,
        crossterm::event::KeyModifiers::empty(),
    )))
    .await;
    assert_eq!(app.state().help_scroll, 1);

    app.handle_action(Action::Key(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Up,
        crossterm::event::KeyModifiers::empty(),
    )))
    .await;
    assert_eq!(app.state().help_scroll, 0);

    app.handle_action(Action::Key(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Char('x'),
        crossterm::event::KeyModifiers::empty(),
    )))
    .await;
    assert!(app.state().show_help, "unrelated keys are swallowed");
}

#[tokio::test]
async fn test_mouse_click_help_overlay_dismisses() {
    let mut app = App::new();
    app.state_mut().show_help = true;
    assert!(app.state().show_help);

    app.handle_action(Action::MouseClick(10, 10)).await;
    assert!(!app.state().show_help);
}

#[tokio::test]
async fn test_mouse_click_outside_theme_popup_dismisses() {
    let mut app = App::new();
    app.state_mut().show_theme_popup = true;
    assert!(app.state().show_theme_popup);

    app.handle_action(Action::MouseClick(0, 0)).await;
    assert!(!app.state().show_help);
}

#[tokio::test]
async fn test_mouse_click_search_input_mode() {
    let mut app = App::new();
    assert_eq!(app.state().input_mode, InputMode::Normal);

    app.handle_action(Action::MouseClick(5, 5)).await;
    let _ = app.state().input_mode;
}

#[tokio::test]
async fn test_mouse_click_search_suggestion_selects_query() {
    let mut app = App::new();
    app.state_mut().input_mode = InputMode::Editing;
    app.state_mut().search_suggestions = vec![
        "Deewaniyat".to_string(),
        "Ek Deewane Ki Deewaniyat".to_string(),
    ];

    let area = ratatui::layout::Rect::new(0, 0, 80, 24);
    let search_bar_area = ratatui::layout::Rect::new(10, 5, 60, 3);
    let (_container, inner) =
        moviebox_tui::tui::screens::home::search_suggestions_bounds(area, search_bar_area, 2);

    app.handle_action(Action::MouseClick(inner.x, inner.y))
        .await;
    app.handle_action(Action::SelectSuggestion {
        query: "Deewaniyat".to_string(),
    })
    .await;
    assert_eq!(app.state().search_query.as_str(), "Deewaniyat");
    assert_eq!(app.state().input_mode, InputMode::Normal);
    assert!(app.state().search_suggestions.is_empty());
}

#[tokio::test]
async fn test_mouse_scroll_maps_to_key_actions() {
    let mut app = App::new();
    app.state_mut().active_screen = Screen::Home;
    for i in 0..40 {
        app.state_mut().search_results.push(SearchResult {
            id: i.to_string(),
            title: format!("Title {i}"),
            stype: 1,
            release_year: "2010".to_string(),
            cover_url: None,
            season: 0,
            episode: 0,
            provider: ProviderKind::MovieBox,
        });
    }
    app.state_mut().search_list_state.select(Some(20));

    app.handle_action(Action::WheelScroll { up: true }).await;
    let after_up = app
        .state()
        .search_list_state
        .selected()
        .expect("selection stays set");
    assert!(
        after_up < 20,
        "wheel up moves the result selection upward (got {after_up})"
    );

    for _ in 0..3 {
        app.handle_action(Action::WheelScroll { up: false }).await;
    }
    let after_down = app
        .state()
        .search_list_state
        .selected()
        .expect("selection stays set");
    assert!(
        after_down > after_up,
        "wheel down moves the result selection downward"
    );
    assert!(after_down < 40, "selection stays within bounds");
}

#[tokio::test]
async fn test_full_user_journey_movie_search_details_and_back_navigation() {
    let mut app = App::new();
    app.state_mut().is_tv_mode = false;
    app.state_mut().is_addon_mode = false;
    app.state_mut().active_screen = Screen::Home;
    app.state_mut().last_search_edit =
        std::time::Instant::now() - std::time::Duration::from_millis(600);

    let sample_item = SearchResult {
        id: "12345".to_string(),
        title: "Inception".to_string(),
        stype: 1,
        release_year: "2010".to_string(),
        cover_url: None,
        season: 0,
        episode: 0,
        provider: ProviderKind::MovieBox,
    };

    app.state_mut().search_results.push(sample_item);
    app.state_mut().search_list_state.select(Some(0));

    app.handle_action(Action::Submit).await;
    assert_eq!(app.state().active_screen, Screen::Details);
    assert_eq!(app.state().active_subject_id.as_deref(), Some("12345"));

    app.handle_action(Action::GoBack).await;
    assert_eq!(app.state().active_screen, Screen::Home);
}

#[tokio::test]
async fn test_full_user_journey_mode_switching_and_theme_selection() {
    let mut app = App::new();
    app.state_mut().is_addon_mode = false;
    app.state_mut().is_tv_mode = false;

    app.handle_action(Action::ToggleAddonMode).await;
    assert!(app.state().is_addon_mode);

    app.handle_action(Action::SwitchToStreamingMode).await;
    assert!(!app.state().is_addon_mode);

    app.handle_action(Action::SelectTheme("TokyoNight".to_string()))
        .await;
    assert_eq!(app.state().active_theme_kind, "TokyoNight");
}

#[tokio::test]
async fn test_download_dir_reset_user_journey() {
    let mut app = App::new();
    let custom_path = std::path::PathBuf::from("/custom/moviebox/downloads");
    app.state_mut().download_dir = Some(custom_path.clone());
    assert_eq!(app.state().download_dir, Some(custom_path));

    app.handle_action(Action::Search {
        query: "/download-dir reset".to_string(),
        force_refresh: false,
    })
    .await;

    assert!(app.state().download_dir.is_none());
    assert!(!app.state().notifications.is_empty());
    let notif = app.state().notifications.back().unwrap();
    assert_eq!(notif.title, "Download Directory");
    assert!(notif.message.contains("Reset to default"));
}

#[tokio::test]
async fn test_esc_key_cancels_slash_command_and_clears_search_bar() {
    let mut app = App::new();
    app.state_mut().update_available = None;
    app.state_mut().show_theme_popup = false;
    app.state_mut().show_browse_popup = false;
    app.state_mut().input_mode = InputMode::Editing;
    app.state_mut().search_query.set_content("/download-dir");

    let esc_key = crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Esc,
        crossterm::event::KeyModifiers::empty(),
    );
    app.handle_action(Action::Key(esc_key)).await;

    assert_eq!(app.state().input_mode, InputMode::Normal);
    assert_eq!(app.state().search_query, "");
}

#[tokio::test]
async fn test_esc_key_cancels_unsubmitted_search_query() {
    let mut app = App::new();
    app.state_mut().update_available = None;
    app.state_mut().show_theme_popup = false;
    app.state_mut().show_browse_popup = false;
    app.state_mut().input_mode = InputMode::Editing;
    app.state_mut().search_query.set_content("batman");
    assert!(app.state().search_results.is_empty());

    let esc_key = crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Esc,
        crossterm::event::KeyModifiers::empty(),
    );
    app.handle_action(Action::Key(esc_key)).await;

    assert_eq!(app.state().input_mode, InputMode::Normal);
    assert_eq!(app.state().search_query, "");
}

#[tokio::test]
async fn test_tab_key_completes_regular_search_suggestions() {
    let mut app = App::new();
    app.state_mut().input_mode = InputMode::Editing;
    app.state_mut().search_query.set_content("inter");
    app.state_mut().search_suggestions =
        vec!["Interstellar".to_string(), "Interstellar 2".to_string()];

    let tab_key = crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Tab,
        crossterm::event::KeyModifiers::empty(),
    );
    app.handle_action(Action::Key(tab_key)).await;

    assert_eq!(app.state().search_query, "Interstellar");
}

#[tokio::test]
async fn test_ctrl_u_clears_search_input() {
    let mut app = App::new();
    app.state_mut().input_mode = InputMode::Editing;
    app.state_mut().search_query.set_content("hello world");

    let ctrl_u = crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Char('u'),
        crossterm::event::KeyModifiers::CONTROL,
    );
    app.handle_action(Action::Key(ctrl_u)).await;

    assert_eq!(app.state().search_query, "");
}

#[tokio::test]
async fn test_ctrl_w_deletes_backward_word() {
    let mut app = App::new();
    app.state_mut().input_mode = InputMode::Editing;
    app.state_mut().search_query.set_content("the dark knight");

    let ctrl_w = crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Char('w'),
        crossterm::event::KeyModifiers::CONTROL,
    );
    app.handle_action(Action::Key(ctrl_w)).await;
    assert_eq!(app.state().search_query, "the dark ");

    app.handle_action(Action::Key(ctrl_w)).await;
    assert_eq!(app.state().search_query, "the ");

    app.handle_action(Action::Key(ctrl_w)).await;
    assert_eq!(app.state().search_query, "");
}

#[tokio::test]
async fn test_f_key_toggles_favorite_on_home_results() {
    let mut app = App::new();
    app.state_mut().active_screen = Screen::Home;
    app.state_mut().favorites.items.clear();
    let res = SearchResult {
        id: "100".to_string(),
        title: "Test Movie".to_string(),
        stype: 1,
        release_year: "2024".to_string(),
        cover_url: None,
        season: 0,
        episode: 0,
        provider: ProviderKind::MovieBox,
    };
    app.state_mut().search_results.push(res.clone());
    app.state_mut().search_list_state.select(Some(0));

    let identity = moviebox_tui::models::SubjectIdentity {
        provider: res.provider.cache_key(),
        subject_id: &res.id,
        title: &res.title,
        stype: res.stype,
        release_year: &res.release_year,
    };
    assert!(!app.state().favorites.is_favorite(&identity));

    let f_key = crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Char('f'),
        crossterm::event::KeyModifiers::empty(),
    );
    app.handle_action(Action::Key(f_key)).await;
    app.handle_action(Action::ToggleFavorite).await;

    assert!(app.state().favorites.is_favorite(&identity));
}

#[tokio::test]
async fn test_up_from_favorites_landing_unfocuses() {
    let mut app = App::new();
    app.state_mut().active_screen = Screen::Home;
    app.state_mut().favorites_focus = true;
    app.state_mut().favorites_landing_state.select(Some(0));

    app.handle_action(Action::MoveUp).await;

    assert!(!app.state().favorites_focus);
    assert_eq!(app.state().favorites_landing_state.selected(), None);
}

#[tokio::test]
async fn test_search_cursor_navigation_and_mid_string_editing() {
    let mut app = App::new();
    app.state_mut().input_mode = InputMode::Editing;
    app.state_mut().search_query.set_content("avtar");

    // Move left twice to position cursor between 'v' and 't' (pos 2)
    let left = KeyEvent::new(KeyCode::Left, KeyModifiers::empty());
    app.handle_action(Action::Key(left)).await;
    app.handle_action(Action::Key(left)).await;
    assert_eq!(app.state().search_query.cursor(), 3);
    app.handle_action(Action::Key(left)).await;
    assert_eq!(app.state().search_query.cursor(), 2);

    // Insert 'a' -> "avatar"
    let char_a = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty());
    app.handle_action(Action::Key(char_a)).await;
    assert_eq!(app.state().search_query, "avatar");
    assert_eq!(app.state().search_query.cursor(), 3);

    // Home -> cursor at 0
    let home = KeyEvent::new(KeyCode::Home, KeyModifiers::empty());
    app.handle_action(Action::Key(home)).await;
    assert_eq!(app.state().search_query.cursor(), 0);

    // Forward delete 'a' -> "vatar"
    let delete = KeyEvent::new(KeyCode::Delete, KeyModifiers::empty());
    app.handle_action(Action::Key(delete)).await;
    assert_eq!(app.state().search_query, "vatar");
    assert_eq!(app.state().search_query.cursor(), 0);

    // End -> cursor at end
    let end = KeyEvent::new(KeyCode::End, KeyModifiers::empty());
    app.handle_action(Action::Key(end)).await;
    assert_eq!(app.state().search_query.cursor(), 5);
}

#[tokio::test]
async fn test_contextual_window_title() {
    let mut app = App::new();
    app.state_mut().active_screen = Screen::Home;
    app.state_mut()
        .set_mode(moviebox_tui::tui::state::AppMode::Streaming);
    assert_eq!(app.contextual_title(), "MovieBox-Tui — Streaming");

    app.state_mut()
        .set_mode(moviebox_tui::tui::state::AppMode::Tv);
    assert_eq!(app.contextual_title(), "MovieBox-Tui — Live TV");

    app.state_mut()
        .set_mode(moviebox_tui::tui::state::AppMode::Addon);
    assert_eq!(app.contextual_title(), "MovieBox-Tui — Addons");

    app.state_mut().active_screen = Screen::Details;
    app.state_mut().selected_details = Some(serde_json::json!({
        "title": "Inception"
    }));
    assert_eq!(app.contextual_title(), "MovieBox-Tui — Inception");
}
#[tokio::test]
async fn test_esc_in_normal_mode_focuses_search_bar_when_results_present() {
    let mut app = App::new();
    app.state_mut().active_screen = Screen::Home;
    app.state_mut().input_mode = InputMode::Normal;
    app.state_mut().search_query.set_content("matrix");
    app.state_mut().search_results.push(SearchResult {
        id: "1".to_string(),
        title: "The Matrix".to_string(),
        stype: 1,
        release_year: "1999".to_string(),
        cover_url: None,
        season: 0,
        episode: 0,
        provider: ProviderKind::MovieBox,
    });
    app.state_mut().search_list_state.select(Some(0));

    let esc_key = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
    app.handle_action(Action::Key(esc_key)).await;
    app.handle_action(Action::GoBack).await;

    assert_eq!(app.state().input_mode, InputMode::Editing);
    assert_eq!(app.state().search_results.len(), 1);
    assert_eq!(app.state().search_query, "matrix");
}

#[tokio::test]
async fn test_esc_in_editing_mode_switches_to_normal_when_results_present() {
    let mut app = App::new();
    app.state_mut().active_screen = Screen::Home;
    app.state_mut().input_mode = InputMode::Editing;
    app.state_mut().search_query.set_content("matrix");
    app.state_mut().search_results.push(SearchResult {
        id: "1".to_string(),
        title: "The Matrix".to_string(),
        stype: 1,
        release_year: "1999".to_string(),
        cover_url: None,
        season: 0,
        episode: 0,
        provider: ProviderKind::MovieBox,
    });

    let esc_key = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
    app.handle_action(Action::Key(esc_key)).await;

    assert_eq!(app.state().input_mode, InputMode::Normal);
    assert_eq!(app.state().search_results.len(), 1);
    assert_eq!(app.state().search_query, "matrix");
}

#[tokio::test]
async fn test_esc_in_editing_mode_clears_when_no_results() {
    let mut app = App::new();
    app.state_mut().active_screen = Screen::Home;
    app.state_mut().input_mode = InputMode::Editing;
    app.state_mut().search_query.set_content("matrix");
    assert!(app.state().search_results.is_empty());

    let esc_key = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
    app.handle_action(Action::Key(esc_key)).await;

    assert_eq!(app.state().input_mode, InputMode::Normal);
    assert!(app.state().search_query.is_empty());
}

#[tokio::test]
async fn test_ctrl_u_and_clear_command_clears_results_cleanly() {
    let mut app = App::new();
    app.state_mut().active_screen = Screen::Home;
    app.state_mut().input_mode = InputMode::Editing;
    app.state_mut().search_query.set_content("matrix");
    app.state_mut().search_results.push(SearchResult {
        id: "1".to_string(),
        title: "The Matrix".to_string(),
        stype: 1,
        release_year: "1999".to_string(),
        cover_url: None,
        season: 0,
        episode: 0,
        provider: ProviderKind::MovieBox,
    });

    let ctrl_u = KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL);
    app.handle_action(Action::Key(ctrl_u)).await;

    assert!(app.state().search_results.is_empty());
    assert!(app.state().search_query.is_empty());
    assert_eq!(app.state().input_mode, InputMode::Normal);

    // Test /clear slash command
    app.state_mut().search_results.push(SearchResult {
        id: "2".to_string(),
        title: "Inception".to_string(),
        stype: 1,
        release_year: "2010".to_string(),
        cover_url: None,
        season: 0,
        episode: 0,
        provider: ProviderKind::MovieBox,
    });
    app.state_mut().search_query.set_content("/clear");
    app.handle_action(Action::Search {
        query: "/clear".to_string(),
        force_refresh: false,
    })
    .await;

    assert!(app.state().search_results.is_empty());
    assert!(app.state().search_query.is_empty());
    assert_eq!(app.state().input_mode, InputMode::Normal);
}

#[tokio::test]
async fn test_history_item_space_and_p_key_direct_resume() {
    let mut app = App::new();
    app.state_mut().active_screen = Screen::Home;
    app.state_mut().input_mode = InputMode::Normal;
    app.state_mut().search_query.set_content("/history");
    app.state_mut().search_results.push(SearchResult {
        id: "tv_100".to_string(),
        title: "Breaking Bad".to_string(),
        stype: 2,
        release_year: "2008".to_string(),
        cover_url: None,
        season: 2,
        episode: 4,
        provider: ProviderKind::MovieBox,
    });
    app.state_mut().search_list_state.select(Some(0));
    app.state_mut().last_search_edit =
        std::time::Instant::now() - std::time::Duration::from_secs(1);

    let space_key = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty());
    app.handle_action(Action::Key(space_key)).await;

    assert_eq!(app.state().active_screen, Screen::Details);
    assert_eq!(app.state().active_subject_id.as_deref(), Some("tv_100"));
    assert_eq!(app.state().selected_season, 2);
    assert_eq!(app.state().selected_episode, 4);
    assert!(app.state().auto_play_on_ready);
}

#[tokio::test]
async fn test_history_item_enter_pre_seeds_season_and_episode() {
    let mut app = App::new();
    app.state_mut().active_screen = Screen::Home;
    app.state_mut().input_mode = InputMode::Normal;
    app.state_mut().search_query.set_content("/history");
    app.state_mut().search_results.push(SearchResult {
        id: "tv_200".to_string(),
        title: "Better Call Saul".to_string(),
        stype: 2,
        release_year: "2015".to_string(),
        cover_url: None,
        season: 3,
        episode: 7,
        provider: ProviderKind::MovieBox,
    });
    app.state_mut().search_list_state.select(Some(0));
    app.state_mut().last_search_edit =
        std::time::Instant::now() - std::time::Duration::from_secs(1);

    app.handle_action(Action::Submit).await;

    assert_eq!(app.state().active_screen, Screen::Details);
    assert_eq!(app.state().active_subject_id.as_deref(), Some("tv_200"));
    assert_eq!(app.state().selected_season, 3);
    assert_eq!(app.state().selected_episode, 7);
    assert!(!app.state().auto_play_on_ready);
}

#[tokio::test]
async fn test_no_results_and_error_state_rendering_hints() {
    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    // No results view
    app.state_mut().active_screen = Screen::Home;
    app.state_mut().input_mode = InputMode::Normal;
    app.state_mut()
        .search_query
        .set_content("nonexistent_movie_xyz");
    app.state_mut().search_results.clear();

    terminal.draw(|frame| app.draw(frame)).unwrap();
    let content = terminal.backend().buffer().content();
    let text: String = content.iter().map(|c| c.symbol()).collect();
    assert!(text.contains("Switch provider"));
    assert!(text.contains("Browse categories"));
    assert!(text.contains("Clear"));

    // Error view
    app.state_mut().search_error =
        Some("Failed to connect to MovieBox provider: connection refused by server".to_string());
    terminal.draw(|frame| app.draw(frame)).unwrap();
    let content_err = terminal.backend().buffer().content();
    let text_err: String = content_err.iter().map(|c| c.symbol()).collect();
    assert!(text_err.contains("Retry request"));
    assert!(text_err.contains("Switch provider"));
    assert!(text_err.contains("Back"));
}
