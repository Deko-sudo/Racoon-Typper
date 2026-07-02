//! WordPackLoader — загрузка словарей из .txt файлов.
//! Формат: одно слово на строку.

use std::collections::HashMap;
use std::sync::OnceLock;

/// Словарь слов для конкретного языка.
pub struct WordPack {
    pub language: String,
    pub words: Vec<String>,
}

impl WordPack {
    pub fn new(language: &str, words: Vec<String>) -> Self {
        Self {
            language: language.to_string(),
            words,
        }
    }

    pub fn len(&self) -> usize {
        self.words.len()
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    pub fn get_word(&self, index: usize) -> Option<&str> {
        self.words.get(index).map(|s| s.as_str())
    }
}

/// Загрузчик словарей. Кэширует загруженные словари.
pub struct WordPackLoader {
    packs: HashMap<String, WordPack>,
}

impl WordPackLoader {
    pub fn new() -> Self {
        let mut packs = HashMap::new();

        macro_rules! load_lang {
            ($code:expr, $path:expr) => {
                let words = load_txt(include_str!($path));
                if !words.is_empty() {
                    packs.insert($code.to_string(), WordPack::new($code, words));
                }
            };
        }

        load_lang!("en", "../../../resources/words/en.txt");
        load_lang!("ru", "../../../resources/words/ru.txt");
        load_lang!("de", "../../../resources/words/de.txt");
        load_lang!("uk", "../../../resources/words/uk.txt");
        load_lang!("cs", "../../../resources/words/cs.txt");
        load_lang!("pl", "../../../resources/words/pl.txt");
        load_lang!("ro", "../../../resources/words/ro.txt");
        load_lang!("it", "../../../resources/words/it.txt");
        load_lang!("fr", "../../../resources/words/fr.txt");
        load_lang!("es", "../../../resources/words/es.txt");
        load_lang!("pt", "../../../resources/words/pt.txt");
        load_lang!("ja", "../../../resources/words/ja.txt");
        load_lang!("zh-hk", "../../../resources/words/zh-hk.txt");
        load_lang!("zh-tw", "../../../resources/words/zh-tw.txt");
        load_lang!("ko", "../../../resources/words/ko.txt");

        Self { packs }
    }

    /// Возвращает словарь для языка.
    pub fn get_pack(&self, language: &str) -> Option<&WordPack> {
        self.packs.get(language)
    }

    /// Возвращает список доступных языков.
    pub fn available_languages(&self) -> Vec<String> {
        self.packs.keys().cloned().collect()
    }

    /// Генерирует N случайных слов из словаря.
    /// Возвращает строку с пробелами между словами.
    pub fn generate_words(&self, language: &str, count: usize) -> Option<String> {
        let pack = self.get_pack(language)?;
        if pack.is_empty() {
            return None;
        }

        let mut result = Vec::with_capacity(count);
        let mut last_index = None;

        for _ in 0..count {
            let idx = random_index(pack.len(), last_index);
            result.push(pack.words[idx].clone());
            last_index = Some(idx);
        }

        Some(result.join(" "))
    }

    /// Генерирует случайные слова для режима Time (бесконечный поток).
    /// Возвращает batch из N слов.
    pub fn generate_batch(&self, language: &str, count: usize) -> Option<String> {
        self.generate_words(language, count)
    }
}

impl Default for WordPackLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Глобальный синглтон для WordPackLoader.
static LOADER: OnceLock<WordPackLoader> = OnceLock::new();

/// Возвращает глобальный WordPackLoader.
pub fn word_pack_loader() -> &'static WordPackLoader {
    LOADER.get_or_init(WordPackLoader::new)
}

/// Парсит .txt файл — одно слово на строку.
fn load_txt(content: &str) -> Vec<String> {
    content
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Простой генератор случайных чисел без rand crate.
fn random_index(max: usize, exclude: Option<usize>) -> usize {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as usize;

    let mut idx = seed % max;
    // Избегаем повторения подряд
    if let Some(ex) = exclude {
        if max > 1 && idx == ex {
            idx = (idx + 1) % max;
        }
    }
    idx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_en_words() {
        let loader = WordPackLoader::new();
        let pack = loader.get_pack("en").unwrap();
        assert!(!pack.is_empty());
        assert!(pack.len() > 100);
    }

    #[test]
    fn load_ru_words() {
        let loader = WordPackLoader::new();
        let pack = loader.get_pack("ru").unwrap();
        assert!(!pack.is_empty());
        assert!(pack.len() > 100);
    }

    #[test]
    fn available_languages() {
        let loader = WordPackLoader::new();
        let langs = loader.available_languages();
        assert!(langs.contains(&"en".to_string()));
        assert!(langs.contains(&"ru".to_string()));
    }

    #[test]
    fn generate_words_count() {
        let loader = WordPackLoader::new();
        let result = loader.generate_words("en", 10).unwrap();
        let words: Vec<&str> = result.split(' ').collect();
        assert_eq!(words.len(), 10);
    }

    #[test]
    fn generate_words_25() {
        let loader = WordPackLoader::new();
        let result = loader.generate_words("en", 25).unwrap();
        let words: Vec<&str> = result.split(' ').collect();
        assert_eq!(words.len(), 25);
    }

    #[test]
    fn generate_words_50() {
        let loader = WordPackLoader::new();
        let result = loader.generate_words("en", 50).unwrap();
        let words: Vec<&str> = result.split(' ').collect();
        assert_eq!(words.len(), 50);
    }

    #[test]
    fn generate_words_100() {
        let loader = WordPackLoader::new();
        let result = loader.generate_words("en", 100).unwrap();
        let words: Vec<&str> = result.split(' ').collect();
        assert_eq!(words.len(), 100);
    }

    #[test]
    fn generate_words_ru() {
        let loader = WordPackLoader::new();
        let result = loader.generate_words("ru", 10).unwrap();
        let words: Vec<&str> = result.split(' ').collect();
        assert_eq!(words.len(), 10);
    }

    #[test]
    fn generate_words_nonexistent_language() {
        let loader = WordPackLoader::new();
        assert!(loader.generate_words("fr", 10).is_none());
    }

    #[test]
    fn generate_words_not_empty() {
        let loader = WordPackLoader::new();
        let result = loader.generate_words("en", 5).unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn generate_batch_same_as_words() {
        let loader = WordPackLoader::new();
        let batch = loader.generate_batch("en", 15).unwrap();
        let words: Vec<&str> = batch.split(' ').collect();
        assert_eq!(words.len(), 15);
    }

    #[test]
    fn parse_txt_skips_empty_lines() {
        let content = "hello\n\nworld\n\n\nfoo\n";
        let words = load_txt(content);
        assert_eq!(words, vec!["hello", "world", "foo"]);
    }

    #[test]
    fn parse_txt_trims_whitespace() {
        let content = "  hello  \n  world  \n";
        let words = load_txt(content);
        assert_eq!(words, vec!["hello", "world"]);
    }

    #[test]
    fn word_pack_get_word() {
        let pack = WordPack::new("en", vec!["hello".to_string(), "world".to_string()]);
        assert_eq!(pack.get_word(0), Some("hello"));
        assert_eq!(pack.get_word(1), Some("world"));
        assert_eq!(pack.get_word(2), None);
    }
}
