use flexi_logger::{Cleanup, Criterion, FileSpec, Logger, Naming, WriteMode};

pub fn init() {
    let default_level = if cfg!(debug_assertions) {
        "info"
    } else {
        "warn"
    };
    let spec = std::env::var("MOVIEBOX_LOG")
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| default_level.to_string());

    let log_dir = dirs::data_dir()
        .map(|dir| dir.join("moviebox-tui").join("logs"))
        .unwrap_or_else(std::env::temp_dir);

    match Logger::try_with_str(&spec) {
        Ok(logger) => {
            if let Err(error) = logger
                .log_to_file(
                    FileSpec::default()
                        .directory(&log_dir)
                        .basename("moviebox-tui"),
                )
                .rotate(
                    Criterion::Size(5 * 1024 * 1024),
                    Naming::Numbers,
                    Cleanup::KeepLogFiles(3),
                )
                .write_mode(WriteMode::Direct)
                .format(flexi_logger::opt_format)
                .start()
            {
                eprintln!("[moviebox-tui] logging unavailable: {error}");
            }
        }
        Err(error) => {
            eprintln!("[moviebox-tui] invalid MOVIEBOX_LOG level: {error}");
        }
    }

    log::info!(
        "session started | moviebox-tui {} | {} | log file: {}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        log_file_path().display()
    );
    eprintln!(
        "[moviebox-tui] logging to {} (MOVIEBOX_LOG={})",
        log_file_path().display(),
        spec
    );
}

pub fn log_file_path() -> std::path::PathBuf {
    dirs::data_dir()
        .map(|dir| {
            dir.join("moviebox-tui")
                .join("logs")
                .join("moviebox-tui_rCURRENT.log")
        })
        .unwrap_or_else(std::env::temp_dir)
}
