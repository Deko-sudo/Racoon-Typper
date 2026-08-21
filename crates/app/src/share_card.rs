//! PNG share-карточка результата теста.
//!
//! Рендерит 900×470 карточку в стиле monkeytype: заголовок, stat-блоки
//! (WPM / Raw / Accuracy / Duration), мини-heatmap клавиатуры и акцентную
//! полосу. Цвета приходят из активной темы (фронтенд читает CSS-переменные
//! через getComputedStyle и передаёт hex-строки). Шрифты DejaVu встроены
//! через include_bytes!.

use ab_glyph::{Font, FontRef, Glyph, Point, PxScale, ScaleFont};
use image::{Rgb, RgbImage};
use racoon_domain::keyboard::KeyHeatData;
use std::collections::BTreeMap;

const FONT_REGULAR: &[u8] = include_bytes!("../../../resources/fonts/DejaVuSans.ttf");
const FONT_BOLD: &[u8] = include_bytes!("../../../resources/fonts/DejaVuSans-Bold.ttf");

pub const CARD_WIDTH: u32 = 900;
pub const CARD_HEIGHT: u32 = 470;

/// Цвета активной темы для карточки. hex-строки вида "#RRGGBB" (или
/// "#RRGGBBAA" — альфа игнорируется), как их отдаёт getComputedStyle.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ThemeColors {
    pub background: String,
    pub surface: String,
    pub text: String,
    pub sub: String,
    pub accent: String,
    pub error: String,
}

/// Статистика для карточки. Отдельно от FinalStats, чтобы контракт команды
/// был явным и стабильным.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ShareStats {
    pub wpm: f64,
    pub raw_wpm: f64,
    pub accuracy: f64,
    pub duration_ms: u64,
    pub mode: String,
    pub language: String,
    pub date: String,
    /// Per-key heatmap (ключ → данные). Может быть пустой.
    pub heatmap: BTreeMap<String, KeyHeatData>,
}

fn parse_hex(hex: &str, fallback: [u8; 3]) -> [u8; 3] {
    let h = hex.trim().trim_start_matches('#');
    let chars: Vec<char> = h.chars().collect();
    if chars.len() >= 6 {
        let r = u8::from_str_radix(&chars[0..2].iter().collect::<String>(), 16);
        let g = u8::from_str_radix(&chars[2..4].iter().collect::<String>(), 16);
        let b = u8::from_str_radix(&chars[4..6].iter().collect::<String>(), 16);
        if let (Ok(r), Ok(g), Ok(b)) = (r, g, b) {
            return [r, g, b];
        }
    }
    fallback
}

fn mix(a: [u8; 3], b: [u8; 3], t: f32) -> Rgb<u8> {
    Rgb([
        (a[0] as f32 * (1.0 - t) + b[0] as f32 * t) as u8,
        (a[1] as f32 * (1.0 - t) + b[1] as f32 * t) as u8,
        (a[2] as f32 * (1.0 - t) + b[2] as f32 * t) as u8,
    ])
}

/// Рисует строку текста шрифтом `font_data`, цветом `color`, начиная с
/// (x, baseline_y). Возвращает ширину отрисованного текста в пикселях.
#[allow(clippy::too_many_arguments)]
fn draw_text(
    img: &mut RgbImage,
    font_data: &[u8],
    text: &str,
    x: f32,
    baseline_y: f32,
    size_px: f32,
    color: [u8; 3],
) -> f32 {
    let font = match FontRef::try_from_slice(font_data) {
        Ok(f) => f,
        Err(_) => return 0.0,
    };
    let scale = PxScale {
        x: size_px,
        y: size_px,
    };
    let scaled = font.as_scaled(scale);
    let mut pen_x = x;

    for ch in text.chars() {
        let glyph_id = font.glyph_id(ch);
        let glyph = Glyph {
            id: glyph_id,
            scale,
            position: Point {
                x: pen_x,
                y: baseline_y,
            },
        };
        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            outlined.draw(|gx, gy, coverage| {
                let px = bounds.min.x as i64 + gx as i64;
                let py = bounds.min.y as i64 + gy as i64;
                if px >= 0 && py >= 0 && (px as u32) < img.width() && (py as u32) < img.height() {
                    let cur = *img.get_pixel(px as u32, py as u32);
                    let blended = mix([cur[0], cur[1], cur[2]], color, coverage.min(1.0));
                    img.put_pixel(px as u32, py as u32, blended);
                }
            });
        }
        pen_x += scaled.h_advance(glyph_id);
    }
    pen_x - x
}

