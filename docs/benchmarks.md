# prani — Benchmarks

Run with:

```sh
cyrius bench tests/prani.bcyr
```

Two groups. **Group A** is the historical series tracked since 2.0.0. **Group B**
mirrors the Rust oracle's criterion suite one-for-one — same species, same
vocalizations, same durations, same 44100 Hz — so the two harnesses can be lined
up mechanically. Group B was added in **2.0.7**, the milestone that had to run
before the Rust oracle could be [retired](development/roadmap.md).

## Per-apparatus coverage (Group B)

Every vocal apparatus is now benchmarked. Before 2.0.7 only the laryngeal path
was, so **a regression in syringeal, stridulatory, vibratile or noise-only
synthesis was invisible.**

x86_64 Linux, AMD Ryzen 7 5800H, single core, cyrius 6.5.36, prani 2.0.6 on
svara 3.5.4. Measured 2026-08-31.

| Apparatus | Benchmark | Time | ns/sample | × realtime |
|---|---|---:|---:|---:|
| laryngeal | `wolf_howl_1s` | 21.92 ms | 497 | 46× |
|  | `lion_roar_1s (infrasonic)` | 21.96 ms | 498 | 46× |
|  | `dragon_roar_1s (subharmonic)` | 23.29 ms | 528 | 43× |
|  | `crocodilian_rumble_1s (subharmonic)` | 21.93 ms | 497 | 46× |
|  | `wolf_howl_shout_1s (vocal effort)` | 21.35 ms | 484 | 47× |
|  | `wolf_alarm_howl_1s (intent)` | 15.29 ms | 495 | 46× |
|  | `cat_purr_500ms (purr path)` | 7.94 ms | 360 | 63× |
|  | `stream_wolf_howl_1s (streaming)` | 17.93 ms | 407 | 56× |
| syringeal | `songbird_trill_500ms` | 11.55 ms | 524 | 43× |
|  | `crow_screech_500ms` | 9.49 ms | 430 | 53× |
| stridulatory | `cricket_stridulate_300ms` | 2.36 ms | 178 | 127× |
| vibratile | `bee_buzz_300ms` | 2.52 ms | 190 | 119× |
| noise-only | `snake_hiss_500ms` | 2.73 ms | 124 | 183× |

**Per-sample cost is essentially rate-independent**, which is the useful number:
~497 ns/sample for a full laryngeal howl regardless of duration or rate. The
apparatus spread is real — noise-only is 4× cheaper per sample than laryngeal
(124 vs 497 ns) because it has no glottal source and no formant bank, just shaped
noise through one naad bandpass.

## The Rust comparison

> **Dated 2026-08-31.** Both harnesses on one host, in one sitting. This is the
> comparison [`roadmap 2.0.7`](development/roadmap.md) required before the oracle
> could be retired, because after removal it can only be run from a tag.

| Benchmark | Rust oracle (f32) | Cyrius port (f64) | Ratio |
|---|---:|---:|---:|
| `crow_screech_500ms` | 546 µs | 9.49 ms | **17.4×** |
| `stream_wolf_howl_1s` | 1.079 ms | 17.93 ms | **16.6×** |
| `crocodilian_rumble_1s` | 1.375 ms | 21.93 ms | **16.0×** |
| `wolf_howl_shout_1s` | 1.338 ms | 21.35 ms | **16.0×** |
| `dragon_roar_1s` | 1.467 ms | 23.29 ms | **15.9×** |
| `wolf_howl_1s` | 1.391 ms | 21.92 ms | **15.8×** |
| `lion_roar_1s` | 1.394 ms | 21.96 ms | **15.7×** |
| `cat_purr_500ms` | 505 µs | 7.94 ms | **15.7×** |
| `songbird_trill_500ms` | 741 µs | 11.55 ms | **15.6×** |
| `wolf_alarm_howl_1s` | 982 µs | 15.29 ms | **15.6×** |
| `bee_buzz_300ms` | 176 µs | 2.52 ms | **14.3×** |
| `snake_hiss_500ms` | 191 µs | 2.73 ms | **14.2×** |
| `cricket_stridulate_300ms` | 238 µs | 2.36 ms | **9.9×** |
| `emotion_evaluate` | 3.20 ns | 151 ns | **47×** |

**The port is ~16× slower than the oracle**, median **15.7×** across the 13
synthesis benchmarks, in a tight 9.9×–17.4× band. A uniform ratio across
unrelated code paths points at something systemic in the substrate rather than
one slow algorithm.

### Two figures this project has published were wrong

| Claim | Where | Measured |
|---|---|---|
| oracle does *~1000× realtime* | this file, since the port began | **719×** |
| port does *~236× realtime* | this file, 2.0.0–2.0.6 | **45.6×** |

The port's 236× was an **artifact of the 8 kHz benchmark**, not a throughput
figure. Per-sample cost barely moves with rate — 533 ns/sample at 0.05 s @ 8 kHz
against 497 ns/sample at 1.0 s @ 44100 Hz — so a benchmark at one eighth the
rate renders one eighth the samples and looks eight times more real-time than
the library actually is. **Quote ns/sample, or quote realtime with its rate.**

### What the 15.7× is, and is not

⚠ **Three things differ between those two columns, and only one is the port.**

