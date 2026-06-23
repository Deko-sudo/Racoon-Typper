//! Finger mapping — какая клавиша каким пальцем нажимается.
//! QWERTY и JCUKEN раскладки.

use serde::{Deserialize, Serialize};

/// Палец.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Finger {
    LeftPinky,
    LeftRing,
    LeftMiddle,
    LeftIndex,
    LeftThumb,
    RightThumb,
    RightIndex,
    RightMiddle,
    RightRing,
    RightPinky,
}

impl Finger {
    pub fn display_name(&self) -> &str {
        match self {
            Finger::LeftPinky => "Left Pinky",
            Finger::LeftRing => "Left Ring",
            Finger::LeftMiddle => "Left Middle",
            Finger::LeftIndex => "Left Index",
            Finger::LeftThumb => "Left Thumb",
            Finger::RightThumb => "Right Thumb",
            Finger::RightIndex => "Right Index",
            Finger::RightMiddle => "Right Middle",
            Finger::RightRing => "Right Ring",
            Finger::RightPinky => "Right Pinky",
        }
    }

    pub fn is_left(&self) -> bool {
        matches!(
            self,
            Finger::LeftPinky
                | Finger::LeftRing
                | Finger::LeftMiddle
                | Finger::LeftIndex
                | Finger::LeftThumb
        )
    }
}

/// Сопоставление клавиша → палец для QWERTY.
pub fn finger_for_key_qwerty(ch: char) -> Finger {
    match ch.to_ascii_lowercase() {
        'q' | 'a' | 'z' | '1' => Finger::LeftPinky,
        'w' | 's' | 'x' | '2' => Finger::LeftRing,
        'e' | 'd' | 'c' | '3' => Finger::LeftMiddle,
        'r' | 'f' | 'v' | 't' | 'g' | 'b' | '4' | '5' => Finger::LeftIndex,
        'y' | 'h' | 'n' | 'u' | 'j' | 'm' | '6' | '7' => Finger::RightIndex,
        'i' | 'k' | ',' | '8' => Finger::RightMiddle,
        'o' | 'l' | '.' | '9' => Finger::RightRing,
        'p' | ';' | '/' | '0' | '-' | '=' | '\'' => Finger::RightPinky,
        ' ' => Finger::RightThumb,
        _ => Finger::LeftPinky,
    }
}

/// Сопоставление клавиша → палец для JCUKEN (русская).
pub fn finger_for_key_jcuken(ch: char) -> Finger {
    match ch.to_lowercase().next().unwrap_or(ch) {
        'ф' | 'я' | 'ё' => Finger::LeftPinky,
        'ы' | 'ч' | 'ц' | '1' => Finger::LeftRing,
        'в' | 'с' | 'у' | '2' => Finger::LeftMiddle,
        'а' | 'п' | 'к' | 'м' | '3' | '4' => Finger::LeftIndex,
        'о' | 'л' | 'д' | 'р' | 'т' | '5' | '6' => Finger::RightIndex,
        'е' | 'г' | 'ш' | '7' => Finger::RightMiddle,
        'н' | 'щ' | 'з' | '8' => Finger::RightRing,
        'ь' | 'б' | 'ю' | 'ъ' | '9' | '0' => Finger::RightPinky,
        ' ' => Finger::RightThumb,
        _ => Finger::LeftPinky,
    }
}

/// Возвращает палец для символа в зависимости от раскладки.
pub fn finger_for_char(ch: char, is_russian: bool) -> Finger {
    if is_russian {
        // Проверяем, русский ли символ
        if "абвгдеёжзийклмнопрстуфхцчшщъыьэюя".contains(ch.to_lowercase().next().unwrap_or(' '))
        {
            finger_for_key_jcuken(ch)
        } else {
            finger_for_key_qwerty(ch)
        }
    } else {
        finger_for_key_qwerty(ch)
    }
}

