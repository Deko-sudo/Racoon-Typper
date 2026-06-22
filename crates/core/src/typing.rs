//! Typing Engine — TextBuffer, TypedChar, CaretState.
//! Обрабатывает нажатия, обновляет позицию, отслеживает ошибки.

use std::time::Instant;

use racoon_domain::stats::TypedChar;
use racoon_domain::CharStatus;

/// Результат обработки нажатия в Typing Engine.
#[derive(Debug, Clone, PartialEq)]
pub enum TypingResult {
    /// Символ введён правильно.
    Correct,
    /// Символ введён неправильно.
    Incorrect,
    /// Backspace на правильном символе — снята пометка.
    UndoneCorrect,
    /// Backspace на ошибочном символе — снята пометка.
    UndoneIncorrect,
    /// Игнор (backspace в начале, ввод после завершения).
    Noop,
    /// Тест завершён, ввод игнорируется.
    TestEnded,
}

/// Текстовый буфер — состояние печатаемого текста.
#[derive(Debug, Clone)]
pub struct TextBuffer {
    /// Полный целевой текст.
    pub full_text: String,
    /// Каждый введённый символ с метаданными.
    pub typed_chars: Vec<TypedChar>,
    /// Индекс текущего ожидаемого символа.
    pub current_position: usize,
    /// Момент старта таймера (первое нажатие).
    pub start_time: Option<Instant>,
    /// Завершён ли тест.
    pub is_complete: bool,
}

impl TextBuffer {
    /// Создаёт новый TextBuffer для заданного текста.
    pub fn new(text: &str) -> Self {
        let typed_chars: Vec<TypedChar> = text.chars().map(TypedChar::new).collect();
        Self {
            full_text: text.to_string(),
            typed_chars,
            current_position: 0,
            start_time: None,
            is_complete: false,
        }
    }

    /// Обрабатывает печатный символ.
    pub fn process_print(&mut self, ch: char, timestamp: u64) -> TypingResult {
        if self.is_complete {
            return TypingResult::TestEnded;
        }
        if self.current_position >= self.typed_chars.len() {
            return TypingResult::Noop;
        }

        // Старт таймера на первое нажатие
        if self.start_time.is_none() {
            self.start_time = Some(Instant::now());
        }

        let tc = &mut self.typed_chars[self.current_position];
        let expected = tc.expected;

        // Заполняем метаданные
        tc.typed = Some(ch);
        tc.timestamp_ms = Some(timestamp);

        // first_typed — первая попытка для этой позиции
        if tc.first_typed.is_none() {
            tc.first_typed = Some(ch);
            tc.first_correct = ch == expected;
        }

        if ch == expected {
            tc.status = CharStatus::Correct;
            self.current_position += 1;

            // Проверка завершения
            if self.current_position >= self.typed_chars.len() {
                self.is_complete = true;
            }
            TypingResult::Correct
        } else {
            tc.status = CharStatus::Incorrect;
            // Не продвигаем caret при ошибке
            TypingResult::Incorrect
        }
    }

    /// Обрабатывает backspace.
    pub fn process_backspace(&mut self) -> TypingResult {
        if self.is_complete {
            return TypingResult::TestEnded;
        }

        // Проверяем: есть ли incorrect символ на текущей позиции?
        // При incorrect caret не двигается, но символ помечен incorrect.
        // Backspace должен снять incorrect, не откатывать previous correct.
        if self.current_position < self.typed_chars.len() {
            let current_status = self.typed_chars[self.current_position].status.clone();
            if current_status == CharStatus::Incorrect {
                let tc = &mut self.typed_chars[self.current_position];
                tc.status = CharStatus::Pending;
                tc.typed = None;
                tc.timestamp_ms = None;
                // first_typed сохраняется для heatmap
                return TypingResult::UndoneIncorrect;
            }
        }

        // Обычный backspace — откат на предыдущую позицию
        if self.current_position == 0 {
            return TypingResult::Noop;
        }

        self.current_position -= 1;
        let tc = &mut self.typed_chars[self.current_position];

        let was_correct = tc.status == CharStatus::Correct;
        let was_incorrect = tc.status == CharStatus::Incorrect;

        tc.status = CharStatus::Backspaced;
        tc.typed = None;
        tc.timestamp_ms = None;

        if was_correct {
            TypingResult::UndoneCorrect
        } else if was_incorrect {
            TypingResult::UndoneIncorrect
        } else {
            TypingResult::Noop
        }
    }

