fn env(name: &str) -> String {
    std::env::var(name).unwrap_or_default()
}

pub fn uses_basic_ui() -> bool {
    let term = env("TERM");
    let term_program = env("TERM_PROGRAM");

    cfg!(target_os = "windows")
        || term == "dumb"
        || term == "linux"
        || term_program == "Apple_Terminal"
        || std::env::var_os("TMUX").is_some()
        || std::env::var_os("SSH_TTY").is_some()
        || std::env::var_os("SSH_CLIENT").is_some()
}

pub fn should_query_images() -> bool {
    let term = env("TERM");
    let term_program = env("TERM_PROGRAM");

    term != "dumb"
        && term != "linux"
        && term_program != "Apple_Terminal"
        && std::env::var_os("TMUX").is_none()
        && std::env::var_os("SSH_TTY").is_none()
        && std::env::var_os("SSH_CLIENT").is_none()
}

pub fn background_is_light() -> bool {
    if let Ok(value) = std::env::var("COLORFGBG")
        && let Some(background) = value
            .split([';', ':'])
            .next_back()
            .and_then(|value| value.parse::<u8>().ok())
    {
        return matches!(background, 7 | 10..=15);
    }

    std::env::var("TERM_BACKGROUND").is_ok_and(|value| value.eq_ignore_ascii_case("light"))
}
