//! Vocal fatigue and habituation system.
//!
//! Tracks calling history and applies realistic degradation:
//!
//! - **Fatigue**: Prolonged calling causes pitch drift downward, increased
//!   breathiness, and reduced amplitude. Recovers during silence.
//! - **Habituation**: Repeated alarm calls without reinforcement lose
//!   intensity over time (the "boy who cried wolf" effect).

use serde::{Deserialize, Serialize};

/// Tracks vocal fatigue and call habituation for a creature voice.
///
/// Feed call events via [`record_call`](Self::record_call) and silence via
/// [`rest`](Self::rest). Query the current modifiers with [`modifiers`](Self::modifiers).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[must_use]
pub struct FatigueState {
    /// Accumulated fatigue (0.0 = fresh, 1.0 = exhausted).
    fatigue: f32,
    /// Habituation level for alarm/threat calls (0.0 = full response, 1.0 = fully habituated).
    alarm_habituation: f32,
    /// Total calling time in the current bout (seconds).
    bout_calling_time: f32,
    /// Number of alarm calls in the current window without reinforcement.
    alarm_count: u32,
    /// Whether the last alarm was reinforced (threat confirmed).
    alarm_reinforced: bool,
}

/// Modifiers produced by the fatigue system, applied to synthesis.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[must_use]
pub struct FatigueModifiers {
    /// Pitch drift (negative = downward drift from fatigue). In Hz.
    pub pitch_offset: f32,
    /// Breathiness increase (additive, 0.0 = no change).
    pub breathiness_delta: f32,
    /// Amplitude scale (1.0 = normal, < 1.0 = fatigued/habituated).
    pub amplitude_scale: f32,
    /// Jitter increase factor (1.0 = normal, > 1.0 = fatigued).
    pub jitter_scale: f32,
}

impl FatigueState {
    /// Creates a fresh (non-fatigued) state.
    pub fn new() -> Self {
        Self {
            fatigue: 0.0,
            alarm_habituation: 0.0,
            bout_calling_time: 0.0,
            alarm_count: 0,
            alarm_reinforced: false,
        }
    }

    /// Returns the current fatigue level (0.0–1.0).
    #[must_use]
    pub fn fatigue(&self) -> f32 {
        self.fatigue
    }

    /// Returns the current alarm habituation level (0.0–1.0).
    #[must_use]
    pub fn alarm_habituation(&self) -> f32 {
        self.alarm_habituation
    }

    /// Records that a call was produced.
    ///
    /// `duration` — call duration in seconds.
    /// `is_alarm` — whether this was an alarm or threat call.
    pub fn record_call(&mut self, duration: f32, is_alarm: bool) {
        self.bout_calling_time += duration;

        // Fatigue accumulates based on calling time.
        // ~30 seconds of continuous calling reaches ~0.5 fatigue.
        // ~60 seconds reaches ~0.75.
        let fatigue_rate = 0.015; // per second of calling
        self.fatigue = (self.fatigue + duration * fatigue_rate).clamp(0.0, 1.0);

        if is_alarm {
            self.alarm_count += 1;
            // Habituation increases with unreinforced alarms.
            // Each unreinforced alarm adds ~0.1 habituation.
            if !self.alarm_reinforced {
                self.alarm_habituation = (self.alarm_habituation + 0.1).clamp(0.0, 1.0);
            }
        }
    }

    /// Records a period of rest (silence).
    ///
    /// `duration` — rest duration in seconds. Fatigue recovers during rest.
    pub fn rest(&mut self, duration: f32) {
        // Recovery: ~50% fatigue reduction per 10 seconds of rest.
        let recovery_rate = 0.07; // per second of rest
        self.fatigue = (self.fatigue - duration * recovery_rate).max(0.0);

        // Habituation decays slowly (~50% per 30 seconds without alarm).
        let habituation_decay = 0.023; // per second
        self.alarm_habituation = (self.alarm_habituation - duration * habituation_decay).max(0.0);

        // If enough rest, reset bout tracking.
        if duration > 5.0 {
            self.bout_calling_time = 0.0;
        }
    }

    /// Reinforces the last alarm (threat was real).
    ///
    /// Resets habituation, confirming that alarm calls are still warranted.
    pub fn reinforce_alarm(&mut self) {
        self.alarm_reinforced = true;
        self.alarm_habituation = (self.alarm_habituation - 0.3).max(0.0);
        self.alarm_count = 0;
    }

    /// Resets the reinforcement flag (call this at the start of each new alarm bout).
    pub fn clear_reinforcement(&mut self) {
        self.alarm_reinforced = false;
    }

    /// Returns the current fatigue and habituation modifiers.
    #[must_use = "returns fatigue modifiers that should be applied to synthesis"]
    pub fn modifiers(&self) -> FatigueModifiers {
        // Pitch drift: up to -20 Hz at full fatigue (vocal fold tension drops).
        let pitch_offset = -self.fatigue * 20.0;

        // Breathiness: up to +0.2 at full fatigue (incomplete glottal closure).
        let breathiness_delta = self.fatigue * 0.2;

        // Amplitude: fatigue reduces to 0.6x, habituation further reduces alarm amplitude.
        let fatigue_amp = 1.0 - self.fatigue * 0.4;
        let habituation_amp = 1.0 - self.alarm_habituation * 0.5;
        let amplitude_scale = fatigue_amp * habituation_amp;

        // Jitter: increases with fatigue (less stable phonation).
        let jitter_scale = 1.0 + self.fatigue * 1.5;

        FatigueModifiers {
            pitch_offset,
            breathiness_delta,
            amplitude_scale,
            jitter_scale,
        }
    }

    /// Resets all fatigue and habituation state.
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for FatigueState {
    fn default() -> Self {
        Self::new()
    }
}
