# prani

**prani** (Sanskrit: प्राणी — living being / creature) — Creature and animal vocal synthesis for [Cyrius](https://github.com/MacCracken/cyrius) / AGNOS.

Procedural synthesis of non-human voices: wolves, cats, lions, birds, snakes, insects, dragons, and fantasy creatures. Built on [svara](https://github.com/MacCracken/svara)'s glottal/formant engine with species-specific vocal tract models, bioacoustic call patterns, and behavioral vocalization mapping.

> **v2.0.0 is a Rust → Cyrius port.** The original Rust crate (through 1.1.0) is
> frozen in git history as the parity oracle — read it with
> `git show 2.0.3:rust-old/src/voice.rs`
> ([ADR-0004](docs/adr/0004-cite-the-oracle-by-tag.md)). Every Cyrius module is
> cross-checked against it function-for-function. See
> [`docs/development/port-audit.md`](docs/development/port-audit.md).

## Features

- **13 species** with bioacoustically-derived vocal parameters
- **5 vocal apparatus types**: Laryngeal, Syringeal, Stridulatory, Vibratile, NoiseOnly
- **14 vocalization types**: Howl, Bark, Growl, Roar, Hiss, Chirp, Trill, and more
- **7 behavioral intents**: Alarm, Territorial, Mating, Distress, Idle, Threat, Social
- **Bioacoustic realism**: cat purr (25-30 Hz cycling), formant transitions, cricket pulse-train chirps, biphonation, nasal resonance, spectral tilt
- **Subharmonics** with time-varying amplitude and deterministic chaos for lions, dragons, crocodilians
- **Dual-source syrinx** for songbirds (two simultaneous pitches)
- **Dragon fire-breath** noise component
- **Spatial audio**: Doppler shift, distance attenuation with atmospheric HF absorption
- **Sequencing**: call bouts, phrases, multi-voice chorus synthesis
- **11 built-in voice presets**: Alpha Wolf, Wolf Pup, House Cat, Kitten, Male Lion, Ancient/Young Dragon, Bald Eagle, Raven, Field Cricket, American Alligator
- **Emotion & fatigue** drive: valence/arousal → vocalization + intent + effort; vocal fatigue + alarm habituation

## Quick Start

prani is a Cyrius library. Build the smoke binary and run a suite:

```sh
cyrius deps                              # resolve svara/naad/hisab/goonj/sakshi
cyrius build src/main.cyr build/prani    # compile
cyrius test tests/voice.tcyr             # run one parity suite
```

Synthesize a wolf howl (Cyrius):

```
# a fresh Wolf voice → a 2-second howl at 44.1 kHz
var voice = crvoice_new(PRANI_SP_WOLF);
var samples = crvoice_vocalize(voice, PRANI_VOC_HOWL, f64_from(44100), f64_from(2));
# samples is a vec of f64 (or a negative PRANI_ERR_* on failure)

# a preset character with a territorial intent
var alpha = preset_build(preset_alpha_wolf());
var call = crvoice_vocalize_with_intent(alpha, PRANI_VOC_HOWL, PRANI_INTENT_TERRITORIAL, f64_from(44100), f64_from(3));

# a wolf-pack chorus
var pack = vec_new();
vec_push(pack, crvoice_new(PRANI_SP_WOLF));
vec_push(pack, crvoice_new(PRANI_SP_WOLF));
var chorus = sequence_synthesize_chorus(pack, PRANI_VOC_HOWL, PRANI_INTENT_SOCIAL, f64_from(44100), f64_from(3), F64_HALF);
```

Five worked programs live in [`docs/examples/`](docs/examples/) — start with
`basic.cyr`. Each one includes the published `dist/prani.cyr` exactly as a
consumer would, and CI builds and runs all five on every push, so they cannot
rot against the API.

## Consuming prani

Downstream Cyrius projects include the distlib bundle and resolve prani's
dependencies from their own `cyrius.cyml`:

```
include "lib/hisab.cyr"
include "lib/goonj.cyr"
include "lib/sakshi.cyr"
include "lib/naad.cyr"
include "lib/svara.cyr"
include "dist/prani.cyr"
```

```toml
[deps.prani]
git = "https://github.com/MacCracken/prani.git"
tag = "2.0.11"
modules = ["dist/prani.cyr"]
```

## License

GPL-3.0-only
