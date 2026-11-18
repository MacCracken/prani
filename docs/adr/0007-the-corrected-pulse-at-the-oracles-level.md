# 0007 — svara's corrected pulse, at the oracle's level

**Status**: Accepted
**Date**: 2026-11-17

## Context

prani's gain stage was set against the oracle's glottal source. In svara 1.0.0, the
Rust prani's dependency, a Rosenberg pulse rose to a peak of 1.0, dropped straight
to 0, and was mixed with aspiration noise as `pulse*(1-b) + noise*b*0.3*gate`. Every
level downstream is relative to that unit-peak pulse: effort and intent amplitude,
the subharmonic and biphonation sines `voice.cyr` adds, and each species'
breathiness.

svara 3.11.0 ([its ADR-0014](https://github.com/MacCracken/svara/blob/3.14.0/docs/adr/0014-the-rosenberg-pulse-and-its-speed-quotient.md))
corrected the pulse to Rosenberg's full B, with a closing phase. It scaled the
pulse by `SV_G_ROSENBERG_GAIN` = 7.13 (+17.06 dB) so that a default male /a/ keeps
its RMS, and left the noise unscaled. The shape is right. The gain is right for
svara: it was solved where a human voice's energy sits, at harmonics 5 to 20.

A creature tract sits lower. Wolf F1 is 420 Hz at an f0 of 400, so its output is in
the first harmonics, and those carry the whole gain. Measured through 2.1.1's
code on svara 3.14.0, at 44.1 kHz for 0.1 s per species and vocalization:

- wolf was 14.4 dB louder at the median, up to 16.7, and dog, cat, crow and fantasy
  were 3 to 14 dB louder;
- a wolf howl+bark phrase peaked at 2.24, where 2.1.1 peaked at 0.40, and three
  more renders passed full scale;
- every species was 17 dB less breathy than its params say, because the pulse grew
  and the noise did not.

Lion, crocodilian and dragon held their level only because their subharmonic sine
dominates. The other renders that held never use the glottal source.

## Decision

**prani takes svara's corrected pulse shape into the oracle's gain stage, in
`src/tract.cyr`, and svara is unchanged.** It does this in two steps.

*Extended in 2.3.2 by [ADR-0014](0014-a-render-does-not-depend-on-the-sample-rate.md):*
the gain solved here is a 44.1 kHz one, and every other rate is now held to the
44.1 kHz render. The breathiness mapping below carries the noise's scale for the rate,
and is this one at 44.1 kHz.

*Re-solved in 2.3.3 by [ADR-0015](0015-a-formant-renders-the-width-it-is-specified.md),*
by this ADR's method against the same 56 renders and 2.1.1's levels, when the formants
began to render their specified widths: the gain is −3.78 dB, from +7.37, and the 56
spread from −4.9 to +7.9 dB around it.

1. **The oracle's excitation, exactly.** svara's mix is linear in its breathiness
   b′, with the pulse at peak g. So prani hands svara `b′ = g·b / (1 − b + g·b)` and
   scales each sample by `(1 − b + g·b) / g`. The tract then hears
   `p·(1 − b) + n·b·0.3·gate`: the oracle's mix, with p the unit-peak corrected
   pulse. g is read from svara (`SV_G_ROSENBERG_GAIN`, for the Rosenberg model
   prani always builds), not copied.
2. **The oracle's loudness.** A unit-peak full-B pulse is darker than the oracle's
   instant drop, so step 1 alone left prani 7.4 dB quieter at the median. Fantasy
   then fell under the oracle's own audibility bar (`max |s| > 0.001`). That is the
   bar svara hit, and prani answers it the way svara did: with one gain solved
   against the old level. `PR_TR_EXC_GAIN` = 2.336552212222851 (+7.37 dB) is the
   gain at which the median RMS change is 0 dB against 2.1.1. The set is every
   species × vocalization the species supports, through `crvoice_vocalize` at
   44.1 kHz for 0.1 s, keeping the 56 renders whose glottal part carries more than
   half the RMS.

## Consequences

- **Positive**: nothing passes full scale. The loudest render in the sweep peaks
  at 0.786, and 2.1.1's peaked at 0.810. Breathiness is the oracle's again, and the
  centre of the voiced repertoire is at 2.1.1's level. The two svara facts this
  relies on (the peak is g, and the mix is linear in b), the helper algebra and the
  calibration are all pinned. 2.1.1's code on the new svara fails the calibration
  group, and so does step 1 without step 2.
- **Negative**: one gain cannot restore every render. Around the median the 56
  spread from −6.9 to +7.1 dB, because the darker closure takes more from a tract
  whose energy sits higher. That is the timbre svara corrected, and undoing it
  would undo svara's fix. Renders at a low sample rate come out louder than at
  44.1 kHz: +5 to +7 dB at 8 kHz, where there is no band above 4 kHz for the closure
  to darken.
- **Negative**: prani now owns a solved constant and a method. If svara changes the
  pulse's shape, the gain must be re-solved, not nudged. The calibration group
  fails when that happens.
- **Neutral**: the solve reads one svara constant by name. A rename fails the
  build, and a recalibration of `SV_G_ROSENBERG_GAIN` flows through step 1.

## Alternatives considered

- **Take svara's level as it is.** This is prani 2.1.1's code on svara 3.14.0. It
  clips and is 17 dB under-breathy, and nothing in prani asked for either.
- **Ask svara to change its gain.** svara's gain is correct for svara's voices, and
  the level mismatch is in prani's gain stage, so the fix is prani's.
- **Pin svara below 3.11.0.** That keeps the old levels by keeping the wrong pulse.
- **Step 1 only, unit peak and no level gain.** This is the most literal reading of
  "the oracle's excitation". It fails the oracle's audibility bar for fantasy, and
  it is 7.4 dB under 2.1.1 at the median.
- **Scale svara's whole sample by 1/g.** This divides the noise down too, so it
  leaves prani 17 dB under-breathy.
- **A gain per species.** Thirteen solved constants would still leave the spread
  across vocalizations within a species, which is up to 7 dB for dog. That spread is
  the same timbre effect, so it buys little for thirteen times the upkeep.
