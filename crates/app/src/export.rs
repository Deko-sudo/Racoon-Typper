//! Export serializers for analytics and profile data.
//!
//! These are pure, dependency-light serializers that turn in-memory statistics
//! into user-facing documents (Markdown report, PNG heatmap card). They never
//! touch the database or the filesystem; the Tauri command layer owns I/O.

use image::{Rgb, RgbImage};
use racoon_domain::keyboard::KeyHeatData;
use std::collections::BTreeMap;

/// A single row of the Markdown history table.
pub struct MarkdownTestRow {
    pub date: String,
    pub mode: String,
    pub language: String,
    pub wpm: f64,
    pub accuracy: f64,
    pub duration_ms: u64,
}

/// Build a Markdown report from dashboard aggregates and a history slice.
///
/// The report is intentionally plain and self-contained: it renders as a
/// readable document in any Markdown viewer and contains no raw typed text.
pub fn build_markdown_report(
    title: &str,
    generated_at: &str,
    summary: &[(&str, String)],
    rows: &[MarkdownTestRow],
) -> String {
    let mut out = String::new();
    out.push_str(&format!("# {title}\n\n"));
    out.push_str(&format!("_Generated {generated_at}_\n\n"));

    if !summary.is_empty() {
        out.push_str("## Summary\n\n");
        out.push_str("| Metric | Value |\n|---|---|\n");
        for (label, value) in summary {
            out.push_str(&format!("| {label} | {value} |\n"));
        }
        out.push('\n');
    }

    if !rows.is_empty() {
        out.push_str("## Test history\n\n");
        out.push_str("| Date | Mode | Language | WPM | Accuracy | Duration (s) |\n");
        out.push_str("|---|---|---|---|---|---|\n");
        for row in rows {
            out.push_str(&format!(
                "| {} | {} | {} | {:.1} | {:.1}% | {:.2} |\n",
                row.date,
                row.mode,
                row.language,
                row.wpm,
                row.accuracy,
                row.duration_ms as f64 / 1000.0,
            ));
        }
        out.push('\n');
    }

    out
}

/// Render a heatmap as a PNG card.
///
/// Each key is drawn as a colored square whose intensity reflects its error
/// rate (red = weak, green = strong). The image is a fixed 640x360 canvas so
/// it can be embedded in a report or shared as a standalone card.
pub fn render_heatmap_png(heatmap: &BTreeMap<String, KeyHeatData>) -> Vec<u8> {
    const WIDTH: u32 = 640;
    const HEIGHT: u32 = 360;
    const CELL: u32 = 40;
    const PAD: u32 = 20;

    let mut img = RgbImage::new(WIDTH, HEIGHT);
    // Background.
    for pixel in img.pixels_mut() {
        *pixel = Rgb([24, 24, 28]);
    }

    let mut col = 0u32;
    let mut row = 0u32;
    for (key, data) in heatmap {
        let error_rate = if data.total_attempts == 0 {
            0.0
        } else {
            data.incorrect as f64 / data.total_attempts as f64
        };
        // Green (strong) -> red (weak).
        let (r, g) = if error_rate < 0.5 { (0, 255) } else { (255, 0) };
        let intensity = (error_rate * 255.0) as u8;
        let color = Rgb([r, g, intensity]);

        let x0 = PAD + col * (CELL + 4);
        let y0 = PAD + row * (CELL + 4);
        for dy in 0..CELL {
            for dx in 0..CELL {
                let x = x0 + dx;
                let y = y0 + dy;
                if x < WIDTH && y < HEIGHT {
                    img.put_pixel(x, y, color);
                }
            }
        }
        let _ = key; // key label is not drawn (no font dependency)

        col += 1;
        if col >= 12 {
            col = 0;
            row += 1;
        }
    }

    let mut bytes = Vec::new();
    let _ = image::DynamicImage::ImageRgb8(img).write_to(
        &mut std::io::Cursor::new(&mut bytes),
        image::ImageFormat::Png,
    );
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_report_contains_summary_and_history() {
        let report = build_markdown_report(
            "Test report",
            "2026-08-09T00:00:00Z",
            &[
                ("Total tests", "42".to_string()),
                ("Best WPM", "120.0".to_string()),
            ],
            &[MarkdownTestRow {
                date: "2026-08-08".to_string(),
                mode: "time".to_string(),
                language: "en".to_string(),
                wpm: 88.5,
                accuracy: 97.2,
                duration_ms: 30_000,
            }],
        );
        assert!(report.contains("# Test report"));
        assert!(report.contains("| Total tests | 42 |"));
        assert!(report.contains("| 2026-08-08 | time | en | 88.5 | 97.2% | 30.00 |"));
    }

    #[test]
    fn markdown_report_empty_rows_omits_table() {
        let report = build_markdown_report("Empty", "now", &[], &[]);
        assert!(report.contains("# Empty"));
        assert!(!report.contains("Test history"));
    }

    #[test]
    fn heatmap_png_renders_valid_png_bytes() {
        let mut heatmap = BTreeMap::new();
        heatmap.insert(
            "a".to_string(),
            KeyHeatData {
                total_attempts: 10,
                correct: 9,
                incorrect: 1,
                avg_wpm_at_key: 60.0,
            },
        );
        heatmap.insert(
            "z".to_string(),
            KeyHeatData {
                total_attempts: 10,
                correct: 2,
                incorrect: 8,
                avg_wpm_at_key: 30.0,
            },
        );
        let bytes = render_heatmap_png(&heatmap);
        // PNG signature.
        assert_eq!(
            &bytes[..8],
            &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]
        );
        assert!(!bytes.is_empty());
    }
}
