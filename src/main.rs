use moviebox_tui::tui::app::App;

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::event::DisableFocusChange
        );
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let stdout = std::io::stdout();
    let backend =
        ratatui::backend::CrosstermBackend::new(std::io::BufWriter::with_capacity(65536, stdout));
    let mut terminal = ratatui::Terminal::new(backend)?;
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableFocusChange
    )?;

    let _guard = TerminalGuard;

    let mut app = App::new();
    app.run(&mut terminal).await
}
