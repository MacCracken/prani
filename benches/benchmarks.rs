//! Criterion benchmarks for prani creature vocal synthesis.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use prani::prelude::*;

fn bench_wolf_howl(c: &mut Criterion) {
    c.bench_function("wolf_howl_1s", |b| {
        let voice = CreatureVoice::new(Species::Wolf);
        b.iter(|| {
            let samples = voice.vocalize(&Vocalization::Howl, 44100.0, 1.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_cat_purr(c: &mut Criterion) {
    c.bench_function("cat_purr_500ms", |b| {
        let voice = CreatureVoice::new(Species::Cat);
        b.iter(|| {
            let samples = voice.vocalize(&Vocalization::Purr, 44100.0, 0.5).unwrap();
            black_box(samples);
        });
    });
}

fn bench_cricket_stridulate(c: &mut Criterion) {
    c.bench_function("cricket_stridulate_300ms", |b| {
        let voice = CreatureVoice::new(Species::Cricket);
        b.iter(|| {
            let samples = voice
                .vocalize(&Vocalization::Stridulate, 44100.0, 0.3)
                .unwrap();
            black_box(samples);
        });
    });
}

fn bench_dragon_roar(c: &mut Criterion) {
    c.bench_function("dragon_roar_1s", |b| {
        let voice = CreatureVoice::new(Species::Dragon);
        b.iter(|| {
            let samples = voice.vocalize(&Vocalization::Roar, 44100.0, 1.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_snake_hiss(c: &mut Criterion) {
    c.bench_function("snake_hiss_500ms", |b| {
        let voice = CreatureVoice::new(Species::Snake);
        b.iter(|| {
            let samples = voice.vocalize(&Vocalization::Hiss, 44100.0, 0.5).unwrap();
            black_box(samples);
        });
    });
}

fn bench_songbird_trill(c: &mut Criterion) {
    c.bench_function("songbird_trill_500ms", |b| {
        let voice = CreatureVoice::new(Species::Songbird);
        b.iter(|| {
            let samples = voice.vocalize(&Vocalization::Trill, 44100.0, 0.5).unwrap();
            black_box(samples);
        });
    });
}

fn bench_lion_roar(c: &mut Criterion) {
    c.bench_function("lion_roar_1s", |b| {
        let voice = CreatureVoice::new(Species::Lion);
        b.iter(|| {
            let samples = voice.vocalize(&Vocalization::Roar, 44100.0, 1.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_wolf_alarm_howl(c: &mut Criterion) {
    c.bench_function("wolf_alarm_howl_1s", |b| {
        let voice = CreatureVoice::new(Species::Wolf);
        b.iter(|| {
            let samples = voice
                .vocalize_with_intent(&Vocalization::Howl, CallIntent::Alarm, 44100.0, 1.0)
                .unwrap();
            black_box(samples);
        });
    });
}

fn bench_bee_buzz(c: &mut Criterion) {
    c.bench_function("bee_buzz_300ms", |b| {
        let voice = CreatureVoice::new(Species::Bee);
        b.iter(|| {
            let samples = voice.vocalize(&Vocalization::Buzz, 44100.0, 0.3).unwrap();
            black_box(samples);
        });
    });
}

fn bench_crow_screech(c: &mut Criterion) {
    c.bench_function("crow_screech_500ms", |b| {
        let voice = CreatureVoice::new(Species::Crow);
        b.iter(|| {
            let samples = voice
                .vocalize(&Vocalization::Screech, 44100.0, 0.5)
                .unwrap();
            black_box(samples);
        });
    });
}

fn bench_crocodilian_rumble(c: &mut Criterion) {
    c.bench_function("crocodilian_rumble_1s", |b| {
        let voice = CreatureVoice::new(Species::Crocodilian);
        b.iter(|| {
            let samples = voice.vocalize(&Vocalization::Rumble, 44100.0, 1.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_vocal_effort_shout(c: &mut Criterion) {
    c.bench_function("wolf_howl_shout_1s", |b| {
        let voice = CreatureVoice::new(Species::Wolf).with_vocal_effort(1.0);
        b.iter(|| {
            let samples = voice.vocalize(&Vocalization::Howl, 44100.0, 1.0).unwrap();
            black_box(samples);
        });
    });
}

fn bench_stream_wolf_howl(c: &mut Criterion) {
    c.bench_function("stream_wolf_howl_1s", |b| {
        let voice = CreatureVoice::new(Species::Wolf);
        b.iter(|| {
            let mut stream = prani::stream::SynthStream::new(
                voice.clone(),
                Vocalization::Howl,
                CallIntent::Social,
                44100.0,
                1.0,
            )
            .unwrap();
            let mut buf = vec![0.0f32; 512];
            while !stream.is_finished() {
                stream.fill_buffer(&mut buf);
            }
            black_box(buf);
        });
    });
}

fn bench_emotion_evaluate(c: &mut Criterion) {
    c.bench_function("emotion_evaluate", |b| {
        let state = EmotionState::with_values(-0.5, 0.8);
        b.iter(|| {
            let _ = black_box(state.evaluate());
        });
    });
}

criterion_group!(
    benches,
    bench_wolf_howl,
    bench_cat_purr,
    bench_cricket_stridulate,
    bench_dragon_roar,
    bench_snake_hiss,
    bench_songbird_trill,
    bench_lion_roar,
    bench_wolf_alarm_howl,
    bench_bee_buzz,
    bench_crow_screech,
    bench_crocodilian_rumble,
    bench_vocal_effort_shout,
    bench_stream_wolf_howl,
    bench_emotion_evaluate,
);

criterion_main!(benches);
