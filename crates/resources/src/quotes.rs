//! QuoteLoader — загрузка цитат из .toml файлов.
//! Формат: [[quotes]] array с полями text и author.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;

/// Одна цитата.
#[derive(Debug, Clone, Deserialize)]
pub struct Quote {
    pub text: String,
    pub author: String,
}

/// Коллекция цитат для конкретного языка.
pub struct QuotePack {
    pub language: String,
    pub quotes: Vec<Quote>,
}

impl QuotePack {
    pub fn new(language: &str, quotes: Vec<Quote>) -> Self {
        Self {
            language: language.to_string(),
            quotes,
        }
    }

    pub fn len(&self) -> usize {
        self.quotes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.quotes.is_empty()
    }

    pub fn get_random(&self) -> Option<&Quote> {
        if self.quotes.is_empty() {
            return None;
        }
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as usize;
        let idx = seed % self.quotes.len();
        self.quotes.get(idx)
    }

    pub fn get_by_index(&self, index: usize) -> Option<&Quote> {
        self.quotes.get(index)
    }
}

/// Загрузчик цитат. Кэширует загруженные коллекции.
pub struct QuoteLoader {
    packs: HashMap<String, QuotePack>,
}

impl QuoteLoader {
    pub fn new() -> Self {
        let mut packs = HashMap::new();

        let en_quotes = load_toml(include_str!("../../../resources/quotes/en.toml"));
        if !en_quotes.is_empty() {
            packs.insert("en".to_string(), QuotePack::new("en", en_quotes));
        }

        let ru_quotes = load_toml(include_str!("../../../resources/quotes/ru.toml"));
        if !ru_quotes.is_empty() {
            packs.insert("ru".to_string(), QuotePack::new("ru", ru_quotes));
        }

        Self { packs }
    }

    /// Возвращает коллекцию цитат для языка.
    pub fn get_pack(&self, language: &str) -> Option<&QuotePack> {
        self.packs.get(language)
    }

    /// Возвращает случайную цитату для языка.
    pub fn get_random_quote(&self, language: &str) -> Option<&Quote> {
        self.get_pack(language)?.get_random()
    }

    /// Возвращает цитату по индексу.
    pub fn get_quote_by_index(&self, language: &str, index: usize) -> Option<&Quote> {
        self.get_pack(language)?.get_by_index(index)
    }

    /// Возвращает список доступных языков.
    pub fn available_languages(&self) -> Vec<String> {
        self.packs.keys().cloned().collect()
    }
}

impl Default for QuoteLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Глобальный синглтон для QuoteLoader.
static LOADER: OnceLock<QuoteLoader> = OnceLock::new();

/// Возвращает глобальный QuoteLoader.
pub fn quote_loader() -> &'static QuoteLoader {
    LOADER.get_or_init(QuoteLoader::new)
}

/// Парсит TOML файл с цитатами.
fn load_toml(content: &str) -> Vec<Quote> {
    #[derive(Deserialize)]
    struct QuoteFile {
        quotes: Vec<Quote>,
    }

    toml::from_str::<QuoteFile>(content)
        .map(|f| f.quotes)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_en_quotes() {
        let loader = QuoteLoader::new();
        let pack = loader.get_pack("en").unwrap();
        assert!(!pack.is_empty());
        assert!(pack.len() >= 10);
    }

    #[test]
    fn load_ru_quotes() {
        let loader = QuoteLoader::new();
        let pack = loader.get_pack("ru").unwrap();
        assert!(!pack.is_empty());
        assert!(pack.len() >= 10);
    }

    #[test]
    fn get_random_quote_en() {
        let loader = QuoteLoader::new();
        let quote = loader.get_random_quote("en").unwrap();
        assert!(!quote.text.is_empty());
        assert!(!quote.author.is_empty());
    }

    #[test]
    fn get_random_quote_ru() {
        let loader = QuoteLoader::new();
        let quote = loader.get_random_quote("ru").unwrap();
        assert!(!quote.text.is_empty());
        assert!(!quote.author.is_empty());
    }

    #[test]
    fn get_quote_by_index() {
        let loader = QuoteLoader::new();
        let quote = loader.get_quote_by_index("en", 0).unwrap();
        assert!(!quote.text.is_empty());
    }

    #[test]
    fn get_quote_nonexistent_language() {
        let loader = QuoteLoader::new();
        assert!(loader.get_pack("fr").is_none());
    }

    #[test]
    fn available_languages_has_both() {
        let loader = QuoteLoader::new();
        let langs = loader.available_languages();
        assert!(langs.contains(&"en".to_string()));
        assert!(langs.contains(&"ru".to_string()));
    }

    #[test]
    fn quote_text_is_not_empty() {
        let loader = QuoteLoader::new();
        let pack = loader.get_pack("en").unwrap();
        for q in &pack.quotes {
            assert!(!q.text.is_empty(), "Empty quote text found");
        }
    }

    #[test]
    fn parse_toml_valid() {
        let toml_content = r#"
[[quotes]]
text = "Hello world"
author = "Test"

[[quotes]]
text = "Goodbye"
author = "Test2"
"#;
        let quotes = load_toml(toml_content);
        assert_eq!(quotes.len(), 2);
        assert_eq!(quotes[0].text, "Hello world");
        assert_eq!(quotes[1].author, "Test2");
    }

    #[test]
    fn parse_toml_invalid_returns_empty() {
        let quotes = load_toml("invalid toml {{{");
        assert!(quotes.is_empty());
    }

    #[test]
    fn quote_pack_get_random_multiple_calls() {
        let pack = QuotePack::new(
            "en",
            vec![
                Quote {
                    text: "one".to_string(),
                    author: "a".to_string(),
                },
                Quote {
                    text: "two".to_string(),
                    author: "b".to_string(),
                },
            ],
        );
        // Множественные вызовы не должны паниковать
        for _ in 0..10 {
            let q = pack.get_random().unwrap();
            assert!(!q.text.is_empty());
        }
    }
}
