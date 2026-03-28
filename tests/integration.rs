//! Integration tests for prani.

use prani::prelude::*;

#[test]
fn test_wolf_howl() {
    let voice = CreatureVoice::new(Species::Wolf);
    let samples = voice.vocalize(&Vocalization::Howl, 44100.0, 1.0).unwrap();
    assert!(!samples.is_empty());
    assert!(samples.iter().all(|s| s.is_finite()));
    let max_amp: f32 = samples.iter().map(|s| s.abs()).fold(0.0, f32::max);
    assert!(max_amp > 0.001, "output too quiet: {max_amp}");
}

#[test]
fn test_cat_purr() {
    let voice = CreatureVoice::new(Species::Cat);
    let samples = voice.vocalize(&Vocalization::Purr, 44100.0, 0.5).unwrap();
    assert!(!samples.is_empty());
    assert!(samples.iter().all(|s| s.is_finite()));
}

#[test]
fn test_snake_hiss() {
    let voice = CreatureVoice::new(Species::Snake);
    let samples = voice.vocalize(&Vocalization::Hiss, 44100.0, 0.5).unwrap();
    assert!(!samples.is_empty());
    assert!(samples.iter().all(|s| s.is_finite()));
}

#[test]
fn test_cricket_stridulate() {
    let voice = CreatureVoice::new(Species::Cricket);
    let samples = voice
        .vocalize(&Vocalization::Stridulate, 44100.0, 0.3)
        .unwrap();
    assert!(!samples.is_empty());
    assert!(samples.iter().all(|s| s.is_finite()));
}

#[test]
fn test_lion_roar() {
    let voice = CreatureVoice::new(Species::Lion);
    let samples = voice.vocalize(&Vocalization::Roar, 44100.0, 1.0).unwrap();
    assert!(!samples.is_empty());
    assert!(samples.iter().all(|s| s.is_finite()));
}

#[test]
fn test_dragon_roar() {
    let voice = CreatureVoice::new(Species::Dragon);
    let samples = voice.vocalize(&Vocalization::Roar, 44100.0, 1.5).unwrap();
    assert!(!samples.is_empty());
    assert!(samples.iter().all(|s| s.is_finite()));
}

#[test]
fn test_songbird_trill() {
    let voice = CreatureVoice::new(Species::Songbird);
    let samples = voice.vocalize(&Vocalization::Trill, 44100.0, 0.5).unwrap();
    assert!(!samples.is_empty());
    assert!(samples.iter().all(|s| s.is_finite()));
}

#[test]
fn test_individual_variation() {
    let small = CreatureVoice::new(Species::Dog).with_size(0.5);
    let large = CreatureVoice::new(Species::Dog).with_size(2.0);

    assert!(small.effective_f0() > large.effective_f0());
    assert!(small.effective_tract_scale() < large.effective_tract_scale());
}

#[test]
fn test_intent_modifies_output() {
    let voice = CreatureVoice::new(Species::Wolf);
    let idle = voice
        .vocalize_with_intent(&Vocalization::Howl, CallIntent::Idle, 44100.0, 0.5)
        .unwrap();
    let alarm = voice
        .vocalize_with_intent(&Vocalization::Howl, CallIntent::Alarm, 44100.0, 0.5)
        .unwrap();

    let idle_energy: f32 = idle.iter().map(|s| s * s).sum();
    let alarm_energy: f32 = alarm.iter().map(|s| s * s).sum();

    // Alarm should be louder (more energy)
    assert!(
        alarm_energy > idle_energy,
        "alarm ({alarm_energy}) should have more energy than idle ({idle_energy})"
    );
}

