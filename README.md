# prani

**prani** (Sanskrit: प्राणी — living being / creature) — Creature and animal vocal synthesis for Rust.

Procedural synthesis of non-human voices: wolves, cats, lions, birds, snakes, insects, dragons, and fantasy creatures. Built on [svara](https://crates.io/crates/svara)'s formant synthesis engine with species-specific vocal tract models, bioacoustic call patterns, and behavioral vocalization mapping.

## Features

- **13 species** with bioacoustically-derived vocal parameters
- **5 vocal apparatus types**: Laryngeal, Syringeal, Stridulatory, Vibratile, NoiseOnly
- **14 vocalization types**: Howl, Bark, Growl, Roar, Hiss, Chirp, Trill, and more
- **7 behavioral intents**: Alarm, Territorial, Mating, Distress, Idle, Threat, Social
- **Bioacoustic realism**: cat purr (25 Hz cycling), formant transitions, cricket pulse-train chirps, biphonation, nasal resonance, spectral tilt
- **Subharmonics** with time-varying amplitude and deterministic chaos for lions, dragons, crocodilians
- **Dual-source syrinx** for songbirds (two simultaneous pitches)
- **Dragon fire-breath** noise component
- **Spatial audio**: Doppler shift, distance attenuation with atmospheric HF absorption
- **Sequencing**: call bouts, phrases, multi-voice chorus synthesis
- **7 built-in voice presets**: Alpha Wolf, Wolf Pup, House Cat, Kitten, Male Lion, Ancient Dragon, Young Dragon
- ~1,000x real-time, `no_std` compatible, all types `Send + Sync + Serialize + Deserialize`

## Quick Start

```rust
use prani::prelude::*;

// Create a wolf voice and synthesize a howl
let voice = CreatureVoice::new(Species::Wolf);
let samples = voice.vocalize(&Vocalization::Howl, 44100.0, 2.0).unwrap();

// Use a preset for a specific character
use prani::preset::presets;
let alpha = presets::ALPHA_WOLF.build();
let howl = alpha.vocalize_with_intent(
    &Vocalization::Howl,
    CallIntent::Territorial,
    44100.0,
    3.0,
).unwrap();

// Synthesize a wolf pack chorus
use prani::sequence::synthesize_chorus;
let pack: Vec<_> = (0..4)
    .map(|i| CreatureVoice::new(Species::Wolf).with_size(0.8 + i as f32 * 0.2))
    .collect();
let chorus = synthesize_chorus(
    &pack, &Vocalization::Howl, CallIntent::Social, 44100.0, 3.0, 0.3,
).unwrap();

// Apply distance attenuation
use prani::spatial::apply_distance_attenuation;
let far_away = apply_distance_attenuation(&samples, 100.0, 1.0, 44100.0);
```

## License

GPL-3.0-only
