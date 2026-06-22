//! Раскладки клавиатуры и finger assignments.

use serde::{Deserialize, Serialize};

/// Раскладка клавиатуры.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum KeyboardLayout {
    #[default]
    Qwerty,
    Jcuken,
    Dvorak,
}

/// Палец.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

/// Назначение пальцев для клавиш (QWERTY reference).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerAssignment {
    pub char: String,
    pub finger: Finger,
    pub hand: Hand,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Hand {
    Left,
    Right,
    Both,
}

/// Данные heatmap для одной клавиши.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyHeatData {
    pub total_attempts: usize,
    pub correct: usize,
    pub incorrect: usize,
    pub avg_wpm_at_key: f64,
}

/// Посимвольная статистика.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CharStat {
    pub correct: usize,
    pub incorrect: usize,
    pub total: usize,
}

pub type CharStatsMap = std::collections::HashMap<String, CharStat>;
pub type HeatmapMap = std::collections::HashMap<String, KeyHeatData>;