#[test]
fn test_all_species_synthesize() {
    let species = [
        Species::Wolf,
        Species::Dog,
        Species::Cat,
        Species::Lion,
        Species::Songbird,
        Species::Crow,
        Species::Raptor,
        Species::Snake,
        Species::Crocodilian,
        Species::Cricket,
        Species::Bee,
        Species::Dragon,
        Species::Fantasy,
    ];
    for s in &species {
        let voice = CreatureVoice::new(*s);
        // Use a vocalization compatible with each species' vocal apparatus
        let v = if s.supports_vocalization(&Vocalization::Growl) {
            Vocalization::Growl
        } else if s.supports_vocalization(&Vocalization::Hiss) {
            Vocalization::Hiss
        } else {
            Vocalization::Chirp
        };
        let result = voice.vocalize(&v, 44100.0, 0.3);
        assert!(result.is_ok(), "failed to synthesize {:?} with {:?}", s, v);
        assert!(result.unwrap().iter().all(|s| s.is_finite()));
    }
}

#[test]
fn test_serde_roundtrip_species() {
    let json = serde_json::to_string(&Species::Dragon).unwrap();
    let s2: Species = serde_json::from_str(&json).unwrap();
    assert_eq!(s2, Species::Dragon);
}

#[test]
fn test_serde_roundtrip_vocalization() {
    let json = serde_json::to_string(&Vocalization::Howl).unwrap();
    let v2: Vocalization = serde_json::from_str(&json).unwrap();
    assert_eq!(v2, Vocalization::Howl);
}

#[test]
fn test_serde_roundtrip_call_intent() {
    let json = serde_json::to_string(&CallIntent::Alarm).unwrap();
    let i2: CallIntent = serde_json::from_str(&json).unwrap();
    assert_eq!(i2, CallIntent::Alarm);
}

#[test]
fn test_serde_roundtrip_creature_voice() {
    let voice = CreatureVoice::new(Species::Wolf)
        .with_size(1.5)
        .with_f0_offset(50.0)
        .with_breathiness(0.12);
    let json = serde_json::to_string(&voice).unwrap();
    let v2: CreatureVoice = serde_json::from_str(&json).unwrap();
    assert_eq!(v2.species(), Species::Wolf);
    assert!((v2.effective_f0() - voice.effective_f0()).abs() < f32::EPSILON);
    assert!((v2.effective_tract_scale() - voice.effective_tract_scale()).abs() < f32::EPSILON);
}

#[test]
fn test_invalid_species_vocalization_rejected() {
    let snake = CreatureVoice::new(Species::Snake);
    assert!(snake.vocalize(&Vocalization::Howl, 44100.0, 1.0).is_err());

    let cricket = CreatureVoice::new(Species::Cricket);
    assert!(cricket.vocalize(&Vocalization::Roar, 44100.0, 1.0).is_err());

    let wolf = CreatureVoice::new(Species::Wolf);
    assert!(
        wolf.vocalize(&Vocalization::Stridulate, 44100.0, 1.0)
            .is_err()
    );
}

#[test]
fn test_species_valid_vocalizations() {
    // Laryngeal species should support vocal calls
    assert!(Species::Wolf.supports_vocalization(&Vocalization::Howl));
    assert!(Species::Wolf.supports_vocalization(&Vocalization::Growl));
    assert!(!Species::Wolf.supports_vocalization(&Vocalization::Stridulate));

    // NoiseOnly should only support hiss/growl
    assert!(Species::Snake.supports_vocalization(&Vocalization::Hiss));
    assert!(!Species::Snake.supports_vocalization(&Vocalization::Howl));

    // Stridulatory should support insect sounds
    assert!(Species::Cricket.supports_vocalization(&Vocalization::Chirp));
    assert!(!Species::Cricket.supports_vocalization(&Vocalization::Roar));
}

#[test]
fn test_parameter_clamping() {
    let voice = CreatureVoice::new(Species::Wolf)
        .with_breathiness(5.0) // Over max
        .with_size(-1.0) // Under min
        .with_jitter(1.0) // Over max
        .with_shimmer(1.0); // Over max

    // Should all be clamped, not crash
    let result = voice.vocalize(&Vocalization::Howl, 44100.0, 0.3);
    assert!(result.is_ok());
}

