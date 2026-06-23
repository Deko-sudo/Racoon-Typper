//! AdaptiveTextGenerator — генерация тренировочного текста
//! на основе heatmap ошибок.

use std::collections::HashMap;

use racoon_domain::keyboard::CharStat;

/// Тренировочный символ с метриками.
#[derive(Debug, Clone)]
pub struct WeakChar {
    pub ch: char,
    pub error_count: usize,
    pub total: usize,
    pub accuracy: f64,
}

/// Trait для генераторов адаптивного текста.
pub trait AdaptiveTextGenerator {
    /// Анализирует heatmap и возвращает проблемные символы.
    fn analyze(&self, char_stats: &HashMap<String, CharStat>) -> Vec<WeakChar>;

    /// Генерирует тренировочный текст из проблемных символов.
    fn generate(&self, weak_chars: &[WeakChar], word_count: usize) -> String;
}

/// FrequencyAdaptiveGenerator — MVP реализация.
/// Алгоритм: топ-N проблемных символов → слова из словаря с этими символами.
pub struct FrequencyAdaptiveGenerator {
    word_list: Vec<String>,
}

impl FrequencyAdaptiveGenerator {
    pub fn new(word_list: Vec<String>) -> Self {
        Self { word_list }
    }

    /// Создаёт генератор из встроенного словаря.
    /// Note: core crate не зависит от resources. Словарь передаётся через конструктор.
    pub fn from_word_list(words: Vec<String>) -> Self {
        Self::new(words)
    }

    /// Фильтрует слова, содержащие хотя бы один из проблемных символов.
    fn filter_words_by_chars(&self, weak_chars: &[WeakChar]) -> Vec<String> {
        let target_chars: Vec<char> = weak_chars.iter().map(|w| w.ch).collect();
        self.word_list
            .iter()
            .filter(|word| word.chars().any(|c| target_chars.contains(&c)))
            .cloned()
            .collect()
    }
}

impl AdaptiveTextGenerator for FrequencyAdaptiveGenerator {
    fn analyze(&self, char_stats: &HashMap<String, CharStat>) -> Vec<WeakChar> {
        let mut weak: Vec<WeakChar> = char_stats
            .iter()
            .filter(|(_, stat)| stat.total > 0)
            .map(|(ch, stat)| {
                let accuracy = if stat.total > 0 {
                    (stat.correct as f64 / stat.total as f64) * 100.0
                } else {
                    100.0
                };
                WeakChar {
                    ch: ch.chars().next().unwrap_or(' '),
                    error_count: stat.incorrect,
                    total: stat.total,
                    accuracy,
                }
            })
            .filter(|w| w.error_count > 0)
            .collect();

        // Сортировка по error_count descending
        weak.sort_by_key(|b| std::cmp::Reverse(b.error_count));
        weak
    }

