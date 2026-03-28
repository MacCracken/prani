//! Creature emotion state machine — valence/arousal model.
//!
//! Maps a 2D emotional state (valence × arousal) to vocalization selection,
//! call intent, and continuous synthesis parameters. Provides smooth
//! transitions between emotional states for natural-sounding behavior.
//!
//! ```text
//! High Arousal
//!   Alarm/Screech ←──── Arousal ────→ Excited/Trill
//!        (−V, +A)                       (+V, +A)
//!            │                              │
//!            │          Neutral             │
//!            │         (Idle/Growl)          │
//!            │                              │
//!   Sullen/Rumble ←──── Valence ───→ Content/Purr
//!        (−V, −A)                       (+V, −A)
//! Low Arousal
//! ```

use serde::{Deserialize, Serialize};

use crate::vocalization::{CallIntent, Vocalization};

/// A creature's emotional state in valence/arousal space.
///
/// This drives vocalization selection, call intent, and continuous
/// parameter modulation (vocal effort, breathiness, pitch scale).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[must_use]
pub struct EmotionState {
    /// Valence: −1.0 (negative/aversive) to +1.0 (positive/appetitive).
    valence: f32,
    /// Arousal: 0.0 (calm/sleepy) to 1.0 (excited/alert).
    arousal: f32,
    /// Smoothing factor for state transitions (0.0 = instant, 1.0 = very slow).
    /// Controls how quickly the creature's emotional state changes.
    smoothing: f32,
}

/// The output of the emotion state machine — parameters to drive synthesis.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[must_use]
pub struct EmotionOutput {
    /// Suggested vocalization for this emotional state.
    pub vocalization: Vocalization,
    /// Behavioral intent.
    pub intent: CallIntent,
    /// Vocal effort (0.0–1.0), derived from arousal.
    pub vocal_effort: f32,
    /// Pitch scale multiplier, derived from valence.
    pub pitch_scale: f32,
    /// Breathiness modifier (additive), derived from arousal extremes.
    pub breathiness_delta: f32,
}

impl EmotionState {
    /// Creates a new emotion state at the neutral point (idle, calm).
    pub fn new() -> Self {
        Self {
            valence: 0.0,
            arousal: 0.2,
            smoothing: 0.1,
        }
    }

    /// Creates an emotion state with specific initial values.
    pub fn with_values(valence: f32, arousal: f32) -> Self {
        Self {
            valence: valence.clamp(-1.0, 1.0),
            arousal: arousal.clamp(0.0, 1.0),
            smoothing: 0.1,
        }
    }

    /// Sets the smoothing factor (0.0 = instant transitions, 0.95 = very sluggish).
    pub fn with_smoothing(mut self, smoothing: f32) -> Self {
        self.smoothing = smoothing.clamp(0.0, 0.95);
        self
    }

    /// Returns the current valence.
    #[must_use]
    pub fn valence(&self) -> f32 {
        self.valence
    }

    /// Returns the current arousal.
    #[must_use]
    pub fn arousal(&self) -> f32 {
        self.arousal
    }

    /// Updates the emotional state toward a target, applying smoothing.
    ///
    /// Call this once per frame or tick. The state exponentially decays
    /// toward `(target_valence, target_arousal)` at a rate controlled
    /// by `smoothing`.
    pub fn update(&mut self, target_valence: f32, target_arousal: f32) {
        let tv = target_valence.clamp(-1.0, 1.0);
        let ta = target_arousal.clamp(0.0, 1.0);
        let s = self.smoothing;
        self.valence = self.valence * s + tv * (1.0 - s);
        self.arousal = self.arousal * s + ta * (1.0 - s);
    }

    /// Sets the emotional state directly (no smoothing).
    pub fn set(&mut self, valence: f32, arousal: f32) {
        self.valence = valence.clamp(-1.0, 1.0);
        self.arousal = arousal.clamp(0.0, 1.0);
    }