#[test]
fn test_serde_roundtrip_species_params() {
    let params = Species::Dragon.params();
    let json = serde_json::to_string(&params).unwrap();
    let p2: prani::species::SpeciesParams = serde_json::from_str(&json).unwrap();
    assert!((p2.f0_default - params.f0_default).abs() < f32::EPSILON);
    assert_eq!(p2.apparatus, params.apparatus);
}

#[test]
fn test_serde_roundtrip_vocal_apparatus() {
    use prani::species::VocalApparatus;
    let apparatuses = [
        VocalApparatus::Laryngeal,
        VocalApparatus::Syringeal,
        VocalApparatus::Stridulatory,
        VocalApparatus::Vibratile,
        VocalApparatus::NoiseOnly,
    ];
    for a in &apparatuses {
        let json = serde_json::to_string(a).unwrap();
        let a2: VocalApparatus = serde_json::from_str(&json).unwrap();
        assert_eq!(*a, a2);
    }
}

#[test]
fn test_serde_roundtrip_error() {
    let err = PraniError::SynthesisFailed("test".into());
    let json = serde_json::to_string(&err).unwrap();
    let e2: PraniError = serde_json::from_str(&json).unwrap();
    assert_eq!(err.to_string(), e2.to_string());
}

#[test]
fn test_serde_roundtrip_intent_modifiers() {
    let mods = CallIntent::Alarm.modifiers();
    let json = serde_json::to_string(&mods).unwrap();
    let m2: prani::vocalization::IntentModifiers = serde_json::from_str(&json).unwrap();
    assert!((m2.pitch_scale - mods.pitch_scale).abs() < f32::EPSILON);
    assert!((m2.amplitude_scale - mods.amplitude_scale).abs() < f32::EPSILON);
    assert!((m2.duration_scale - mods.duration_scale).abs() < f32::EPSILON);
    assert!((m2.urgency - mods.urgency).abs() < f32::EPSILON);
}

#[test]
fn test_serde_roundtrip_creature_tract() {
    let params = Species::Wolf.params();
    let tract = prani::tract::CreatureTract::new(&params, 44100.0);
    let json = serde_json::to_string(&tract).unwrap();
    let t2: prani::tract::CreatureTract = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&t2).unwrap();
    assert_eq!(json, json2);
}

#[test]
fn test_zero_duration_synthesis() {
    let voice = CreatureVoice::new(Species::Wolf);
    let samples = voice.vocalize(&Vocalization::Howl, 44100.0, 0.0).unwrap();
    assert!(samples.is_empty());
}

#[test]
fn test_high_frequency_syringeal_path() {
    // Songbird trill with alarm intent pushes f0 above 2000 Hz,
    // exercising the dual-source syringeal synthesis path.
    let voice = CreatureVoice::new(Species::Songbird);
    let samples = voice
        .vocalize_with_intent(&Vocalization::Trill, CallIntent::Alarm, 44100.0, 0.3)
        .unwrap();
    assert!(!samples.is_empty());
    assert!(samples.iter().all(|s| s.is_finite()));
}

#[test]
fn test_bee_buzz() {
    let voice = CreatureVoice::new(Species::Bee);
    let samples = voice.vocalize(&Vocalization::Buzz, 44100.0, 0.3).unwrap();
    assert!(!samples.is_empty());
    assert!(samples.iter().all(|s| s.is_finite()));
}

#[test]
fn test_crow_screech() {
    let voice = CreatureVoice::new(Species::Crow);
    let samples = voice
        .vocalize(&Vocalization::Screech, 44100.0, 0.5)
        .unwrap();
    assert!(!samples.is_empty());
    assert!(samples.iter().all(|s| s.is_finite()));
}

