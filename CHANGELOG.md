# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-03-27

### Added

- Initial scaffold of the prani crate
- `Species` enum: 13 species (Wolf, Dog, Cat, Lion, Songbird, Crow, Raptor, Snake, Crocodilian, Cricket, Bee, Dragon, Fantasy)
- `VocalApparatus` enum: Laryngeal, Syringeal, Stridulatory, NoiseOnly
- `SpeciesParams`: Per-species vocal parameters (f0 range, tract scale, breathiness, jitter, shimmer)
- `CreatureTract`: Species-specific vocal tract wrapping svara's VocalTract with apparatus-dependent synthesis
- `Vocalization` enum: 14 call types (Howl, Bark, Growl, Roar, Hiss, Chirp, Trill, Whine, Rumble, Purr, Yelp, Screech, Stridulate, Buzz)
- `CallIntent` enum: 7 behavioral intents (Alarm, Territorial, Mating, Distress, Idle, Threat, Social) with prosodic modifiers
- `CreatureVoice`: Species instance with individual variation (size, f0 offset, breathiness) and builder pattern
- `PraniError`: Error type with svara error conversion
- Integration tests: all species synthesize, intent modifies output, individual variation, serde roundtrips
- Criterion benchmarks: wolf howl, cat purr, cricket stridulate, dragon roar, snake hiss
- `no_std` support via `libm` + `alloc`
- Feature flags: `std` (default), `logging`, `full`
- Strict `deny.toml` matching hisab production patterns
- Send/Sync compile-time assertions on all public types
