use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// MusicalKey
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MusicalKey {
    CMajor,
    DMinor,
    EMinor,
    FMajor,
    GMajor,
    AMinor,
    BFlatMajor,
    Chromatic,
}

// ---------------------------------------------------------------------------
// Tempo
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tempo {
    pub bpm: f64,
    pub is_changing: bool,
    pub target_bpm: f64,
}

impl Tempo {
    pub fn new(bpm: f64) -> Self {
        Self {
            bpm,
            is_changing: false,
            target_bpm: bpm,
        }
    }

    pub fn accelerate(&mut self, target: f64) {
        self.target_bpm = target.max(self.bpm);
        self.is_changing = true;
    }

    pub fn decelerate(&mut self, target: f64) {
        self.target_bpm = target.min(self.bpm);
        self.is_changing = true;
    }

    /// Smoothly approaches `target_bpm` by 1 % per tick.
    pub fn tick(&mut self) {
        if !self.is_changing {
            return;
        }
        let diff = self.target_bpm - self.bpm;
        if diff.abs() < 0.1 {
            self.bpm = self.target_bpm;
            self.is_changing = false;
        } else {
            self.bpm += diff * 0.01;
        }
    }

    pub fn is_fast(&self) -> bool {
        self.bpm > 120.0
    }

    pub fn is_slow(&self) -> bool {
        self.bpm < 80.0
    }
}

