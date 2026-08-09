use moviebox_tui::tui::app::App;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

struct TerminalGuard;

fn restore_terminal() {
    let _ = crossterm::execute!(
        std::io::stdout(),
        crossterm::cursor::Show,
        crossterm::event::DisableMouseCapture,
        crossterm::event::DisableFocusChange,
        crossterm::terminal::LeaveAlternateScreen
    );
    let _ = crossterm::terminal::disable_raw_mode();
}

fn purge_stale_subtitles() {
    tokio::task::spawn_blocking(|| {
        let max_age = 24 * 60 * 60;
        if let Ok(entries) = std::fs::read_dir(std::env::temp_dir()) {
            for entry in entries.flatten() {
                if entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("moviebox_sub_")
                    && let Ok(metadata) = entry.metadata()
                    && let Ok(modified) = metadata.modified()
                    && let Ok(elapsed) = modified.elapsed()
                    && elapsed.as_secs() > max_age
                {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }
    });
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        restore_terminal();
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    moviebox_tui::logging::init();

    std::panic::set_hook(Box::new(|info| {
        log::error!("panic: {info}");
        restore_terminal();
        eprintln!("{info}");
    }));

    let args: Vec<String> = std::env::args().collect();
    if args
        .iter()
        .any(|arg| arg == "--version" || arg == "-v" || arg == "-V")
    {
        println!("moviebox-tui {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let stdout = std::io::stdout();
    let backend =
        ratatui::backend::CrosstermBackend::new(std::io::BufWriter::with_capacity(65536, stdout));
    let mut terminal = ratatui::Terminal::new(backend)?;
    crossterm::terminal::enable_raw_mode()?;
    let _guard = TerminalGuard;
    crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture,
        crossterm::event::EnableFocusChange
    )?;

    moviebox_tui::cache::clean_old_cache_background();
    purge_stale_subtitles();

    let mut app = App::new();
    app.run(&mut terminal).await
}
