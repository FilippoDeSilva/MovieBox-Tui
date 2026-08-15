use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

pub fn width(value: &str) -> usize {
    UnicodeWidthStr::width(value)
}

pub fn remove_last_grapheme(value: &mut String) {
    if let Some((index, _)) = value.grapheme_indices(true).next_back() {
        value.truncate(index);
    }
}

pub fn truncate_width(value: &str, max_width: usize) -> String {
    if width(value) <= max_width {
        return value.to_string();
    }
    if max_width <= 3 {
        return ".".repeat(max_width);
    }

    let content_width = max_width - 3;
    let mut output = String::new();
    let mut used = 0;
    for grapheme in value.graphemes(true) {
        let grapheme_width = width(grapheme);
        if used + grapheme_width > content_width {
            break;
        }
        output.push_str(grapheme);
        used += grapheme_width;
    }
    output.push_str("...");
    output
}

pub fn truncate_middle_width(value: &str, max_width: usize) -> String {
    if width(value) <= max_width {
        return value.to_string();
    }
    if max_width == 0 {
        return String::new();
    }
    if max_width <= 3 {
        return ".".repeat(max_width);
    }

    let content_width = max_width - 1;
    let start_width = content_width.div_ceil(2);
    let end_width = content_width - start_width;

    let mut start = String::new();
    let mut used = 0;
    for grapheme in value.graphemes(true) {
        let grapheme_width = width(grapheme);
        if used + grapheme_width > start_width {
            break;
        }
        start.push_str(grapheme);
        used += grapheme_width;
    }

    let mut end = Vec::new();
    used = 0;
    for grapheme in value.graphemes(true).rev() {
        let grapheme_width = width(grapheme);
        if used + grapheme_width > end_width {
            break;
        }
        end.push(grapheme);
        used += grapheme_width;
    }
    end.reverse();

    format!("{start}…{}", end.concat())
}

pub fn sanitize_language_label(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .filter(|c| !matches!(*c as u32, 0x064B..=0x065F | 0x0670 | 0x06D6..=0x06ED))
        .collect();

    match cleaned.trim() {
        "العربية" | "Arabic" | "ara" | "ar" => "Arabic".to_string(),
        "اردو" | "أردو" | "Urdu" | "urd" | "ur" => "Urdu".to_string(),
        "বাংলা" | "Bengali" | "ben" | "bn" => "Bengali".to_string(),
        "हिन्दी" | "हिंदी" | "Hindi" | "hin" | "hi" => "Hindi".to_string(),
        "Filipino" | "Tagalog" | "fil" | "tl" => "Filipino".to_string(),
        "Indonesian" | "ind" | "id" => "Indonesian".to_string(),
        "English" | "eng" | "en" => "English".to_string(),
        "Español" | "Spanish" | "spa" | "es" => "Spanish".to_string(),
        "Français" | "French" | "fra" | "fre" | "fr" => "French".to_string(),
        "Deutsch" | "German" | "deu" | "ger" | "de" => "German".to_string(),
        "Italiano" | "Italian" | "ita" | "it" => "Italian".to_string(),
        "Português" | "Portuguese" | "por" | "pt" => "Portuguese".to_string(),
        "Русский" | "Russian" | "rus" | "ru" => "Russian".to_string(),
        "Türkçe" | "Turkish" | "tur" | "tr" => "Turkish".to_string(),
        "Tiếng Việt" | "Vietnamese" | "vie" | "vi" => "Vietnamese".to_string(),
        "中文" | "Chinese" | "zho" | "chi" | "zh" => "Chinese".to_string(),
        "日本語" | "Japanese" | "jpn" | "ja" => "Japanese".to_string(),
        "한국어" | "Korean" | "kor" | "ko" => "Korean".to_string(),
        "ไทย" | "Thai" | "tha" | "th" => "Thai".to_string(),
        "தமிழ்" | "Tamil" | "tam" | "ta" => "Tamil".to_string(),
        "తెలుగు" | "Telugu" | "tel" | "te" => "Telugu".to_string(),
        "മലയാളം" | "Malayalam" | "mal" | "ml" => "Malayalam".to_string(),
        "ಕನ್ನಡ" | "Kannada" | "kan" | "kn" => "Kannada".to_string(),
        "मराठी" | "Marathi" | "mar" | "mr" => "Marathi".to_string(),
        "ગુજરાતી" | "Gujarati" | "guj" | "gu" => "Gujarati".to_string(),
        "ਪੰਜਾਬੀ" | "Punjabi" | "pan" | "pa" => "Punjabi".to_string(),
        "فارسی" | "Persian" | "fas" | "per" | "fa" => "Persian".to_string(),
        "עברית" | "Hebrew" | "heb" | "he" => "Hebrew".to_string(),
        "Ελληνικά" | "Greek" | "ell" | "gre" | "el" => "Greek".to_string(),
        "Polski" | "Polish" | "pol" | "pl" => "Polish".to_string(),
        "Nederlands" | "Dutch" | "nld" | "dut" | "nl" => "Dutch".to_string(),
        "Svenska" | "Swedish" | "swe" | "sv" => "Swedish".to_string(),
        "Norsk" | "Norwegian" | "nor" | "no" => "Norwegian".to_string(),
        "Dansk" | "Danish" | "dan" | "da" => "Danish".to_string(),
        "Suomi" | "Finnish" | "fin" | "fi" => "Finnish".to_string(),
        "Čeština" | "Czech" | "ces" | "cze" | "cs" => "Czech".to_string(),
        "Magyar" | "Hungarian" | "hun" | "hu" => "Hungarian".to_string(),
        "Română" | "Romanian" | "ron" | "rum" | "ro" => "Romanian".to_string(),
        "Українська" | "Ukrainian" | "ukr" | "uk" => "Ukrainian".to_string(),
        "" => "Unknown".to_string(),
        other => other.to_string(),
    }
}
