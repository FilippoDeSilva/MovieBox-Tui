use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
};

use crate::tui::theme::Theme;

fn theme_color(style: Style, fallback: Color) -> Color {
    style.fg.unwrap_or(fallback)
}

pub fn resolution_badge_spans<'a>(
    resolution: i64,
    theme: &'a Theme,
    basic_terminal: bool,
) -> Vec<Span<'a>> {
    if basic_terminal {
        let (label, style) = match resolution {
            2160 | 4320 => ("[4K]", theme.rating.add_modifier(Modifier::BOLD)),
            1080 => ("[1080p]", theme.highlight.add_modifier(Modifier::BOLD)),
            720 => ("[720p]", theme.teal.add_modifier(Modifier::BOLD)),
            480 | 540 | 576 => ("[SD]", theme.text_dim.add_modifier(Modifier::BOLD)),
            _ if resolution > 0 => (
                match resolution {
                    2160 => "[4K]",
                    1080 => "[1080p]",
                    720 => "[720p]",
                    480 => "[480p]",
                    360 => "[360p]",
                    _ => "[HD]",
                },
                theme.text.add_modifier(Modifier::BOLD),
            ),
            _ => ("[SD]", theme.text_dim),
        };
        return vec![Span::styled(format!("{:<8}", label), style)];
    }

    let (badge_bg, contrast_fg, label) = match resolution {
        2160 | 4320 => (
            theme_color(theme.rating, Color::Rgb(249, 226, 175)),
            if theme.is_light {
                Color::White
            } else {
                theme_color(theme.crust, Color::Rgb(17, 17, 27))
            },
            " 4K ",
        ),
        1080 => (
            theme_color(theme.sapphire, Color::Rgb(116, 199, 236)),
            if theme.is_light {
                Color::White
            } else {
                theme_color(theme.crust, Color::Rgb(17, 17, 27))
            },
            " 1080p ",
        ),
        720 => (
            theme_color(theme.teal, Color::Rgb(148, 226, 213)),
            if theme.is_light {
                Color::White
            } else {
                theme_color(theme.crust, Color::Rgb(17, 17, 27))
            },
            " 720p ",
        ),
        480 | 540 | 576 => (
            theme_color(theme.surface2, Color::Rgb(88, 91, 112)),
            theme_color(theme.text, Color::White),
            " SD ",
        ),
        _ if resolution > 0 => (
            theme_color(theme.surface2, Color::Rgb(88, 91, 112)),
            theme_color(theme.text, Color::White),
            match resolution {
                2160 => " 4K ",
                1080 => " 1080p ",
                720 => " 720p ",
                480 => " 480p ",
                360 => " 360p ",
                _ => " HD ",
            },
        ),
        _ => (
            theme_color(theme.surface2, Color::Rgb(88, 91, 112)),
            theme_color(theme.text_dim, Color::Gray),
            " SD ",
        ),
    };

    vec![
        Span::styled(
            label,
            Style::default()
                .bg(badge_bg)
                .fg(contrast_fg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
    ]
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MediaTags {
    pub hdr: Option<&'static str>,
    pub audio: Option<&'static str>,
    pub codec: Option<&'static str>,
    pub source: Option<&'static str>,
}

pub fn extract_media_tags(title: &str, codec_name: &str) -> MediaTags {
    let lower_title = title.to_ascii_lowercase();
    let lower_codec = codec_name.to_ascii_lowercase();

    let hdr = if lower_title.contains("hdr10+") || lower_title.contains("hdr10plus") {
        Some("HDR10+")
    } else if lower_title.contains("dovi")
        || lower_title.contains("dolby vision")
        || lower_title.contains("dolbyvision")
        || lower_title.contains(".dv.")
        || lower_title.contains(" dv ")
        || lower_title.contains("-dv")
    {
        Some("DV")
    } else if lower_title.contains("hdr") {
        Some("HDR")
    } else {
        None
    };

    let audio = if lower_title.contains("atmos") {
        Some("ATMOS")
    } else if lower_title.contains("7.1") {
        Some("7.1")
    } else if lower_title.contains("5.1")
        || lower_title.contains("ddp5.1")
        || lower_title.contains("dd5.1")
        || lower_title.contains("ac3")
    {
        Some("5.1")
    } else {
        None
    };

    let codec = if lower_codec.contains("hevc")
        || lower_codec.contains("x265")
        || lower_codec.contains("h265")
        || lower_title.contains("hevc")
        || lower_title.contains("x265")
        || lower_title.contains("h.265")
        || lower_title.contains("h265")
    {
        Some("HEVC")
    } else if lower_codec.contains("av1") || lower_title.contains("av1") {
        Some("AV1")
    } else if lower_codec.contains("h264")
        || lower_codec.contains("x264")
        || lower_codec.contains("avc")
        || lower_title.contains("x264")
        || lower_title.contains("h.264")
        || lower_title.contains("h264")
        || lower_title.contains("avc")
    {
        Some("H.264")
    } else {
        None
    };

    let source = if lower_title.contains("remux") {
        Some("REMUX")
    } else if lower_title.contains("bluray")
        || lower_title.contains("bdrip")
        || lower_title.contains("brrip")
    {
        Some("BluRay")
    } else if lower_title.contains("web-dl")
        || lower_title.contains("webdl")
        || lower_title.contains("webrip")
    {
        Some("WEB-DL")
    } else {
        None
    };

    MediaTags {
        hdr,
        audio,
        codec,
        source,
    }
}

pub fn render_media_tag_spans<'a>(
    tags: &MediaTags,
    theme: &'a Theme,
    basic_terminal: bool,
) -> Vec<Span<'a>> {
    let mut spans = Vec::new();

    if let Some(hdr) = tags.hdr {
        if basic_terminal {
            spans.push(Span::styled(
                format!("[{hdr}] "),
                theme.rating.add_modifier(Modifier::BOLD),
            ));
        } else {
            let bg = theme_color(theme.surface1, Color::Rgb(73, 76, 94));
            let fg = theme_color(theme.rating, Color::Yellow);
            spans.push(Span::styled(
                format!(" {hdr} "),
                Style::default().bg(bg).fg(fg).add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::raw(" "));
        }
    }

    if let Some(audio) = tags.audio {
        if basic_terminal {
            spans.push(Span::styled(
                format!("[{audio}] "),
                theme.sapphire.add_modifier(Modifier::BOLD),
            ));
        } else {
            let bg = theme_color(theme.surface0, Color::Rgb(56, 58, 74));
            let fg = theme_color(theme.sapphire, Color::Cyan);
            spans.push(Span::styled(
                format!(" {audio} "),
                Style::default().bg(bg).fg(fg).add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::raw(" "));
        }
    }

    if let Some(codec) = tags.codec {
        if basic_terminal {
            spans.push(Span::styled(
                format!("[{codec}] "),
                theme.teal.add_modifier(Modifier::BOLD),
            ));
        } else {
            let bg = theme_color(theme.surface0, Color::Rgb(56, 58, 74));
            let fg = theme_color(theme.teal, Color::Rgb(148, 226, 213));
            spans.push(Span::styled(
                format!(" {codec} "),
                Style::default().bg(bg).fg(fg),
            ));
            spans.push(Span::raw(" "));
        }
    }

    if let Some(source) = tags.source {
        if basic_terminal {
            spans.push(Span::styled(
                format!("[{source}] "),
                theme.lavender.add_modifier(Modifier::BOLD),
            ));
        } else {
            let bg = theme_color(theme.surface0, Color::Rgb(56, 58, 74));
            let fg = theme_color(theme.lavender, Color::Rgb(180, 190, 254));
            spans.push(Span::styled(
                format!(" {source} "),
                Style::default().bg(bg).fg(fg),
            ));
            spans.push(Span::raw(" "));
        }
    }

    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_media_tags() {
        let tags = extract_media_tags(
            "Dune.Part.Two.2024.2160p.UHD.BluRay.x265.Atmos.TrueHD.7.1.DV.HDR",
            "hevc",
        );
        assert_eq!(tags.hdr, Some("DV"));
        assert_eq!(tags.audio, Some("ATMOS"));
        assert_eq!(tags.codec, Some("HEVC"));
        assert_eq!(tags.source, Some("BluRay"));
    }

    #[test]
    fn test_resolution_badge_spans_4k() {
        let theme = Theme::default();
        let spans_normal = resolution_badge_spans(2160, &theme, false);
        assert_eq!(spans_normal.len(), 2);

        let spans_basic = resolution_badge_spans(2160, &theme, true);
        assert_eq!(spans_basic.len(), 1);
        assert!(spans_basic[0].content.contains("[4K]"));
    }

    #[test]
    fn test_render_media_tag_spans() {
        let theme = Theme::default();
        let tags = MediaTags {
            hdr: Some("HDR"),
            audio: Some("5.1"),
            codec: Some("HEVC"),
            source: Some("WEB-DL"),
        };
        let spans = render_media_tag_spans(&tags, &theme, false);
        assert_eq!(spans.len(), 8);
    }
}
