# prani — Benchmarks

Hot-path benchmarks for the 2.0.0 Cyrius port. Run with:

```sh
cyrius bench tests/prani.bcyr
```

Scalar reference numbers (x86_64 Linux, cyrius 6.3.45, single core). These are
the inner-loop / per-call functions consumers hit; hosts own any SIMD dispatch.

| Benchmark | Time | Notes |
|---|---|---|
| `dcblocker_process` | **19 ns/sample** | Single-pole DC blocker applied to every synthesis buffer. |
| `prani_rng_next_f32` | **15 ns/sample** | PCG32 draw (aspiration / jitter / shimmer noise). |
| `emotion_evaluate` | **83 ns/frame** | 2D valence/arousal → vocalization + intent + effort + pitch + breathiness (per-frame game-AI call). |
| `crvoice_vocalize` (Wolf howl, 0.05 s @ 8 kHz = 400 samples) | **227 µs/call** (~0.57 µs/sample) | Full synthesis through the whole svara/naad stack (glottal source → vocal tract → contour → post-processing). **≈ 220× realtime.** |

## Method

- Batch pattern (`bench_batch_start` / tight loop / `bench_batch_stop`) with
  N = 1e6 for per-sample ops (amortizes clock-read overhead), 1e5 for
  `emotion_evaluate`, and 200 for the full-synthesis path.
- The full-synthesis benchmark builds a fresh `CreatureVoice` each iteration
  (matching a cold call from a game engine) and synthesizes a complete howl,
  exercising species params → svara glottal/formant/tract → pitch/formant
  contours → the post-processing chain (tilt, envelope, DC block).

## Parity note

The Rust oracle (1.1.0) reported ~1000× realtime for full synthesis in f32; the
Cyrius port is f64 throughout (svara/naad/hisab are f64-only) and routes through
the ported svara stack, so absolute numbers differ. A like-for-like Rust-vs-Cyrius
comparison on identical hardware is a follow-up (see
[`development/state.md`](development/state.md)).