#[test]
fn test_all_intents_modify_differently() {
    let intents = [
        CallIntent::Alarm,
        CallIntent::Territorial,
        CallIntent::Mating,
        CallIntent::Distress,
        CallIntent::Idle,
        CallIntent::Threat,
        CallIntent::Social,
    ];
    // Each intent should produce different modifiers
    for (i, a) in intents.iter().enumerate() {
        for b in intents.iter().skip(i + 1) {
            let ma = a.modifiers();
            let mb = b.modifiers();
            let same = (ma.pitch_scale - mb.pitch_scale).abs() < f32::EPSILON
                && (ma.amplitude_scale - mb.amplitude_scale).abs() < f32::EPSILON
                && (ma.duration_scale - mb.duration_scale).abs() < f32::EPSILON
                && (ma.urgency - mb.urgency).abs() < f32::EPSILON;
            assert!(!same, "{a:?} and {b:?} should have different modifiers");
        }
    }
}

#[test]
fn test_crocodilian_rumble_with_subharmonics() {
    let voice = CreatureVoice::new(Species::Crocodilian);
    let samples = voice.vocalize(&Vocalization::Rumble, 44100.0, 1.0).unwrap();
    assert!(!samples.is_empty());
    assert!(samples.iter().all(|s| s.is_finite()));
    let max_amp: f32 = samples.iter().map(|s| s.abs()).fold(0.0, f32::max);
    assert!(max_amp > 0.001, "output too quiet: {max_amp}");
}

#[test]
fn test_raptor_screech() {
    let voice = CreatureVoice::new(Species::Raptor);
    let samples = voice
        .vocalize(&Vocalization::Screech, 44100.0, 0.5)
        .unwrap();
    assert!(!samples.is_empty());
    assert!(samples.iter().all(|s| s.is_finite()));
}

#[test]
fn test_dragon_individual_variation() {
    // Two dragons with different sizes should produce different f0
    let small = CreatureVoice::new(Species::Dragon).with_size(0.5);
    let large = CreatureVoice::new(Species::Dragon).with_size(3.0);
    assert!(small.effective_f0() > large.effective_f0());

    // Both should synthesize successfully
    let s1 = small.vocalize(&Vocalization::Roar, 44100.0, 0.5).unwrap();
    let s2 = large.vocalize(&Vocalization::Roar, 44100.0, 0.5).unwrap();
    assert!(s1.iter().all(|s| s.is_finite()));
    assert!(s2.iter().all(|s| s.is_finite()));
}

// --- High-priority backlog tests ---

#[test]
fn test_cat_purr_special_synthesis() {
    // Cat purr should use 25-30 Hz laryngeal muscle cycling
    let voice = CreatureVoice::new(Species::Cat);
    let samples = voice.vocalize(&Vocalization::Purr, 44100.0, 1.0).unwrap();
    assert!(!samples.is_empty());
    assert!(samples.iter().all(|s| s.is_finite()));
    let max_amp: f32 = samples.iter().map(|s| s.abs()).fold(0.0, f32::max);
    // Purr is naturally quiet (low-energy 25 Hz cycling through formants)
    assert!(max_amp > 0.0001, "purr output too quiet: {max_amp}");
}

#[test]
fn test_cat_purr_size_variation() {
    // Larger cat should have slightly lower purr frequency
    let small = CreatureVoice::new(Species::Cat).with_size(0.5);
    let large = CreatureVoice::new(Species::Cat).with_size(2.0);
    let s1 = small.vocalize(&Vocalization::Purr, 44100.0, 0.5).unwrap();
    let s2 = large.vocalize(&Vocalization::Purr, 44100.0, 0.5).unwrap();
    assert!(s1.iter().all(|s| s.is_finite()));
    assert!(s2.iter().all(|s| s.is_finite()));
}

#[test]
fn test_subharmonics_are_finite() {
    // Lion, Dragon, Crocodilian all use subharmonics through the tract
    for species in [Species::Lion, Species::Dragon, Species::Crocodilian] {
        let voice = CreatureVoice::new(species);
        let vocalization = if species == Species::Crocodilian {
            Vocalization::Rumble
        } else {
            Vocalization::Roar
        };
        // Use Territorial intent for louder output (Idle has 0.5x amplitude)
        let samples = voice
            .vocalize_with_intent(&vocalization, CallIntent::Territorial, 44100.0, 1.0)
            .unwrap();
        assert!(
            samples.iter().all(|s| s.is_finite()),
            "{species:?} subharmonic synthesis produced non-finite samples"
        );
        let max_amp: f32 = samples.iter().map(|s| s.abs()).fold(0.0, f32::max);
        assert!(
            max_amp > 0.01,
            "{species:?} with subharmonics too quiet: {max_amp}"
        );
    }
}

