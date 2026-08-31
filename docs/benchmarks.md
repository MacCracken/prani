# prani — Benchmarks

Hot-path benchmarks for the Cyrius port. Run with:

```sh
cyrius bench tests/prani.bcyr
```

Scalar reference numbers (x86_64 Linux, cyrius 6.5.36, single core). These are
the inner-loop / per-call functions consumers hit; hosts own any SIMD dispatch.

| Benchmark | Time | 2.0.2 | 2.0.0 | Notes |
|---|---|---|---|---|
| `dcblocker_process` | **19 ns/sample** | 20 ns | 19 ns | Single-pole DC blocker applied to every synthesis buffer. |
| `prani_rng_next_f32` | **14 ns/sample** | 14 ns | 15 ns | PCG32 draw (aspiration / jitter / shimmer noise). |
| `emotion_evaluate` | **69 ns/frame** | 70 ns | 83 ns | 2D valence/arousal → vocalization + intent + effort + pitch + breathiness (per-frame game-AI call). |
| `crvoice_vocalize` (Wolf howl, 0.05 s @ 8 kHz = 400 samples) | **211 µs/call** (~0.53 µs/sample) | 214 µs | 227 µs | Full synthesis through the whole svara/naad stack (glottal source → vocal tract → contour → post-processing). **≈ 236× realtime.** |

**2.0.3 makes no performance claim.** Every row is within noise of 2.0.2 (three
back-to-back runs of the same binary spread 211.3–211.8 µs on the synthesis row);
its optimization work is measured in bytes retained, not nanoseconds — see the
allocation budget below. The 2.0.0 column is the same harness on the same host
under cyrius 6.3.45: `emotion_evaluate` (~16%) and the full synthesis path (~6%)
came out ahead on the 124-release toolchain bump, while the two per-sample
figures have only ever moved by one nanosecond in either direction, which is the
resolution of the measurement rather than a result.

## Allocation budget

Cyrius's allocator is a bump arena that never frees, so an allocation on a
per-frame or per-block path is retained for the life of the process. These are
measured with `alloc_used()` in `tests/hardening.tcyr`, not calculated, and are
asserted on every run:

| Path | 2.0.2 | 2.0.3 |
|---|---|---|
| `prani_ffi_voice_set_size` — a real-time parameter a host may call every frame, per creature | **168 B retained per call** (`sizeof(CreatureVoice)` 40 + `sizeof(SpeciesParams)` 128; 168,000 B over 1000 calls) | **0 B** |
| `crvoice_vocalize` block loop | one `SynthesisOptions` per 20 ms of audio, i.e. 50 per second, each retained | one per call |

At 60 fps across 100 creatures the first row was about 1 MB/second retained
permanently. svara's `streaming.cyr` states the rule it broke: *an allocation
inside an audio callback is not a leak that grows slowly, it is one that ends the
process.*

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
the ported svara stack, so absolute numbers differ. Until the two are measured on
one host with one method, neither number means anything next to the other — and
note that `lib/bench.cyr` has measured and subtracted the timer floor since
cyrius 6.5.19, which a Rust harness will not.

That comparison, and the coverage gap behind it (no benchmark touches syringeal,
stridulatory, vibratile or noise-only synthesis, the streaming path, or the
sequence/chorus paths), are owned by **roadmap 2.0.7** —
[`development/roadmap.md`](development/roadmap.md).

⚠ **That item gates `rust-old/`'s retirement (roadmap 2.0.8), and it is the only
one that does.** The comparison needs the oracle present and buildable; after
removal it can only be run from a tag, and it depends on `svara 1.0.0` /
`naad 1.0.0` / `hisab 1.2.0` staying published on crates.io. `cargo fetch
--locked` succeeds as of 2026-08-31 — that is a window, not a guarantee. The Rust
harness's 10 benchmarks also cover every vocal apparatus, so mirroring them
closes the coverage gap and makes the comparison like-for-like at the same time.
