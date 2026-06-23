//! Viewport logic tests — проверка скроллинга текста.

/// Вычисляет видимое окно текста вокруг курсора.
pub fn calc_viewport(
    caret_pos: usize,
    text_len: usize,
    window: usize,
    padding: usize,
) -> (usize, usize) {
    if text_len <= window {
        return (0, text_len);
    }
    let start = caret_pos.saturating_sub(padding);
    let end = (start + window).min(text_len);
    (start, end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_short_text_full() {
        let (start, end) = calc_viewport(5, 20, 80, 20);
        assert_eq!(start, 0);
        assert_eq!(end, 20);
    }

    #[test]
    fn viewport_start_of_text() {
        let (start, end) = calc_viewport(0, 200, 80, 20);
        assert_eq!(start, 0);
        assert_eq!(end, 80);
    }

    #[test]
    fn viewport_middle_of_text() {
        let (start, end) = calc_viewport(100, 200, 80, 20);
        assert_eq!(start, 80);
        assert_eq!(end, 160);
    }

    #[test]
    fn viewport_end_of_text() {
        let (start, end) = calc_viewport(199, 200, 80, 20);
        assert_eq!(start, 179);
        assert_eq!(end, 200);
    }

    #[test]
    fn viewport_caret_always_visible() {
        // Проверяем что для любой позиции курсор в окне
        for pos in 0..500 {
            let (start, end) = calc_viewport(pos, 500, 80, 20);
            assert!(pos >= start, "caret {} < start {} for text 500", pos, start);
            assert!(pos < end, "caret {} >= end {} for text 500", pos, end);
        }
    }

    #[test]
    fn viewport_ru_text() {
        // RU текст: символы могут быть multibyte, но позиции char-based
        let (start, end) = calc_viewport(50, 100, 80, 20);
        assert_eq!(start, 30);
        assert_eq!(end, 100);
    }

    #[test]
    fn viewport_empty_text() {
        let (start, end) = calc_viewport(0, 0, 80, 20);
        assert_eq!(start, 0);
        assert_eq!(end, 0);
    }

    #[test]
    fn viewport_exact_text_length() {
        let (start, end) = calc_viewport(80, 80, 80, 20);
        assert_eq!(start, 0);
        assert_eq!(end, 80);
    }

    #[test]
    fn viewport_padding_less_than_caret() {
        let (start, end) = calc_viewport(10, 200, 80, 20);
        assert_eq!(start, 0); // 10 - 20 = negative → 0
        assert_eq!(end, 80);
    }

    #[test]
    fn viewport_long_text_120s() {
        // 200 слов ~ 1200 символов
        let (start, end) = calc_viewport(600, 1200, 80, 20);
        assert_eq!(start, 580);
        assert_eq!(end, 660);
        assert!(600 >= start && 600 < end);
    }
}