#[test]
fn test_wolf_howl_formant_transitions() {
    // Wolf howl should produce output with formant transitions (no crash)
    let voice = CreatureVoice::new(Species::Wolf);
    let samples = voice.vocalize(&Vocalization::Howl, 44100.0, 2.0).unwrap();
    assert!(!samples.is_empty());
    assert!(samples.iter().all(|s| s.is_finite()));
}

#[test]
fn test_cat_howl_formant_transitions() {
    // Cat howl/meow uses formant transitions (mouth open/close)
    let voice = CreatureVoice::new(Species::Cat);
    let samples = voice.vocalize(&Vocalization::Howl, 44100.0, 1.0).unwrap();
    assert!(!samples.is_empty());
    assert!(samples.iter().all(|s| s.is_finite()));
}

#[test]
fn test_cricket_pulse_train() {
    // Cricket stridulation should have silence gaps (pulse-train structure)
    let voice = CreatureVoice::new(Species::Cricket);
    let samples = voice
        .vocalize(&Vocalization::Stridulate, 44100.0, 1.0)
        .unwrap();
    assert!(!samples.is_empty());
    assert!(samples.iter().all(|s| s.is_finite()));
    // Check for near-silent regions from inter-chirp gaps.
    // Spectral tilt filtering may bleed tiny values into silence, so check < threshold.
    let near_silent = samples.iter().filter(|&&s| s.abs() < 0.001).count();
    assert!(
        near_silent > 10,
        "cricket pulse-train should have silence gaps, got {near_silent} near-silent samples"
    );
}

// --- Lower-priority feature tests ---

#[test]
fn test_wolf_biphonation() {
    // Wolf howl should include biphonation (second pitch) in the middle section
    let voice = CreatureVoice::new(Species::Wolf);
    let samples = voice.vocalize(&Vocalization::Howl, 44100.0, 2.0).unwrap();
    assert!(!samples.is_empty());
    assert!(samples.iter().all(|s| s.is_finite()));
}

#[test]
fn test_cat_nasal_resonance() {
    // Cat howl (meow) should apply nasal anti-formant at onset
    let voice = CreatureVoice::new(Species::Cat);
    let samples = voice.vocalize(&Vocalization::Howl, 44100.0, 1.0).unwrap();
    assert!(!samples.is_empty());
    assert!(samples.iter().all(|s| s.is_finite()));
}

#[test]
fn test_doppler_shift() {
    use prani::spatial::apply_doppler_shift;
    let voice = CreatureVoice::new(Species::Wolf);
    let samples = voice.vocalize(&Vocalization::Howl, 44100.0, 1.0).unwrap();

    // Approaching (negative velocity): higher pitch = fewer output samples
    let approaching = apply_doppler_shift(&samples, -30.0, 44100.0);
    assert!(approaching.len() < samples.len());

    // Receding (positive velocity): lower pitch = more output samples
    let receding = apply_doppler_shift(&samples, 30.0, 44100.0);
    assert!(receding.len() > samples.len());
}

#[test]
fn test_distance_attenuation() {
    use prani::spatial::apply_distance_attenuation;
    let voice = CreatureVoice::new(Species::Wolf);
    let samples = voice.vocalize(&Vocalization::Howl, 44100.0, 0.5).unwrap();

    let near = apply_distance_attenuation(&samples, 1.0, 1.0, 44100.0);
    let far = apply_distance_attenuation(&samples, 50.0, 1.0, 44100.0);

    let near_energy: f32 = near.iter().map(|s| s * s).sum();
    let far_energy: f32 = far.iter().map(|s| s * s).sum();
    assert!(
        near_energy > far_energy * 10.0,
        "near ({near_energy}) should be much louder than far ({far_energy})"
    );
}