// ---------------------------------------------------------------------------
// Layer / MusicLayer
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Layer {
    Melody,
    Harmony,
    Bass,
    Percussion,
    Ambient,
    Effects,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicLayer {
    pub layer_type: Layer,
    pub volume: f64,
    pub active: bool,
    pub pattern_id: Option<u32>,
}

impl MusicLayer {
    pub fn new(layer_type: Layer) -> Self {
        Self {
            layer_type,
            volume: 0.0,
            active: false,
            pattern_id: None,
        }
    }

    pub fn fade_in(&mut self) {
        self.volume = (self.volume + 0.1).min(1.0);
        if self.volume > 0.0 {
            self.active = true;
        }
    }

    pub fn fade_out(&mut self) {
        self.volume = (self.volume - 0.1).max(0.0);
        if self.volume <= 0.0 {
            self.active = false;
        }
    }
}

// ---------------------------------------------------------------------------
// SoundtrackState
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundtrackState {
    pub key: MusicalKey,
    pub tempo: Tempo,
    pub layers: Vec<MusicLayer>,
    pub mood_tag: String,
}

impl SoundtrackState {
    pub fn new() -> Self {
        Self {
            key: MusicalKey::CMajor,
            tempo: Tempo::new(100.0),
            layers: vec![
                MusicLayer::new(Layer::Melody),
                MusicLayer::new(Layer::Harmony),
                MusicLayer::new(Layer::Bass),
                MusicLayer::new(Layer::Percussion),
                MusicLayer::new(Layer::Ambient),
                MusicLayer::new(Layer::Effects),
            ],
            mood_tag: String::from("calm"),
        }
    }

    /// Set the mood, adjusting key / tempo / layers accordingly.
    pub fn set_mood(&mut self, mood: &str) {
        self.mood_tag = mood.to_string();
        // First deactivate + silence everything.
        for layer in &mut self.layers {
            layer.active = false;
            layer.volume = 0.0;
        }

        let activate = |layers: &mut [MusicLayer], types: &[Layer], vol: f64| {
            for l in layers.iter_mut() {
                if types.contains(&l.layer_type) {
                    l.active = true;
                    l.volume = vol;
                }
            }
        };

        match mood {
            "calm" => {
                self.key = MusicalKey::FMajor;
                self.tempo = Tempo::new(70.0);
                activate(&mut self.layers, &[Layer::Melody, Layer::Ambient], 0.6);
            }
            "exciting" => {
                self.key = MusicalKey::GMajor;
                self.tempo = Tempo::new(130.0);
                activate(&mut self.layers, &[Layer::Melody, Layer::Harmony, Layer::Bass, Layer::Percussion, Layer::Ambient, Layer::Effects], 0.8);
            }
            "mysterious" => {
                self.key = MusicalKey::AMinor;
                self.tempo = Tempo::new(90.0);
                activate(&mut self.layers, &[Layer::Ambient, Layer::Effects], 0.5);
            }
            "triumphant" => {
                self.key = MusicalKey::CMajor;
                self.tempo = Tempo::new(120.0);
                activate(&mut self.layers, &[Layer::Melody, Layer::Harmony, Layer::Bass, Layer::Percussion, Layer::Ambient, Layer::Effects], 1.0);
            }
            "sad" => {
                self.key = MusicalKey::DMinor;
                self.tempo = Tempo::new(60.0);
                activate(&mut self.layers, &[Layer::Melody], 0.4);
            }
            "scary" => {
                self.key = MusicalKey::EMinor;
                self.tempo = Tempo::new(100.0);
                activate(&mut self.layers, &[Layer::Bass, Layer::Effects], 0.7);
            }
            _ => {
                self.key = MusicalKey::Chromatic;
                self.tempo = Tempo::new(100.0);
            }
        }
    }
}

impl Default for SoundtrackState {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// SoundtrackEngine
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundtrackEngine {
    pub room_soundtracks: HashMap<String, SoundtrackState>,
}

impl SoundtrackEngine {
    pub fn new() -> Self {
        Self {
            room_soundtracks: HashMap::new(),
        }
    }

    pub fn ensure_room(&mut self, room: &str) {
        self.room_soundtracks
            .entry(room.to_string())
            .or_default();
    }

    /// Vibe (0..1) drives tempo: `vibe * 40 + 80 = bpm`.
    pub fn update_from_vibe(&mut self, room: &str, vibe: f64) {
        self.ensure_room(room);
        if let Some(state) = self.room_soundtracks.get_mut(room) {
            let target = vibe * 40.0 + 80.0;
            if target > state.tempo.bpm {
                state.tempo.accelerate(target);
            } else {
                state.tempo.decelerate(target);
            }
        }
    }

    /// Named emotion drives key + layers via `set_mood`.
    pub fn update_from_emotion(&mut self, room: &str, emotion: &str) {
        self.ensure_room(room);
        if let Some(state) = self.room_soundtracks.get_mut(room) {
            state.set_mood(emotion);
        }
    }

    pub fn get_audio_params(&self, room: &str) -> Option<&SoundtrackState> {
        self.room_soundtracks.get(room)
    }
}

impl Default for SoundtrackEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- MusicalKey --
    #[test]
    fn musical_key_variants_serialize() {
        let keys = vec![
            MusicalKey::CMajor,
            MusicalKey::DMinor,
            MusicalKey::EMinor,
            MusicalKey::FMajor,
            MusicalKey::GMajor,
            MusicalKey::AMinor,
            MusicalKey::BFlatMajor,
            MusicalKey::Chromatic,
        ];
        let json = serde_json::to_string(&keys).unwrap();
        let de: Vec<MusicalKey> = serde_json::from_str(&json).unwrap();
        assert_eq!(keys, de);
    }

    // -- Tempo basics --
    #[test]
    fn tempo_new_sets_fields() {
        let t = Tempo::new(100.0);
        assert_eq!(t.bpm, 100.0);
        assert!(!t.is_changing);
        assert_eq!(t.target_bpm, 100.0);
    }

    #[test]
    fn tempo_is_fast_and_slow() {
        let fast = Tempo::new(150.0);
        assert!(fast.is_fast());
        assert!(!fast.is_slow());

        let slow = Tempo::new(50.0);
        assert!(slow.is_slow());
        assert!(!fast.is_slow());
    }

    #[test]
    fn tempo_accelerate_sets_target() {
        let mut t = Tempo::new(100.0);
        t.accelerate(140.0);
        assert!(t.is_changing);
        assert_eq!(t.target_bpm, 140.0);
    }

    #[test]
    fn tempo_accelerate_lower_is_noop() {
        let mut t = Tempo::new(100.0);
        t.accelerate(80.0);
        // target should stay at 100 (max)
        assert_eq!(t.target_bpm, 100.0);
    }

    #[test]
    fn tempo_decelerate_sets_target() {
        let mut t = Tempo::new(120.0);
        t.decelerate(60.0);
        assert!(t.is_changing);
        assert_eq!(t.target_bpm, 60.0);
    }

    #[test]
    fn tempo_decelerate_higher_is_noop() {
        let mut t = Tempo::new(80.0);
        t.decelerate(120.0);
        assert_eq!(t.target_bpm, 80.0);
    }

    #[test]
    fn tempo_tick_converges() {
        let mut t = Tempo::new(100.0);
        t.accelerate(200.0);
        // After many ticks we should get close.
        for _ in 0..2000 {
            t.tick();
        }
        assert!(!t.is_changing);
        assert!((t.bpm - 200.0).abs() < 0.5);
    }

    #[test]
    fn tempo_tick_noop_when_stable() {
        let mut t = Tempo::new(100.0);
        t.tick();
        assert!(!t.is_changing);
        assert_eq!(t.bpm, 100.0);
    }

    // -- MusicLayer --
    #[test]
    fn music_layer_fade_in() {
        let mut l = MusicLayer::new(Layer::Melody);
        assert!(!l.active);
        for _ in 0..5 {
            l.fade_in();
        }
        assert!(l.active);
        assert!((l.volume - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn music_layer_fade_in_clamps_at_one() {
        let mut l = MusicLayer::new(Layer::Melody);
        for _ in 0..20 {
            l.fade_in();
        }
        assert!(l.volume <= 1.0);
    }

    #[test]
    fn music_layer_fade_out() {
        let mut l = MusicLayer::new(Layer::Melody);
        l.volume = 0.3;
        l.active = true;
        l.fade_out();
        assert!((l.volume - 0.2).abs() < f64::EPSILON);
        l.fade_out();
        l.fade_out();
        assert!(!l.active);
        assert_eq!(l.volume, 0.0);
    }

    #[test]
    fn music_layer_fade_out_clamps_at_zero() {
        let mut l = MusicLayer::new(Layer::Bass);
        l.fade_out();
        assert_eq!(l.volume, 0.0);
        assert!(!l.active);
    }

    #[test]
    fn music_layer_new_fields() {
        let l = MusicLayer::new(Layer::Percussion);
        assert_eq!(l.layer_type, Layer::Percussion);
        assert_eq!(l.volume, 0.0);
        assert!(!l.active);
        assert!(l.pattern_id.is_none());
    }

    // -- SoundtrackState moods --
    #[test]
    fn mood_calm() {
        let mut s = SoundtrackState::new();
        s.set_mood("calm");
        assert_eq!(s.key, MusicalKey::FMajor);
        assert!((s.tempo.bpm - 70.0).abs() < f64::EPSILON);
        assert!(find_layer(&s.layers, Layer::Melody).unwrap().active);
        assert!(find_layer(&s.layers, Layer::Ambient).unwrap().active);
        assert!(!find_layer(&s.layers, Layer::Percussion).unwrap().active);
    }

    #[test]
    fn mood_exciting() {
        let mut s = SoundtrackState::new();
        s.set_mood("exciting");
        assert_eq!(s.key, MusicalKey::GMajor);
        assert!((s.tempo.bpm - 130.0).abs() < f64::EPSILON);
        assert!(s.layers.iter().all(|l| l.active));
    }

    #[test]
    fn mood_mysterious() {
        let mut s = SoundtrackState::new();
        s.set_mood("mysterious");
        assert_eq!(s.key, MusicalKey::AMinor);
        assert!(!find_layer(&s.layers, Layer::Melody).unwrap().active);
        assert!(find_layer(&s.layers, Layer::Ambient).unwrap().active);
        assert!(find_layer(&s.layers, Layer::Effects).unwrap().active);
    }

    #[test]
    fn mood_triumphant() {
        let mut s = SoundtrackState::new();
        s.set_mood("triumphant");
        assert_eq!(s.key, MusicalKey::CMajor);
        assert!((s.tempo.bpm - 120.0).abs() < f64::EPSILON);
        assert!(s.layers.iter().all(|l| l.active));
        assert!(s.layers.iter().all(|l| (l.volume - 1.0).abs() < f64::EPSILON));
    }

    #[test]
    fn mood_sad() {
        let mut s = SoundtrackState::new();
        s.set_mood("sad");
        assert_eq!(s.key, MusicalKey::DMinor);
        assert!((s.tempo.bpm - 60.0).abs() < f64::EPSILON);
        assert!(find_layer(&s.layers, Layer::Melody).unwrap().active);
        assert!(!find_layer(&s.layers, Layer::Bass).unwrap().active);
    }

    #[test]
    fn mood_scary() {
        let mut s = SoundtrackState::new();
        s.set_mood("scary");
        assert_eq!(s.key, MusicalKey::EMinor);
        assert!((s.tempo.bpm - 100.0).abs() < f64::EPSILON);
        assert!(find_layer(&s.layers, Layer::Bass).unwrap().active);
        assert!(find_layer(&s.layers, Layer::Effects).unwrap().active);
        assert!(!find_layer(&s.layers, Layer::Melody).unwrap().active);
    }

    #[test]
    fn mood_unknown_defaults_chromatic() {
        let mut s = SoundtrackState::new();
        s.set_mood("unknown_mood");
        assert_eq!(s.key, MusicalKey::Chromatic);
    }

    #[test]
    fn mood_tag_stored() {
        let mut s = SoundtrackState::new();
        s.set_mood("scary");
        assert_eq!(s.mood_tag, "scary");
    }

    // -- SoundtrackEngine --
    #[test]
    fn engine_update_from_vibe() {
        let mut eng = SoundtrackEngine::new();
        eng.update_from_vibe("forest", 0.75);
        let state = eng.get_audio_params("forest").unwrap();
        assert!(state.tempo.is_changing);
        assert!((state.tempo.target_bpm - 110.0).abs() < f64::EPSILON);
    }

    #[test]
    fn engine_update_from_emotion() {
        let mut eng = SoundtrackEngine::new();
        eng.update_from_emotion("castle", "scary");
        let state = eng.get_audio_params("castle").unwrap();
        assert_eq!(state.key, MusicalKey::EMinor);
    }

    #[test]
    fn engine_missing_room_returns_none() {
        let eng = SoundtrackEngine::new();
        assert!(eng.get_audio_params("void").is_none());
    }

    #[test]
    fn engine_multiple_rooms() {
        let mut eng = SoundtrackEngine::new();
        eng.update_from_emotion("room_a", "calm");
        eng.update_from_emotion("room_b", "exciting");
        assert_eq!(eng.get_audio_params("room_a").unwrap().key, MusicalKey::FMajor);
        assert_eq!(eng.get_audio_params("room_b").unwrap().key, MusicalKey::GMajor);
    }

    #[test]
    fn engine_vibe_then_emotion() {
        let mut eng = SoundtrackEngine::new();
        eng.update_from_vibe("lab", 0.5);
        eng.update_from_emotion("lab", "mysterious");
        let s = eng.get_audio_params("lab").unwrap();
        assert_eq!(s.key, MusicalKey::AMinor);
        // set_mood overwrites tempo to mood default (90 bpm)
        assert!((s.tempo.bpm - 90.0).abs() < f64::EPSILON);
    }

    #[test]
    fn serde_roundtrip_engine() {
        let mut eng = SoundtrackEngine::new();
        eng.update_from_emotion("dungeon", "scary");
        eng.update_from_vibe("dungeon", 0.3);
        let json = serde_json::to_string(&eng).unwrap();
        let back: SoundtrackEngine = serde_json::from_str(&json).unwrap();
        assert_eq!(back.room_soundtracks["dungeon"].key, MusicalKey::EMinor);
    }

    #[test]
    fn soundtrack_state_default() {
        let s = SoundtrackState::default();
        assert_eq!(s.key, MusicalKey::CMajor);
        assert_eq!(s.layers.len(), 6);
    }

    #[test]
    fn engine_default() {
        let eng = SoundtrackEngine::default();
        assert!(eng.room_soundtracks.is_empty());
    }

    // helper
    fn find_layer(layers: &[MusicLayer], lt: Layer) -> Option<&MusicLayer> {
    layers.iter().find(|l| l.layer_type == lt)
}
}