fn text_width(font_data: &[u8], text: &str, size_px: f32) -> f32 {
    let font = match FontRef::try_from_slice(font_data) {
        Ok(f) => f,
        Err(_) => return 0.0,
    };
    let scale = PxScale {
        x: size_px,
        y: size_px,
    };
    let scaled = font.as_scaled(scale);
    text.chars()
        .map(|ch| scaled.h_advance(font.glyph_id(ch)))
        .sum()
}

/// Рисует stat-блок: значение (крупно, акцент) + подпись (мелко, sub).
fn draw_stat_block(
    img: &mut RgbImage,
    fonts: &ShareFonts,
    value: &str,
    label: &str,
    x: f32,
    y_baseline: f32,
    colors: &ParsedColors,
) {
    let vw = draw_text(img, fonts.bold, value, x, y_baseline, 56.0, colors.accent);
    let _ = vw;
    draw_text(
        img,
        fonts.regular,
        label,
        x,
        y_baseline + 28.0,
        20.0,
        colors.sub,
    );
}

struct ShareFonts {
    regular: &'static [u8],
    bold: &'static [u8],
}

struct ParsedColors {
    background: [u8; 3],
    surface: [u8; 3],
    text: [u8; 3],
    sub: [u8; 3],
    accent: [u8; 3],
    error: [u8; 3],
}

