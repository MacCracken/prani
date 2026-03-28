# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

#### CI/Infrastructure
- CI/CD pipeline: GitHub Actions workflows (ci.yml, release.yml) matching svara
- Makefile with check, fmt, clippy, test, audit, deny, bench, coverage, build, doc targets
- rust-toolchain.toml, codecov.yml, scripts/bench-history.sh
- Expanded .gitignore

#### High-Priority Features
- **Cat purr special-case synthesis**: 25-30 Hz laryngeal muscle cycling with asymmetric waveform and formant-filtered resonance
- **Formant transitions**: Dynamic formant changes during cat meow (nasal → open → closing) and wolf howl (gradual mouth shape)
- **Cricket discrete pulse-train chirps**: 3-5 pulse groups at ~30 Hz with inter-chirp silence at ~2.5 Hz
- **Spectral tilt** (`spectral_tilt` field on `SpeciesParams`): dB/octave roll-off per species (lion: -6, bird: -1)

#### Medium-Priority Features
- **Time-varying subharmonic amplitude**: Peaks during middle of roar/bellow, not constant
- **Deterministic chaos**: Noise modulated by subharmonic period during peak intensity (lion/dragon/crocodilian roars)
- **AM patterns**: Bird trill rapid amplitude modulation at 20 Hz
- **`VocalApparatus::Vibratile`**: New variant for bees (thoracic flight muscle vibration, not friction-based stridulation)
- **Improved bird vocal tract**: Wider bandwidths for syringeal species (less defined formants)
- **Increased crow breathiness** (0.15 → 0.18): More realistic harsh/noisy corvid calls

#### Lower-Priority Features
- **Nasal resonance**: Anti-formant (spectral notch at ~250 Hz) during nasal phases of cat meow and wolf howl onset
- **Biphonation for canids**: Second independent pitch (~minor seventh) during middle of wolf/dog howls and whines
- **`spatial` module**: Distance-based attenuation with atmospheric HF absorption; Doppler pitch shift for moving sources
- **`sequence` module**: `CallBout` (repeated calls with intervals), `CallPhrase` (ordered vocalization sequences), `synthesize_chorus` (multiple voices with timing spread)
- **`preset` module**: Named voice presets (`VoicePreset`) with 7 built-in presets (Alpha Wolf, Wolf Pup, House Cat, Kitten, Male Lion, Ancient Dragon, Young Dragon)

#### Quality
- `#[must_use]` on `SpeciesParams`, `IntentModifiers`, `VoicePreset`
- Tracing warning for out-of-range formant fallback
- 26 new tests (46 total), 11 benchmarks
- docs/architecture/overview.md, docs/development/roadmap.md

### Changed

- Dragon fire-breath RNG seed derived from species params (was hardcoded)
- Bee species now uses `VocalApparatus::Vibratile` (was `Stridulatory`)
- Subharmonics now have time-varying envelope with chaos injection (was constant 0.3 amplitude)
- Removed unused f64 math module and unused RNG methods
- Removed `#[allow(dead_code)]` from math and rng modules

### Performance

- Intentional regressions from new features (spectral tilt, formant transitions, biphonation, nasal resonance, chaos, AM patterns):
  - wolf_howl_1s: 1.06 -> 1.49 ms (+40%) — added biphonation, nasal resonance, formant transitions, spectral tilt
  - wolf_alarm_howl_1s: 745 -> 1040 us (+40%) — same pipeline additions
  - lion_roar_1s: 1.29 -> 1.47 ms (+14%) — added time-varying subharmonics + chaos
  - dragon_roar_1s: 1.38 -> 1.55 ms (+12%) — same
  - songbird_trill_500ms: 671 -> 802 us (+20%) — added trill AM pattern + spectral tilt
- snake_hiss_500ms: 261 -> 252 us (-4%) — improved

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
