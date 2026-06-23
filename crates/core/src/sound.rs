//! Sound Engine — воспроизведение звуков при событиях печати.
//! MVP: генерация простых тонов через частоты. Без внешних файлов.

use std::collections::HashMap;

/// Тип звукового события.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SoundEvent {
    KeyPress,
    Error,
    LessonComplete,
    AchievementUnlocked,
}

impl SoundEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            SoundEvent::KeyPress => "key_press",
            SoundEvent::Error => "error",
            SoundEvent::LessonComplete => "lesson_complete",
            SoundEvent::AchievementUnlocked => "achievement_unlocked",
        }
    }

    /// Частота тона (Hz) для каждого события.
    pub fn frequency(&self) -> f64 {
        match self {
            SoundEvent::KeyPress => 800.0,
            SoundEvent::Error => 200.0,
            SoundEvent::LessonComplete => 880.0,
            SoundEvent::AchievementUnlocked => 1200.0,
        }
    }

    /// Длительность в миллисекундах.
    pub fn duration_ms(&self) -> u64 {
        match self {
            SoundEvent::KeyPress => 20,
            SoundEvent::Error => 80,
            SoundEvent::LessonComplete => 300,
            SoundEvent::AchievementUnlocked => 500,
        }
    }
}

/// Конфигурация звука.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SoundConfig {
    pub enabled: bool,
    pub volume: f64,
}

impl Default for SoundConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            volume: 0.5,
        }
    }
}

impl SoundConfig {
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn effective_volume(&self) -> f64 {
        self.volume.clamp(0.0, 1.0)
    }
}

/// Sound Engine — управляет воспроизведением звуков.
pub struct SoundEngine {
    config: SoundConfig,
    last_played: HashMap<SoundEvent, u64>,
    cooldown_ms: u64,
}

impl SoundEngine {
    pub fn new(config: SoundConfig) -> Self {
        Self {
            config,
            last_played: HashMap::new(),
            cooldown_ms: 50,
        }
    }

    pub fn set_config(&mut self, config: SoundConfig) {
        self.config = config;
    }

    pub fn set_volume(&mut self, volume: f64) {
        self.config.volume = volume.clamp(0.0, 1.0);
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
    }

    /// Проверяет, нужно ли играть звук (cooldown).
    pub fn should_play(&self, event: SoundEvent, now_ms: u64) -> bool {
        if !self.config.enabled {
            return false;
        }
        match self.last_played.get(&event) {
            Some(last) => now_ms.saturating_sub(*last) >= self.cooldown_ms,
            None => true,
        }
    }

    /// Возвращает параметры звука если должен играть.
    pub fn try_play(&mut self, event: SoundEvent, now_ms: u64) -> Option<SoundOutput> {
        if !self.should_play(event, now_ms) {
            return None;
        }
        self.last_played.insert(event, now_ms);
        Some(SoundOutput {
            frequency: event.frequency(),
            duration_ms: event.duration_ms(),
            volume: self.config.effective_volume(),
        })
    }

    /// Сброс cooldown (для тестов).
    pub fn reset_cooldown(&mut self) {
        self.last_played.clear();
    }
}

