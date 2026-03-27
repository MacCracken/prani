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
    let voice = CreatureVoice::new(Species::Wolf).with_size(1.5);
    let json = serde_json::to_string(&voice).unwrap();
    let v2: CreatureVoice = serde_json::from_str(&json).unwrap();
    assert_eq!(v2.species(), Species::Wolf);
}

#[test]
fn test_serde_roundtrip_error() {
    let err = PraniError::SynthesisFailed("test".into());
    let json = serde_json::to_string(&err).unwrap();
    let e2: PraniError = serde_json::from_str(&json).unwrap();
    assert_eq!(err.to_string(), e2.to_string());
}
