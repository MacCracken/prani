# 0009 — A render does not depend on how it is cut

**Status**: Accepted
**Date**: 2026-11-17

## Context

The [2026-11-17 sweep](../audit/2026-11-17-audit.md) found six defects in how prani
renders. Each one was faithful to the oracle, so parity had preserved them. CLAUDE.md
sets the port's bar as "matches what Rust did" and allows a divergence only with an
ADR. This is that ADR.

Five of the six share a shape. The audio depends on where a render is cut into pieces:
into vocalize's 20 ms blocks, into the host's callback buffers, or into a mix shorter
than its calls. The sixth renders something other than what the caller built. All six
were measured on the 2.2.0 tree.

1. **B1: the tract had no time base across calls.** The syringeal dual-source,
   stridulatory and vibratile arms computed time as `i / sample_rate` from the start of
   each call (`2.0.3:rust-old/src/tract.rs:185`, `:257-258`, `:286`).
   `crvoice_vocalize` renders in 20 ms blocks, so those oscillators restarted at t = 0
   every 20 ms. A cricket chirps for 133 ms of every 400, and the 267 ms gap could not
   happen through vocalize at any duration. In a 1 s cricket CHIRP at 44.1 kHz, vocalize
   gave **0** of 50 windows of 20 ms below a 1e-3 peak, against **29** for the same
   1 s rendered in one call. The oracle's own `test_cricket_pulse_train` passes on the
   per-syllable envelope, not on the gap its comment names.
2. **B2: an estimated formant above Nyquist discarded the species.** The tract estimates
   F4 and F5 as 1.3× and 1.5× F3 (`2.0.3:rust-old/src/tract.rs:59-64`, and `:357-363`
   for a blend). At a rate whose Nyquist lies below those estimates, svara refuses the
   whole target, and the tract keeps svara's default vowel with a warning. A dog at
   8 kHz (F5 = 4125 Hz) rendered with formants at 523/1588/2469 Hz instead of its own
   550/1650/2750.
3. **B3: the stream's audio depended on the host's buffer size.** `stream_fill_buffer`
   rendered one tract call per host fill (`2.0.3:rust-old/src/stream.rs:144-187`). Each
   fill got its own contour point, its own boundary boost, a fresh glottal source and a
   fresh tilt filter state. Draining the same 0.1 s HOWL at 8 kHz with fills of 32, 64,
   160, 256, 512 and 800 samples gave RMS spreads of **6.0 dB** (wolf), **6.1 dB**
   (crow), **8.7 dB** (raptor) and **20.4 dB** (lion).
4. **B4: a stream ignored the voice it was given.** The stream built its tract from the
   species table (`self.voice.species().params()`, `:154`), not from the voice. A
   voice's breathiness, jitter, shimmer and formant overrides never reached a stream,
   and neither did vocalize's effort breathiness delta. A breathier wolf streamed
   bit-identical to a default one.
5. **B10: the stream built its tract on the first fill** (`get_or_insert_with`, `:153`).
   A sample rate svara refuses was reported by the first fill as 0 samples, not by
   `stream_new`, and not at all for a stream too short to have a fill. The first
   callback also allocated the tract: **10,504 B** at 2.2.0, against 560 B for every
   later fill.
6. **C3: a chorus was shorter than its calls.** `sequence_synthesize_chorus` sized the
   mix from the unscaled `duration` (`2.0.3:rust-old/src/sequence.rs:110-112`). Each
   call is rendered at `duration × duration_scale` (`:119`), so MATING (×2) cut every
   call in half: **800** of its 1,600 samples at 8 kHz for 0.1 s.

None of these is a contract. No oracle document states it, no oracle test pins it, and
each makes the output depend on something the caller did not choose: a block size, a
callback size, or a sample rate's distance from a guessed formant.

## Decision

**prani repairs all six. A render depends on what was asked for, not on how it was
cut.**

1. **The tract keeps a sample clock.** `CreatureTract.clock` counts the samples the
   tract has rendered. The three arms take time as `(clock + i) / sample_rate` and
   advance the clock, so rendering in blocks is rendering in one call.
   `crtract_reset` zeroes it. It roundtrips through JSON as `"clock"`, a plain integer,
   because ADR-0005 restores prani's own state. A document written before 2.2.1 has no
   clock and decodes at 0, the time base such a tract had.
   `crtract_osc_ok(f0, end_clock, sr)` refuses a render whose largest oscillator phase
   would not be finite (audit B7).
2. **An estimate that does not fit goes into the band.** `crtract_in_band(est, below,
   nyquist)` keeps an estimated F4 or F5 below Nyquist, and moves one at or above it to
   midway between the formant below and Nyquist, so the order holds. F1–F3 are the
   species' own and are not moved. If they do not fit, svara still refuses the target
   and the tract keeps the oracle's fallback.
   *Extended in 2.3.2 by [ADR-0013](0013-a-formant-past-nyquist-is-placed-in-band.md)
   (audit D3):* F1–F3 are placed the same way, and every placed formant, an estimate
   included, carries the level its resonance has where it is placed.