    /// Evaluates the current emotional state into synthesis parameters.
    ///
    /// This is the core mapping from the 2D emotion space to concrete
    /// vocalization choices and parameter values.
    #[must_use = "returns synthesis parameters derived from the emotional state"]
    pub fn evaluate(&self) -> EmotionOutput {
        let vocalization = self.select_vocalization();
        let intent = self.select_intent();

        // Vocal effort driven primarily by arousal
        // Low arousal → whisper (0.2), high arousal → shout (0.9)
        let vocal_effort = 0.2 + self.arousal * 0.7;

        // Pitch scale from valence: positive = slightly higher, negative = lower
        let pitch_scale = 1.0 + self.valence * 0.15;

        // Breathiness increases at arousal extremes (panting when very excited,
        // breathy when very calm/sleepy)
        let breathiness_delta = if self.arousal > 0.8 {
            (self.arousal - 0.8) * 0.75 // up to +0.15 at max arousal
        } else if self.arousal < 0.2 {
            (0.2 - self.arousal) * 0.5 // up to +0.1 at min arousal
        } else {
            0.0
        };

        EmotionOutput {
            vocalization,
            intent,
            vocal_effort,
            pitch_scale,
            breathiness_delta,
        }
    }

    /// Selects the most appropriate vocalization for the current state.
    fn select_vocalization(&self) -> Vocalization {
        match (self.valence_zone(), self.arousal_zone()) {
            // High arousal, negative valence → alarm/threat sounds
            (ValenceZone::Negative, ArousalZone::High) => {
                if self.valence < -0.5 {
                    Vocalization::Screech
                } else {
                    Vocalization::Bark
                }
            }
            // High arousal, positive valence → excited sounds
            (ValenceZone::Positive, ArousalZone::High) => {
                if self.valence > 0.5 {
                    Vocalization::Trill
                } else {
                    Vocalization::Chirp
                }
            }
            // High arousal, neutral valence → alert
            (ValenceZone::Neutral, ArousalZone::High) => Vocalization::Bark,

            // Low arousal, negative valence → sullen
            (ValenceZone::Negative, ArousalZone::Low) => Vocalization::Growl,
            // Low arousal, positive valence → content
            (ValenceZone::Positive, ArousalZone::Low) => Vocalization::Purr,
            // Low arousal, neutral → idle rumble
            (ValenceZone::Neutral, ArousalZone::Low) => Vocalization::Rumble,

            // Mid arousal, negative → threat
            (ValenceZone::Negative, ArousalZone::Mid) => Vocalization::Growl,
            // Mid arousal, positive → social
            (ValenceZone::Positive, ArousalZone::Mid) => Vocalization::Howl,
            // Mid arousal, neutral → idle
            (ValenceZone::Neutral, ArousalZone::Mid) => Vocalization::Whine,
        }
    }

    /// Selects the call intent from the current state.
    fn select_intent(&self) -> CallIntent {
        match (self.valence_zone(), self.arousal_zone()) {
            (ValenceZone::Negative, ArousalZone::High) => {
                if self.valence < -0.7 {
                    CallIntent::Distress
                } else {
                    CallIntent::Alarm
                }
            }
            (ValenceZone::Positive, ArousalZone::High) => CallIntent::Mating,
            (ValenceZone::Neutral, ArousalZone::High) => CallIntent::Alarm,
            (ValenceZone::Negative, ArousalZone::Low) => CallIntent::Threat,
            (ValenceZone::Positive, ArousalZone::Low) => CallIntent::Idle,
            (ValenceZone::Neutral, ArousalZone::Low) => CallIntent::Idle,
            (ValenceZone::Negative, ArousalZone::Mid) => CallIntent::Threat,
            (ValenceZone::Positive, ArousalZone::Mid) => CallIntent::Social,
            (ValenceZone::Neutral, ArousalZone::Mid) => CallIntent::Social,
        }
    }

    fn valence_zone(&self) -> ValenceZone {
        if self.valence < -0.2 {
            ValenceZone::Negative
        } else if self.valence > 0.2 {
            ValenceZone::Positive
        } else {
            ValenceZone::Neutral
        }
    }

    fn arousal_zone(&self) -> ArousalZone {
        if self.arousal < 0.33 {
            ArousalZone::Low
        } else if self.arousal > 0.66 {
            ArousalZone::High
        } else {
            ArousalZone::Mid
        }
    }
}

impl Default for EmotionState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
enum ValenceZone {
    Negative,
    Neutral,
    Positive,
}

#[derive(Debug, Clone, Copy)]
enum ArousalZone {
    Low,
    Mid,
    High,
}