| | Rust oracle | Cyrius port |
|---|---|---|
| float width | **f32** | **f64** |
| DSP stack | svara 1.0.0 / naad 1.0.0 / hisab 1.2.0 | svara **3.5.4** / naad **2.2.2** / hisab **2.11.2** |
| compiler | LLVM, `cargo --release` | cycc 6.5.36 |

Three major versions of the DSP dependency sit inside that ratio, and svara 3.x
added work 1.0.0 never did — subglottal coupling, a nasal antiformant,
source-filter interaction. **This measures the two stacks as shipped, not one
algorithm in two languages,** and it is not evidence that Cyrius is 15× slower
than Rust. The honest prior is that codegen dominates and float width is second.

Closing the float-width variable is [roadmap 2.1.0 Lane A](development/roadmap.md),
filed as **P0 on naad and svara** — prani cannot convert alone, because the
per-sample svara calls are f64 on both sides. It is no longer *forced*: ganita
1.1.4 ships an f32 scalar tier and prani calls nothing outside it.

## Method

- **Host**: x86_64 Linux 7.1.10, AMD Ryzen 7 5800H, single core, no pinning, no
  governor changes. Both harnesses run back to back on an otherwise idle machine.
- **Rust**: `cargo bench --bench benchmarks` at `--release`, criterion defaults
  (3 s warmup, 100 samples), `CARGO_TARGET_DIR` **outside the repo** so the
  oracle tree stayed unmodified. The point estimate is quoted.
- **Cyrius**: `cyrius bench tests/prani.bcyr`, batch pattern
  (`bench_batch_start` / loop / `bench_batch_stop`), one batch mean per benchmark.
- ⚠ **The two harnesses do not measure identically.** `lib/bench.cyr` **measures
  the timer floor and subtracts it from every sample** (cyrius 6.5.19+; 1.33 µs
  on this host); criterion does not. Criterion also warms up, adapts its sample
  count and rejects outliers; the Cyrius harness reports one batch mean, so its
  min/max columns equal the mean by construction and must not be read as a
  distribution. At millisecond scale the floor is noise, but at
  `emotion_evaluate`'s 3 ns it is **400× the measurement** — which is why that
  row's 47× ratio is not comparable to the synthesis rows and is reported
  separately.
- **Voice construction is outside the timed loop** in Group B, matching
  criterion. Group A's `wolf_howl_0.05s@8k` deliberately builds a voice per
  iteration (a cold call from a game engine), which is why the two groups'
  wolf-howl numbers are not comparable to each other either.
- **`wolf_alarm_howl_1s` renders 0.7 s, not 1.0 s** — Alarm's `duration_scale`
  is 0.7. Its realtime figure is computed on the audio actually produced;
  reading it as a 1.0 s call would overstate it by 43%.
- **Three known mirror inexactnesses**, all small against a millisecond: the
  oracle's `voice.clone()` into `SynthStream::new` has no port analogue (the
  handle is passed directly); `vec![0.0f32; 512]` becomes `vec_new()` + 512
  pushes, so ~6 doubling reallocs against Rust's one allocation, inside the
  timed region as criterion has it; and `black_box` has no equivalent, so results
  are assigned to a sink — cycc's DCE is function-granularity, so a call is never
  elided, but this is a difference in kind rather than spelling.

## Group A — the historical series

Tracked since 2.0.0 on the same host and harness. Kept unchanged so the series
stays comparable; **do not renumber or reparameterise these.**

| Benchmark | 2.0.7 | 2.0.3 | 2.0.2 | 2.0.0 | Notes |
|---|---|---|---|---|---|
| `dcblocker_process` | **19 ns/sample** | 19 ns | 20 ns | 19 ns | Single-pole DC blocker on every synthesis buffer. |
| `prani_rng_next_f32` | **14 ns/sample** | 14 ns | 14 ns | 15 ns | PCG32 draw (aspiration / jitter / shimmer). |
| `emotion_evaluate` (per-frame) | **118 ns** | 69 ns | 70 ns | 83 ns | 2.0.10 recovered 37 of the 82 ns 2.0.5's guards cost; the rest is guarded evaluation. |
| `crvoice_vocalize` wolf howl 0.05 s @ 8 kHz | **~213 µs** | 211 µs | 214 µs | 227 µs | Cold call: builds a fresh voice each iteration. |

**`emotion_evaluate`: 69 → 155 → 118 ns.** 2.0.5's input-range guards cost 82 ns
on a per-frame path by validating the same two fields **four times** per call —
`evaluate` reaches both zone functions twice, through `select_vocalization` and
`select_intent`. 2.0.10 split each into a public checked wrapper over an internal
unchecked core (the `species_params_into` shape) and validates once at the entry
point, recovering 37 ns. **The remaining ~45 ns is the entry-point validation
itself and is not a defect** — guarded evaluation costs what it costs. Output is
bit-identical across a 6,451-state sweep.

⚠ **This row is why the benchmark harness cannot be trusted without a fresh
bundle.** `tests/prani.bcyr` measures `dist/prani.cyr` — deliberately, since that
is what consumers get — so a stale bundle reports the last bundled code. While
fixing this, `cyrius audit` read **155 ns** from a stale `dist/` for a change that
actually measured **114 ns**. CI now runs bundle coherence *before* the audit;
locally, run `cyrius distlib` before believing a bench number after editing
`src/`. Roadmap 2.0.11.

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

