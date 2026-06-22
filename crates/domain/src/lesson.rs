//! Уроки, курсы, модули, упражнения.

use serde::{Deserialize, Serialize};

/// Полный курс слепой печати.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Course {
    pub name: String,
    pub language: String,
    pub layout: String,
    pub version: String,
    pub modules: Vec<Module>,
}

/// Модуль курса (группа уроков).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub id: String,
    pub title: String,
    pub target_keys: Vec<String>,
    pub lessons: Vec<Lesson>,
}

/// Урок.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lesson {
    pub id: String,
    pub title: String,
    pub target_keys: Vec<String>,
    pub prerequisite_keys: Vec<String>,
    pub passing_wpm: u32,
    pub passing_accuracy: f64,
    pub max_errors_per_exercise: u32,
    pub difficulty: Difficulty,
    pub exercises: Vec<Exercise>,
}

/// Упражнение.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exercise {
    #[serde(rename = "type")]
    pub exercise_type: String,
    pub text: String,
    pub min_wpm: u32,
    pub min_accuracy: f64,
}

/// Уровень сложности.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    #[default]
    Beginner,
    Intermediate,
    Advanced,
}

/// Статус прохождения урока.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum LessonStatus {
    #[default]
    NotStarted,
    InProgress,
    Completed,
    Mastered,
}
