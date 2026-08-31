# prani — Benchmarks

Hot-path benchmarks for the Cyrius port. Run with:

```sh
cyrius bench tests/prani.bcyr
```

Scalar reference numbers (x86_64 Linux, cyrius 6.5.36, single core). These are
the inner-loop / per-call functions consumers hit; hosts own any SIMD dispatch.

| Benchmark | Time | vs 2.0.0 | Notes |
|---|---|---|---|
| `dcblocker_process` | **20 ns/sample** | 19 ns | Single-pole DC blocker applied to every synthesis buffer. |
| `prani_rng_next_f32` | **14 ns/sample** | 15 ns | PCG32 draw (aspiration / jitter / shimmer noise). |
| `emotion_evaluate` | **70 ns/frame** | 83 ns | 2D valence/arousal → vocalization + intent + effort + pitch + breathiness (per-frame game-AI call). |
| `crvoice_vocalize` (Wolf howl, 0.05 s @ 8 kHz = 400 samples) | **214 µs/call** (~0.53 µs/sample) | 227 µs | Full synthesis through the whole svara/naad stack (glottal source → vocal tract → contour → post-processing). **≈ 234× realtime.** |

The `vs 2.0.0` column is the same harness on the same host under cyrius 6.3.45.
`emotion_evaluate` (~16%) and the full synthesis path (~6%) came out ahead on the
124-release toolchain bump; the two per-sample figures moved by one nanosecond in
opposite directions, which is the resolution of the measurement rather than a
result.

## Method

- Batch pattern (`bench_batch_start` / tight loop / `bench_batch_stop`) with
  N = 1e6 for per-sample ops (amortizes clock-read overhead), 1e5 for
  `emotion_evaluate`, and 200 for the full-synthesis path.
- Since cyrius **6.5.19**, `lib/bench.cyr` also *measures* the timer floor and
  subtracts it from every sample; the harness prints its own figure
  (`bench_clock_overhead_ns()`, 1.31 µs on this host) rather than quoting a
  constant. A clock read spans ~15 ns to ~3,550 ns across hosts, so no single
  written-down number survives the move. The batch pattern already amortized it,
  which is why the 2.0.0 column above is still comparable.
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
