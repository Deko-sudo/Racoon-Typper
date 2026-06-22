//! Input Engine — классификация и валидация клавиатурных событий.

use serde::{Deserialize, Serialize};

/// Действие клавиши после классификации.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyAction {
    /// Печатный символ.
    Print(char),
    /// Удаление предыдущего символа.
    Backspace,
    /// Подтверждение / следующая строка.
    Enter,
    /// Прерывание теста.
    Escape,
    /// Quick restart / переход.
    Tab,
    /// Игнор (modifier-only и т.д.).
    Ignore,
}

/// Сырое клавиатурное событие из frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEvent {
    pub key: String,
    pub code: String,
    pub timestamp: u64,
}

/// Результат валидации ввода.
#[derive(Debug, Clone, PartialEq)]
pub enum InputValidation {
    /// Принято, классифицировано.
    Ok(KeyAction),
    /// Тест не активен.
    NoActiveTest,
    /// Тест завершён.
    TestEnded,
}

/// Классификатор клавиш.
pub struct KeyClassifier;

impl KeyClassifier {
    /// Классифицирует сырое событие в KeyAction.
    pub fn classify(key: &str, code: &str) -> KeyAction {
        // Специальные клавиши
        match key {
            "Backspace" => return KeyAction::Backspace,
            "Enter" => return KeyAction::Enter,
            "Escape" => return KeyAction::Escape,
            "Tab" => return KeyAction::Tab,
            _ => {}
        }

        // Игнорируем modifier-only нажатия
        let modifier_codes = [
            "ShiftLeft",
            "ShiftRight",
            "ControlLeft",
            "ControlRight",
            "AltLeft",
            "AltRight",
            "MetaLeft",
            "MetaRight",
            "CapsLock",
            "ContextMenu",
        ];
        if modifier_codes.contains(&code) {
            return KeyAction::Ignore;
        }

        // Печатный символ
        // key может быть "a", "A", "1", " " и т.д.
        // Берем первый char
        if let Some(ch) = key.chars().next() {
            // Проверяем что это печатный символ (не control char)
            if !ch.is_control() || ch == ' ' || ch == '\t' {
                return KeyAction::Print(ch);
            }
        }

        KeyAction::Ignore
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_print() {
        assert_eq!(KeyClassifier::classify("a", "KeyA"), KeyAction::Print('a'));
        assert_eq!(KeyClassifier::classify("A", "KeyA"), KeyAction::Print('A'));
        assert_eq!(
            KeyClassifier::classify("1", "Digit1"),
            KeyAction::Print('1')
        );
    }

    #[test]
    fn classify_space() {
        assert_eq!(KeyClassifier::classify(" ", "Space"), KeyAction::Print(' '));
    }

    #[test]
    fn classify_backspace() {
        assert_eq!(
            KeyClassifier::classify("Backspace", "Backspace"),
            KeyAction::Backspace
        );
    }

    #[test]
    fn classify_enter() {
        assert_eq!(KeyClassifier::classify("Enter", "Enter"), KeyAction::Enter);
    }

    #[test]
    fn classify_escape() {
        assert_eq!(
            KeyClassifier::classify("Escape", "Escape"),
            KeyAction::Escape
        );
    }

    #[test]
    fn classify_tab() {
        assert_eq!(KeyClassifier::classify("Tab", "Tab"), KeyAction::Tab);
    }

    #[test]
    fn classify_shift_ignored() {
        assert_eq!(
            KeyClassifier::classify("Shift", "ShiftLeft"),
            KeyAction::Ignore
        );
        assert_eq!(
            KeyClassifier::classify("Control", "ControlRight"),
            KeyAction::Ignore
        );
    }

    #[test]
    fn classify_empty_ignored() {
        assert_eq!(KeyClassifier::classify("", ""), KeyAction::Ignore);
    }
}
