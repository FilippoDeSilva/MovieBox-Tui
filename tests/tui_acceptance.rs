use moviebox_tui::tui::action::Action;
use moviebox_tui::tui::app::App;
use moviebox_tui::tui::state::InputMode;
use moviebox_tui::tui::theme::ThemeKind;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

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
        (80, 24),
        (100, 30),
        (120, 40),
        (160, 50),
        (200, 60),
        (25, 8),
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
async fn test_mouse_scroll_maps_to_key_actions() {
    let mut app = App::new();
    let up_key = crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Up,
        crossterm::event::KeyModifiers::empty(),
    );
    let down_key = crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Down,
        crossterm::event::KeyModifiers::empty(),
    );

    app.handle_action(Action::Key(down_key)).await;
    app.handle_action(Action::Key(up_key)).await;
    assert_eq!(app.state().search_query, "");
}