    /// Возвращает текущий ожидаемый символ.
    pub fn current_expected(&self) -> Option<char> {
        if self.current_position < self.typed_chars.len() {
            Some(self.typed_chars[self.current_position].expected)
        } else {
            None
        }
    }

    /// Время от старта в мс (если таймер запущен).
    pub fn elapsed_ms(&self) -> u64 {
        self.start_time
            .map(|t| t.elapsed().as_millis() as u64)
            .unwrap_or(0)
    }

    /// Количество корректных символов (не backspaced).
    pub fn correct_chars(&self) -> usize {
        self.typed_chars
            .iter()
            .filter(|tc| tc.status == CharStatus::Correct)
            .count()
    }

    /// Количество ошибочных символов (финальных, не backspaced).
    pub fn incorrect_chars(&self) -> usize {
        self.typed_chars
            .iter()
            .filter(|tc| tc.status == CharStatus::Incorrect)
            .count()
    }

    /// Количество backspace операций.
    pub fn backspace_count(&self) -> usize {
        self.typed_chars
            .iter()
            .filter(|tc| tc.status == CharStatus::Backspaced)
            .count()
    }

    /// Статус символа на позиции для рендера.
    pub fn char_status_at(&self, pos: usize) -> Option<CharStatus> {
        self.typed_chars.get(pos).map(|tc| tc.status.clone())
    }