3. **A stream renders vocalize's blocks.** The stream renders internal blocks of
   `sample_rate × 0.02` samples (vocalize's 20 ms), with the contour and boundary boost
   taken at each block's start and the tilt filter's state carried from block to block.
   It serves the host from those blocks. The output is bit-identical at every buffer
   size, and `tests/stream.tcyr` pins it against a reference render.
4. **A stream renders its voice.** The tract is built from the voice's own params plus
   vocalize's effort breathiness delta. The cat purr takes them unchanged, as
   `crvoice_vocalize`'s purr does.
5. **`stream_new` builds the tract and sizes the block.** A rate svara refuses is
   `PRANI_ERR_SVARA` from `stream_new`, the same code `crvoice_vocalize` gives. The
   first fill costs what every later fill costs.
6. **A chorus spans its calls.** The mix length uses `call_seconds = duration ×
   duration_scale`, the length each call is rendered at.

## Consequences

- **Positive**: measured over the 716,010-line sweep against 2.2.0. 141,097 of 709,301
  sample values moved, in 109 of 404 renders. No render changed its status or length,
  and none became non-finite.
  - At 44.1 kHz only four species move. Songbird is −0.00 to +0.07 dB and raptor
    −0.15 to +0.00 dB (B1, the dual source). Cricket is −1.31 to −1.45 dB, because
    its gaps are now silent and a 1 s CHIRP has its 29 quiet windows (B1). Bee
    changes in its low bits only. The other nine are bit-identical to 2.2.0.
  - At 8 kHz the dog moves −1.5 to +7.4 dB, now that it renders its own formants
    (B2). Songbird and raptor move −0.8 to +0.5 dB, and a half-size raptor chirp
    −1.6 dB. One cat howl moves +2.4 dB. The cat's own F3 (4200 Hz) is past
    Nyquist, so its tract keeps the oracle's fallback, but the howl's formant blend
    estimates F4/F5 the way B2 repairs. With only the blend's in-band step reverted,
    that howl is bit-identical to 2.2.0.
  - Streams at 8 kHz move −1.8 to +7.6 dB per species. They now render at vocalize's
    block structure, which 2.2.0 came within 0.5% of only at a 160-sample fill
    (B3).
  - The one JSON change is the tract's new `"clock"` field.
- **Positive**: `tests/layers.tcyr` (new) pins the RMS of 15 renders to a relative
  1e-9. On 2.2.0 it fails at exactly the four of them this ADR moves (songbird trill,
  cricket, and two streams), and it catches every one of the sweep's 14 mutants of
  the render paths. A level move from here on is a test failure, not a sweep finding.
- **Negative**: the audio changes. Anyone holding 2.2.0 output will not match it. prani
  has no consumers yet, which is why this is done now.
- **Negative**: a fill's CPU cost is now uneven. A fill renders a whole 20 ms block when
  it runs past the last one, and nothing otherwise. A 512-sample fill at 44.1 kHz renders
  one block or none: 560, 560, 0, 560, 0 B across a 2,205-sample drain. A host with
  small buffers sees most callbacks cost almost nothing and one in several cost a
  block. That is the price of output that does not depend on the callback size.
- **Negative**: `stream_new` does more and can fail in more ways. It builds the tract
  and allocates the block, which 2.2.0 deferred to the first fill, and each refusal is
  reported (ADR-0008).
- **Neutral**: the oracle's streaming and one-shot paths still differ in their post
  chain (subharmonics, biphonation, nasal anti-formant, AM, envelope), so a stream is
  still not sample-for-sample a vocalize. The two now share their block structure, and
  that is all they share.

## Alternatives considered

- **Keep parity, document the defects.** That keeps prani matching a Rust program
  nobody runs. The bar exists so the port is correct, and here the oracle was not.
- **B3: render exactly the host's slice, but with a continuous glottal source.** The
  output would still depend on the buffer, through the contour and boost taken at each
  fill's start. It would also need the cached glottal source that
  `docs/benchmarks.md` sets aside as a behaviour decision of its own.
- **B2: clamp the estimate just under Nyquist.** That stacks F4 and F5 on the same edge,
  which the order check refuses. The midpoint keeps the order and keeps the formant
  inside the band.
- **B2: drop F4 and F5 when they do not fit.** svara's target takes five formants, so
  this would need a target shape svara does not have.
- **C3: render each call at the unscaled duration.** That changes what MATING means: a
  call twice as long becomes a call of the same length. The intent's duration scale is
  the point of the intent.
