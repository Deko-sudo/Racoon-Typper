//! Script validation for bundled language content.
//!
//! A lesson must never mix writing systems: Russian and Ukrainian content is
//! Cyrillic-only, Latin courses stay Latin, and CJK courses stay inside their
//! own scripts. The loaders and the regression tests share these helpers so a
//! stray character cannot reach a running session unnoticed.

/// Writing systems used by bundled content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Script {
    Cyrillic,
    Latin,
    Han,
    Hiragana,
    Katakana,
    Hangul,
    Bopomofo,
    /// An alphanumeric character from any other writing system.
    Other,
}

impl Script {
    /// Name used in validation messages.
    pub fn name(self) -> &'static str {
        match self {
            Script::Cyrillic => "Cyrillic",
            Script::Latin => "Latin",
            Script::Han => "Han",
            Script::Hiragana => "Hiragana",
            Script::Katakana => "Katakana",
            Script::Hangul => "Hangul",
            Script::Bopomofo => "Bopomofo",
            Script::Other => "another script",
        }
    }
}

/// Returns the writing system of `ch`.
///
/// Whitespace, digits, punctuation and symbols return `None`: they are neutral
/// and may appear in any language.
pub fn script_of(ch: char) -> Option<Script> {
    if ch.is_numeric() || !ch.is_alphanumeric() {
        return None;
    }

    Some(match ch as u32 {
        0x0400..=0x052F => Script::Cyrillic,
        0x0041..=0x005A | 0x0061..=0x007A | 0x00C0..=0x024F | 0x1E00..=0x1EFF => Script::Latin,
        0x3005 | 0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF => Script::Han,
        0x3040..=0x309F => Script::Hiragana,
        0x30A0..=0x30FF => Script::Katakana,
        0x1100..=0x11FF | 0x3130..=0x318F | 0xAC00..=0xD7AF => Script::Hangul,
        0x3100..=0x312F => Script::Bopomofo,
        _ => Script::Other,
    })
}

/// Writing systems allowed for a bundled language code.
///
/// Returns `None` for languages without a script policy; callers then skip the
/// check instead of rejecting content they cannot judge.
pub fn allowed_scripts(language: &str) -> Option<&'static [Script]> {
    match language {
        "ru" | "uk" => Some(&[Script::Cyrillic]),
        "en" | "de" | "it" | "fr" | "es" | "pt" | "ro" | "cs" | "pl" => Some(&[Script::Latin]),
        // CJK courses may include common Latin loanwords ("OK", "Wi-Fi").
        "ja" => Some(&[
            Script::Hiragana,
            Script::Katakana,
            Script::Han,
            Script::Latin,
        ]),
        "zh-hk" | "zh-tw" => Some(&[Script::Han, Script::Bopomofo, Script::Latin]),
        "ko" => Some(&[Script::Hangul, Script::Han, Script::Latin]),
        _ => None,
    }
}

/// Returns every character of `text` written in a script that is not allowed
/// for `language`. Neutral characters never appear in the result.
pub fn find_foreign_characters(language: &str, text: &str) -> Vec<char> {
    let Some(allowed) = allowed_scripts(language) else {
        return Vec::new();
    };

    text.chars()
        .filter(|ch| script_of(*ch).is_some_and(|script| !allowed.contains(&script)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cyrillic_and_latin_are_distinguished() {
        assert_eq!(script_of('а'), Some(Script::Cyrillic));
        assert_eq!(script_of('ё'), Some(Script::Cyrillic));
        assert_eq!(script_of('a'), Some(Script::Latin));
        assert_eq!(script_of('é'), Some(Script::Latin));
        assert_eq!(script_of('ß'), Some(Script::Latin));
    }

    #[test]
    fn cjk_scripts_are_classified() {
        assert_eq!(script_of('导'), Some(Script::Han));
        assert_eq!(script_of('々'), Some(Script::Han));
        assert_eq!(script_of('あ'), Some(Script::Hiragana));
        assert_eq!(script_of('ア'), Some(Script::Katakana));
        assert_eq!(script_of('한'), Some(Script::Hangul));
        assert_eq!(script_of('ㄅ'), Some(Script::Bopomofo));
    }

    #[test]
    fn neutral_characters_are_ignored() {
        for ch in [' ', '\t', '1', '.', ',', '!', '?', '—', '«', '»', '…', ':'] {
            assert_eq!(script_of(ch), None, "{ch:?}");
        }
    }

    #[test]
    fn foreign_scripts_are_flagged() {
        assert_eq!(find_foreign_characters("ru", "вода 导电"), vec!['导', '电']);
        assert_eq!(
            find_foreign_characters("en", "hello мир"),
            vec!['м', 'и', 'р']
        );
        assert_eq!(
            find_foreign_characters("ru", "вода слон"),
            Vec::<char>::new()
        );
        assert_eq!(
            find_foreign_characters("en", "hello world"),
            Vec::<char>::new()
        );
    }

    #[test]
    fn mixed_script_words_are_flagged() {
        // A Latin look-alike inside a Cyrillic word must not pass.
        assert_eq!(find_foreign_characters("ru", "cобака"), vec!['c']);
        assert_eq!(find_foreign_characters("ru", "отвод导电"), vec!['导', '电']);
    }

    #[test]
    fn cjk_content_allows_its_own_scripts() {
        assert!(find_foreign_characters("ja", "日本語のテスト").is_empty());
        assert!(find_foreign_characters("zh-tw", "繁體中文").is_empty());
        assert!(find_foreign_characters("ko", "한국어").is_empty());
        assert_eq!(find_foreign_characters("ja", "ロシア語привет").len(), 6);
    }

    #[test]
    fn unknown_language_has_no_policy() {
        assert!(allowed_scripts("xx").is_none());
        assert!(find_foreign_characters("xx", "导电 мир hello").is_empty());
    }
}
