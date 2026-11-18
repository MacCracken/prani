# 0013 — A formant past Nyquist is placed in band, not lost

**Status**: Accepted
**Date**: 2026-11-17

## Context

Audit D3, from the 2.3 sweep ([`docs/audit/2026-11-17-2.3-audit.md`](../audit/2026-11-17-2.3-audit.md)).
svara refuses a five-formant target if any formant is not below Nyquist
(`svara_formant_validate_target`), and the tract then keeps what it had. At
construction that is svara's default vowel: schwa (523, 1588, 2469, 3300, 3750 Hz), or,
below a 7.5 kHz rate, where schwa is past Nyquist too, a single 500 Hz formant.

ADR-0009 placed prani's two estimates, F4 and F5 (1.3× and 1.5× F3), inside the band
(2.2.1, audit B2). A species whose own F2 or F3 is past Nyquist still lost all five
formants:

- **at construction**, a cat at 8 kHz (F3 4200 Hz), a crow, a raptor or a songbird at
  8 or 11.025 kHz, and a raptor or a songbird at 16 kHz rendered through schwa. A crow
  at 5 kHz, or a wolf at svara's 1201 Hz floor, rendered through the single 500 Hz
  formant. The Rust warned and rendered on (`2.0.3:rust-old/src/tract.rs`, `warn!`
  "species formants out of range"), and so did prani through 2.3.1;
- **per block**, `crvoice_vocalize` discarded `crtract_set_formant_blend`'s refusal, as
  the Rust did (`let _ =`, `2.0.3:rust-old/src/voice.rs:238,248`), and the block kept
  the last block's formants. A cat's meow at 8 kHz was refused in 152 of its 208
  blocks, so the transition moved piecemeal. The syringeal source-filter coupling was
  refused in every block for the crow, the songbird and the raptor below 22.05 kHz, so
  it never happened at all.

Both returned success. ADR-0001's rule is that prani reports what it cannot do rather
than rendering something else as success, and schwa is someone else's voice. A
band-limited voice, though, is not a failure: a cat at 8 kHz is a telephone-quality
cat, and its F1 and F2 are in band.

What a band-limited voice should be is measurable: the same call rendered at 44.1 kHz,
where every species' formants are in band, and cut to the lower rate's Nyquist. A
resonance past Nyquist leaves only its lower skirt in band, at the level its analog
response has there: b·f / √((F² − f²)² + (b·f)²) of its peak, for a formant at F Hz
of bandwidth b.

## Decision

**Every formant is kept in band, at the level its own resonance has where it is
placed.** `crtract_in_band` places a finite formant at or past Nyquist halfway between
the formant below it (0 for F1) and Nyquist, as ADR-0009 placed the estimates. F1 to
F3 are placed in order, then F4 and F5 above the placed F3. `crtract_skirt_amp` then
scales svara's amplitude for that slot by the resonance's gain at the placed frequency.
For a cat at 8 kHz, F3 (4200 Hz, 200 Hz wide) goes to 3250 Hz, at 0.046: its resonance's 0.091 there, times svara's 0.5 for the slot. A
formant in band keeps its place and svara's amplitude, so every render whose formants
were all in band, every species at 44.1 kHz, is unchanged. `crtract_new` and
`crtract_set_formant_blend` both place their five. svara copies a record's values and
does not check an amplitude, so the amplitudes are written into the tract's own record
and the bank rebuilt from it once: nothing is allocated.

**What svara still refuses is returned.** After placement svara can refuse only a
formant or bandwidth that is not a positive finite number (its bandwidth bound, 222 ×
the rate, is far above prani's 28 kHz maximum). A params record holds one of those
only through a raw setter, since `species_params_in_range` refuses it everywhere else.
So `crtract_new` returns the refusal, as any tract error, and `crvoice_vocalize`
returns a per-block one. A NaN or infinite formant is not placed: it is not a formant
past Nyquist, and placing it would invent one nobody asked for.

## Consequences

- **Positive.** A species keeps its own formants at any rate. `tests/tract.tcyr`'s D3
  group pins the placed frequencies and the cat's placed F3 level, all exact in both
  formats, and fails on 2.3.1 (19 of the 21 assertions it had then). The renders that
  were schwa now render the species. The one runtime WARN prani had is gone.
- **Negative.** A placement is still not the formant. Over 49 renders that place at
  least one formant (cat, crow, raptor, songbird, cricket and dog calls at 8, 11.025
  and 16 kHz), each scored by 1/3-octave band levels against the same call at 44.1 kHz
  cut to the lower Nyquist, the mean error is 17.9 dB. Most of that was not the
  formants: every low-rate render was louder than at 44.1 kHz, a dog at 8 kHz by 17.8 dB
  with only F5 placed, and with the crow's and the raptor's own formants two 8 kHz
  renders of the sweep peaked past full scale. That was audit V1, repaired in the same
  release by [ADR-0014](0014-a-render-does-not-depend-on-the-sample-rate.md).
- **Neutral.** A render through a raw-set NaN formant, which rendered schwa through
  2.3.1, is now refused with `PRANI_ERR_SVARA`. `prani_log_warn` has no caller left in
  prani, and stays as API. ADR-0009's placed F4/F5 now carry the skirt level too, so a
  dog at 8 kHz renders differently from 2.2.1.

## Alternatives considered

- **Keep the fallback.** The Rust's behaviour, and a render of another voice as success.
- **Refuse a formant past Nyquist**, as ADR-0011 refuses a pitch. A formant is a
  resonance of the filter, not a source: past Nyquist it is band-limited away, as a
  harmonic is, and refusing it would refuse a cat at 8 kHz.
- **Place it at full amplitude**, ADR-0009's rule as it stood. That piled up to four
  resonances just below Nyquist that the voice does not have there. Over the same 49
  renders it was the worst of the three rules here (18.6 dB mean error), and its sweep
  put seven renders past full scale, a crow's bark bout at 1.85, where 2.3.1's loudest
  render anywhere was 0.79. It was 2.3.2's first repair, and the sweep is what caught it.
- **Drop it (amplitude 0).** As accurate as the skirt level over the 49 renders (17.9
  dB). But a voice whose F1 is past Nyquist, a cricket at 8 kHz, would render silence
  as success, which is what ADR-0001 refuses.
- **Log the refusal and render on.** A log line is not a return value, and the block
  would still render someone else's formants.