/// Результат — параметры для воспроизведения.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct SoundOutput {
    pub frequency: f64,
    pub duration_ms: u64,
    pub volume: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sound_event_as_str() {
        assert_eq!(SoundEvent::KeyPress.as_str(), "key_press");
        assert_eq!(SoundEvent::Error.as_str(), "error");
        assert_eq!(SoundEvent::LessonComplete.as_str(), "lesson_complete");
        assert_eq!(
            SoundEvent::AchievementUnlocked.as_str(),
            "achievement_unlocked"
        );
    }

    #[test]
    fn sound_event_frequencies() {
        assert!((SoundEvent::KeyPress.frequency() - 800.0).abs() < 0.01);
        assert!((SoundEvent::Error.frequency() - 200.0).abs() < 0.01);
        assert!((SoundEvent::LessonComplete.frequency() - 880.0).abs() < 0.01);
        assert!((SoundEvent::AchievementUnlocked.frequency() - 1200.0).abs() < 0.01);
    }

    #[test]
    fn sound_event_durations() {
        assert_eq!(SoundEvent::KeyPress.duration_ms(), 20);
        assert_eq!(SoundEvent::Error.duration_ms(), 80);
        assert_eq!(SoundEvent::LessonComplete.duration_ms(), 300);
        assert_eq!(SoundEvent::AchievementUnlocked.duration_ms(), 500);
    }

    #[test]
    fn sound_config_default() {
        let c = SoundConfig::default();
        assert!(!c.enabled);
        assert!((c.volume - 0.5).abs() < 0.01);
    }

    #[test]
    fn sound_config_volume_clamped() {
        let mut c = SoundConfig::default();
        c.volume = 1.5;
        assert!((c.effective_volume() - 1.0).abs() < 0.01);
        c.volume = -0.5;
        assert!((c.effective_volume() - 0.0).abs() < 0.01);
    }

    #[test]
    fn sound_engine_disabled_no_play() {
        let mut engine = SoundEngine::new(SoundConfig {
            enabled: false,
            volume: 0.5,
        });
        assert!(engine.try_play(SoundEvent::KeyPress, 0).is_none());
    }

    #[test]
    fn sound_engine_enabled_plays() {
        let mut engine = SoundEngine::new(SoundConfig {
            enabled: true,
            volume: 0.5,
        });
        let out = engine.try_play(SoundEvent::KeyPress, 0);
        assert!(out.is_some());
        let out = out.unwrap();
        assert!((out.frequency - 800.0).abs() < 0.01);
        assert!((out.volume - 0.5).abs() < 0.01);
    }

    #[test]
    fn sound_engine_cooldown() {
        let mut engine = SoundEngine::new(SoundConfig {
            enabled: true,
            volume: 0.5,
        });
        assert!(engine.try_play(SoundEvent::KeyPress, 0).is_some());
        assert!(engine.try_play(SoundEvent::KeyPress, 10).is_none()); // within cooldown
        assert!(engine.try_play(SoundEvent::KeyPress, 60).is_some()); // after cooldown
    }

    #[test]
    fn sound_engine_different_events_independent() {
        let mut engine = SoundEngine::new(SoundConfig {
            enabled: true,
            volume: 0.5,
        });
        assert!(engine.try_play(SoundEvent::KeyPress, 0).is_some());
        assert!(engine.try_play(SoundEvent::Error, 0).is_some()); // different event, no cooldown
    }

    #[test]
    fn sound_engine_set_volume() {
        let mut engine = SoundEngine::new(SoundConfig {
            enabled: true,
            volume: 0.5,
        });
        engine.set_volume(0.8);
        let out = engine.try_play(SoundEvent::KeyPress, 0).unwrap();
        assert!((out.volume - 0.8).abs() < 0.01);
    }

    #[test]
    fn sound_engine_set_enabled() {
        let mut engine = SoundEngine::new(SoundConfig {
            enabled: true,
            volume: 0.5,
        });
        engine.set_enabled(false);
        assert!(engine.try_play(SoundEvent::KeyPress, 0).is_none());
    }

    #[test]
    fn sound_engine_reset_cooldown() {
        let mut engine = SoundEngine::new(SoundConfig {
            enabled: true,
            volume: 0.5,
        });
        engine.try_play(SoundEvent::KeyPress, 0);
        assert!(engine.try_play(SoundEvent::KeyPress, 10).is_none());
        engine.reset_cooldown();
        assert!(engine.try_play(SoundEvent::KeyPress, 10).is_some());
    }

    #[test]
    fn sound_output_serializable() {
        let out = SoundOutput {
            frequency: 800.0,
            duration_ms: 20,
            volume: 0.5,
        };
        let json = serde_json::to_string(&out).unwrap();
        assert!(json.contains("800"));
        assert!(json.contains("20"));
    }
}