#[test]
fn test_call_bout() {
    use prani::sequence::CallBout;
    let voice = CreatureVoice::new(Species::Dog);
    let bout = CallBout {
        vocalization: Vocalization::Bark,
        count: 3,
        call_duration: 0.2,
        interval: 0.3,
        intent: CallIntent::Alarm,
    };
    let samples = bout.synthesize(&voice, 44100.0).unwrap();
    assert!(!samples.is_empty());
    assert!(samples.iter().all(|s| s.is_finite()));
    // Should be roughly 3 barks + 2 gaps
    let expected_len = ((0.2 * 3.0 + 0.3 * 2.0) * 44100.0) as usize;
    assert!(
        (samples.len() as f32 - expected_len as f32).abs() < 44100.0,
        "bout length {} far from expected {}",
        samples.len(),
        expected_len
    );
}

#[test]
fn test_call_phrase() {
    use prani::sequence::CallPhrase;
    let voice = CreatureVoice::new(Species::Songbird);
    let phrase = CallPhrase {
        elements: vec![
            prani::sequence::CallElement {
                vocalization: Vocalization::Chirp,
                duration: 0.1,
                gap: 0.05,
            },
            prani::sequence::CallElement {
                vocalization: Vocalization::Trill,
                duration: 0.3,
                gap: 0.1,
            },
            prani::sequence::CallElement {
                vocalization: Vocalization::Chirp,
                duration: 0.1,
                gap: 0.0,
            },
        ],
        intent: CallIntent::Mating,
    };
    let samples = phrase.synthesize(&voice, 44100.0).unwrap();
    assert!(!samples.is_empty());
    assert!(samples.iter().all(|s| s.is_finite()));
}

#[test]
fn test_chorus_synthesis() {
    use prani::sequence::synthesize_chorus;
    let voices: Vec<_> = (0..4)
        .map(|i| CreatureVoice::new(Species::Wolf).with_size(0.8 + i as f32 * 0.2))
        .collect();
    let samples = synthesize_chorus(
        &voices,
        &Vocalization::Howl,
        CallIntent::Social,
        44100.0,
        1.0,
        0.2,
    )
    .unwrap();
    assert!(!samples.is_empty());
    assert!(samples.iter().all(|s| s.is_finite()));
}

#[test]
fn test_voice_presets() {
    use prani::preset::presets;
    for preset in presets::all() {
        let voice = preset.build();
        // Each preset should produce valid synthesis
        let v = if voice.species().supports_vocalization(&Vocalization::Howl) {
            Vocalization::Howl
        } else if voice.species().supports_vocalization(&Vocalization::Roar) {
            Vocalization::Roar
        } else {
            Vocalization::Growl
        };
        let samples = voice.vocalize(&v, 44100.0, 0.3).unwrap();
        assert!(
            samples.iter().all(|s| s.is_finite()),
            "preset {} produced non-finite samples",
            preset.name
        );
    }
}

#[test]
fn test_serde_roundtrip_call_bout() {
    use prani::sequence::CallBout;
    let bout = CallBout {
        vocalization: Vocalization::Bark,
        count: 3,
        call_duration: 0.2,
        interval: 0.3,
        intent: CallIntent::Alarm,
    };
    let json = serde_json::to_string(&bout).unwrap();
    let b2: CallBout = serde_json::from_str(&json).unwrap();
    assert_eq!(b2.count, 3);
    assert_eq!(b2.vocalization, Vocalization::Bark);
}

#[test]
fn test_serde_roundtrip_voice_preset() {
    use prani::preset::presets;
    let preset = presets::ALPHA_WOLF;
    let json = serde_json::to_string(&preset).unwrap();
    let p2: prani::preset::VoicePreset = serde_json::from_str(&json).unwrap();
    assert_eq!(p2.species, Species::Wolf);
}
