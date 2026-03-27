# prani

**prani** (Sanskrit: प्राणी — living being / creature) — Creature and animal vocal synthesis for Rust.

Procedural synthesis of non-human voices: wolves, cats, lions, birds, snakes, insects, dragons, and fantasy creatures. Built on [svara](https://crates.io/crates/svara)'s formant synthesis engine with species-specific vocal tract models, bioacoustic call patterns, and behavioral vocalization mapping.

## Features

- **13 species** with bioacoustically-derived vocal parameters
- **4 vocal apparatus types**: Laryngeal, Syringeal, Stridulatory, NoiseOnly
- **14 vocalization types**: Howl, Bark, Growl, Roar, Hiss, Chirp, Trill, and more
- **7 behavioral intents**: Alarm, Territorial, Mating, Distress, Idle, Threat, Social
- **Pitch contours** per vocalization (howls rise-fall, roars crescendo, screeches descend)
- **Subharmonics** for lions, dragons, crocodilians
- **Dual-source syrinx** for songbirds (two simultaneous pitches)
- **Dragon fire-breath** noise component
- **Species-specific formants** derived from vocal tract geometry
- ~1,000x real-time, `no_std` compatible, all types `Send + Sync`

## Quick Start

```rust
use prani::prelude::*;

let voice = CreatureVoice::new(Species::Wolf);
let samples = voice.vocalize(&Vocalization::Howl, 44100.0, 2.0).unwrap();
```

## License

GPL-3.0-only
