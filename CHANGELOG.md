# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `naad` as optional dependency with `naad-backend` feature flag (default on), matching svara
- Non-stationary jitter/shimmer: perturbation scales with call urgency and position (stronger at boundaries and during alarm/distress)
- `Species::bout_template()`: species-specific default `CallBout` for all 13 species (e.g., dogs bark 5x at 0.25s intervals, wolves howl 3x with 2s gaps)
- 4 new voice presets: Bald Eagle, Raven, Field Cricket, American Alligator (11 total)
- Spectral envelope per vocalization: growls/rumbles darker (-2 dB/oct offset), screeches/hisses brighter (+1.5 to +2 dB/oct)
- Source-filter coupling for birds: F1 tracks toward f0 at 40% coupling strength, simulating syrinx-tract interaction
- 4 new tests: bout templates, spectral envelope, source-filter coupling, non-stationary perturbation (50 total)
- CI/CD pipeline: GitHub Actions workflows (ci.yml, release.yml) matching svara
- Makefile, rust-toolchain.toml, codecov.yml, scripts/bench-history.sh
- `spatial` module: `apply_distance_attenuation` (inverse-distance + atmospheric HF absorption), `apply_doppler_shift` (linear interpolation resampling)
- `sequence` module: `CallBout` (repeated calls with intervals), `CallPhrase` (ordered vocalization sequences), `synthesize_chorus` (multiple voices with timing spread)
- `preset` module: `VoicePreset` with 7 built-in presets (Alpha Wolf, Wolf Pup, House Cat, Kitten, Male Lion, Ancient Dragon, Young Dragon)
- `VocalApparatus::Vibratile`: new variant for bees (thoracic flight muscle vibration)
- `spectral_tilt` field on `SpeciesParams`: per-species dB/octave roll-off (lion: -6, bird: -1)
- Cat purr special-case synthesis: 25-30 Hz laryngeal muscle cycling with asymmetric waveform through vocal tract
- Formant transitions: dynamic formant changes during cat meow (nasal -> open -> closing) and wolf howl
- Cricket discrete pulse-train chirps: 3-5 pulse groups at ~30 Hz with inter-chirp silence
- Time-varying subharmonic amplitude for lion/dragon/crocodilian (peaks during middle of call)
- Deterministic chaos injection during peak intensity of roars (period-doubling roughness)
- Biphonation for canids: second independent pitch (~minor seventh) during wolf/dog howls
- Nasal resonance: anti-formant notch at ~250 Hz during nasal phases of cat meow and wolf howl
- AM patterns: bird trill rapid amplitude modulation at 20 Hz
- `#[must_use]` on `SpeciesParams`, `IntentModifiers`, `VoicePreset`
- Tracing warning when species formants fall out of svara's valid range
- 26 new integration tests (46 total), covering all new modules and features
- docs/architecture/overview.md with full data flow diagram
- docs/development/roadmap.md

### Changed

- Bee species now uses `VocalApparatus::Vibratile` (was `Stridulatory`)
- Bird species (Songbird, Crow, Raptor) have wider formant bandwidths for less defined resonances
- Crow breathiness increased (0.15 -> 0.18) for more realistic harsh/noisy calls
- Dragon fire-breath RNG seed derived from species params (was hardcoded 8888)
- Subharmonics now have time-varying envelope with chaos (was constant 0.3 amplitude sine)
- Removed unused f64 math module and unused RNG methods (poisson, next_f32_range, next_f32_unsigned)
- Removed `#[allow(dead_code)]` suppressions

### Performance

- New features add processing to the synthesis pipeline. Regressions are proportional to added complexity:
  - wolf_howl_1s: 1.29 -> 1.49 ms (+15%) — biphonation, nasal resonance, formant transitions, spectral tilt
  - wolf_alarm_howl_1s: 773 -> 1040 us (+35%) — same pipeline additions
  - lion_roar_1s: 1.64 -> 1.47 ms (-10%) — net improvement despite new subharmonic envelope + chaos
  - dragon_roar_1s: 1.52 -> 1.55 ms (+2%) — near-neutral
  - songbird_trill_500ms: 872 -> 802 us (-8%) — improved despite new AM pattern
  - snake_hiss_500ms: 519 -> 252 us (-51%) — improved (dead code removal, no new processing)
  - cricket_stridulate_300ms: 221 -> 235 us (+6%) — pulse-train replaces continuous AM

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