    fn generate(&self, weak_chars: &[WeakChar], word_count: usize) -> String {
        if weak_chars.is_empty() {
            // Fallback: случайные слова
            return self
                .word_list
                .iter()
                .take(word_count)
                .cloned()
                .collect::<Vec<_>>()
                .join(" ");
        }

        let filtered = self.filter_words_by_chars(weak_chars);
        let source = if filtered.is_empty() {
            &self.word_list
        } else {
            &filtered
        };

        if source.is_empty() {
            return weak_chars
                .iter()
                .map(|w| w.ch.to_string())
                .collect::<Vec<_>>()
                .join(" ");
        }

        // Генерируем слова, приоритизируя слова с наибольшим количеством проблемных символов
        let target_chars: Vec<char> = weak_chars.iter().map(|w| w.ch).collect();
        let mut scored: Vec<(usize, &String)> = source
            .iter()
            .map(|w| {
                let score = w.chars().filter(|c| target_chars.contains(c)).count();
                (score, w)
            })
            .collect();
        scored.sort_by_key(|b| std::cmp::Reverse(b.0));

        let mut result = Vec::with_capacity(word_count);
        let mut last_idx = None;
        for _ in 0..word_count {
            let idx = if let Some(li) = last_idx {
                if source.len() > 1 {
                    let mut i = scored.len() % source.len();
                    if i == li {
                        i = (i + 1) % source.len();
                    }
                    i
                } else {
                    0
                }
            } else {
                0
            };
            result.push(scored[idx].1.clone());
            last_idx = Some(idx);
        }

        result.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_char_stat(correct: usize, incorrect: usize) -> CharStat {
        CharStat {
            correct,
            incorrect,
            total: correct + incorrect,
        }
    }

    fn make_stats() -> HashMap<String, CharStat> {
        let mut m = HashMap::new();
        m.insert("a".to_string(), make_char_stat(5, 10)); // 33% acc, 10 errors
        m.insert("e".to_string(), make_char_stat(8, 5)); // 61% acc, 5 errors
        m.insert("i".to_string(), make_char_stat(10, 1)); // 90% acc, 1 error
        m.insert("o".to_string(), make_char_stat(20, 0)); // 100% acc, 0 errors
        m
    }

    #[test]
    fn analyze_finds_weak_chars() {
        let gen = FrequencyAdaptiveGenerator::new(vec![]);
        let weak = gen.analyze(&make_stats());
        assert_eq!(weak.len(), 3); // a, e, i (o has 0 errors)
    }

    #[test]
    fn analyze_sorts_by_error_count() {
        let gen = FrequencyAdaptiveGenerator::new(vec![]);
        let weak = gen.analyze(&make_stats());
        assert_eq!(weak[0].ch, 'a'); // 10 errors
        assert_eq!(weak[1].ch, 'e'); // 5 errors
        assert_eq!(weak[2].ch, 'i'); // 1 error
    }

    #[test]
    fn analyze_calculates_accuracy() {
        let gen = FrequencyAdaptiveGenerator::new(vec![]);
        let weak = gen.analyze(&make_stats());
        assert!((weak[0].accuracy - 33.33).abs() < 1.0);
    }

    #[test]
    fn analyze_empty_stats() {
        let gen = FrequencyAdaptiveGenerator::new(vec![]);
        let weak = gen.analyze(&HashMap::new());
        assert!(weak.is_empty());
    }

    #[test]
    fn analyze_no_errors() {
        let gen = FrequencyAdaptiveGenerator::new(vec![]);
        let mut m = HashMap::new();
        m.insert("x".to_string(), make_char_stat(10, 0));
        let weak = gen.analyze(&m);
        assert!(weak.is_empty());
    }

    #[test]
    fn generate_from_weak_chars() {
        let gen = FrequencyAdaptiveGenerator::new(vec![
            "cat".to_string(),
            "bat".to_string(),
            "rat".to_string(),
            "hello".to_string(),
        ]);
        let weak = vec![WeakChar {
            ch: 'a',
            error_count: 10,
            total: 15,
            accuracy: 33.0,
        }];
        let text = gen.generate(&weak, 3);
        let words: Vec<&str> = text.split(' ').collect();
        assert_eq!(words.len(), 3);
        // Все слова должны содержать 'a'
        for w in &words {
            assert!(w.contains('a'), "Word '{}' should contain 'a'", w);
        }
    }

    #[test]
    fn generate_empty_weak_falls_back() {
        let gen = FrequencyAdaptiveGenerator::new(vec!["hello".to_string(), "world".to_string()]);
        let text = gen.generate(&[], 2);
        let words: Vec<&str> = text.split(' ').collect();
        assert_eq!(words.len(), 2);
    }

    #[test]
    fn generate_no_matching_words_uses_fallback() {
        let gen = FrequencyAdaptiveGenerator::new(vec!["xyz".to_string(), "qwe".to_string()]);
        let weak = vec![WeakChar {
            ch: 'a',
            error_count: 5,
            total: 10,
            accuracy: 50.0,
        }];
        let text = gen.generate(&weak, 2);
        let words: Vec<&str> = text.split(' ').collect();
        assert_eq!(words.len(), 2);
    }

    #[test]
    fn generate_correct_word_count() {
        let gen = FrequencyAdaptiveGenerator::new(vec![
            "cat".to_string(),
            "bat".to_string(),
            "rat".to_string(),
        ]);
        let weak = vec![WeakChar {
            ch: 'a',
            error_count: 5,
            total: 10,
            accuracy: 50.0,
        }];
        let text = gen.generate(&weak, 10);
        let words: Vec<&str> = text.split(' ').collect();
        assert_eq!(words.len(), 10);
    }

    #[test]
    fn generate_empty_word_list_uses_chars() {
        let gen = FrequencyAdaptiveGenerator::new(vec![]);
        let weak = vec![
            WeakChar {
                ch: 'a',
                error_count: 5,
                total: 10,
                accuracy: 50.0,
            },
            WeakChar {
                ch: 'e',
                error_count: 3,
                total: 10,
                accuracy: 70.0,
            },
        ];
        let text = gen.generate(&weak, 5);
        assert!(!text.is_empty());
    }

    #[test]
    fn from_word_list_en() {
        let gen = FrequencyAdaptiveGenerator::from_word_list(vec![
            "the".to_string(),
            "be".to_string(),
            "to".to_string(),
        ]);
        assert!(!gen.word_list.is_empty());
    }

    #[test]
    fn from_word_list_ru() {
        let gen = FrequencyAdaptiveGenerator::from_word_list(vec![
            "и".to_string(),
            "в".to_string(),
            "не".to_string(),
        ]);
        assert!(!gen.word_list.is_empty());
    }

    #[test]
    fn from_word_list_empty() {
        let gen = FrequencyAdaptiveGenerator::from_word_list(vec![]);
        assert!(gen.word_list.is_empty());
    }
}
