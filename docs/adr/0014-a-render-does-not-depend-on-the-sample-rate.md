# 0014 — A render's level does not depend on the sample rate

**Status**: Accepted
**Date**: 2026-11-17

## Context

Audit V1, from the 2.3 sweep ([`docs/audit/2026-11-17-2.3-audit.md`](../audit/2026-11-17-2.3-audit.md)),
found while verifying D3 and older than 2.3. prani's levels are calibrated at 44.1 kHz:
ADR-0007 solved its excitation gain over 56 renders at that rate. A render at another
rate should be that render, cut to its band. It was not. Against the same call at
44.1 kHz cut to the lower Nyquist, scored by 1/3-octave band levels over 127 renders a
rate, the median render was:

| Rate | 8 kHz | 11.025 | 16 | 22.05 | 32 | 48 | 96 |
|---|---|---|---|---|---|---|---|
| Level, 2.3.1 | +18.8 dB | +15.0 | +11.1 | +7.9 | +3.6 | −0.9 | −8.7 |

With D3's correct formants, two 8 kHz renders of the sweep peaked past full scale.
Splitting a wolf's chain by stage at 8 and 96 kHz found the glottal source rate-invariant
(0.00 dB) and four stages whose response is fixed per sample rather than per second:

- **svara's lip radiation**, `y = x − 0.97·x[n−1]` at every rate. Against 44.1 kHz its
  gain is +11.7 to +14.4 dB at 8 kHz (200 Hz to 3.5 kHz). Its source-filter interaction
  feeds that radiated signal back, so the loop changed with the rate as well.
- **svara's formant bandwidths.** Its resonator, α = sin ω₀ · sinh(πb / sr), is about
  b · sin(2πf / sr) wide at −3 dB, not b. A wolf's 100 Hz F1 at 420 Hz is 6.0 Hz wide
  at 44.1 kHz, 32 at 8 kHz and 2.7 at 96 kHz. The 8 kHz bank passed 12.9 dB more of the
  voice, and the 96 kHz one 6.3 dB less. naad's bandpass, α = sin ω₀ / 2Q, narrows
  toward Nyquist the same way, and halved a snake's 2500 Hz band at 8 kHz.
- **prani's one-poles.** The spectral tilt ignored its `sample_rate` argument, as the
  Rust did, and the DC blocker's pole was 0.995 at every rate. Each corner moved with
  the rate: 5.5 times lower at 8 kHz.
- **white noise.** Every source, svara's aspiration and prani's own (the syringeal
  breath, the noise-only voice, the purr's closed phase, the chaos and fire-breath
  layers), draws a fixed variance per sample, a power density that falls as the rate
  rises: 7.4 dB more of it in band at 8 kHz.

## Decision

**prani holds every rate-fixed stage to its 44.1 kHz response.** 44.1 kHz is the
reference rate. At exactly that rate nothing changes, so every 44.1 kHz render is the
same bits it was. At any other rate:

1. **Lip radiation.** `1 − c z⁻¹` with c = e^(−a/sr) is about (s + a)/sr, so its zero
   keeps its place in Hz at c = 0.97^(44100/sr). The gain sr/44100 goes in svara's output
   gain, and in the interaction strength, whose feedback taps the radiated signal ahead
   of that gain. svara clamps the strength at 0.3, which its default 0.05 reaches only
   past 264.6 kHz.
2. **Bandwidths.** A constant-peak bandpass passes noise power α/(1 + α) · sr/2 at any
   centre frequency (measured). So each formant's α is carried from 44.1 kHz at equal
   noise power in Hz, and the bandwidth handed to svara solved from it:
   b' = (sr/π) · asinh(α/sin ω₀). That stays under svara's bandwidth bound however close
   to Nyquist the formant sits. The noise-only voice's Q for naad is carried the same
   way. A formant at or past 22.05 kHz, which the reference has no band for, keeps its b.
3. **One-poles.** A coefficient p calibrated at 44.1 kHz becomes p^(44100/sr), the same
   corner in Hz. The tilt now reads its sample rate, and refuses one that is not
   positive and finite, as every other reader of one does.
4. **Noise.** Each source is scaled by √(sr/44100). svara's aspiration is reached through
   ADR-0007's breathiness mapping: b' = q·g·b / (1 − b + q·g·b), scale (1 − b + q·g·b)/g,
   which at q = 1 is ADR-0007's.

svara is not changed. prani sets what svara exposes: its lip coefficient, interaction
strength and output gain, and the bandwidths it is given.

## Consequences

- **Positive.** Over 861 renders (127 at each of seven rates), the median level error
  is now within ±0.3 dB at every rate, the mean |error| 0.25 dB, its 95th percentile
  0.83 dB and its worst 3.1, where they were 7.2, 21.0 and 26.2 dB. The mean spectral
  error is 1.3 dB, from 8.5. The f32 build scores the same. No sweep render now peaks
  past full scale. `tests/tract.tcyr` pins each stage's setting at 8 kHz against its
  formula and its 44.1 kHz setting as untouched. It renders a wolf's 400 Hz harmonic at
  8, 16 and 96 kHz within 0.5 dB of 44.1 kHz (measured −0.34, −0.15, +0.05), which
  fails on 2.3.1, and checks three noise sources against twins fed by hand.
- **Negative.** What remains is what no per-rate setting reaches. The non-band-limited
  glottal pulse aliases at low rates. A first-order radiation filter cannot follow a
  differentiator near Nyquist: it is 2 dB low at 2.5 kHz at 8 kHz. A resonance that
  straddles Nyquist cannot be matched exactly. The largest are a cat's purr at 8 kHz
  (−3.1 dB) and a raptor's howl at 11.025 kHz (+3.0). Every render at another rate
  moves, by design.
- **Neutral.** svara's bandwidths are narrower than the species' values at 44.1 kHz too:
  a wolf's F1 renders 6.0 Hz wide, not 100. This ADR keeps the 44.1 kHz render, the
  calibrated one, and makes every rate match it. Rendering the specified bandwidths
  would change the 44.1 kHz voice itself and ADR-0007's gain, and is not decided here.

## Alternatives considered

- **Leave it, and document the rate dependence.** A caller at 8 kHz got a voice 19 dB
  louder, with wider formants, than the one prani is calibrated to.
- **Fit the lip filter to 44.1 kHz's at two frequencies.** It halves the lip filter's own
  error (0.62 dB RMS at 8 kHz, from 1.19). Over the 861 renders, though, it made the
  whole worse: a mean level error of 0.38 dB against 0.25. The analog-equivalent rule is
  the one measured best.
- **Match α · sr rather than the noise power α/(1 + α) · sr/2.** Within 0.3 dB of it,
  0.27 dB mean level error against 0.25.
- **Resample: render at 44.1 kHz, then convert.** Exact by construction, but it is a
  resampler prani does not have, a cost at every rate, and a latency a stream cannot pay.
- **Change svara.** svara is right for what it is. The stages are svara's, and the
  calibration is prani's, as ADR-0007's was.
