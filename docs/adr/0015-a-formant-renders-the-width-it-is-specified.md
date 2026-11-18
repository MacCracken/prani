# 0015 — A formant renders the width it is specified

**Status**: Accepted
**Date**: 2026-11-17

## Context

Found while repairing audit V1 in 2.3.2
([ADR-0014](0014-a-render-does-not-depend-on-the-sample-rate.md)), and older than the
port. A species gives each formant a centre and a bandwidth in Hz: a wolf's F1 is
420 Hz, 100 Hz wide. svara's resonator takes the same two numbers and computes
α = sin ω₀ · sinh(πb/sr), the line svara ported from its own Rust 1.0.0
(`src/formant.rs:253-254`). That α makes the resonance about b · sin(2πf/sr) wide at
−3 dB, not b:

| Formant | Specified | Rendered at 8 kHz | 44.1 kHz | 96 kHz |
|---|---|---|---|---|
| wolf F1, 420 Hz | 100 Hz | 32 Hz | 6.0 Hz | 2.7 Hz |
| wolf F3, 2100 Hz | 150 Hz | 149 Hz | 44 Hz | 21 Hz |
| songbird F1, 3500 Hz | 500 Hz | 1210 Hz | 239 Hz | 114 Hz |

svara's own voices are rendered the same way: its schwa's 60 Hz F1 at 523 Hz is 4.5 Hz
wide at 44.1 kHz. Port parity carried the formula through every release, because it
was the Rust's. ADR-0014 then held every rate to the 44.1 kHz render, so 2.3.2 is
consistent across rates and too narrow at all of them.

The bar is accuracy. The reference for "b Hz wide at f" is the analog resonance
b s / (s² + b s + ω₀²), whose −3 dB width is exactly b. Over species-like centres and
widths at 8 to 96 kHz, svara's rendering is 9.4 dB RMS off it near the peak, and its
in-band power 5.1 dB low on average, 19 dB at worst.

## Decision

**prani hands svara the bandwidth at which svara renders the width specified.** For a
constant-peak bandpass, the α closest to the analog resonance is πb/sr. So
`crtract_bw64` hands svara b' = (sr/π) · asinh(πb / (sr · sin ω₀)), at which svara's
own α is exactly that, at every rate, 44.1 kHz included. b' stays under svara's
bandwidth bound however close to Nyquist the formant sits. A bandwidth svara refuses (a
non-finite one, or one past `SV_F_BW_SAFE` × sr) is handed over unchanged, for svara to
refuse as before. The noise-only voice's naad bandpass, specified as Q 2, renders f/Q
Hz the same way: Q' = sin ω₀ / (2πb/sr).

Measured against the analog resonance over the same set, three rules were tested:

| α rule | Near-peak RMS | In-band power, mean \|err\| | Worst |
|---|---|---|---|
| **πb/sr** | **0.45 dB** | **0.12 dB** | **1.3 dB** |
| −3 dB edges (the RBJ cookbook's octave form) | 0.49 dB | 0.13 dB | 3.0 dB |
| equal noise power | 0.84 dB | 0.17 dB | 3.8 dB |
| svara's own | 9.4 dB | 5.1 dB | 19 dB |

**The gain is re-solved.** Wider resonances pass more of the voice: at 2.3.2's
excitation gain the 56 renders ADR-0007 calibrates on were 10.82 dB louder at the
median. By ADR-0007's own method (renders are linear in the glottal part, so two
renders give any gain), the gain is now −3.78 dB, from +7.37, and the median is again
2.1.1's level. Around it the 56 spread from −4.9 to +7.9 dB, the timbre this release
corrects, and the voice suite's bound on that spread is re-recorded at ±8 dB.

This supersedes ADR-0014's bandwidth rule, which carried each formant's 44.1 kHz width
to other rates. ADR-0014's lip, pole and noise rules stand: they hold a stage to the
44.1 kHz response, and with this rule the 44.1 kHz response is now the specified one.

## Consequences

- **Positive.** A formant is as wide as the species says. `tests/tract.tcyr` drives a
  sine through one svara resonance built as `crtract_new` builds it, and finds it
  within 0.25 dB of −3 dB at the analog edges, at 44.1, 8, 16 and 96 kHz (measured
  −3.01 dB well below Nyquist, −2.91 and −3.14 dB for a songbird's F1 at 16 kHz). With
  svara's widths a wolf's F1 is 24.5 dB down there. Against 44.1 kHz, renders at other
  rates are closer than in 2.3.2, since the reference itself is now accurate: over 861
  renders the mean level error is 0.225 dB (0.253), the worst 2.6 (3.1), and the mean
  spectral error 1.10 dB (1.27).
- **Negative.** Every render through svara's formants changes, at 44.1 kHz too. The
  pinned layer levels moved −0.10 to +3.91 dB and are re-recorded: the songbird's
  trill most, then the two streams and the cat's howl. The cricket and the bee, whose
  paths never reach svara's formants, did not move.
- **Neutral.** svara is unchanged; prani sets the bandwidth it hands svara. Recording
  the cat's howl found the f32 build's nasal notch recursing in f32 at poles 0.994 from
  the unit circle, 1.28e-6 off the f64 build in RMS once the formants widened. It now
  recurses in f64 (ADR-0010), 1.43e-7, and the f64 build is unchanged by it.

## Alternatives considered

- **Keep svara's widths** (2.3.2). Consistent across rates, and 3 to 16 times too
  narrow at 44.1 kHz for the low formants.
- **The −3 dB-edge or equal-noise-power α.** Each measured, and each less accurate than
  πb/sr on both the shape near the peak and the in-band power.
- **Keep 2.3.2's gain.** The voice would be 10.8 dB louder at the median than the level
  ADR-0007 keeps prani at.
