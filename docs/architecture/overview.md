# Architecture Overview

## Module Map

```text
prani/
  src/
    lib.rs          — Crate root, prelude, trait assertions
    species.rs      — Species enum, VocalApparatus, SpeciesParams
    tract.rs        — CreatureTract (wraps svara::VocalTract)
    vocalization.rs — Vocalization enum, CallIntent, IntentModifiers
    voice.rs        — CreatureVoice (species + individual variation + synthesis)
    error.rs        — PraniError, Result alias
    math.rs         — no_std math shim (sin via libm)
    rng.rs          — PCG32 PRNG for stochastic synthesis
```

## Data Flow

```text
Species::params() -> SpeciesParams
                         |
CreatureVoice::new(species)
    |-- stores SpeciesParams + individual offsets (f0, size, breathiness)
    |
vocalize(Vocalization, sample_rate, duration)
    |
    |-- validates species supports vocalization
    |-- applies CallIntent modifiers (pitch, amplitude, duration, urgency)
    |-- creates CreatureTract from SpeciesParams
    |-- selects synthesis path by VocalApparatus:
    |     |-- Laryngeal:    svara::GlottalSource -> VocalTract
    |     |-- Syringeal:    GlottalSource (<=2kHz) or dual-sine (>2kHz) -> VocalTract
    |     |-- NoiseOnly:    PRNG noise -> FormantFilter
    |     |-- Stridulatory: AM-modulated sine + harmonics
    |-- applies pitch contour (vocalization-specific f0 envelope)
    |-- adds subharmonics (Lion, Dragon, Crocodilian)
    |-- adds fire-breath noise (Dragon)
    |-- applies amplitude scaling from intent
    |-- applies attack/release envelope
    |
    v
Vec<f32> output samples
```

## Consumers

- **kiran** (game engine) — real-time creature audio
- **joshua** (game manager) — triggered vocalizations
- Any AGNOS component needing non-human vocal synthesis

## Key Dependencies

- **svara** — GlottalSource, FormantFilter, VocalTract, VowelTarget
- **hisab** — ease_in_out_smooth for envelopes
- **libm** — no_std transcendental math fallback
