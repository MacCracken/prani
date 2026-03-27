//! Vocalization types and call intent mapping.
//!
//! Defines the patterns and shapes of creature calls: howls, growls, chirps,
//! roars, hisses, and more. Each vocalization has a characteristic f0 contour,
//! amplitude envelope, and duration profile.

use serde::{Deserialize, Serialize};

/// A vocalization pattern — the shape and character of a creature call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Vocalization {
    /// Long sustained call with pitch rise then fall (wolves, dogs).
    Howl,
    /// Short, sharp burst (dogs, seals).
    Bark,
    /// Low-frequency sustained vibration (cats, bears).
    Growl,
    /// Very loud, powerful sustained call (lions, dragons).
    Roar,
    /// Breathy turbulent noise (snakes, cats).
    Hiss,
    /// High-pitched short call (birds, small mammals).
    Chirp,
    /// Rapid frequency modulation (songbirds).
    Trill,
    /// Ascending pitch call (dogs, wolves).
    Whine,
    /// Low rumbling (crocodilians, elephants).
    Rumble,
    /// Steady low-frequency vibration with amplitude modulation (cats).
    Purr,
    /// Very short sharp call (small dogs, rodents).
    Yelp,
    /// High-pitched descending scream (raptors).
    Screech,
    /// Rhythmic stridulation (crickets, cicadas).
    Stridulate,
    /// Wing-based tonal vibration (bees, flies).
    Buzz,
}

/// The behavioral intent behind a vocalization.
///
/// Maps to prosodic modifications: alarm calls are louder and higher-pitched,
/// mating calls are longer and more melodic, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CallIntent {
    /// Warning of danger — high urgency, elevated pitch, sharp onset.
    Alarm,
    /// Claiming territory — loud, sustained, assertive.
    Territorial,
    /// Attracting a mate — melodic, varied, extended duration.
    Mating,
    /// Expressing pain or distress — high pitch, irregular, urgent.
    Distress,
    /// Neutral/background vocalization — relaxed, low effort.
    Idle,
    /// Aggression/threat — loud, low pitch, growling quality.
    Threat,
    /// Social bonding/communication — moderate, varied.
    Social,
}

/// Parameters that modify a vocalization based on intent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[must_use]
pub struct IntentModifiers {
    /// Pitch multiplier (1.0 = species default).
    pub pitch_scale: f32,
    /// Amplitude multiplier.
    pub amplitude_scale: f32,
    /// Duration multiplier.
    pub duration_scale: f32,
    /// Urgency: affects jitter and onset sharpness (0.0-1.0).
    pub urgency: f32,
}

impl CallIntent {
    /// Returns the prosodic modifiers for this intent.
    pub fn modifiers(self) -> IntentModifiers {
        match self {
            Self::Alarm => IntentModifiers {
                pitch_scale: 1.4,
                amplitude_scale: 1.5,
                duration_scale: 0.7,
                urgency: 0.9,
            },
            Self::Territorial => IntentModifiers {
                pitch_scale: 0.9,
                amplitude_scale: 1.3,
                duration_scale: 1.5,
                urgency: 0.5,
            },
            Self::Mating => IntentModifiers {
                pitch_scale: 1.1,
                amplitude_scale: 1.0,
                duration_scale: 2.0,
                urgency: 0.2,
            },
            Self::Distress => IntentModifiers {
                pitch_scale: 1.5,
                amplitude_scale: 1.4,
                duration_scale: 0.8,
                urgency: 1.0,
            },
            Self::Idle => IntentModifiers {
                pitch_scale: 1.0,
                amplitude_scale: 0.5,
                duration_scale: 1.0,
                urgency: 0.0,
            },
            Self::Threat => IntentModifiers {
                pitch_scale: 0.7,
                amplitude_scale: 1.4,
                duration_scale: 1.2,
                urgency: 0.7,
            },
            Self::Social => IntentModifiers {
                pitch_scale: 1.0,
                amplitude_scale: 0.8,
                duration_scale: 1.0,
                urgency: 0.3,
            },
        }
    }
}
