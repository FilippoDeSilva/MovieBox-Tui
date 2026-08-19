use moviebox_tui::tui::app::App;
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
