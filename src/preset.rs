//! Named voice presets — specific individuals within a species.
//!
//! Presets provide ready-made `CreatureVoice` configurations for
//! recognizable individual characters (e.g., a large alpha wolf
//! vs. a small pup).

use alloc::borrow::Cow;
use serde::{Deserialize, Serialize};

use crate::species::Species;
use crate::voice::CreatureVoice;

/// A named creature voice preset.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[must_use]
pub struct VoicePreset {
    /// Display name for this preset.
    pub name: Cow<'static, str>,
    /// Species.
    pub species: Species,
    /// Size scale.
    pub size: f32,
    /// F0 offset from species default (Hz).
    pub f0_offset: f32,
    /// Breathiness override.
    pub breathiness: f32,
}

impl VoicePreset {
    /// Creates a `CreatureVoice` from this preset.
    #[must_use]
    pub fn build(&self) -> CreatureVoice {
        CreatureVoice::new(self.species)
            .with_size(self.size)
            .with_f0_offset(self.f0_offset)
            .with_breathiness(self.breathiness)
    }
}

/// Built-in voice presets.
pub mod presets {
    use super::*;

    /// Large alpha wolf — deep, resonant howl.
    pub const ALPHA_WOLF: VoicePreset = VoicePreset {
        name: Cow::Borrowed("Alpha Wolf"),
        species: Species::Wolf,
        size: 1.4,
        f0_offset: -80.0,
        breathiness: 0.03,
    };

    /// Wolf pup — high-pitched, breathy.
    pub const WOLF_PUP: VoicePreset = VoicePreset {
        name: Cow::Borrowed("Wolf Pup"),
        species: Species::Wolf,
        size: 0.4,
        f0_offset: 200.0,
        breathiness: 0.12,
    };

    /// House cat — typical domestic cat.
    pub const HOUSE_CAT: VoicePreset = VoicePreset {
        name: Cow::Borrowed("House Cat"),
        species: Species::Cat,
        size: 1.0,
        f0_offset: 0.0,
        breathiness: 0.08,
    };

    /// Kitten — small, high-pitched.
    pub const KITTEN: VoicePreset = VoicePreset {
        name: Cow::Borrowed("Kitten"),
        species: Species::Cat,
        size: 0.4,
        f0_offset: 150.0,
        breathiness: 0.15,
    };

    /// Male lion — deep, powerful roar.
    pub const MALE_LION: VoicePreset = VoicePreset {
        name: Cow::Borrowed("Male Lion"),
        species: Species::Lion,
        size: 1.3,
        f0_offset: -15.0,
        breathiness: 0.12,
    };

    /// Ancient dragon — massive, earth-shaking.
    pub const ANCIENT_DRAGON: VoicePreset = VoicePreset {
        name: Cow::Borrowed("Ancient Dragon"),
        species: Species::Dragon,
        size: 4.0,
        f0_offset: -30.0,
        breathiness: 0.25,
    };

    /// Young dragon — smaller, higher.
    pub const YOUNG_DRAGON: VoicePreset = VoicePreset {
        name: Cow::Borrowed("Young Dragon"),
        species: Species::Dragon,
        size: 0.8,
        f0_offset: 50.0,
        breathiness: 0.15,
    };

    /// Bald eagle — piercing screech.
    pub const BALD_EAGLE: VoicePreset = VoicePreset {
        name: Cow::Borrowed("Bald Eagle"),
        species: Species::Raptor,
        size: 1.2,
        f0_offset: -200.0,
        breathiness: 0.1,
    };

    /// Raven — deep, rough croak.
    pub const RAVEN: VoicePreset = VoicePreset {
        name: Cow::Borrowed("Raven"),
        species: Species::Crow,
        size: 1.3,
        f0_offset: -100.0,
        breathiness: 0.2,
    };

    /// Field cricket — steady chirping.
    pub const FIELD_CRICKET: VoicePreset = VoicePreset {
        name: Cow::Borrowed("Field Cricket"),
        species: Species::Cricket,
        size: 1.0,
        f0_offset: 0.0,
        breathiness: 0.0,
    };

    /// American alligator — deep bellow.
    pub const ALLIGATOR: VoicePreset = VoicePreset {
        name: Cow::Borrowed("American Alligator"),
        species: Species::Crocodilian,
        size: 1.5,
        f0_offset: -10.0,
        breathiness: 0.18,
    };

    /// Returns all built-in presets.
    #[must_use = "returns the list of all built-in voice presets"]
    pub fn all() -> &'static [VoicePreset] {
        &[
            ALPHA_WOLF,
            WOLF_PUP,
            HOUSE_CAT,
            KITTEN,
            MALE_LION,
            ANCIENT_DRAGON,
            YOUNG_DRAGON,
            BALD_EAGLE,
            RAVEN,
            FIELD_CRICKET,
            ALLIGATOR,
        ]
    }
}
