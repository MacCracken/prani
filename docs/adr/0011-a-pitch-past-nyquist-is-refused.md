# 0011 — A pitch the sample rate cannot represent is refused

**Status**: Accepted
**Date**: 2026-11-17

## Context

Found while measuring ADR-0009, and deferred from the 2.2.1 sweep as roadmap 2.3.x
(audit V4). A source whose fundamental is at or above Nyquist, sample_rate / 2, cannot
be rendered:

- **At exactly Nyquist** a sine samples as sin(π·n), zero at every sample: a cricket
  sized to sing at 4 kHz rendered silence through an 8 kHz tract. A half-size cricket
  sings at 8 kHz, the rate itself, whose alias is 0 Hz, and rendered 0.1 s of numerical
  silence, RMS 3.3e-14. (2.3.1 said that one sat at Nyquist; it sits at sr, and is
  past it just the same. Audit D4, 2.3.2.)
- **Past Nyquist** a tone aliases to another pitch. A cricket's 4500 Hz song at 8 kHz
  came out at 8000 − 4500 = 3500 Hz.

Both returned success. Neither svara nor the Rust refuses such a pitch:
`svara_glottal_new` checks f0 against its own [20, 2000] Hz, not against the sample
rate, so at rates below 4 kHz a laryngeal voice can pass Nyquist too. prani's own
oscillators (the syringeal dual source, the stridulatory and vibratile carriers, the
purr's cycle) never checked at all. The purr was bounded since 2.2.1 by the sample rate
itself, a full octave too high.

The bar is accuracy, and ADR-0001's rule is that prani reports what it cannot do rather
than rendering something else as success. A render at the wrong pitch, or silence, is
the case that rule exists for.

## Decision

**Every path refuses a pitch it cannot represent with `PRANI_ERR_INVALID_TRACT`.**
`crtract_pitch_ok(f0, sr)` is |f0| < sr / 2. Each path checks it once per call, on the
pitch it actually renders, after its own clamp:

- **laryngeal**: the clamped f0, which svara's glottal source takes;
- **syringeal**: the clamped f0 through svara up to 2 kHz, and above that the second of
  the two tones, 1.498 × f0, the higher;
- **stridulatory and vibratile**: the carrier's magnitude;
- **the purr**: its cycle rate, which tightens 2.2.1's bound from sr to sr / 2;
- **voice's own sources** (*added in 2.3.2, audit D1*): the canid howl's and whine's
  biphonation, a second voice at 1.78 × f0, and the subharmonic layer at 0.5 × f0.
  `crvoice_vocalize` adds each as a sine of its own, from the unclamped f0, after the
  render. 2.3.1 checked only the tract's pitch, so a dog with an f0 offset of 1900 Hz
  howling at 8 kHz rendered its 4984 Hz second voice at 3016 Hz, and a default wolf
  howl at 1201 Hz its 712 Hz one at 489 Hz, both as success. The check comes after the
  tract is built, so svara's own refusal of a rate still wins.

The rule is about the fundamental of each independently pitched source. Harmonics past
Nyquist alias in any non-band-limited synthesis, svara's glottal pulse included. They
are synthesis quality, not a request for a pitch that does not exist.

*Added in 2.3.2 (audit D5).* A partial a path writes as a sine of its own is a
different case: it can be left out exactly. The stridulatory and vibratile paths add
0.3 × sin(2π·2f0·t), and a cricket at 16 kHz, a rate this ADR says holds it, rendered
its 9 kHz partial at 7 kHz. Since 2.3.2 that partial is left out when 2f0 is past
Nyquist, and the carrier renders alone.

The error reaches every caller as any tract error does. `crvoice_vocalize` returns it,
since a pitch contour that crosses Nyquist mid-call fails the call. A stream retires on
the fill that meets it, with `stream_last_error` naming it (ADR-0003).

## Consequences

- **Positive.** A caller learns that its rate cannot hold the voice, rather than
  receiving silence or an alias as success. The rule is the same for every apparatus.
- **Negative.** Some species cannot sing at low rates. At 8 kHz all 12 of the
  songbird's vocalizations, 6 of the raptor's (its second syrinx source passes 4 kHz) and
  3 of the cricket's are refused. All 21 render at 16 kHz, and through 2.3.0 every one
  of them was an alias at 8 kHz. prani is now stricter than svara for a laryngeal voice
  below a 4 kHz rate.
- **Neutral.** `tests/allocrefused.tcyr`'s every-pair sweep renders those 21 at 16 kHz,
  and pins the count. `tests/hardening.tcyr`'s V4 group fails on 2.3.0 in all ten
  refusals, and passes in both formats. *Since 2.3.2* its D1 group pins the
  biphonation refusals, and its D2/D4 group pins where each check sits: a wolf tract at
  8 kHz asked for 5000 Hz renders, since it clamps to 1200, and so does a crow at 2000 Hz
  at 5 kHz, which svara renders as one source (audit D2). `tests/tract.tcyr`'s D5 group
  measures the 2f0 partial's absence by Goertzel. 2.3.1's mutants of both placements
  passed every suite.

## Alternatives considered

- **Keep rendering.** That is the Rust's and svara's behaviour, but it reports a render
  that is not the pitch asked for as success.
- **Clamp the pitch below Nyquist.** That renders a pitch nobody asked for: the same
  fabrication, quieter.
- **Refuse only prani's own oscillators, and leave laryngeal to svara.** A laryngeal
  voice past Nyquist aliases just the same, and one rule is easier to state and test.
- **Refuse any harmonic past Nyquist.** That would refuse most voices at ordinary rates,
  for a property every pulse-based source has.
