//! ReplayEngine — загрузка, воспроизведение, seek, speed.

use crate::analytics::SessionReplay;

/// Состояние воспроизведения.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplayState {
    Paused,
    Playing,
}

/// Скорость воспроизведения.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub enum ReplaySpeed {
    Half,
    Normal,
    Double,
    Quad,
}

impl ReplaySpeed {
    pub fn multiplier(&self) -> f64 {
        match self {
            ReplaySpeed::Half => 0.5,
            ReplaySpeed::Normal => 1.0,
            ReplaySpeed::Double => 2.0,
            ReplaySpeed::Quad => 4.0,
        }
    }

    pub fn from_f64(speed: f64) -> Self {
        if speed <= 0.5 {
            ReplaySpeed::Half
        } else if speed <= 1.0 {
            ReplaySpeed::Normal
        } else if speed <= 2.0 {
            ReplaySpeed::Double
        } else {
            ReplaySpeed::Quad
        }
    }
}

/// Результат загрузки replay.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LoadedReplay {
    pub replay: SessionReplay,
    pub total_frames: usize,
    pub total_duration_ms: u64,
}

/// ReplayEngine — управляет воспроизведением.
pub struct ReplayEngine {
    replay: SessionReplay,
    current_frame: usize,
    state: ReplayState,
    speed: ReplaySpeed,
}

impl ReplayEngine {
    pub fn new(replay: SessionReplay) -> Self {
        Self {
            replay,
            current_frame: 0,
            state: ReplayState::Paused,
            speed: ReplaySpeed::Normal,
        }
    }

    pub fn play(&mut self) {
        self.state = ReplayState::Playing;
    }

    pub fn pause(&mut self) {
        self.state = ReplayState::Paused;
    }

    pub fn seek(&mut self, frame: usize) {
        self.current_frame = frame.min(self.replay.events.len().saturating_sub(1));
    }

    pub fn set_speed(&mut self, speed: ReplaySpeed) {
        self.speed = speed;
    }

    pub fn current_frame(&self) -> usize {
        self.current_frame
    }

    pub fn current_event(&self) -> Option<&crate::analytics::ReplayEvent> {
        self.replay.events.get(self.current_frame)
    }

    pub fn state(&self) -> &ReplayState {
        &self.state
    }

    pub fn speed(&self) -> ReplaySpeed {
        self.speed
    }

    pub fn total_frames(&self) -> usize {
        self.replay.events.len()
    }

    pub fn total_duration_ms(&self) -> u64 {
        self.replay.total_duration_ms
    }

    pub fn advance_frame(&mut self) -> bool {
        if self.current_frame + 1 < self.replay.events.len() {
            self.current_frame += 1;
            true
        } else {
            self.state = ReplayState::Paused;
            false
        }
    }

    pub fn progress(&self) -> f64 {
        if self.replay.events.is_empty() {
            return 0.0;
        }
        (self.current_frame as f64 / self.replay.events.len() as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytics::{ReplayEvent, SessionReplay};

    fn make_replay() -> SessionReplay {
        let mut r = SessionReplay::new("hello".to_string());
        r.add_event(0, "h", "h", true);
        r.add_event(100, "e", "e", true);
        r.add_event(200, "l", "l", true);
        r.add_event(300, "l", "l", true);
        r.add_event(400, "o", "o", true);
        r
    }

    #[test]
    fn replay_engine_new() {
        let r = make_replay();
        let engine = ReplayEngine::new(r);
        assert_eq!(engine.total_frames(), 5);
        assert_eq!(engine.current_frame(), 0);
        assert!(engine.state() == &ReplayState::Paused);
    }

    #[test]
    fn replay_play_pause() {
        let r = make_replay();
        let mut engine = ReplayEngine::new(r);
        engine.play();
        assert!(engine.state() == &ReplayState::Playing);
        engine.pause();
        assert!(engine.state() == &ReplayState::Paused);
    }

    #[test]
    fn replay_seek() {
        let r = make_replay();
        let mut engine = ReplayEngine::new(r);
        engine.seek(3);
        assert_eq!(engine.current_frame(), 3);
    }

    #[test]
    fn replay_seek_beyond_end() {
        let r = make_replay();
        let mut engine = ReplayEngine::new(r);
        engine.seek(100);
        assert_eq!(engine.current_frame(), 4); // clamped to last
    }

    #[test]
    fn replay_speed() {
        let r = make_replay();
        let mut engine = ReplayEngine::new(r);
        engine.set_speed(ReplaySpeed::Double);
        assert_eq!(engine.speed(), ReplaySpeed::Double);
    }

    #[test]
    fn replay_speed_multiplier() {
        assert!((ReplaySpeed::Half.multiplier() - 0.5).abs() < 0.01);
        assert!((ReplaySpeed::Normal.multiplier() - 1.0).abs() < 0.01);
        assert!((ReplaySpeed::Double.multiplier() - 2.0).abs() < 0.01);
        assert!((ReplaySpeed::Quad.multiplier() - 4.0).abs() < 0.01);
    }

    #[test]
    fn replay_speed_from_f64() {
        assert_eq!(ReplaySpeed::from_f64(0.5), ReplaySpeed::Half);
        assert_eq!(ReplaySpeed::from_f64(1.0), ReplaySpeed::Normal);
        assert_eq!(ReplaySpeed::from_f64(2.0), ReplaySpeed::Double);
        assert_eq!(ReplaySpeed::from_f64(4.0), ReplaySpeed::Quad);
    }

    #[test]
    fn replay_current_event() {
        let r = make_replay();
        let engine = ReplayEngine::new(r);
        let event = engine.current_event().unwrap();
        assert_eq!(event.key, "h");
        assert!(event.correct);
    }

    #[test]
    fn replay_advance_frame() {
        let r = make_replay();
        let mut engine = ReplayEngine::new(r);
        assert!(engine.advance_frame());
        assert_eq!(engine.current_frame(), 1);
        let event = engine.current_event().unwrap();
        assert_eq!(event.key, "e");
    }

    #[test]
    fn replay_advance_past_end() {
        let r = make_replay();
        let mut engine = ReplayEngine::new(r);
        engine.seek(4);
        assert!(!engine.advance_frame());
        assert!(engine.state() == &ReplayState::Paused);
    }

    #[test]
    fn replay_progress() {
        let r = make_replay();
        let mut engine = ReplayEngine::new(r);
        assert!((engine.progress() - 0.0).abs() < 0.01);
        engine.seek(2);
        assert!((engine.progress() - 40.0).abs() < 0.01);
        engine.seek(4);
        assert!((engine.progress() - 80.0).abs() < 0.01);
    }

    #[test]
    fn replay_empty() {
        let r = SessionReplay::new("".to_string());
        let engine = ReplayEngine::new(r);
        assert_eq!(engine.total_frames(), 0);
        assert!(engine.current_event().is_none());
        assert!((engine.progress() - 0.0).abs() < 0.01);
    }

    #[test]
    fn replay_total_duration() {
        let r = make_replay();
        let engine = ReplayEngine::new(r);
        assert_eq!(engine.total_duration_ms(), 400);
    }

    #[test]
    fn replay_seek_to_zero() {
        let r = make_replay();
        let mut engine = ReplayEngine::new(r);
        engine.seek(3);
        engine.seek(0);
        assert_eq!(engine.current_frame(), 0);
    }

    #[test]
    fn replay_advance_all_frames() {
        let r = make_replay();
        let mut engine = ReplayEngine::new(r);
        for _ in 0..4 {
            assert!(engine.advance_frame());
        }
        assert!(!engine.advance_frame()); // 5th advance fails
    }
}