/// Home Row клавиши для QWERTY.
pub const QWERTY_HOME_ROW: &[char] = &['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';'];

/// Home Row клавиши для JCUKEN.
pub const JCUKEN_HOME_ROW: &[char] = &['ф', 'ы', 'в', 'а', 'п', 'о', 'л', 'д', 'ж', 'э'];

/// Проверяет, находится ли символ на Home Row.
pub fn is_home_row(ch: char, is_russian: bool) -> bool {
    let lower = ch.to_lowercase().next().unwrap_or(ch);
    if is_russian {
        JCUKEN_HOME_ROW.contains(&lower)
    } else {
        QWERTY_HOME_ROW.contains(&lower)
    }
}

/// QWERTY строки для отображения клавиатуры.
pub const QWERTY_ROWS: &[&[&str]] = &[
    &["q", "w", "e", "r", "t", "y", "u", "i", "o", "p"],
    &["a", "s", "d", "f", "g", "h", "j", "k", "l"],
    &["z", "x", "c", "v", "b", "n", "m"],
];

/// JCUKEN строки для отображения клавиатуры.
pub const JCUKEN_ROWS: &[&[&str]] = &[
    &["й", "ц", "у", "к", "е", "н", "г", "ш", "щ", "з", "х", "ъ"],
    &["ф", "ы", "в", "а", "п", "р", "о", "л", "д", "ж", "э"],
    &["я", "ч", "с", "м", "и", "т", "ь", "б", "ю"],
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finger_for_qwerty_home_row() {
        assert_eq!(finger_for_key_qwerty('a'), Finger::LeftPinky);
        assert_eq!(finger_for_key_qwerty('s'), Finger::LeftRing);
        assert_eq!(finger_for_key_qwerty('d'), Finger::LeftMiddle);
        assert_eq!(finger_for_key_qwerty('f'), Finger::LeftIndex);
        assert_eq!(finger_for_key_qwerty('j'), Finger::RightIndex);
        assert_eq!(finger_for_key_qwerty('k'), Finger::RightMiddle);
        assert_eq!(finger_for_key_qwerty('l'), Finger::RightRing);
        assert_eq!(finger_for_key_qwerty(';'), Finger::RightPinky);
    }

    #[test]
    fn finger_for_qwerty_top_row() {
        assert_eq!(finger_for_key_qwerty('q'), Finger::LeftPinky);
        assert_eq!(finger_for_key_qwerty('w'), Finger::LeftRing);
        assert_eq!(finger_for_key_qwerty('e'), Finger::LeftMiddle);
        assert_eq!(finger_for_key_qwerty('r'), Finger::LeftIndex);
        assert_eq!(finger_for_key_qwerty('y'), Finger::RightIndex);
        assert_eq!(finger_for_key_qwerty('i'), Finger::RightMiddle);
        assert_eq!(finger_for_key_qwerty('o'), Finger::RightRing);
        assert_eq!(finger_for_key_qwerty('p'), Finger::RightPinky);
    }

    #[test]
    fn finger_for_qwerty_bottom_row() {
        assert_eq!(finger_for_key_qwerty('z'), Finger::LeftPinky);
        assert_eq!(finger_for_key_qwerty('x'), Finger::LeftRing);
        assert_eq!(finger_for_key_qwerty('c'), Finger::LeftMiddle);
        assert_eq!(finger_for_key_qwerty('v'), Finger::LeftIndex);
        assert_eq!(finger_for_key_qwerty('n'), Finger::RightIndex);
        assert_eq!(finger_for_key_qwerty('m'), Finger::RightIndex);
    }

    #[test]
    fn finger_for_space() {
        assert_eq!(finger_for_key_qwerty(' '), Finger::RightThumb);
    }

    #[test]
    fn finger_for_uppercase() {
        assert_eq!(finger_for_key_qwerty('A'), Finger::LeftPinky);
        assert_eq!(finger_for_key_qwerty('F'), Finger::LeftIndex);
        assert_eq!(finger_for_key_qwerty('J'), Finger::RightIndex);
    }

    #[test]
    fn finger_for_jcuken_home_row() {
        assert_eq!(finger_for_key_jcuken('ф'), Finger::LeftPinky);
        assert_eq!(finger_for_key_jcuken('ы'), Finger::LeftRing);
        assert_eq!(finger_for_key_jcuken('в'), Finger::LeftMiddle);
        assert_eq!(finger_for_key_jcuken('а'), Finger::LeftIndex);
        assert_eq!(finger_for_key_jcuken('о'), Finger::RightIndex);
    }

    #[test]
    fn finger_for_char_russian() {
        assert_eq!(finger_for_char('ф', true), Finger::LeftPinky);
        assert_eq!(finger_for_char('а', true), Finger::LeftIndex);
    }

    #[test]
    fn finger_for_char_english() {
        assert_eq!(finger_for_char('a', false), Finger::LeftPinky);
        assert_eq!(finger_for_char('f', false), Finger::LeftIndex);
    }

    #[test]
    fn finger_is_left() {
        assert!(Finger::LeftPinky.is_left());
        assert!(Finger::LeftIndex.is_left());
        assert!(!Finger::RightIndex.is_left());
        assert!(!Finger::RightPinky.is_left());
    }

    #[test]
    fn finger_display_name() {
        assert_eq!(Finger::LeftIndex.display_name(), "Left Index");
        assert_eq!(Finger::RightPinky.display_name(), "Right Pinky");
    }

    #[test]
    fn is_home_row_qwerty() {
        assert!(is_home_row('a', false));
        assert!(is_home_row('s', false));
        assert!(is_home_row('f', false));
        assert!(is_home_row('j', false));
        assert!(!is_home_row('q', false));
        assert!(!is_home_row('z', false));
    }

    #[test]
    fn is_home_row_jcuken() {
        assert!(is_home_row('ф', true));
        assert!(is_home_row('ы', true));
        assert!(!is_home_row('й', true));
    }

    #[test]
    fn is_home_row_uppercase() {
        assert!(is_home_row('A', false));
        assert!(is_home_row('F', false));
    }
}