/// Рендерит карточку и возвращает PNG-байты.
pub fn render_share_card(stats: &ShareStats, colors: &ThemeColors) -> Result<Vec<u8>, String> {
    let fonts = ShareFonts {
        regular: FONT_REGULAR,
        bold: FONT_BOLD,
    };
    let pc = ParsedColors {
        background: parse_hex(&colors.background, [13, 15, 18]),
        surface: parse_hex(&colors.surface, [21, 24, 29]),
        text: parse_hex(&colors.text, [231, 233, 237]),
        sub: parse_hex(&colors.sub, [140, 148, 160]),
        accent: parse_hex(&colors.accent, [197, 203, 212]),
        error: parse_hex(&colors.error, [220, 141, 141]),
    };

    let mut img = RgbImage::from_pixel(CARD_WIDTH, CARD_HEIGHT, Rgb(pc.background));

    // Акцентная полоса сверху.
    for y in 0..6u32 {
        for x in 0..CARD_WIDTH {
            img.put_pixel(x, y, Rgb(pc.accent));
        }
    }

    // Заголовок.
    draw_text(
        &mut img,
        fonts.bold,
        "RACOON TYPPER",
        48.0,
        76.0,
        34.0,
        pc.accent,
    );
    let subtitle = format!("{} · {} · {}", stats.mode, stats.language, stats.date);
    draw_text(
        &mut img,
        fonts.regular,
        &subtitle,
        48.0,
        108.0,
        18.0,
        pc.sub,
    );

    // Разделитель.
    let divider = mix(pc.background, pc.sub, 0.25);
    for x in 48..(CARD_WIDTH - 48) {
        img.put_pixel(x, 140, divider);
    }

    // Stat-блоки: WPM / Raw WPM / Accuracy / Duration.
    let baseline = 260.0;
    let cols = [
        (48.0f32, format!("{:.1}", stats.wpm), "WPM"),
        (264.0f32, format!("{:.1}", stats.raw_wpm), "RAW WPM"),
        (480.0f32, format!("{:.1}%", stats.accuracy), "ACCURACY"),
        (
            696.0f32,
            format!("{:.1}s", stats.duration_ms as f64 / 1000.0),
            "TIME",
        ),
    ];
    for (x, value, label) in cols {
        draw_stat_block(&mut img, &fonts, &value, label, x, baseline, &pc);
    }

    // Разделитель перед heatmap.
    for x in 48..(CARD_WIDTH - 48) {
        img.put_pixel(x, 330, divider);
    }
    draw_text(
        &mut img,
        fonts.regular,
        "KEYBOARD HEATMAP",
        48.0,
        366.0,
        16.0,
        pc.sub,
    );

    // Мини-heatmap: буквы EN-раскладки, 3 ряда, клетка 24px.
    const ROWS: &[&str] = &["qwertyuiop", "asdfghjkl", "zxcvbnm"];
    const CELL: u32 = 24;
    const GAP: u32 = 4;
    const HEAT_TOP: u32 = 384;
    for (row_idx, row) in ROWS.iter().enumerate() {
        // Ступенчатость рядов как на ANSI-клавиатуре (0 / 14 / 28 px).
        let row_offset = (row_idx as u32) * 14;
        for (col_idx, ch) in row.chars().enumerate() {
            let data = stats.heatmap.get(&ch.to_string());
            let (fill, label_color) = match data {
                Some(d) if d.total_attempts > 0 => {
                    let acc = d.correct as f32 / d.total_attempts as f32;
                    // accuracy 1.0 → accent, 0.0 → error.
                    (mix(pc.accent, pc.error, 1.0 - acc), pc.text)
                }
                _ => (mix(pc.surface, pc.sub, 0.15), pc.sub),
            };
            let x0 = 48 + row_offset + (col_idx as u32) * (CELL + GAP);
            let y0 = HEAT_TOP + (row_idx as u32) * (CELL + GAP);
            let rounded = 5u32;
            for dy in 0..CELL {
                for dx in 0..CELL {
                    if x0 + dx >= CARD_WIDTH || y0 + dy >= CARD_HEIGHT {
                        continue;
                    }
                    // Скругление углов: пропускаем угловые пиксели.
                    let corner = (dx < rounded || dx >= CELL - rounded)
                        && (dy < rounded || dy >= CELL - rounded);
                    let in_corner_circle = |cx: u32, cy: u32| {
                        let ddx = (dx as i64 - cx as i64).abs();
                        let ddy = (dy as i64 - cy as i64).abs();
                        ddx * ddx + ddy * ddy <= (rounded as i64) * (rounded as i64)
                    };
                    let skip = corner
                        && !in_corner_circle(
                            if dx < rounded {
                                rounded - 1
                            } else {
                                CELL - rounded
                            },
                            if dy < rounded {
                                rounded - 1
                            } else {
                                CELL - rounded
                            },
                        );
                    if skip {
                        continue;
                    }
                    img.put_pixel(x0 + dx, y0 + dy, fill);
                }
            }
            // Символ клавиши по центру клетки.
            let ch_str = ch.to_string();
            let tw = text_width(fonts.regular, &ch_str, 16.0);
            let cap_height = 12.0;
            draw_text(
                &mut img,
                fonts.regular,
                &ch_str,
                x0 as f32 + (CELL as f32 - tw) / 2.0,
                y0 as f32 + (CELL as f32 + cap_height) / 2.0,
                16.0,
                label_color,
            );
        }
    }

    let mut bytes = Vec::new();
    image::DynamicImage::ImageRgb8(img)
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .map_err(|e| format!("PNG encode failed: {e}"))?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_colors() -> ThemeColors {
        ThemeColors {
            background: "#0d0f12".to_string(),
            surface: "#15181d".to_string(),
            text: "#e7e9ed".to_string(),
            sub: "#8c94a0".to_string(),
            accent: "#c5cbd4".to_string(),
            error: "#dc8d8d".to_string(),
        }
    }

    fn sample_stats() -> ShareStats {
        let mut heatmap = BTreeMap::new();
        heatmap.insert(
            "a".to_string(),
            KeyHeatData {
                total_attempts: 12,
                correct: 11,
                incorrect: 1,
                avg_wpm_at_key: 0.0,
            },
        );
        heatmap.insert(
            "z".to_string(),
            KeyHeatData {
                total_attempts: 8,
                correct: 2,
                incorrect: 6,
                avg_wpm_at_key: 0.0,
            },
        );
        ShareStats {
            wpm: 87.4,
            raw_wpm: 94.2,
            accuracy: 96.8,
            duration_ms: 30_000,
            mode: "time 30".to_string(),
            language: "en".to_string(),
            date: "2026-08-20".to_string(),
            heatmap,
        }
    }

    #[test]
    fn share_card_renders_valid_png() {
        let bytes = render_share_card(&sample_stats(), &sample_colors()).unwrap();
        assert_eq!(
            &bytes[..8],
            &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A],
            "PNG magic bytes"
        );
        assert!(
            bytes.len() > 10_000,
            "card should be substantial, got {} bytes",
            bytes.len()
        );
    }

    #[test]
    fn share_card_bad_colors_fall_back() {
        let mut colors = sample_colors();
        colors.background = "not-a-hex".to_string();
        let bytes = render_share_card(&sample_stats(), &colors).unwrap();
        assert_eq!(
            &bytes[..8],
            &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]
        );
    }

    #[test]
    fn share_card_empty_heatmap_still_renders() {
        let mut stats = sample_stats();
        stats.heatmap.clear();
        let bytes = render_share_card(&stats, &sample_colors()).unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn parse_hex_variants() {
        assert_eq!(parse_hex("#ff0000", [0, 0, 0]), [255, 0, 0]);
        assert_eq!(parse_hex("00ff00", [0, 0, 0]), [0, 255, 0]);
        assert_eq!(parse_hex("#00ff0080", [9, 9, 9]), [0, 255, 0]);
        assert_eq!(parse_hex("zzz", [9, 9, 9]), [9, 9, 9]);
    }
}
