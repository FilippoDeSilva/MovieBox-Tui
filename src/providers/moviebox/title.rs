pub fn clean_moviebox_title(raw_title: &str) -> String {
    let mut title = raw_title.trim();

    if let Some(pos) = title.find('[') {
        title = title[..pos].trim();
    }

    if let Some(pos) = title.find('(') {
        let inside = &title[pos + 1..];
        let inside_content = inside.split(')').next().unwrap_or("").trim();
        let is_year = inside_content.len() == 4
            && inside_content.chars().all(|c| c.is_ascii_digit())
            && inside_content
                .parse::<u32>()
                .is_ok_and(|y| (1900..=2099).contains(&y));
        if !is_year {
            title = title[..pos].trim();
        }
    }

    if let Some(pos) = title.rfind(" - ") {
        let suffix = title[pos + 3..].to_lowercase();
        let is_tag = suffix.contains("hindi")
            || suffix.contains("tamil")
            || suffix.contains("telugu")
            || suffix.contains("kannada")
            || suffix.contains("malayalam")
            || suffix.contains("bengali")
            || suffix.contains("marathi")
            || suffix.contains("punjabi")
            || suffix.contains("gujarati")
            || suffix.contains("urdu")
            || suffix.contains("english")
            || suffix.contains("spanish")
            || suffix.contains("french")
            || suffix.contains("german")
            || suffix.contains("italian")
            || suffix.contains("japanese")
            || suffix.contains("korean")
            || suffix.contains("chinese")
            || suffix.contains("russian")
            || suffix.contains("portuguese")
            || suffix.contains("turkish")
            || suffix.contains("arabic")
            || suffix.contains("dub")
            || suffix.contains("audio")
            || suffix.contains("multi")
            || suffix.contains("season")
            || (suffix.starts_with('s')
                && suffix[1..].chars().all(|c| c.is_ascii_digit() || c == '-'));
        if is_tag {
            title = title[..pos].trim();
        }
    }

    if let Some(s_idx) = title.rfind(" S") {
        let suffix = &title[s_idx + 2..];
        let is_season = suffix
            .chars()
            .all(|c| c.is_ascii_digit() || c == '-' || c == 'S');
        if is_season && suffix.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            title = title[..s_idx].trim();
        }
    }

    if let Some(s_idx) = title.to_lowercase().rfind(" season ") {
        title = title[..s_idx].trim();
    }

    title
        .trim_end_matches(['-', ':', '_', '.', ' '])
        .trim()
        .to_string()
}