    /// Введённый символ на позиции (для рендера).
    pub fn typed_at(&self, pos: usize) -> Option<char> {
        self.typed_chars.get(pos).and_then(|tc| tc.typed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_character() {
        let mut buf = TextBuffer::new("hello");
        let result = buf.process_print('h', 100);
        assert_eq!(result, TypingResult::Correct);
        assert_eq!(buf.current_position, 1);
        assert_eq!(buf.char_status_at(0), Some(CharStatus::Correct));
        assert_eq!(buf.typed_at(0), Some('h'));
    }

    #[test]
    fn incorrect_character() {
        let mut buf = TextBuffer::new("hello");
        let result = buf.process_print('x', 100);
        assert_eq!(result, TypingResult::Incorrect);
        assert_eq!(buf.current_position, 0); // не двигается
        assert_eq!(buf.char_status_at(0), Some(CharStatus::Incorrect));
        assert_eq!(buf.typed_at(0), Some('x'));
    }

    #[test]
    fn backspace_on_correct() {
        let mut buf = TextBuffer::new("hello");
        buf.process_print('h', 100);
        assert_eq!(buf.current_position, 1);

        let result = buf.process_backspace();
        assert_eq!(result, TypingResult::UndoneCorrect);
        assert_eq!(buf.current_position, 0);
        assert_eq!(buf.char_status_at(0), Some(CharStatus::Backspaced));
    }

    #[test]
    fn backspace_on_incorrect() {
        let mut buf = TextBuffer::new("hello");
        buf.process_print('x', 100); // incorrect
        assert_eq!(buf.current_position, 0);

        // Backspace на incorrect — теперь снимает incorrect пометку (Pending)
        let result = buf.process_backspace();
        assert_eq!(result, TypingResult::UndoneIncorrect);
        assert_eq!(buf.current_position, 0); // caret не двигается
    }

    #[test]
    fn backspace_in_beginning_is_noop() {
        let mut buf = TextBuffer::new("hello");
        let result = buf.process_backspace();
        assert_eq!(result, TypingResult::Noop);
        assert_eq!(buf.current_position, 0);
    }

    #[test]
    fn first_typed_preserved_after_backspace() {
        let mut buf = TextBuffer::new("hello");
        buf.process_print('x', 100); // incorrect, first_typed = 'x'

        // Backspace на incorrect снимает пометку, но first_typed сохраняется
        buf.process_backspace(); // UndoneIncorrect

        // Проверяем что first_typed сохранён
        let tc = &buf.typed_chars[0];
        assert_eq!(tc.first_typed, Some('x'));
        assert!(!tc.first_correct);

        // Теперь вводим правильно
        buf.process_print('h', 200); // correct
        assert_eq!(buf.current_position, 1);
    }

    #[test]
    fn text_completion() {
        let mut buf = TextBuffer::new("hi");
        buf.process_print('h', 100);
        buf.process_print('i', 200);
        assert!(buf.is_complete);
        assert_eq!(buf.current_position, 2);

        // Ввод после завершения
        let result = buf.process_print('x', 300);
        assert_eq!(result, TypingResult::TestEnded);
    }

    #[test]
    fn timer_starts_on_first_key() {
        let mut buf = TextBuffer::new("hello");
        assert!(buf.start_time.is_none());

        buf.process_print('h', 100);
        assert!(buf.start_time.is_some());
    }

    #[test]
    fn full_sentence_typing() {
        let text = "The quick brown fox jumps over the lazy dog";
        let mut buf = TextBuffer::new(text);

        for ch in text.chars() {
            let result = buf.process_print(ch, 0);
            assert_eq!(result, TypingResult::Correct, "Failed on char: '{}'", ch);
        }

        assert!(buf.is_complete);
        assert_eq!(buf.correct_chars(), text.len());
        assert_eq!(buf.incorrect_chars(), 0);
    }

    #[test]
    fn backspace_after_advance() {
        let mut buf = TextBuffer::new("abc");
        buf.process_print('a', 100);
        buf.process_print('b', 200);
        assert_eq!(buf.current_position, 2);

        // Backspace на 'b' (correct)
        let result = buf.process_backspace();
        assert_eq!(result, TypingResult::UndoneCorrect);
        assert_eq!(buf.current_position, 1);

        // Вводим 'b' снова
        let result = buf.process_print('b', 300);
        assert_eq!(result, TypingResult::Correct);
        assert_eq!(buf.current_position, 2);
    }

    #[test]
    fn backspace_then_retype_preserves_first_typed() {
        let mut buf = TextBuffer::new("abc");
        buf.process_print('x', 100); // incorrect at pos 0
        buf.process_print('a', 200); // still at pos 0, now correct

        // first_typed для pos 0 должен быть 'x' (первая попытка)
        assert_eq!(buf.typed_chars[0].first_typed, Some('x'));
        assert!(!buf.typed_chars[0].first_correct);

        // Но статус сейчас Correct
        assert_eq!(buf.char_status_at(0), Some(CharStatus::Correct));
    }

    #[test]
    fn unicode_cyrillic_text() {
        let text = "привет мир";
        let mut buf = TextBuffer::new(text);

        for ch in text.chars() {
            let result = buf.process_print(ch, 0);
            assert_eq!(result, TypingResult::Correct, "Failed on char: '{}'", ch);
        }
        assert!(buf.is_complete);
        assert_eq!(buf.correct_chars(), 10); // 10 chars including space
    }

    #[test]
    fn unicode_emoji_text() {
        let text = "hello 🦝 world";
        let mut buf = TextBuffer::new(text);

        // 'h','e','l','l','o',' ','🦝',' ','w','o','r','l','d'
        // 🦝 — один char в Rust (4 байта UTF-8)
        let chars: Vec<char> = text.chars().collect();
        assert_eq!(chars.len(), 13);

        for ch in text.chars() {
            let result = buf.process_print(ch, 0);
            assert_eq!(result, TypingResult::Correct, "Failed on char: '{}'", ch);
        }
        assert!(buf.is_complete);
    }

    #[test]
    fn unicode_mixed_script() {
        let text = "hello мир 123";
        let mut buf = TextBuffer::new(text);

        for ch in text.chars() {
            buf.process_print(ch, 0);
        }
        assert!(buf.is_complete);
        assert_eq!(buf.correct_chars(), text.chars().count());
    }

    #[test]
    fn utf8_multibyte_position_correct() {
        let text = "ёж";
        let mut buf = TextBuffer::new(text);
        // 'ё' — 2 байта в UTF-8, но 1 char в Rust
        buf.process_print('ё', 100);
        assert_eq!(buf.current_position, 1);
        buf.process_print('ж', 200);
        assert_eq!(buf.current_position, 2);
        assert!(buf.is_complete);
    }

    #[test]
    fn wrong_backspace_correct_sequence() {
        let mut buf = TextBuffer::new("hello");

        // 1. Wrong: 'x' вместо 'h'
        let r = buf.process_print('x', 100);
        assert_eq!(r, TypingResult::Incorrect);
        assert_eq!(buf.current_position, 0);

        // first_typed = 'x', first_correct = false
        assert_eq!(buf.typed_chars[0].first_typed, Some('x'));
        assert!(!buf.typed_chars[0].first_correct);

        // 2. Backspace на incorrect — снимает incorrect (UndoneIncorrect)
        let r = buf.process_backspace();
        assert_eq!(r, TypingResult::UndoneIncorrect);
        assert_eq!(buf.current_position, 0);

        // 3. Correct: 'h'
        let r = buf.process_print('h', 200);
        assert_eq!(r, TypingResult::Correct);
        assert_eq!(buf.current_position, 1);

        // first_typed сохранён = 'x' (для heatmap)
        assert_eq!(buf.typed_chars[0].first_typed, Some('x'));
        assert!(!buf.typed_chars[0].first_correct);

        // Статус = Correct
        assert_eq!(buf.char_status_at(0), Some(CharStatus::Correct));
    }

    #[test]
    fn wrong_backspace_correct_after_advance() {
        let mut buf = TextBuffer::new("abc");

        // Правильно: 'a'
        buf.process_print('a', 100);
        assert_eq!(buf.current_position, 1);

        // Неправильно: 'x' вместо 'b'
        buf.process_print('x', 200);
        assert_eq!(buf.current_position, 1); // не двигается

        // Backspace на incorrect — снимает incorrect (UndoneIncorrect)
        let r = buf.process_backspace();
        assert_eq!(r, TypingResult::UndoneIncorrect);
        assert_eq!(buf.current_position, 1);

        // Теперь вводим правильно: 'b'
        buf.process_print('b', 400);
        assert_eq!(buf.current_position, 2);
    }

    #[test]
    fn long_text_1000_chars() {
        let text = "a ".repeat(500); // 1000 chars
        assert_eq!(text.len(), 1000);

        let mut buf = TextBuffer::new(&text);
        let chars: Vec<char> = text.chars().collect();
        assert_eq!(chars.len(), 1000);

        for ch in text.chars() {
            let r = buf.process_print(ch, 0);
            assert_eq!(r, TypingResult::Correct);
        }
        assert!(buf.is_complete);
        assert_eq!(buf.correct_chars(), 1000);
    }

    #[test]
    fn long_text_with_errors_and_backspace() {
        // 500 символов с ошибками и исправлениями
        let text = "the quick brown fox ".repeat(25); // 500 chars
        let mut buf = TextBuffer::new(&text);

        let chars: Vec<char> = text.chars().collect();
        for (i, ch) in chars.iter().enumerate() {
            if i % 50 == 10 {
                // Ошибка каждые 50 символов
                buf.process_print('X', i as u64);
                // Исправляем
                buf.process_backspace();
            }
            buf.process_print(*ch, i as u64 + 1000);
        }
        assert!(buf.is_complete);
        assert_eq!(buf.correct_chars(), text.chars().count());
    }
}
