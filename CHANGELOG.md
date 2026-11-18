# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

> **Reading `rust-old/…` citations in the entries below.** The Rust oracle this
> project was ported from lived at `rust-old/` until **2.0.8** removed it. Entries
> written before then cite it by bare path, and those paths are left as written —
> a released changelog is a record of what was true at the time, not a document to
> retrofit. To resolve any of them, name the tag
> ([ADR-0004](docs/adr/0004-cite-the-oracle-by-tag.md)):
>
> ```sh
> git show 2.0.3:rust-old/src/voice.rs   # port era: 2.0.0 and later
> git show 1.1.0:src/voice.rs            # Rust era: the Rust WAS src/
> ```
>
> The path changed at 2.0.0 because the port created `rust-old/` by moving the
> Rust aside, so the era decides which form resolves.

## [2.3.3] - the specified formant widths

Patch release: every formant renders the width its species specifies, at every rate
([ADR-0015](docs/adr/0015-a-formant-renders-the-width-it-is-specified.md)). Found while
repairing audit V1 in 2.3.2, and older than the port.

No dependency moves: cyrius 6.7.18, svara 3.14.0, naad 2.3.5, hisab 3.5.0, goonj
2.3.2, sakshi 2.5.10. The consumer floor stays cyrius 6.7.2, for both formats.
**2,686 assertions / 22 suites, in each format.**

### Changed

- ⚠ **A formant renders the width it is specified.** svara's resonator computes
  α = sin ω₀ · sinh(πb/sr), the line its port carried from its own Rust 1.0.0
  (`src/formant.rs:253-254`), and that renders a b Hz bandwidth about b · sin(2πf/sr)
  wide. A wolf's 100 Hz F1 at 420 Hz was 6.0 Hz wide at 44.1 kHz, its 150 Hz F3 44 Hz,
  and 2.3.2 held every other rate to those widths. Against the analog resonance, that was
  9.4 dB RMS off near the peak and 5.1 dB low in in-band power. prani now hands svara
  b' = (sr/π) · asinh(πb / (sr · sin ω₀)), at which svara's α is πb/sr. Of three rules
  measured, that is the closest to the analog resonance: 0.45 dB near the peak and
  0.12 dB in power. The noise-only voice's naad Q renders its f/Q band the same way. A
  bandwidth svara refuses is still handed over unchanged, for svara to refuse.
- ⚠ **The excitation gain is re-solved** (ADR-0007's method, against the same 56 renders
  and 2.1.1's levels). The wider resonances made the median 10.82 dB louder. The gain is
  now −3.78 dB, from +7.37, and the median is 2.1.1's level again. Around it the 56
  spread from −4.9 to +7.9 dB, so the voice suite's bound on that spread is re-recorded
  at ±8 dB.
- **ADR-0014's bandwidth rule is superseded.** Its lip, pole and noise rules stand, and
  now hold each stage to a 44.1 kHz render whose formants are accurate. Over 861 renders
  at seven rates, the mean level error against 44.1 kHz is 0.225 dB (2.3.2: 0.253), the
  worst 2.6 (3.1), and the mean spectral error 1.10 dB (1.27).

### Fixed

- **The f32 build's nasal notch recursed in f32**, with poles 0.994 from the unit circle,
  which amplified its own rounding. A cat's howl, whose meow opens on it, was up to
  4.2e-6 of its peak off the f64 build over the nasal phase, and 1.28e-6 off in RMS once
  its formants widened, against 1.6e-7 at most for every other pinned layer. It now
  recurses in f64 and rounds each sample once: 1.43e-7. The f64 build is unchanged by it
  (ADR-0010, amended).

### Verified

- **The widths are measured, not assumed.** `tests/tract.tcyr` drives a steady sine
  through one svara resonance built as `crtract_new` builds it, and finds it within
  0.25 dB of −3 dB at the analog resonance's edges, at 44.1, 8, 16 and 96 kHz: −3.01 dB
  well below Nyquist, −2.91 and −3.14 dB for a songbird's F1 at 16 kHz. Handed svara's
  widths, as 2.3.2 did, a wolf's F1 is 24.5 dB down there. That mutant fails 9
  assertions, among them every width case.
- **The audio moved where the widths are rendered.** In the whole-surface sweep against
  tag 2.3.2, in each format:
  - 283 of 404 renders moved, 122 of the 127 at 44.1 kHz among them. At 44.1 kHz their
    RMS changed by −3.0 to +8.4 dB around a median of 0.00, 51 of them by less than
    1 dB. The five that did not move are the cricket's and the bee's, whose paths never
    reach svara's formants;
  - no render changed its status or length, none became non-finite, all 26 JSON
    documents are byte-identical, and no hostile case changed its status;
  - the loudest render peaks at 0.657 (2.3.2: 0.665).
- **The pinned levels moved, and are re-recorded** (`tests/layers.tcyr`): 13 of 15, by
  −0.10 to +3.91 dB. The songbird's trill moved most, then the two streams and the
  cat's howl. The cricket and the bee did not move.
- **Gates.**
  - All 22 suites pass in each format (2,686 assertions each), and so do the fuzz
    harness and the five examples.
  - `cyrius audit` exits 0, `scripts/check-alloc.sh` and `gen-f64.sh --check` pass,
    every file is clean under `cyrfmt` and lint, and `cyrius deps --verify` is 50/50.
  - The consumer floor is still cyrius 6.7.2 in both formats: the from-scratch consumer
    builds there warning-free, and its output is byte-identical to 6.7.18's.

### Docs

- ADR-0015, and amendments to ADR-0007, ADR-0010 and ADR-0014 (status: its bandwidth
  rule superseded).

## [2.3.2] - the 2.3 sweep

Patch release: an independent review of the 2.3.0 and 2.3.1 work, and the repair of
everything it found ([audit](docs/audit/2026-11-17-2.3-audit.md)). Four reviews
confirmed 25 findings, 4 HIGH, 6 MEDIUM and 15 LOW, and six constant mutants that no
test caught in either format. All are repaired. Each repair has a regression test, run
against tag 2.3.1 in both formats, that fails there. Verifying them found one more,
older than 2.3: a render's level depended on the sample rate, by 19 dB at 8 kHz. That is
repaired too ([ADR-0014](docs/adr/0014-a-render-does-not-depend-on-the-sample-rate.md)).

No dependency moves: cyrius 6.7.18, svara 3.14.0, naad 2.3.5, hisab 3.5.0, goonj
2.3.2, sakshi 2.5.10. The consumer floor stays cyrius 6.7.2, for both formats.
**2,681 assertions / 22 suites, in each format.**

### Changed

- ⚠ **A render's level does not depend on the sample rate**
  ([ADR-0014](docs/adr/0014-a-render-does-not-depend-on-the-sample-rate.md), audit V1).
  prani's levels are calibrated at 44.1 kHz (ADR-0007), and a render at another rate is
  now that render cut to its band. Through 2.3.1 the median render was +18.8 dB at 8 kHz,
  +11.1 at 16 and −8.7 at 96, against the same call at 44.1 kHz cut to the lower Nyquist
  (127 renders a rate, by 1/3-octave band). Four stages had a response fixed per sample:
  - svara's lip radiation, `y = x − 0.97·x[n−1]` at every rate: its zero now keeps its
    place in Hz, and its gain is restored in svara's output gain and interaction strength;
  - svara's formant bandwidths, which render b · sin(2πf/sr) wide, not b, so a wolf's F1
    was 6.0 Hz wide at 44.1 kHz and 32 at 8: each is now handed svara at the width that
    passes the noise power in Hz it does at 44.1 kHz, as is the snake's naad bandpass;
  - the spectral tilt and the DC blocker, whose corners fell with the rate: each pole is
    now the 44.1 kHz one moved to the rate. The tilt reads its sample rate, which the
    Rust left unused, and refuses one that is not positive and finite when it applies;
  - every white-noise source, svara's aspiration included, now scaled by √(sr/44100)
    to keep its power density.

  Over 861 renders at seven rates, the mean level error is 0.25 dB (worst 3.1), from 7.2
  (worst 26.2), and the median at every rate is within ±0.3 dB. Every 44.1 kHz render is
  bit-identical; every render at another rate moves.
- ⚠ **A formant past Nyquist is placed in band, not lost**
  ([ADR-0013](docs/adr/0013-a-formant-past-nyquist-is-placed-in-band.md), audit D3).
  One species formant at or past Nyquist made svara refuse all five, and the tract
  rendered svara's default vowel as the species: schwa for a cat at 8 kHz, for a crow,
  a raptor or a songbird at 8 or 11.025 kHz, and for a raptor or a songbird at 16 kHz,
  and a single 500 Hz formant below 7.5 kHz. Per block, vocalize discarded the refusal,
  as the Rust did, so a cat's meow at 8 kHz kept its last accepted formants in 152 of
  208 blocks, and the syringeal coupling never ran below 22.05 kHz. Each formant is now
  placed as ADR-0009 places the F4/F5 estimates, halfway from the one below it to
  Nyquist, at the level its own resonance has there. A cat's F3 at 8 kHz, 4200 Hz, goes
  to 3250 Hz with amplitude 0.046, not svara's 0.5 for that slot. ADR-0009's estimates
  carry that level too, so a dog at 8 kHz renders differently from 2.2.1. A formant
  svara still refuses, which only a raw setter can produce, is returned as
  `PRANI_ERR_SVARA` instead of rendered, so prani's one runtime WARN is gone.
- ⚠ **Voice's own sources are held to Nyquist too**
  ([ADR-0011](docs/adr/0011-a-pitch-past-nyquist-is-refused.md), audit D1). A canid
  howl or whine adds a second voice at 1.78 × f0, and lions, crocodilians and dragons
  a subharmonic at 0.5 × f0, after the render and from the unclamped f0. 2.3.1 checked
  only the tract's pitch. A dog with an f0 offset of 1900 Hz howling at 8 kHz rendered
  its 4984 Hz second voice at 3016 Hz, and a default wolf howl at 1201 Hz its 712 Hz one
  at 489 Hz, both as success. Both are now `PRANI_ERR_INVALID_TRACT`. A call of no
  samples renders neither source, and is still an empty buffer.
- ⚠ **The 2·f0 partial is left out past Nyquist** (ADR-0011, audit D5). The
  stridulatory and vibratile paths add 0.3 × sin(2π·2f0·t). A cricket at 16 kHz, a rate
  2.3.1 said holds it, rendered its 9 kHz partial at 7 kHz. The carrier now renders
  alone when 2f0 is past Nyquist.
- **An emotion update never passes its target** (audit B5). `v·s + t·(1 − s)` could be
  rounded past t once v had all but reached it, so a target set on a zone bound read
  the zone beyond. Over 76 paths toward the bounds, 2.3.1 misread 5 in f64 (1,364
  steps) and 9 in f32 (2,588). The blend is held between v and t, which only moves it
  toward the exact value, and the f32 build blends in f64. The f64 build's bits change
  only where the old result passed the target.

### Fixed

- **f32 accuracy** (ADR-0010, amended).
  - **HIGH: Doppler resampling** (audit B1). The read position `i × ratio` was f32, so
    a 10 s buffer was 9.2e-3 off and past 190 s it was nearest-neighbour. It is f64,
    with the output length.
  - **HIGH: a process exit** (audit B2). The nasal anti-formant bounded its count with
    `pr_from(len)`, which in f32 rounds 16,777,219 up, and a fraction of 1.0 on such a
    buffer read past it and exited. The count is bounded in integers.
  - **MEDIUM: documents at a builder's maximum** (audit B3). The f32 0.05 and 0.1 round
    above the decimals, so an f32 voice at the jitter or shimmer ceiling was refused by
    the f64 build. The builders' ceilings, and `crvoice_with_f0_offset`'s range, round
    toward zero.
  - **Phase accumulators** (audit B4). The purr and the laryngeal subharmonic add in
    f64 within a call, where f32 re-rounded each increment: a purr drifted 0.27 cycles
    in 10 minutes.
  - **The random draw** (audit C3). 2.3.1 truncated it to 24 bits and called each draw
    within an ulp of the f64 build's; 25% were not, by up to 499,712 ulps. It is now the
    f64 draw correctly rounded, with the top 64 values clamped to 1 − 2^-24.
- **Gates that could pass a wrong build.**
  - **`gen-f64.sh`** (audit A1–A4). A comment after `#else` or `#endif` silently
    dropped f64 code, and every gate passed. The generator now recognises a directive by
    its first word and fails on an unbalanced one, leaves strings alone, checks that
    `[lib].modules` pairs each source with its twin (two missing twins passed every
    suite and example), and lints the f32-compiled code for f64 arithmetic.
  - **`check-alloc.sh`** (audit A5) counted its funnel's helper lines, not calls, so a
    raw `alloc` on a helper's line passed.
- **Tests that did not test what they said.**
  - **HIGH: four of ADR-0010's six time-base arms were untested** (audit C1): the
    syringeal dual source, and voice's trill AM, subharmonic and biphonation, each of
    which could be written in f32 time with every suite passing. The syringeal arm has
    a 1000 s case, and voice's three share `crvoice_tone`, held to the true sine.
  - **Per-format inputs** (audit C2, C6). In f32, `pr_of64` of an f64 subnormal is 0
    and of f64::MAX is inf, so six assertions tested another branch; spatial's
    reciprocal guard could be deleted from the f32 build unnoticed. Two overflow inputs
    in `tests/hardening.tcyr` were one value in f32, and one of them did not overflow
    in f64.
  - **`assert_real`** (audit C5) measured an f32-build word with high bits set as the
    real in its low half, and passed equal NaNs in f64 only.
  - **`pr_neg(1)` as −1** (audit A6), and **f32 reals passed to naad** (A7).
  - **Six constants no mutant of which failed a test** (audit C, K2–K16): the purr's
    noise scale, the contour's epsilon, the fatigue duration bound, the notch's pole
    ceiling, the arousal Low/Mid bound and the jitter bound. Each is now pinned on both
    sides, in both formats.
- **Docs.** CONTRIBUTING recommended `!pr_ge`, which cycc 6.7.2, the consumer floor,
  miscompiles (audit C4). ADR-0011 put a half-size cricket at Nyquist; it sings at the
  rate itself (D4). An example printed "f64 samples" in the f32 build (C7). `pr_from`'s
  comment claimed one rounding above 2^53 (A8).

### Verified

- **Every defect's regression group fails on 2.3.1.** Each was extracted with what it
  needs and run on a snapshot of the tag, in both formats:
  - in both: D3 (19 of 21 assertions; the two controls pass), D5 (3 of 7), D1 (2 of 5),
    B5 (2 of 3);
  - in f32, the f32 build's defects, each passing in f64: B1 (1 of 2), B3 (3 of 5), B4
    (2 of 4), C3, and B2, which exits the process ("vec: index out of bounds").
- **Mutants.** The groups that pin what 2.3.1 already did right (D2's two placements, and
  the six constants) pass on 2.3.1, and each fails on its mutant. Every mutant the
  reviews found surviving now fails a suite, each planted in a fresh copy with the
  twins regenerated: K2, K4 (epsilon halved and doubled), K6, K7, K10 and K16 in each
  format, at the reviewers' values and at a second value each; T1 (the syringeal arm in
  f32 time), voice's time-base arm in f32 time (12 of 15 samples miss), and G1
  (spatial's reciprocal guard deleted), in f32; D2a and D2b. So do the 2.3.2 repairs'
  own: the emotion hold removed (1,364 misread steps), its f32 blend in f32 (the 0.92
  pin, 1 ulp), the formant placement without its finiteness check (3 assertions), and
  the placed formants at full amplitude (4) or with the bank left unrebuilt (2).
- **The audio moved only where the repairs put it.** In the whole-surface sweep against
  tag 2.3.1, in each format:
  - in f64, every render at a rate other than 44.1 kHz moved (161 of 404: the intents,
    bouts, streams, builders, phrase, chorus and presets, all at 8 kHz), by ADR-0014,
    and by D3 where a formant is placed. None of the 127 renders at 44.1 kHz moved, nor
    the purr there;
  - in f32, 225 moved: those, and at 44.1 kHz the draws now correctly rounded (C3) and
    the purr's phase (B4);
  - no render changed its status or length, none became non-finite, and all 26 JSON
    documents are byte-identical;
  - 1 of 953 hostile cases changed status: a wolf howl of 0.01 s at 1201 Hz is refused
    (D1). A zero-length one is still an empty buffer. 62 changed their samples, every
    one at a rate other than 44.1 kHz;
  - the loudest render now peaks at 0.665, where 2.3.1's peaked at 0.786. None is past
    full scale.

  Against the f64 build, the f32 build's purr is now 1.0e-5 of peak off, where 2.3.1's
  was 0.21 (B4), and vocalize's 127 renders at most 8.8e-5, where they were 1.2e-4.
- **Gates.**
  - All 22 suites pass in each format (2,681 assertions each), and so do the fuzz
    harness and the five examples.
  - `cyrius audit` exits 0, `scripts/check-alloc.sh` and `gen-f64.sh --check` pass,
    every file is clean under `cyrfmt` and lint, and `cyrius deps --verify` is 50/50.
  - The consumer floor is still cyrius 6.7.2 in both formats: a from-scratch consumer
    that reaches every 2.3.2 path builds there warning-free, and its output is
    byte-identical to 6.7.18's.


### Docs

- ADR-0013 and ADR-0014, and amendments to ADR-0007, ADR-0009, ADR-0010 and ADR-0011,
  marked with the release.
- The 2.3 audit report.
- `input-ranges.md`: the biphonation guard, formant placement, the builders' ceilings,
  the tilt's sample rate, renders at any rate.
- CONTRIBUTING: negated compares.

## [2.3.1] - the deferred decisions

Patch release: the three items the 2.2.1 sweep left on roadmap 2.3.x, each settled by
ADR-0010's bar of accuracy over the Rust's numbers. Two change what a render returns,
and one is CI's.

No dependency moves: cyrius 6.7.18, svara 3.14.0, naad 2.3.5, hisab 3.5.0, goonj
2.3.2, sakshi 2.5.10. The consumer floor stays cyrius 6.7.2, for both formats.
**2,546 assertions / 22 suites, in each format.**

### Changed

- ⚠ **A pitch the sample rate cannot represent is refused**
  ([ADR-0011](docs/adr/0011-a-pitch-past-nyquist-is-refused.md), audit V4), with
  `PRANI_ERR_INVALID_TRACT`. Through 2.3.0 a source at or past Nyquist rendered as
  success: silence at exactly sample_rate / 2 (a half-size cricket at 8 kHz, RMS
  3.3e-14), and an alias past it (a cricket's 4500 Hz at 8 kHz came out at 3500 Hz).
  Each path checks the pitch it renders, after its own clamp:
  - the laryngeal f0;
  - the syringeal f0 up to 2 kHz, and the second source, 1.498 × f0, above it;
  - the stridulatory and vibratile carriers;
  - the purr's cycle rate, which 2.2.1 bounded only by the sample rate itself.

  At 8 kHz that refuses all 12 of the songbird's vocalizations, 6 of the raptor's and 3
  of the cricket's. Every one of them was an alias there through 2.3.0, and all 21
  render at 16 kHz. A stream retires on the fill that meets it and names the cause.
- ⚠ **A steady wind does not shift the pitch**
  ([ADR-0012](docs/adr/0012-a-steady-wind-does-not-shift-the-pitch.md), audit C10).
  `bridge_doppler_velocity_from_wind` returns 0 for every finite input, the radial
  velocity a steady wind adds to the Doppler shift between a source and a listener at
  rest: f_r / f_s = (c + w·n̂) / (c + w·n̂) = 1. Through 2.3.0 it returned the Rust's
  `wind × cos(angle)`, which spatial reads as a receding source. A 10 m/s tailwind
  turned 1000 samples into 1029 and a headwind into 970. A non-finite input returns
  NaN.

### Fixed

- **CI and the release workflow fetch cyrius's installer at the pinned tag**, not from
  `main`. The tag is read from `cyrius.cyml`, as the version already was. An unchanged
  prani commit now always installs the same way. Exercised for 2.3.1, which review D
  could not do: the tag's installer installs cyrius 6.7.18 with its checksum and Ed25519
  signature verified.

### Verified

- **Every new regression assertion fails on 2.3.0.** The V4 group's ten refusals all
  render as success there, its seven controls pass, and so do the wind checks (1029 and
  970 samples). Both groups pass in both formats.
- **The audio moved only where ADR-0011 puts it**, in the whole-surface sweep against
  tag 2.3.0, in each format:
  - 20 of 404 renders changed from success to `PRANI_ERR_INVALID_TRACT`, every one a
    pitch past Nyquist: the songbird and the raptor at 8 kHz, through intents, bouts and
    the builders, and the half-size cricket;
  - the songbird's and the raptor's streams at 8 kHz retire on their first fill, so
    those 8 stream buffers are the only samples that moved (1,728 of 690,429);
  - 3 of 953 hostile cases changed, all refusals: the songbird, the crow and the raptor
    howling through a 1201 Hz tract, whose Nyquist is 600 Hz;
  - no other sample or length moved, and all 26 JSON documents are byte-identical.
- **Gates.**
  - All 22 suites pass in each format (2,546 assertions each), and so do the
    fuzz harness and the five examples.
  - `cyrius audit` exits 0, and `scripts/check-alloc.sh` and `gen-f64.sh --check`
    pass. Every file is clean under `cyrfmt` and lint, and `cyrius deps --verify` is
    50/50.

### Docs

- ADR-0011 and ADR-0012, and the ADR index.
- `input-ranges.md`: the pitch guards.
- CONTRIBUTING: the pinned installer.
- Roadmap: the 2.3.x lane is done and removed.

## [2.3.0] - two float formats

Minor release: prani builds in either float format, **f64 or f32**
([ADR-0010](docs/adr/0010-two-float-formats.md)). Roadmap 2.3.0 was "f32
throughout". It lands as a second format beside f64, not a replacement, and the f32
build is held to **accuracy against the f64 build**. The Rust defined what prani does,
not its numbers.

No dependency moves: cyrius 6.7.18, svara 3.14.0, naad 2.3.5, hisab 3.5.0, goonj
2.3.2, sakshi 2.5.10. The consumer floor stays **cyrius 6.7.2, for both formats**.
**2,520 assertions / 22 suites, in each format.**

**The f64 build is 2.2.1, bit for bit.** The whole-surface sweep (716,010 lines: 404
renders, 26 JSON documents, 953 hostile cases) is byte-identical to tag 2.2.1, and its
benchmarks are 2.2.1's: a median of medians of 1.001× over six alternating rounds,
every synthesis benchmark within 0.992–1.019×.

### Added

- ⚠ **The f32 build.** Define `PRANI_F32` (`defines = ["PRANI_F32"]` in a consumer's
  `[build]`, or `-D PRANI_F32`), and every prani value is an f32. That covers samples,
  sample rates, durations, parameters and every field. An f32 is its binary32 bits in
  the low 32 bits of the word. One bundle serves both formats.
- **`src/real.cyr`, the format layer.** Its primitives (`pr_add` … `pr_ge`, `pr_clamp`,
  `pr_sin`, `pr_from`, `pr_to`, `pr_to64` / `pr_of64`) and constants (`PR_ONE`
  … `PR_TAU`) are single f32 operations in the f32 build. A consumer that writes its
  own arithmetic over them builds in either format, as the five examples now do.
- **`src/f64/`, the f64 build, generated by `scripts/gen-f64.sh`.** cycc's `#inline`
  copies each argument into a slot of its own, and through the primitives the f64
  build measured 1.09× 2.2.1 (`emotion_evaluate` 1.30×). The twins spell each
  primitive as its f64 builtin, which is 2.2.1's code. `gen-f64.sh --check` fails CI on
  a stale twin.
- **`tests/real.tcyr`, a 22nd suite**: the format layer, the primitives' NaN, signed
  zero and saturation behaviour in both formats, and document portability.
  **`tests/realt.cyr`**, its helpers: `assert_real` is exact in the f64 build and
  within a measured ulp budget in the f32 build, and `t_per` states a value each
  format decides for itself.

### Changed

- **Where f32 has a choice, it takes the accurate one** (ADR-0010):
  - **Transcendentals are computed in f64 and rounded once.**
  - **An oscillator's time and phase stay f64** in the f32 build. These are the
    tract's syringeal, stridulatory and vibratile arms, and voice's trill AM,
    subharmonic and biphonation. In f32 the sample index is inexact past 2^24, and
    the phase w·t loses precision long before that: a 3 kHz tone 10 s in has phase
    noise at −36 dB. A cricket and a bee 1000 s into their clock render what they
    did at 0, and with f32 time all three cases fail (`tests/tract.tcyr`).
  - **A random draw stays in [0, 1).** Rounding a 31-bit draw to f32, as the Rust
    did, returns exactly 1.0 once in 2^25 draws, or about 13 minutes of a 44.1 kHz
    noise source. The f32 build keeps the 24 high bits (`tests/rng.tcyr` solves for
    the top state).
- **A document carries f64 bit patterns in both builds.** The f32 build writes its
  values widened, which is exact, and reads by rounding once, so either build reads
  the other's documents and 2.2.x's. The nine derive-serialized types that hold
  reals do this in the f32 build through a 64-bit shadow struct, under their own
  function names.
- **The tests run in both formats.** Each expected real is the f64 value it should
  be: exact in the f64 build, and within the measured error of that computation in
  the f32 build, recorded beside it. Boundary inputs ("one ulp past 1.0", the next
  count past a limit) are the build's own neighbours.
- The examples are written for either format, and `scripts/run-examples.sh` takes
  `PRANI_FORMAT=f64|f32`.

### Verified

- **The f32 build's accuracy against the f64 build**, from the sweep:
  - no render changes its status, and none is non-finite;
  - vocalize's 127 renders have a median relative sample error of 9.7e-7, a
    largest of 1.2e-4 of peak, and RMS unchanged to 0.0000 dB;
  - the 15 pinned layer levels are within a relative 6.4e-7;
  - a single computation is 0 to 1 f32 ulp from the f64 value, rounded;
  - lengths can differ, because each format truncates its own product: 21 of 404
    renders by 1 to 3 samples. For 0.1 s × 0.7 × 8 kHz, f64 gives 559 and f32
    gives the exact 560.
- **Speed.** The f32 build is 1.085× the f64 build today (`emotion_evaluate` 1.30×,
  the cricket 1.16×, the bee 1.14×), over six alternating rounds. That is the profile
  the primitives cost the f64 build before the twins (1.087×, 1.30×, 1.18×, 1.14×), so
  the cost is cycc's `#inline` argument copy, not f32 arithmetic. Against the Rust,
  f32 against f32 in one sitting, the f32 build is a median 10.2× and the f64 build
  9.3×. Float width does not explain the gap (`docs/benchmarks.md`).
- **Consumer floor.** A from-scratch consumer builds and runs on cyrius 6.7.2 in both
  formats, and its output (emotion, bridge, rng, a howl and a cricket render) is
  byte-identical to 6.7.18's. That was not true until verification. The first form
  wrote four range checks as `!pr_ge(..)`, and cycc 6.7.2 miscompiles `!` on a float
  builtin's result. A fresh emotion read as out of range, and the smoke render failed,
  so the floor had silently moved to 6.7.3. They are `== 0` again, as in 2.2.1.
- **Gates.**
  - All 22 suites pass in each format (2,520 assertions each), and so do the fuzz
    harness and the five examples.
  - `cyrius audit` exits 0, and `scripts/check-alloc.sh` and `gen-f64.sh --check`
    pass. Every file is clean under `cyrfmt` and lint.
  - aarch64 under qemu passes all 22 suites in both formats, and
    `cyrius deps --verify` is 50/50.

### Docs

- [ADR-0010](docs/adr/0010-two-float-formats.md).
- CLAUDE.md: the Rust defines functionality, and accuracy is the bar for numbers.
- CONTRIBUTING (the two-format workflow and gates), README, `port-audit.md`,
  `input-ranges.md` (f32's range), the architecture overview, the examples' README,
  and `benchmarks.md` (both formats, and the f32-vs-f32 comparison with the Rust).
- Roadmap: the 2.3.0 lane is done and removed. A watch item records when the
  generated twins could go.

## [2.2.1] - the second P(-1) sweep

Patch release: a whole-tree audit, refactor, hardening and optimization sweep of
2.2.0, and its repairs ([`docs/audit/2026-11-17-audit.md`](docs/audit/2026-11-17-audit.md)).
Four independent reviews confirmed 53 findings (6 HIGH, 18 MEDIUM, 29 LOW), and
verifying them turned up 4 more. **52 of the 57 are repaired**, each with a
regression test that fails on 2.2.0. Six of the repaired defects were the oracle's
own, and [ADR-0009](docs/adr/0009-renders-do-not-depend-on-how-they-are-cut.md)
records the divergence.

No dependency moves: cyrius 6.7.18, svara 3.14.0, naad 2.3.5, hisab 3.5.0, goonj
2.3.2, sakshi 2.5.10. The consumer floor stays cyrius 6.7.2.
**2,477 assertions / 21 suites.**

**The audio changes where ADR-0009 says it does, and nowhere else.** At 44.1 kHz,
9 of the 13 species render bit-identical to 2.2.0. Streams and the dog at 8 kHz
move the most. No render changed its status or length, and none became non-finite.

### Changed

- ⚠ **A render no longer depends on how it is cut** (ADR-0009). Each of these was
  faithful to `2.0.3:rust-old/`:
  - **The tract keeps a sample clock** (B1). The syringeal dual source, the
    stridulatory and the vibratile arms restarted time at every call, so through
    vocalize's 20 ms blocks a cricket never fell silent between chirps. A 1 s
    cricket chirp had 0 of 50 quiet windows; it now has 29, the same as one call.
    `crtract_reset` zeroes the clock, and the tract's JSON carries it as `"clock"`.
    A document without one, as 2.2.0 wrote, decodes at 0.
  - **An estimated F4/F5 above Nyquist is placed in band** (B2). Through 2.2.0, svara
    refused the whole target and the species was replaced by svara's default
    vowel: a dog at 8 kHz rendered at 523/1588/2469 Hz instead of its own
    550/1650/2750.
  - **A stream renders vocalize's 20 ms blocks** (B3), with the tilt filter's state
    carried across them. Through 2.2.0 it rendered one block per host fill, and its
    level depended on the host's buffer size: an RMS spread of 6.0 dB (wolf) to
    20.4 dB (lion) across fills of 32–800 samples. It is now bit-identical at every
    buffer size.
  - **A stream renders the voice it was given** (B4): the voice's own params plus
    vocalize's effort breathiness delta, not the species table.
  - **`stream_new` builds the tract** (B10). A sample rate svara refuses is
    `PRANI_ERR_SVARA` from `stream_new`, not a first fill that returns 0. The first
    fill costs what every later one does: 10,504 B at 2.2.0, at most one
    `svara_glottal_new` now.
  - **A chorus spans its calls** (C3). MATING doubles a call, and the mix used to cut
    each one to half: 800 of 1,600 samples.
- ⚠ **The four hand-written decoders require every field, present and an integer**
  (A2, B5, C4, C5): `crvoice_from_json_str`, its nested params,
  `crtract_from_json_str` and `sequence_call_phrase_from_json_str`. Through 2.2.0 an
  absent or mistyped field read as 0. A voice without `"species"` decoded as a Wolf,
  `"vocal_effort":0.9` as an effort of 0.0, `{}` as an empty phrase, and a tract
  without its rng pair as one whose noise was exact silence.
  - `spectral_tilt` stays optional, as the oracle's `#[serde(default)]` made it.
  - The tract decoder refuses an even `rng_inc`, which `prani_rng_new` never makes.
  - The voice decoder refuses a `params.apparatus` that is not its species' (V1). A
    "wolf" whose params said vibratile used to pass the HOWL gate and render 4,410
    samples of wing vibration as a successful howl.
- ⚠ **The point-of-use guards range the voice's own `vocal_effort` (in [0, 1]) and
  `size_scale` (> 0)** (A1, A5), in vocalize and the cat purr. A raw-set effort of 2.0
  or a NaN size used to render as success. `crvoice_set_vocal_effort` and
  `crvoice_apply_lombard_effect` ignore a NaN.
- ⚠ **More inputs are refused** (B6, B7, B11, C6, C9):
  - a `purr_f0` outside (0, sample_rate);
  - an oscillator whose phase would overflow;
  - a species range whose clamped `lo` exceeds `hi`;
  - a fatigue state outside [0, 1] in `fatigue_modifiers`;
  - a NaN in three bridge functions (`intent_from_threat_level`, `species_from_f0`,
    `perturbation_from_urgency`).
- **`preset_make` stores its fields clamped**, as the builders clamp, so a preset's own
  JSON always reads back (A4). `preset_to_json` escapes the name, and a null name
  is refused (A3). The escaping is prani's own, built on `str_builder_putc`, and
  writes the stdlib's `str_builder_add_json_cstr` bytes, held to them for every
  byte 1–255. The stdlib function is newer than cyrius 6.7.2, prani's consumer
  floor.
- **The bout ceiling counts `count − 1` gaps**, so a valid bout at the limit is no
  longer refused (C8).
- **Refactors.** `prani_vec_scale` replaces three copies of a scaling loop (A10).
  `crvoice_params_in_range` delegates to `species_params_in_range`, which it had
  drifted from (A8).

### Fixed

- **Faults reachable through public functions.** Every function that takes a handle
  now answers a null or error handle without reading it (A6, B9, C1, C2). A void
  mutator does nothing, and every other function returns the code, or
  `prani_handle_code`'s category code for null. Through 2.2.0 each of these was a
  SIGSEGV:
  - the FFI's three RTPC setters given an error handle;
  - an error element in a phrase;
  - every emotion and fatigue function but `emotion_evaluate`;
  - the voice accessors and `to_json` functions;
  - `preset_make(0, …)` followed by `preset_to_json`;
  - a null message to any `prani_log_*`.
- **The spatial functions pass an incoming error through** rather than rewriting
  it (C7).
- **A refused tilt retires the stream** with `stream_last_error` set, as ADR-0003
  says (B8). Its flag used to be a dead store.

### Added

- **`emotion_evaluate_into`, `fatigue_modifiers_into` and
  `bridge_perturbation_from_urgency_into`**, which write into a struct the caller
  owns. They allocate nothing; the allocating forms keep 40, 32 and 16 B per call,
  for life (C11).
- **`tests/layers.tcyr`**, a 20th suite. It pins the RMS of 15 renders to a
  relative 1e-9: every layer past the tract, which no test pinned (D2). It catches
  each of 14 single-edit mutants of those layers, and on 2.2.0 it fails at exactly
  the four renders ADR-0009 moves.
- **`tests/logging.tcyr`**, a 21st suite. It counts each wrapper's delivered
  events through sakshi's ring, including the two wrappers prani never calls (D16).
- **A real fuzz harness**, `tests/prani.fcyr` (D17). It feeds 1,500 mutated
  documents to all four decoders: byte overwrites, truncations, spans, structural
  characters and JSON escapes. Each answer must be a handle or a prani code. An
  accepted document must re-encode, decode again and encode to the same bytes, and
  a voice or tract it accepts must render. On 2.2.0 it finds A3 at round 786.
- **`scripts/bench-check.sh --write`** records `benches/baseline.csv` as each
  benchmark's fastest of three runs, with the host, the toolchain and the tree in its
  header. A compare warns when the baseline's toolchain is not the one running (D5).
- [ADR-0009](docs/adr/0009-renders-do-not-depend-on-how-they-are-cut.md) and the
  [audit report](docs/audit/2026-11-17-audit.md).

### Gates

- **`check-alloc.sh` catches what it claims** (D4). Through 2.2.0 it matched only
  `name(` on one line; `alloc (n)`, a call split across lines, and `&alloc` compiled
  and passed. It now scans `src/` recursively.
- **`run-examples.sh` requires the five examples, and checks each with `cyrfmt` and
  lint** (D11, D15). It used to pass with one.
- **`bench-check.sh` prints a failing harness's output** and cleans up after itself
  (D12).
- **O2 is a real budget** (D3): the marginal block cost must equal one
  `svara_glottal_new`, not fit under 1,000,000 bytes.
- **`t_any_nonzero` compares numbers** (D1), so a buffer of −0.0 no longer counts as
  sound.
- **`benches/baseline.csv` is re-recorded on cyrius 6.7.18.** The old one was 6.5.36's,
  so 16 of 18 rows read 0.45–0.82x and would have hidden a 2.4× regression.

### Docs

- `input-ranges.md`: the fatigue rows (D7), two placeholder cells and "f64_to
  wraps" (D14), the builders' row (A9), and a 2.2.1 note.
- `port-audit.md`: the allocation funnel in its conventions (D8), and re-measured
  counts, bundle size, error variants and the sakshi leaf (D9).
- The examples check every constructor (D10). `streaming.cyr` measures each fill
  against one `svara_glottal_new` (D6), and `error_handling.cyr` shows the eager
  tract.
- Every bare `src` line citation in the tests and docs is now a function name (D13).
  The comments and headers that no longer matched the code are corrected (A9, B12,
  C12, D17).
- Roadmap: completed items removed. Open work is pinned to 2.3.0, 2.3.x and 2.x.x,
  and the upstream-blocked section is kept, with the A7 pre-sizing added to it.

### Verified

- **The audio changes are where ADR-0009 puts them.** The 716,010-line sweep against
  2.2.0 moved 141,097 of 709,301 sample values, in 109 of 404 renders. None changed
  its status or length, and none became non-finite.
  - At 44.1 kHz: songbird −0.00 to +0.07 dB, raptor −0.15 to +0.00 dB, cricket
    −1.31 to −1.45 dB, bee in its low bits. The other nine species are bit-identical.
  - At 8 kHz: the dog −1.5 to +7.4 dB, streams −1.8 to +7.6 dB.
  - Of the 26 JSON documents, only the tract's changed, by its new `"clock"`.
  - Of 953 hostile cases, none changed its answer.
- **Every regression test fails on 2.2.0.** The hardening suite faults on 2.2.0 at
  its dead-handle group, so the B2/B4/C3 group was run on its own there.
- **Timing**, over six rounds alternating 2.2.0 and this tree on one machine. The
  host was carrying an unrelated workload throughout, so the absolute times are not
  comparable to earlier releases' figures, but each round ran both trees under the
  same load, so the ratios hold:
  - every synthesis benchmark is 0.98–1.01× (median of medians 1.004×);
  - the 1 s stream is **1.024×**, the cost of rendering in its own 20 ms blocks and
    carrying the tilt state;
  - `emotion_evaluate` is **1.04×** (182 → 189 ns, median), for the call layer the
    `_into` form shares. The first draft validated the state twice, once in
    `emotion_evaluate` and again in the `_into` it called. It measured **1.37×**, so
    each form now validates once.
- **Gates.**
  - `cyrius audit` exits 0, and all 21 suites pass.
  - `scripts/check-alloc.sh` passes, and so do the five examples, now under `cyrfmt`
    and lint.
  - The fuzz harness passes, the suite passes for aarch64 under qemu,
    `cyrius distlib` is current, and `cyrius deps --verify` passes 50 of 50.
- **The consumer floor stays cyrius 6.7.2.** A from-scratch consumer builds and runs
  warning-free on 6.7.2 and 6.7.18, and 6.7.1 still fails inside naad, as at 2.2.0.
  This check caught the first draft escaping preset names with
  `str_builder_add_json_cstr`, which 6.7.2 does not have.

## [2.2.0] - a refused allocation is reported

Minor release, and the first public-surface change since the port. Roadmap 2.2.0's
Lane B (the 2.0.3 audit's F9) ships here, alone: Lane A, f32 throughout, becomes
2.3.0 by the roadmap's own rule. **Every prani constructor and entry point that
allocates now reports a refused allocation as the new `PRANI_ERR_ALLOC` (−6)**
([ADR-0008](docs/adr/0008-a-refused-allocation-is-reported.md)). The builders that
had no error channel until now also refuse a NaN, which 2.0.5 deferred to this
release.

No dependency moves: cyrius 6.7.18, svara 3.14.0, naad 2.3.5, hisab 3.5.0, goonj
2.3.2, sakshi 2.5.10. **Valid-input output is byte-identical to 2.1.2.** What changes
is what happens on a refusal, and three hostile inputs that a builder now refuses
itself. The consumer floor stays cyrius 6.7.2. **2,287 assertions / 19 suites.**

### Breaking

- ⚠ **Every constructor can return a negative code.** This includes those that
  could not fail before:
  - `crvoice_new`, `dcblocker_new`, `prani_rng_new`, `species_params`,
    `species_params_make`, `crtract_options_new` and `crtract_options_default`;
  - the sequence and preset constructors, `preset_all`;
  - `emotion_new`, `emotion_with_values`, `fatigue_new`, `fatigue_modifiers`,
    `prani_intent_modifiers` and `bridge_perturbation_from_urgency`.

  Test every result with `prani_is_err`. The renderers pass an error handle
  straight back, so checking a chain's final result is enough.
- ⚠ **The builders refuse a NaN.** A NaN passes every `f64_clamp`, so through 2.1.2
  it was stored, and caught only where the voice was used.
  - `crvoice_with_size`, `_f0_offset`, `_breathiness`, `_jitter`, `_shimmer` and
    `_vocal_effort` return `PRANI_ERR_INVALID_SPECIES`; `emotion_with_values` and
    `emotion_with_smoothing` return `PRANI_ERR_INVALID_VOCALIZATION`. Each leaves
    its object as it was.
  - An infinity still clamps to the rail, as 2.0.5 decided, and so does any finite
    value.
- ⚠ **`crvoice_new` refuses an unknown species tag**, with `PRANI_ERR_INVALID_SPECIES`.
  `crvoice_new(99)` used to return a voice that only vocalize refused.
- ⚠ **More constructors check their input.**
  - `preset_make` refuses a NaN or an unknown species.
  - `species_params_make` ranges what it built.
  - `fatigue_modifiers_make` refuses a non-finite field, where it used to log it
    and return it unaltered.
- ⚠ **`stream_next_block` reports a failed fill.** When the fill failed, it used to
  return the zero-filled block as if it were rendered audio. Through 2.1.2 a sample
  rate svara rejects did exactly that. It now returns the failure's code.

### Added

- **`PRANI_ERR_ALLOC`** (−6), named `"allocation refused"` by `prani_err_name`.
- **The allocation funnel** in `src/error.cyr`: `prani_alloc`, `prani_vec_new` and
  `prani_vec_push`, drawing from prani's own Allocator (`prani_allocator`,
  `prani_alloc_raw`). It serves the same global arena, through `alloc`.
- **`stream_last_error`** and **`prani_ffi_stream_last_error`**. A failed fill
  still returns 0 and retires the stream (ADR-0003, revisited and kept); these say
  why, as `PRANI_ERR_ALLOC`, `PRANI_ERR_SVARA` and so on.
- `prani_is_nan`, `crvoice_formant_transition_at_into`.
- **`tests/allocrefused.tcyr`, a 19th suite (172 assertions).** It redefines
  `prani_alloc_raw` to refuse prani's requests and no one else's. Each of those
  requests is refused singly, then from that request on, in every public path:
  - every constructor;
  - all five apparatus, and every species × every vocalization it supports;
  - the builder chain, bouts, phrases and the chorus;
  - presets, spatial rendering and the four decoders;
  - the stream (laryngeal and purr), `next_block` and the FFI.

  Every refusal must answer exactly `PRANI_ERR_ALLOC` (the FFI: null, or a fill's
  `last_error`). Each case has a floor, and a control refuses one request past the
  last and requires the unrefused samples, bit for bit.

  It also redefines the stdlib's `alloc` to reach naad's two reporting
  constructors, in the windows it measures:
  - a refused bandpass in `crtract_new` is `PRANI_ERR_ALLOC`;
  - a refused noise or LFO in svara's glottal source is `PRANI_ERR_SVARA`, never
    audio.
- **`scripts/check-alloc.sh`** and its CI step. It fails on any raw `alloc`,
  `vec_new` or `vec_push` in `src/` outside the funnel: the suite can only see
  allocations that go through it. Against the 2.1.2 tree it lists 105.
- [ADR-0008](docs/adr/0008-a-refused-allocation-is-reported.md).

### Changed

- **An error handle passes through.** These functions return an error they are
  handed, rather than reading it:
  - the `with_*` builders, `preset_build` and `crtract_new` (its `params`);
  - the vocalize and render entry points, and `crvoice_synthesize_cat_purr`;
  - `crtract_set_formant_blend`, `emotion_evaluate` and `stream_new`;
  - the sequence renderers, and the two stream counters.

  `stream_is_finished` answers 1 for a null or error handle, so a drain loop
  terminates.
- **A refused bandpass inside naad** (`NAAD_ERR_ALLOCATION`, naad ADR-0010) is
  `PRANI_ERR_ALLOC` from `crtract_new`. Before, it built a snake tract with no
  filter, which then failed at its first render.
- **Five paths allocate nothing now**, which is the cheapest refusal there is. Measured
  against 2.1.2, all output byte-identical:

  | Path | 2.1.2 | 2.2.0 |
  |---|---:|---:|
  | cat-purr stream fill | 8,088 B/call | **0** |
  | `species_supports_vocalization` | 128 B/call | **0** |
  | `crtract_set_formant_blend` | 80 B/call | **0** |
  | 1 s wolf howl (`crvoice_vocalize`) | 1,117,600 B | 1,095,752 B |
  | 1 s crow chirp | 1,114,384 B | 1,095,280 B |

  - The species row is read into a stack buffer.
  - svara's target is built on the stack with `svara_vowel_target_set`, where svara
    allocated one unchecked per call.
  - The vocalize loop's blend storage is allocated once, not per 20 ms block.
  - The purr fill renders into the stream's reused block.

  `tests/allocbudget.tcyr` pins the first three and the howl, and each of those
  assertions fails on 2.1.2. The two stack sizes (`SpeciesParams` 128 B,
  `SvVowelTarget` 80 B) are pinned in `tests/species.tcyr` and `tests/tract.tcyr`.
- **`fatigue_modifiers()` off a poisoned state is refused.** A NaN in a
  `FatigueState` document used to come back as NaN modifiers.
- **Docs.**
  - ADR-0006 gains a 2.2.0 note on the builders, and ADR-0003 is marked revisited.
  - `input-ranges.md`'s "Not guarded" section marks what 2.2.0 settled.
  - CONTRIBUTING's rule now reads "allocate only through the funnel".
  - The `error_handling` example lists six codes.
  - The README shows the stdlib-leaf `sakshi` and checking a chain's result.

### Fixed

- **A refused allocation ended the process.** Measured on the 2.1.2 tree:
  - refusing `crvoice_new`'s first request was a SIGSEGV;
  - refusing a `vec_push` grow was `vec: alloc failed` and exit 1.

  Both are `PRANI_ERR_ALLOC` now. Refusals inside svara's own allocations are
  svara's, and still stop at the null page (svara ADR-0009). ADR-0008 draws that
  line.

### Verified

- **Valid-input output is unchanged**: the 716,010-line sweep against 2.1.2 moved
  0 of 709,301 sample values and none of the 26 JSON documents. 3 of 953 hostile
  cases changed: `crvoice_with_size(NaN)` and `crvoice_with_vocal_effort(NaN)` now
  fail at the builder with `PRANI_ERR_INVALID_SPECIES`, where vocalize returned
  `PRANI_ERR_INVALID_TRACT`, and `crvoice_new(99)` is now an error.
- **The suite can fail.** Eight mutants each fail `tests/allocrefused.tcyr`:
  - three crash it: an unchecked `dcblocker_new`, an unchecked `species_params` and
    an unchecked samples vec in vocalize;
  - five fail an assertion: a synthesis loop ignoring a refused push, `crtract_new`
    dropping naad's code, `next_block` returning zeros, a retiring fill not
    recording why, and `preset_all` storing a refused preset.

  The builder refusals fail on 2.1.2 (5 of 5 probed).
- **Timing**, over six rounds alternating the 2.1.2 tree and this one on one machine:
  - every synthesis benchmark is 0.98–1.02× (median 0.996×);
  - **`emotion_evaluate` costs about 9 ns more, 69 → 78 ns (median).** It allocates
    its result each frame, and that allocation now crosses the funnel and the
    error-handle check. A first version went through the Allocator vtable for it,
    and cost 16 ns (+22%); `prani_alloc` calls its allocator directly since.
- **Gates.** `cyrius audit` exits 0, `scripts/check-alloc.sh` passes, the five
  examples run, the suite passes for aarch64 under qemu, and `cyrius distlib` is
  current.

## [2.1.2] - svara's corrected pulse, at prani's level

Dependency release, and the repair the new svara needed in prani. `svara` 3.6.3 ->
**3.14.0**, `naad` 2.3.3 -> **2.3.5**, `hisab` 3.4.1 -> **3.5.0**, `goonj` 2.3.1 ->
**2.3.2**. sakshi stays **2.5.10**, now a stdlib leaf rather than a package. cyrius
stays **6.7.18**, still its newest tag, with bayan 1.11.1 and ganita 1.2.15. Each dep
is its project's newest tag, and they agree: each pins cyrius 6.7.18, and svara, naad
and goonj all pin hisab 3.5.0.

No API change, and the consumer floor stays cyrius 6.7.2. **The audio changes on
purpose.** svara 3.11.0 corrected its glottal pulse, and prani now feeds that pulse
into the oracle's gain stage
([ADR-0007](docs/adr/0007-the-corrected-pulse-at-the-oracles-level.md)). The
huge-finite-bandwidth NaN that 2.1.1 found is gone, because svara now refuses that
bandwidth.

### Fixed

- ⚠ **prani's gain stage on svara's corrected pulse**
  ([ADR-0007](docs/adr/0007-the-corrected-pulse-at-the-oracles-level.md)). svara
  3.11.0 made its Rosenberg pulse the full B, with a closing phase. It scales the
  pulse by 7.13 so that a human /a/ keeps its loudness, and leaves the aspiration
  noise unscaled. A creature tract's energy sits in the lowest harmonics, which
  carry the whole gain. Measured through 2.1.1's code on svara 3.14.0:
  - wolf came out 14.4 dB louder at the median, up to 16.7, and dog, cat, crow and
    fantasy 3 to 14 dB louder;
  - a wolf howl+bark phrase peaked at **2.24**, where 2.1.1 peaked at 0.40, and three
    more renders passed full scale;
  - every species came out 17 dB less breathy than its params say.

  `src/tract.cyr` now converts the breathiness it hands svara and scales the sample,
  so the tract hears the oracle's mix, `pulse*(1-b) + noise*b*0.3*gate`, with the
  corrected pulse at the oracle's unit peak. One gain, `PR_TR_EXC_GAIN` (+7.37 dB),
  is solved as svara solved its own. With it, the median level of the 56
  glottal-dominated renders (every species × vocalization at 44.1 kHz) is 2.1.1's.
  - **Levels against 2.1.1, now:** wolf +0 to +7 dB, dog −3 to +4, cat −3 to +3,
    crow −2 to +2, fantasy −7 to +4. Lion, crocodilian, dragon, songbird, snake,
    cricket and bee are unchanged.
  - **The spread around the median is timbre, not error.** The 56 run from −6.9 to
    +7.1 dB, because the corrected closure is darker, and more so for a tract whose
    energy sits higher. No single gain undoes that, and none should.
  - **Nothing passes full scale.** The loudest render in the sweep peaks at 0.786,
    and 2.1.1's peaked at 0.810.

  svara is unchanged. Two new test groups pin all of this:
  - `tests/tract.tcyr` pins the two svara facts the maths rests on (the pulse peaks
    at `SV_G_ROSENBERG_GAIN`, and the mix is linear in breathiness), the helper
    algebra, and that `crtract_glottal_new` hands svara the converted breathiness.
  - `tests/voice.tcyr` pins the calibration against 2.1.1's recorded RMS for all 56:
    the median within 0.05 dB, every render within ±7.5 dB, no render at full scale,
    and the phrase under it.

  2.1.1's code on svara 3.14.0 fails three of those assertions. The fix without its
  gain fails two, plus the oracle's own audibility bar for fantasy.

### Changed

- **Every render that voices through svara's glottal source moves.** That is 530,008
  of 709,301 sweep values, in 242 of 404 renders, from the pulse above. Lengths,
  error codes and finiteness are unchanged in every render, and all 26 JSON documents
  prani writes are byte-identical. Of 953 hostile cases, 5 change outcome, all of
  them the bandwidth refusals below. The rest keep their error code, length and
  non-finite count.
- **`crtract_set_formant_blend` refuses a finite bandwidth whose coefficients
  overflow.** svara 3.6.5 (its ADR-0010) bounds it, which answers the issue prani
  filed at 2.1.1.
  - At 44.1 kHz `f64::MAX`, `f32::MAX`, 1e300, 9,973,281 Hz and 1e7 Hz used to return
    `PRANI_ERR_NONE` and render 2,205 of 2,205 samples as NaN. They now return
    `PRANI_ERR_SVARA`, and the tract keeps rendering finite audio.
  - 9,973,280 Hz is still accepted, which is svara's bound.

  The blend group in `tests/tract.tcyr` gains the case, as the roadmap's watch item
  said it should: four assertions, three of which fail on 2.1.1. prani adds no guard
  of its own, and the bound is not restated.
- **A steady-state stream fill retains 560 B, up from 488.** It is still exactly one
  `svara_glottal_new`, whose source gained nine fields: a tilt memo (svara 3.9.0) and
  a tone's state (3.13.0). The two fill budgets in `tests/allocbudget.tcyr` and
  `tests/stream.tcyr` now measure one source in the suite and hold prani's own share
  at 0 B. They no longer write svara's object size down as `<= 512`.
- **sakshi is a stdlib leaf, not a `[deps.sakshi]` package.** hisab 3.4.2 dropped
  its package, and a package of that name now draws `refusing to overwrite stdlib
  leaf 'sakshi'`. `lib/sakshi.cyr` has the same bytes as before. `cyrius.lock` has 4
  commit rows, down from 5.
- **`lib/` and `cyrius.lock` are a clean resolve at the pins.** There are 50 files.
  46 are byte-identical to the 6.7.18 snapshot and 4 to their tag's `dist/`.
  `cyrius deps --verify` passes all 50, and `cyrius deps --locked` confirms them.
- **`build/prani` grows from 4.23 MB to 4.59 MB**, mostly from svara's and hisab's
  larger bundles.

### Removed

- **The six `#inline` markers the compiler ignored**, in `dsp.cyr`, `rng.cyr` (two),
  `tract.cyr` (two) and `voice.cyr`. They were the build's only warnings, so the
  build is now warning-free. The sweep's output is byte-identical with and without
  them. This closes the roadmap watch item.

### Verified

- **The reachable-function walk.** prani calls 43 dep functions directly, which
  reach 278 svara, naad and hisab functions.
  - The 21 that prani consumes by name keep their signatures. Along the call trees,
    23 svara functions changed body and 16 were added.
  - naad's 4 changes are allocation results (naad's ADR-0010), and hisab's 2 are
    allocation routing.
  - Two return values changed meaning:
    - `svara_glottal_next_sample`'s pulse now peaks at 7.13, repaired above.
    - `svara_glottal_new` and `filter_biquad_new` can now return an allocation error.
      prani already treats every negative return from both as a failure.
- **Gates.** 18 suites with **2,058 assertions** (2,025 at 2.1.1), and `cyrius audit`
  exits 0 (fmt, lint, docs, tests and bench). The suite also passes for aarch64 under
  qemu (`cyrius test --aarch64 tests`). The five examples build and run, and
  `cyrius distlib` is current.
- **The consumer floor**, re-measured with a from-scratch consumer of
  `dist/prani.cyr`. 6.7.2, 6.7.5, 6.7.9, 6.7.12, 6.7.17 and 6.7.18 build with no
  warning and run the smoke. 6.7.0 and 6.7.1 fail at compile time, now inside naad's
  bundle.

## [2.1.1] - cyrius 6.7.18, and the newest deps

Toolchain and dependency release. `cyrius` 6.7.9 -> **6.7.18**, `svara` 3.6.0 ->
**3.6.3**, `naad` 2.3.0 -> **2.3.3**, `hisab` 3.4.0 -> **3.4.1**, `goonj` 2.1.0 ->
**2.3.1**, `sakshi` 2.5.8 -> **2.5.10**, with bayan 1.5.13 -> **1.11.1** from the
snapshot; ganita stays 1.2.15. Each is its project's newest tag, and this time the
newest tags are also the matched set: svara, naad, hisab and goonj each pin exactly
6.7.18, svara pins the other three at these tags, and sakshi 2.5.10 is what hisab
declares and what the toolchain bundles.

No API change and no logic change in prani. Every sample value, every JSON document
prani writes and every hostile-input result 2.1.0 produced is the same here, and the
consumer floor stays cyrius 6.7.2. **Two contracts move, each because a dependency
now refuses an input it used to accept**, and both are repairs: an infinite
bandwidth in `crtract_set_formant_blend`, and a duplicate key or leading zero in
prani's four hand-written JSON decoders.

### Changed

- ⚠ **The hand-written JSON decoders refuse a duplicate key or a leading zero.**
  bayan 1.11.1 refuses both, the zero per RFC 8259 section 6. This reaches
  `crvoice_from_json_str`, `crtract_from_json_str`, `preset_from_json_str` and
  `sequence_call_phrase_from_json_str`.
  - Through 2.1.0 they took the **first** of two equal keys, so
    `{"species":1,"species":0,...}` restored a dog from a document that also said
    wolf, and they read `00` as 0.
  - They now return their usual error: `PRANI_ERR_INVALID_SPECIES`, or
    `PRANI_ERR_INVALID_VOCALIZATION` for a phrase.
  - prani's own writer emits neither, so every document it writes still round-trips.

  A new `tests/hardening.tcyr` group pins the change for all four decoders, with a
  control for each, and its five refusals fail on 2.1.0.
- **`crtract_set_formant_blend` refuses an infinite target bandwidth.** svara 3.6.3
  added it to what `svara_formant_validate` rejects.
  - Through svara 3.6.0 the call returned `PRANI_ERR_NONE` and every sample after it
    was NaN: 2,205 of 2,205 at 44.1 kHz, measured on 2.1.0 and on 2.0.14.
  - It now returns `PRANI_ERR_SVARA`, and the tract keeps rendering finite audio with
    its old formants.

  prani adds no guard of its own, for the reason `tests/tract.tcyr` gives for a NaN
  blend: the rule is svara's, and a copy here is what goes stale. The new assertion
  pins svara's refusal and fails on 2.1.0. Only a direct caller can reach this,
  because prani's own blends use built-in contours and species bandwidths
  range-checked against `PR_SP_BW_MAX`.
- **A steady-state stream fill retains 488 B, down from 512.** svara 3.6.3 reuses a
  24 B scratch that every `svara_glottal_new` used to allocate. Measured per fill
  over 40 fills: 512 on every one at 2.1.0, 488 on every one here. The `<= 512`
  budgets in `tests/allocbudget.tcyr` and `tests/stream.tcyr` are unchanged; the
  comments and `docs/benchmarks.md` that gave 512 as today's figure now say 488.
- **JSON decoding costs about 1.5x what it did**, by bayan's own measure, because
  its parsers now read through bounds-checked slices. prani has no JSON benchmark,
  so this is bayan's figure, not one measured here. No synthesis path parses JSON.
- **`lib/` gains `simd.cyr`, `slice.cyr` and seven `unicode/` files.** goonj 2.3.1's
  bundle includes `simd`, and bayan 1.11.1 includes `slice` and `unicode/`. The
  resolve brings them in. None is declared in `[deps].stdlib`, because prani calls
  none of them.
- **`lib/hashmap_fast.cyr` is gone.** Nothing includes it, and `cyrius deps` 6.7.18
  names it outside the declared footprint on every build. `cyrius lib sync` still
  copies it, so it has to be dropped again after every re-vendor, with the command
  the note gives: `rm lib/hashmap_fast.cyr && cyrius deps --lock`.
- **`lib/` and `cyrius.lock` are a clean resolve at the pin.** There are 50 files.
  46 are byte-identical to the 6.7.18 snapshot and 4 to their tag's `dist/`, with
  five commit rows. `cyrius deps --verify` passes all 50, and `cyrius deps --locked`
  confirms them.
- **`build/prani` grows from 3.10 MB to 4.23 MB.** hisab's bundle goes from 2.64 MB
  to 3.66 MB and bayan's from 0.74 MB to 1.00 MB. In `dist/prani.cyr`, only the
  version line and one comment change.

### Fixed

- **Three places said `cyrius audit`'s fmt gate is stricter than
  `cyrius fmt <file> --check`:** `CONTRIBUTING.md`, `state.md` and the CI
  workflow's comment. That has not been true since cyrius 6.7.12, where the audit
  takes `cyrfmt --check`'s verdict. Both now accept a 4-space continuation step as
  well as the canonical 2.

### Found, not fixed

- **A finite but huge bandwidth still renders NaN, inside svara.** A bandwidth of
  `f64::MAX`, or `f32::MAX` as an f64, passes svara 3.6.3's check. The call returns
  success and every sample after it is NaN. A probe of svara alone reproduces it,
  with `svara_tract_set_formants_from_target` and then `svara_tract_process_sample`,
  and 2.0.14 and 2.1.0 do the same through `crtract_set_formant_blend`. The rule is
  svara's, so it is on the roadmap's watch list rather than guarded here. svara's
  ADR-0009 leaves it open on purpose, on the grounds that any finite ceiling would be
  invented. prani filed an issue in svara arguing that the formula defines the
  ceiling: `sinh` overflows past about 226 x the sample rate, 9.97 MHz at 44.1 kHz.
  The issue is `docs/development/issues/2026-11-07-finite-bandwidth-past-sinh-overflow-renders-nan.md`.
- **prani's derived decoders still take the last of two equal keys, and accept
  `05`.** They are the compiler's `#derive(Serialize)` scanner, not bayan, so their
  behaviour is unchanged. This covers `PrRng`, `DcBlocker`, the fatigue and emotion
  types, `IntentModifiers`, `SpeciesParams`, `CallElement` and `CallBout`. prani now
  answers a duplicate key two ways, depending on the type.
- **svara and naad closed the f32 work that roadmap Lane A waits on.** Both decided
  their synthesis core stays f64 (svara ADR-0008, naad ADR-0008). The lane's blocker
  is now a standing decision, so the lane needs a decision too, and the roadmap says
  so. This release does not make it.

### Upgrading

- **Move the pins together:** cyrius 6.7.18, svara 3.6.3, naad 2.3.3, hisab 3.4.1,
  goonj 2.3.1 and sakshi 2.5.10.
- **After `cyrius lib sync`, drop `lib/hashmap_fast.cyr`** as above, or `cyrius deps`
  names it on every build.
- **JSON that prani did not write may now be refused:** a repeated key, or a number
  with a leading zero. A document from prani's own `*_to_json` is unaffected.
- **A caller that relied on an infinite bandwidth being accepted** now gets
  `PRANI_ERR_SVARA` from `crtract_set_formant_blend`. What it got before was NaN audio.
- **`cyrius tests` no longer exists** (removed in cyrius 6.7.12): use
  `cyrius test tests`. Nothing in this repo called it.

### Verified

Every figure below was measured on x86_64, against this tree. The test suite also
ran for aarch64 under qemu, as noted; nothing else here ran on aarch64.

- **Value sweep, 2.1.0 on 6.7.9 against this tree on 6.7.18.** All 3,198,946 values
  across 1,799 renders are **byte-identical**. This is the same sweep that moved
  2,682,303 values at 2.0.14, so it can see a change.
- **JSON sweep, new at this release.** bayan moves from 1.5.13 to 1.11.1, and no
  earlier prani sweep compared serialized output. Every type with a codec was
  covered: 10 derived and 4 hand-rolled. Each was written, decoded and written again,
  with 24 integer edges in each field (NaN and infinity bit patterns, -0, `INT64_MIN`
  and `INT64_MAX`, denormals, `f64::MAX`, 2^53 and out-of-range enum codes). All 2,863
  lines are **byte-identical** to 2.1.0. They include 1,959 exact round trips and the
  879 decodes that the validating codecs refuse. The sweep feeds only prani's own
  output, which is why it could not see the duplicate-key change above. That came
  from bayan's changelog and a separate probe.
- **Hostile inputs, one process per case.** 408 cases. All exit 0, and the output is
  byte-identical to 2.1.0.
- **The consumed surface was diffed.** Of the 59 external names prani calls, five
  changed:
  - `bayan_json_v_is_int` now returns `bool`. prani compares it with `== 0`.
  - `svara_tract_set_formants_from_target` writes through a scratch target. This is
    svara's stale-upper-formant fix.
  - `vec_get`, `vec_set` and `vec_push` now exit through `_vec_abort`, which runs an
    optional hook and then exits exactly as before. `VEC_CAP_MAX` is still 2^28.

  Following every call down, 55 svara functions are reachable from prani's and 7 of
  them changed: four refactors svara reports as bit-identical, the stale-formant fix,
  a slot-index refusal in `svara_formant_bank_update`, and the bandwidth refusal
  above. The value sweep shows that none of them moves a prani sample.
- **The toolchain alone was isolated.** One build used the 6.7.18 compiler and
  stdlib (bayan 1.11.1 included) with the 2.1.0 dep bundles; the other was the 2.1.0
  tree on 6.7.9. On every synthesis path, the RNG, the emotion grid and every
  round trip, the two agree byte for byte, allocation counts included. The fmt, lint
  and doc tools give the same verdicts on every file under both. No new keyword
  touches prani.
- **svara 3.6.3's upstream decoder hang cannot reach prani.** The derived decoder
  loops forever on a `Vec<struct>` field holding a stray `}`. None of prani's 10
  derived types has a vec field, and `CallPhrase`'s element vec is decoded by hand
  through bayan.
- **Benchmarks, six alternating rounds of the 2.1.0 tree and this one.** The median is
  1.005x across 18 benchmarks, ranging from 0.958 to 1.037.
- **The consumer floor was re-measured** with a from-scratch consumer at the new pins.
  6.7.2, 6.7.9, 6.7.17 and 6.7.18 build and run. 6.7.1 fails at compile time, in
  `lib/goonj.cyr`.
- **aarch64, for the first time.** `cyrius test --aarch64 tests` cross-compiles all
  18 suites and runs them under `qemu-aarch64`: 2025 assertions pass. That is user-mode
  emulation, not hardware.
- **Gate:** `cyrius audit` exit 0, 18 suites / 2025 assertions (`tract` 154 -> 156,
  `hardening` 120 -> 130); `cyrius fuzz`, `scripts/run-examples.sh`,
  `cyrius distlib --check`, `cyrius deps --verify` and `cyrius deps --locked` all
  clean.

## [2.1.0] - cyrius 6.7.9, and a new floor for consumers

Toolchain and dependency release. `cyrius` 6.6.20 -> **6.7.9**, `svara` 3.5.6 ->
**3.6.0**, `naad` 2.2.5 -> **2.3.0**, `hisab` 3.3.7 -> **3.4.0**, `goonj` 2.0.6 ->
**2.1.0**, `sakshi` 2.5.7 -> **2.5.8**, with bayan 1.5.12 -> 1.5.13 and ganita 1.2.14 ->
1.2.15 from the snapshot. This is the matched set for 6.7.9: svara, naad, hisab and goonj
each pin it exactly and agree on each other's sub-pins, sakshi 2.5.8 is the copy the
toolchain bundles, and the next tag of each of the four needs cyrius 6.7.18.

It is a minor because the floor for consumers moves: **they now need cyrius 6.7.2 or
later.** No API change, no logic change, and no computed value moves. The `src/` diff
has no non-comment line.

**This is not the 2.1.0 the roadmap described.** That milestone, the allocation-failure
contract and f32 throughout, is renumbered **2.2.0** and none of it is here.

### Changed

- ⚠ **Consumers need cyrius 6.7.2 or later.** sakshi 2.5.8 declares its syscall numbers
  `const`, which arrives in 6.7.2, and goonj pulls sakshi in. Measured with a from-scratch
  consumer of this bundle: 6.7.2, 6.7.5, 6.7.8, 6.7.9 and 6.7.18 build and run; 6.6.20,
  6.7.0 and 6.7.1 fail at compile time, in `lib/goonj.cyr`. The failure is loud, not
  silent. 2.0.14 is the last prani that builds below this floor.
- **The roadmap's 2.1.0 is now 2.2.0.** 33 references in comments, tests and docs moved
  with it. The dated records keep the old number, and the roadmap and
  [ADR-0006](docs/adr/0006-reject-non-finite-numeric-input.md) say so.
- **sakshi is 2.5.8, not the newest tag that pins 6.7.9.** 2.5.9 and 2.5.10 pin it too,
  but 2.5.8 is what hisab 3.4.0 declares and what the toolchain bundles.
- **`build/prani` grows from 2.77 MB to 3.10 MB**, because hisab's bundle does.
  `dist/prani.cyr` changes only its version line.
- **`lib/` and `cyrius.lock` are a clean resolve at the pin.** 42 files, 38 byte-identical
  to the snapshot and four to their tag's `dist/`; five commit rows. `cyrius deps --verify`
  passes all 42, and `cyrius deps --locked`, new in 6.7.x, confirms `lib/` and the lock
  are exactly what the tags resolve.

### Fixed

- **`CONTRIBUTING.md` still said `vec_push` returns -1.** It exits the process on a failed
  allocation; `vec_push_a` is the form that returns -1. 2.0.14 corrected the same claim
  in the roadmap and missed this copy.

### Upgrading

- **Move the pins together:** cyrius 6.7.9, svara 3.6.0, naad 2.3.0, hisab 3.4.0 and
  goonj 2.1.0. A consumer that calls one of hisab 3.4.0's 27 retired names itself gets a
  build error; hisab's changelog has the replacements.
- **A consumer that cannot reach cyrius 6.7.2 stays on 2.0.14.**

### Verified

Every figure below was measured on x86_64, against this tree, and nothing here ran on
aarch64.

- **Value sweep, 2.0.14 on 6.6.20 against this tree on 6.7.9.** All 3,198,946 values
  across 1,799 renders are **byte-identical**: every species, vocalization and intent at
  two rates, the builders, streams, a bout, a phrase, a chorus, spatial, the nasal path and
  the RNG. The same sweep moved 2,682,303 values last release, so it can see a change.
- **The consumed surface was diffed.** Of the 58 external names prani calls, 41 have
  identical bodies, `ease_in_out_smooth` among them, and 16 are compiler builtins. One
  changed: `bayan_json_v_parse_buf` names its 48-byte state through a constant, which is
  still 48. prani calls none of hisab's 27 retired names.
- **The language changes do not reach prani's source.** Outside strings and comments it
  has no lone `!` (a no-op until 6.7.3, so a real one would have flipped), no `~`, no
  `>>>`, no unary `-` on a non-literal, and no name that became a keyword. 6.7.7's arity
  check on forward calls compiles the whole tree.
- **Hostile inputs, one process per case.** The 2.0.14 probe: 12 values through 34
  entry-point and parameter combinations, 408 cases per tree, 25 s each. All 408 exit 0
  on both trees, and **no case moved**: the same exit, return code, length and
  non-finite count throughout.
- **The 2.0.14 guards still hold on 6.7.9.** An off-by-one in either ceiling helper,
  `stream_next_block` ignoring its return, `prani_vec_push_zeros` without its sum check
  and an unguarded Doppler each still fail their suite.
- **Benchmarks are flat**: six alternating rounds on one machine, best of each, median
  0.99x and every row from 0.89x to 1.02x. The `f64_sin`-heavy paths 2.0.14 slowed did
  not move again.
- **Gate:** `cyrius audit` exit 0, 18 suites / 2013 assertions; `cyrius fuzz`,
  `scripts/run-examples.sh`, `cyrius distlib --check`, `cyrius deps --verify` and
  `cyrius deps --locked` all clean.

## [2.0.14] - cyrius 6.6.20, and a guard that saturation walked past

Toolchain and dependency release. `cyrius` 6.6.3 -> **6.6.20**, `svara` 3.5.4 ->
**3.5.6**, `naad` 2.2.2 -> **2.2.5**, `hisab` 2.11.2 -> **3.3.7**, `goonj` 2.0.4 ->
**2.0.6**, `sakshi` -> **2.5.7**, with bayan 1.5.6 -> 1.5.12 and ganita 1.2.5 -> 1.2.14
from the snapshot. This is the matched set for 6.6.20: svara, naad, hisab and goonj
each pin it exactly and agree on each other's sub-pins, sakshi 2.5.7 is the copy the
toolchain bundles, and the next tag of each of the four needs cyrius 6.7.x.

No API change, and no accepted input renders differently beyond the last bits. One
input that used to be refused started killing the process instead, which is the
reason this is more than a pin bump.

### Fixed

⭐ **`crvoice_vocalize` aborted the host process on an overflowing sample count.**
Cyrius 6.6.8 made `f64_to` saturate: +Inf and anything at or past 2^63 went from
`i64::MIN` to `i64::MAX`, and NaN to 0. `num_samples < 0` had been catching the old
sentinel by accident. The new one walks past it, into a render loop that pushes until
`vec: capacity overflow` ends the process.

| Input | 2.0.13 on 6.6.3 | 2.0.13's source on 6.6.20 | 2.0.14 |
|---|---|---|---|
| 1e300 s at 44.1 kHz | `INVALID_VOCALIZATION` | **process abort** | `INVALID_VOCALIZATION` |

The count is now ranged in f64 **before** the cast, as `stream_new` already did. That
check is shared as `prani_sample_count_ok`, `PR_ST_MAX_SAMPLES` is gone, and
`crvoice_vocalize` applies the tighter limit below.

This reaches released versions. **Any prani up to 2.0.13 has this defect if it is
built with cyrius 6.6.8 or later, or for aarch64**, where cyrius's 6.6.8 notes record
that `f64_to` already gave `i64::MAX` for +Inf. `SECURITY.md` says so.

Every other `f64_to` site was read and needs no change: each is ranged first or
bounded by a clamp. `block_size` at an absurd rate now saturates high rather than
flooring to 1, which is what the oracle's `as usize` and `.min()` give, and it is
bounded by the remaining count either way.

⭐ **A request too long for one buffer ended the same way, on every toolchain.** A vec
cannot grow past `VEC_CAP_MAX`, 2^28 samples, about 101 minutes at 44.1 kHz: the next
push is `vec: capacity overflow` and a process exit, with no return value to check.
`crvoice_vocalize` had no ceiling, so 3e9 s at 44.1 kHz rendered for minutes and then
died, on 2.0.13 as well. A rate of 2^63 Hz or 1e19 Hz for 0.01 s had the same shape: the
product fits an i64, and those ran past a 25 s timeout. The same exit was reachable
without going through `vocalize` at all:

| Path | Way in |
|---|---|
| `crvoice_vocalize`, `_with_intent` | a duration or rate past the limit, on every apparatus and the purr path |
| `crvoice_synthesize_cat_purr`, `crtract_synthesize*` | a count past the limit |
| `stream_next_block` | one block past the limit: `remaining` can be 2^53, and the call ignored `prani_vec_push_zeros`'s return |
| `spatial_apply_doppler_shift` | a buffer past ~1.41e8 samples moving away, which stretches it up to 1.9x |

Each now returns `PRANI_ERR_INVALID_VOCALIZATION` before the first push, through
`prani_buffer_count_ok` and `prani_buffer_len_ok`. The limit is `lib/vec.cyr`'s own
constant, not a copy, so it refuses exactly what could never have succeeded, and every
render that worked still does. `stream_new` keeps its 2^53 ceiling because a stream is
not one buffer, so it carries anything longer. This diverges from the oracle, whose
`Vec` had no such limit, and [ADR-0006](docs/adr/0006-reject-non-finite-numeric-input.md)
records it.

Not covered: an allocation refused below the limit is also a process exit, because
`vec_push` exits on that failure too. That is roadmap 2.1.0 Lane B, whose entry wrongly
said `vec_push` returns -1 and is corrected here.

**Two assertions were still passing, for a reason that was no longer true.**
`tests/sequence.tcyr` asserted `f64_to(NaN) * 2 == 0` "because i64::MIN wraps", and
`tests/hardening.tcyr` asserted a NaN count appends nothing "because it is i64::MIN".
Both pass on 6.6.20 only because NaN is now 0. They are rewritten, along with the
nine probes that did fail. A NaN `interval` or `gap` is the case this took a backstop
from: `prani_vec_push_zeros` used to report the count, and can no longer see it. The
pre-pass catches it and always did, but only +inf was tested, so NaN is pinned there
now.

**The comments and docs that restated the old conversion**: seven `src/` modules,
`input-ranges.md`, the `error_handling` example's printed text, and an amendment on
[ADR-0006](docs/adr/0006-reject-non-finite-numeric-input.md), whose decision stands.

**`README.md`'s consumer snippet included `dist/prani.cyr`**, a path a consumer does
not have: a dep vendors to `lib/prani.cyr`, and the snippet as written fails with
`cannot open include file`. Its tag still said 2.0.11, as did `SECURITY.md`'s table,
and `state.md` still said pin 6.5.36.

### Changed

- **39 `bayan_json_v_obj_get` calls moved to `bayan_json_v_obj_get_by_cstr`.** bayan
  1.5.10 deprecated the old name, which forwards to the new one: same lookup, same
  answers, 39 fewer warnings on every build.
- **`dist/prani.deps` is gone, and CI's comment with it.** `cyrius distlib` 6.6.18 and
  later names only what a fold needs beyond its named deps, and for prani that is
  nothing: svara's own sidecar brings `bayan`.
- **The `sakshi` pin now describes what ships.** `lib/sakshi.cyr` was the 6.6.3
  snapshot's 2.5.2, not the pinned 2.4.12: the resolver kept the snapshot and skipped
  the dep, and said so on every build. Both are 2.5.7 now, byte-identical, and the
  resolver has nothing left to skip.
- **`cyrius.lock` and `lib/` are a clean resolve at the pin.** 42 files, 38
  byte-identical to the snapshot and four to their tag's `dist/`; five commit rows;
  `cyrius deps --verify` passes all 42. `alloc_cx.cyr`, `chrono.cyr` and `poison.cyr`
  arrive.
- **Seven `crtract_synthesize*` wrappers gained a doc comment.** cyrdoc used to read
  `#must_use` as documentation, and `cyrius audit` failed on these.
- **Build warnings 136 -> 6**, on a clean tree. naad and svara dropped their dead
  `#inline` markers; the six left are prani's own, and are on the roadmap's watch list.
- **Samples move by up to 6.8e-14 of peak.** 84% of sample values differ in their last
  bits. The likely source is the toolchain's transcendentals: cyrius 6.6.9 took x86 sin
  and cos off x87 `fsin` / `fcos`. The RNG stream is bit-identical. One garbage input
  improved with it: a 1e-297 Hz rate into `crvoice_apply_nasal_antiformant` went from
  438 non-finite samples of 441 to none.
- ⚠ **Three paths got slower; the rest are flat.** Six alternating rounds on one
  machine, best of each: median 1.00x, with `bee_buzz` +29%, `cricket_stridulate` +16%
  and `songbird_trill` +11%. `f64_sin` alone went from 55 to 91 ns a call when it left
  x87 `fsin`, and those three are the paths that call it most per sample. That is the
  likely cause, not an isolated one. The ~2x `scripts/bench-check.sh` reports is not a
  result: 2.0.13 measures about as far from `benches/baseline.csv` on this machine
  today. `baseline.csv` is unchanged.

### Upgrading

- **Move the pins together:** cyrius 6.6.20, svara 3.5.6, naad 2.2.5, hisab 3.3.7 and
  goonj 2.0.6. naad 2.2.5 needs hisab 3.x, so no older set can sit beside it.
- After a pin move, delete `cyrius.lock` before `cyrius deps`, which keeps old commit
  rows beside the new ones.
- Golden values taken from an earlier prani may need a tolerance rather than bit
  equality.
- A consumer includes `lib/prani.cyr`, and needs no `dist/prani.deps`.
- A single buffer past 2^28 samples, about 101 minutes at 44.1 kHz, is now
  `PRANI_ERR_INVALID_VOCALIZATION` rather than a process exit. A longer render goes
  through `stream_new`, in blocks of at most that size.

### Verified

Every figure below was measured on x86_64 against this tree, and neither the sweeps
nor the benchmarks ran on aarch64.

- **Value sweep, 2.0.13 on 6.6.3 against this tree on 6.6.20.** 3,198,946 values across
  1,799 renders: every species, vocalization and intent at two rates, the builders,
  streams, a bout, a phrase, a chorus, spatial, the nasal path and the RNG. 2,682,303
  differ, none by more than 6.8e-14 of peak, with no length, error-code, sign or
  non-finite change.
- **Hostile inputs, one process per case.** 12 values (NaN, ±Inf, f64::MAX, 2^63, 1e19,
  3e9, the smallest denormal, 1e-297, 0, -1, 1e300) through 34 entry-point and
  parameter combinations, 408 cases per tree, 25 s each. On this tree all 408 exit 0,
  with no abort and no timeout; 2.0.13 had 12 timeouts. Thirteen cases moved: eleven
  that ran past the window are now refused, the nasal case above, and one 30M-sample
  render that finished inside the window on this run.
- **The ceiling sits on the real boundary.** One sample past `VEC_CAP_MAX` is refused at
  once. A render of exactly `VEC_CAP_MAX` samples through `crvoice_vocalize` returned
  268,435,456 samples, all finite, at a 4.1 GiB peak. 3e9 s at 44.1 kHz went from a
  process exit, 87 to 345 s in depending on the run, to an instant refusal.
- **The guards were mutated.** For the overflow fix, a floor-only check, a finite-only
  check and a 2^63 ceiling each abort inside the new hardening group, and a guard that
  refuses everything fails its controls. For the ceiling, an off-by-one in either helper,
  `stream_next_block` ignoring the return, `prani_vec_push_zeros` without its sum check
  and an unguarded Doppler each fail, and weakening the vocalize or purr guard aborts
  inside its group.
- **The consumed surface was diffed, not assumed.** Of the 58 external names prani
  calls, 39 have identical bodies, including every svara entry point. The rest are 13
  compiler builtins, `f64_le` / `f64_ge` / `f64_trunc` (builtins since 6.6.13, NaN
  behaviour probed), `alloc_init`'s chunk size, the bayan rename, and
  `ease_in_out_smooth`, which gained `public` and renamed three constants with the
  same values. This compares entry points only: what they call is covered by the value
  sweep.
- **A from-scratch consumer** declaring only svara, naad, hisab, goonj, prani and a
  minimal stdlib builds and runs `lib/prani.cyr`, with no sidecar.
- **Gate:** `cyrius audit` exit 0, 18 suites / 2013 assertions (`hardening` 96 -> 120);
  `cyrius fuzz`, `scripts/run-examples.sh`, `cyrius distlib --check` and
  `cyrius deps --verify` all clean.

## [2.0.13] - Unblocked by cyrius 6.6.3

Toolchain-pin release. `cyrius` 6.6.2 -> **6.6.3**; no source change.

6.6.2 could not compile this project. `#inline` became a real directive at cyrius
v6.5.63, which taught the compiler's PASS 2 to arm the directive — but PASS 1, the
declaration-collection scan, was never taught to CONSUME the token. An unconsumed
directive there falls through to the catchall and **terminates the scan**, so every
declaration after the first `#inline` went unregistered and the next `#derive`d
struct reached the parser as an unknown top-level token. The error named the struct,
which was innocent, hundreds of lines from the real trigger.

cyrius 6.6.3 closes both halves in all seven per-target compiler forks. Re-vendored
and rebuilt against it; tests green.

## [2.0.12] - Documentation sweep: 51 corrections after eight releases

Docs only — no code change; the `src/` diff is **0 non-comment lines**. Eight
releases (2.0.4 → 2.0.11) landed in rapid succession and the documentation was
never checked as a whole against the tree they produced.

### Fixed — the two that would have cost a user directly

⭐ **`SECURITY.md`'s supported-versions table had exactly one row: `1.0.x | Yes`.**
That is the **pre-port Rust crate**, frozen in git history and unmaintained — so
the file told a reporter that the only supported prani was one that no longer
exists, while 2.0.0–2.0.5 shipped a CRITICAL segfault, a process abort across the
whole (1000, 7500] sample-rate band, a NaN-buffer-returned-as-success, two null
derefs and a buffer overrun. Now a four-row table naming the repaired classes.
This is the same defect svara found in its own `SECURITY.md` at 3.5.3.

⭐ **`README.md`'s "Consuming prani" snippet pinned `tag = "2.0.3"`** — eight
releases stale. Anyone who copied it got a version predating every repair above.

### Fixed — the convention that governs future work

`port-audit.md`'s standing rule, applied to every module since 2.0.0:

> **f32 → f64 everywhere** (svara/naad/hisab are f64-only; widening is **forced**)

The first half still describes the code. **The justification was false**: ganita
1.1.4 ships a 23-function f32 scalar tier, already vendored in `lib/ganita.cyr`,
and prani calls nothing outside it. It is a *choice nobody had revisited* — now
recorded as such, and filed as roadmap 2.1.0 Lane A plus P0 on naad and svara.

### Fixed — a threshold that had moved, in seven places

svara's sample-rate floor was `<= 1000 Hz` until **svara 3.5.4** repaired two
defects prani found; it is **>1200 Hz** now. Stale in `ADR-0001`,
`input-ranges.md`, `svara-bridge-api.md`, the roadmap watch item that exists
*specifically to catch a moved threshold*, and three `src/` comments. Corrected
or dated everywhere; ADR-0001's measurement is kept with a dated note, because it
is what the decision was made against.

### Fixed — counts, links and tense

Assertion and suite counts (770/17 → 1946/18), benchmark counts (4 → 18),
`svara 3.5.3` → `3.5.4`, module counts, bundle line counts, and 15 per-module
test figures in `port-audit.md`'s ledger. **Zero dangling links tree-wide**,
verified by extracting every relative link and anchor and resolving it —
including two backticked paths to `dist/svara.cyr` / `dist/naad.cyr`, which have
never existed (the bundles are in `lib/`).

`rust-test-parity.md` now carries a dated header: its counts are **as of 2.0.4**
and stay that way, because rewriting them would destroy the evidence its
conclusions rest on. Its future-tense framing of `rust-old/`'s removal — an event
completed in 2.0.8 — is now past tense.

### Verification

Four sweep agents, then an independent pass that re-measured every corrected
number and hunted what they missed. It caught the sweep making
`docs/benchmarks.md`'s table and prose disagree (151 vs 155, 113 vs 118) —
reconciled to one measured figure, **114 ns**, taken from three runs.

⚠ **Left for a human**: `SECURITY.md` gives `security@agnos.org` while
`CODE_OF_CONDUCT.md` gives `conduct@agnos.dev` — two TLDs for one organisation.
A security contact is not something to guess at.

## [2.0.11] - The benchmark suite compares something now

`cyrius audit` has always run `tests/prani.bcyr` and reported
**`1 passed, 0 failed`** — because all the bench harness checks is that it *ran*.
It compared nothing. **Three regressions reached a release through that gap**,
and every one was caught by a human reading numbers:

- **2.0.5** shipped a 2.2× slowdown in `emotion_evaluate`, found two releases
  later by diffing a table in `docs/benchmarks.md`.
- **2.0.6** found `stream_fill_buffer` retaining 8,800 B/call — by someone
  *writing an example*.
- **2.0.10** found the harness measuring a **stale `dist/` bundle**: it read
  155 ns for a change that actually measured 114 ns.

Suite **1931 → 1946 assertions / 18 suites**, `cyrius audit` exit 0.

### The split: allocation is gated, timing is recorded

⭐ **This is the design decision, and it is deliberate asymmetry.**

`alloc_used()` is **deterministic** — same tree, same byte count, any host, any
load. So allocation is a **hard gate**: `tests/allocbudget.tcyr` (new, 15
assertions) budgets every benchmarked path, and `cyrius audit` runs it.

Wall-clock on a shared CI runner is **not** deterministic. A 20% swing on an
unchanged tree is ordinary, and **a gate that cries wolf gets disabled within a
month, which is worse than no gate.** So timing is *recorded*:
`scripts/bench-check.sh` compares a run against the committed
`benches/baseline.csv` and prints a delta per benchmark, exiting 0.
`BENCH_GATE=<factor>` opts into failing — to be turned on only once recorded
history shows the runner is quiet enough to earn it.

The cost is stated rather than hidden: **2.0.5's regression would NOT have failed
this gate.** What it buys is a gate nobody will be tempted to switch off. It does
fail hard on one timing condition — a baseline benchmark that stops running,
which is a real regression rather than noise.

### The budgets

| Path | Budget | Why |
|---|---|---|
| `dcblocker_process`, `prani_rng_next_f32` | **0 B** | per-sample; one byte here is 44 KB/s retained |
| `prani_ffi_voice_set_size` | **0 B** | an RTPC a host calls per frame (pins 2.0.3's F7) |
| `emotion_evaluate` | ≤ **40 B** | one `PrEmotionOut`, which the caller receives |
| `stream_fill_buffer` | ≤ **512 B** | the audio-callback path (was 8,800 at 2.0.6) |
| `crvoice_vocalize` | **sub-linear in duration** | allocation may scale with audio produced, never with calls |

Every budget carries a **control** — a `<= N` assertion passes trivially if the
path never runs. Verified adversarially: injecting one `vec_new()` into
`dcblocker_process` fails **two** budgets, the per-sample one directly and the
stream fill it cascades into.

The `crvoice_vocalize` budget is expressed as *the marginal cost of doubling the
duration*, not an absolute byte count. That cancels fixed setup and survives any
buffer-growth change, while still catching the 2.0.9 defect — per-block overhead
being retained.

### Fixed — the baseline was gitignored

Found on the last check before release: `.gitignore`'s `*.csv` would have
excluded `benches/baseline.csv`, so CI on a fresh checkout would have failed with
*"no baseline"* — a gate that never ran. Negated explicitly, with the reason.

## [2.0.10] - The two performance defects the arc's own measurements found

Closes roadmap **2.0.9** and **2.0.10** together — both were found by the 2.0.x
arc measuring itself, and both are fixed in one tree state. Suite **1900 → 1931
assertions / 17 suites**, `cyrius audit` exit 0. No API change; output
bit-identical on every path.

### Fixed — `stream_fill_buffer` retained 8,800 bytes per call → 512

Found by 2.0.6's `streaming.cyr`, the first thing to drive the streaming path the
way a host does. At 44100 Hz with 512-sample blocks that was **~757 KB/s retained
for the life of the process** on a path the module header advertises for *"audio
callbacks (Wwise, FMOD, Godot, JACK)"*.

**94% removed.** `stream.cyr`'s own share is now **0**: the contour vec, the
`SynthesisOptions`, and a per-fill `prani_intent_modifiers` — a **fourth** source
the original analysis had missed, allocating a whole struct to read one immutable
field — are hoisted into `SynthStream`. The big one, the 8,088-byte block vec,
needed `crtract_synthesize_into` alongside the allocating wrapper (the
`species_params_into` split, following 2.0.3's F7/F8 precedent). `voice.cyr`'s
block loop reuses one buffer too, saving **~400 KB per 1 s call**.

⚠ **Residual 512 B/fill is a behaviour question, not a perf one.** One
`svara_glottal_new` per block. svara exposes `svara_glottal_set_f`, so caching the
source looks like a free win — **it is not**: a fresh source per block resets its
phase and jitter/shimmer state, a cached one carries them forward, and the audio
changes. The oracle rebuilt per block too, so rebuilding is parity-correct.
Closing it needs a decision that the continuity change is wanted.

### Fixed — `emotion_evaluate` regressed 2.2×; 37 of the 82 ns recovered

2.0.5's guards validated the same two fields **four times** per call. Split into
public checked wrappers over internal unchecked cores; `evaluate` validates once.
**154 → 114 ns**, bit-identical across a 6,451-state sweep.

**The 69 ns baseline was not reached and is not claimed.** The remaining ~45 ns is
the entry-point validation 2.0.5 correctly added. `docs/benchmarks.md` is
re-baselined at 118 ns rather than carrying a target nobody intends to hit.

### Fixed — the benchmark harness was measuring stale code

⭐ Found while verifying the above, and the sharpest of the three.
`tests/prani.bcyr` benchmarks `dist/prani.cyr` — the generated bundle,
deliberately, since that is what a consumer gets. **So a stale bundle makes the
bench report on the last bundled code.** `cyrius audit` read **155 ns** for a
change that actually measured **114 ns**: the fix ran, the suite went green, and
the benchmark disagreed with reality by 36%.

CI's step order is corrected — bundle coherence now runs **before** the audit, so
staleness fails before anything measures it. The local hazard remains and is
documented: run `cyrius distlib` before believing a bench number after editing
`src/`.

### Filed — 2.0.11: the suite still has no baseline

All three of the above were caught by a human reading numbers. `cyrius audit`
reports `1 passed, 0 failed` for the bench because **all it checks is that the
harness ran** — it compares nothing. Filed with three options and a
recommendation (gate on deterministic `alloc_used()` first; treat timing gates as
having to earn their place against a noisy shared runner).

## [2.0.8] - `rust-old/` retired

**The 2.0.x arc's destination.** The frozen Rust oracle prani was ported from —
**26 files, 4,646 lines** — is gone from the working tree. Patch-level because it
changes nothing a consumer sees: no API, no behaviour, no bundle content. What it
changes is what prani's parity guarantee *rests on*. **From here, parity is
asserted by the suites alone.**

`cyrius audit` exit 0 (17 suites, **1900 assertions**), five examples clean, with
the directory absent.

### The gate, and what it caught

The arc was preserve-first: nothing the oracle held could be lost before it went.
Every item was checked, and two of them found problems:

- **2.0.4** — all 73 Rust tests audited against the suites; 41 shortfalls closed,
  zero behavioural defects. Suite 770 → 1200.
- **2.0.6** — five runnable examples, CI-gated; `streaming.cyr` drives the full
  FFI lifecycle, which is what consumer-green was standing in for.
- **2.0.7** — the Rust comparison, captured while the oracle still built. The
  arc's only hard ordering constraint.
- **Move-aside proof** — `mv rust-old ../`, then audit + examples + `distlib`, all
  green, **before** deleting. Nothing in the build, tests, or bundle read it.
- **Reference sweep** — every live citation converted to the tag-qualified form
  from [ADR-0004](docs/adr/0004-cite-the-oracle-by-tag.md).

### Fixed — the sweep's own search pattern was too narrow

⭐ The gate said *"83 citations across 42 files"*. The real figure was **297
across 53** — 2.0.4–2.0.7 added heavily-citing docs and the number was never
re-measured. That is the same drift that produced the wrong 176 figure 2.0.4
corrected, and the second time a gate number measured once has gone stale.

Worse, and only caught by the verification pass: **`git grep rust-old` was the
wrong search.** Citations like `voice.rs:227-231` name an oracle file with **no
`rust-old/` prefix at all**, so they were invisible to it and just as
unresolvable — 77 of them, including one in `src/voice.cyr`. Most turned out to
be module names in prose (`prani's rng.rs and svara's rng.rs are identical`),
which need nothing; the line-numbered ones were converted.

Also normalised: citations written `rust-old/…:904-915 @ 2.0.3` or
`2.0.3: rust-old/…` were tag-qualified but **not paste-able into `git show`** —
which, with the directory gone, is the entire point. All 33 are now the canonical
`2.0.3:rust-old/…`.

### How to read the oracle now

```sh
git show 2.0.3:rust-old/src/voice.rs           # port era (2.0.0+)
git show 1.1.0:src/voice.rs                    # Rust era — the Rust WAS src/
git grep 'fn vocalize' 2.0.3 -- rust-old/src   # grep the whole oracle
```

The path changed at 2.0.0 because the port created `rust-old/` by moving the Rust
aside, so the era decides which form resolves. `CHANGELOG.md` entries written
before this release cite bare paths and are **left as written** — a released
changelog records what was true at the time; a note at the head of this file
carries the incantation instead.

### ⚠ Follow-up — push the tags

The recovery rule names tags, and they could not be verified against `origin`
from the dev environment. **Run `git push --tags`.** The risk is bounded rather
than open: all three oracle-bearing tags are **ancestors of `main`**, and
`origin/main` matched local `main`, so the oracle's *content* is already on
origin — only the readable tag refs may be missing, and pushing them later works
against commits origin already has. Until then, citations resolve by hash only.

## [2.0.7] - The Rust comparison, and two published figures that were wrong

The measurement that had to happen before `rust-old/` could be retired: after
2.0.8 it can only be run from a tag, against three crates.io releases staying
published. Benchmarks **4 → 18** (14 mirroring the oracle's criterion suite
one-for-one), `cyrius audit` exit 0, 1900 assertions. `rust-old/` unmodified —
built with `CARGO_TARGET_DIR` outside the repo.

### The comparison

Both harnesses on one host, one sitting, 2026-08-31. AMD Ryzen 7 5800H, single
core. Same species, same durations, same 44100 Hz.

| | Rust oracle (f32) | Cyrius port (f64) | |
|---|---:|---:|---:|
| `wolf_howl_1s` | 1.39 ms | 21.9 ms | **15.8× slower** |
| median, 13 synthesis benchmarks | | | **15.7×** |
| range | | | 9.9× – 17.4× |

The band is *tight* — every apparatus, every duration. A uniform ratio across
unrelated code paths is the signature of something systemic in the substrate,
not one slow algorithm.

⚠ **This measures two stacks as shipped, not one algorithm in two languages.**
Three things differ and only one is float width: the oracle is prani 1.1.0 on
svara 1.0.0 / naad 1.0.0 compiled by LLVM `--release`; the port is 2.0.6 on
svara **3.5.4** / naad **2.2.2** compiled by cycc. svara 3.x added work 1.0.0
never did. **It is not evidence that Cyrius is 15× slower than Rust**, and the
honest prior is that codegen dominates and float width is second.

### Fixed — two figures this project has published since the port began

| Claim | Measured |
|---|---|
| oracle does *~1000× realtime* | **719×** |
| port does *~236× realtime* | **45.6×** |

⭐ The port's 236× was an **artifact of the 8 kHz benchmark**. Per-sample cost
barely moves with rate — 533 ns/sample at 0.05 s @ 8 kHz against 497 ns/sample at
1.0 s @ 44100 Hz — so a benchmark at one eighth the rate renders one eighth the
samples and looks eight times more real-time than the library is. Neither figure
had ever been checked. `docs/benchmarks.md` now quotes **ns/sample**, or realtime
with its rate attached.

### Added — every vocal apparatus is now benchmarked

Before this, only the laryngeal path was, so **a regression in syringeal,
stridulatory, vibratile or noise-only synthesis was invisible**. The spread is
real: noise-only costs 124 ns/sample against laryngeal's 497, having no glottal
source and no formant bank.

### Found — `emotion_evaluate` regressed 2.2×, and 2.0.5 did it

⭐ The only historical-series row that moved: **69 ns → 151 ns**, on a path this
file calls a *"per-frame game-AI call"*. Caused by 2.0.5's own input-range
guards. `evaluate` calls `select_vocalization` and `select_intent`, and **each
calls both zone functions**, so the same two fields are range-checked **four
times** per call, plus the smoothing check — five validations where one would do.

The lesson is not "fewer guards". It is that 2.0.5 added 109 of them and ran the
benchmark suite green, because **the suite never compared against a baseline** —
it only checked the harness ran. A guard on a per-frame path needs a benchmark
delta attached before it ships. Filed as roadmap **2.0.10** with the fix
(validate once at the entry point; split public checked wrappers from internal
unchecked paths, as 2.0.3 did for `species_params`).

### Note — the f32 constraint this port was built on is gone

`port-audit.md` has said since 2.0.0 that *"f32 → f64 everywhere … widening is
**forced**"*. **ganita 1.1.4 ships a 23-function f32 scalar tier and is already
vendored in `lib/`**, and prani calls nothing outside it — no `tanh`, `sinh`,
`cosh`, `asin` or `acos` anywhere in `src/`. Not forced; a choice nobody had
revisited.

prani still cannot convert alone: `svara_glottal_next_sample` and
`svara_tract_process_sample` are f64 on both sides of every per-sample call.
Filed as **P0 on naad and svara** (naad first — it is the bottom of the stack)
and as roadmap **2.1.0 Lane A** here. Both P0s carry the caveat above: measure
one hot path before converting a module, because a measured "no" closes the
question for everyone downstream.

## [2.0.6] - Runnable examples, and the two defects they found

Five worked programs in [`docs/examples/`](docs/examples/), built and run by CI on
every push via [`scripts/run-examples.sh`](scripts/run-examples.sh). Suite
**1894 → 1900 assertions / 17 suites**, `cyrius audit` exit 0.

`CLAUDE.md` had advertised *"[`docs/examples/`](docs/examples/) — Runnable
examples"* over a directory containing a `.gitkeep`. More to the point:
`rust-old/` is removed in 2.0.8, and after that **a worked example is the only
executable statement of how this API is driven.**

Each example includes **`dist/prani.cyr`** — the published bundle — rather than
`src/*.cyr`, so building them exercises prani exactly as a consumer does. That
makes the examples the closest thing the project has to a consumer integration
test, which matters because neither consumer has ported up the stack.

| Example | Shows |
|---|---|
| `basic.cyr` | a wolf howl end to end, and the vec-or-negative-code convention |
| `species_tour.cyr` | one call per vocal apparatus, with parameters read back at runtime |
| `error_handling.cyr` | every failure shape, and the ADR divergences behind them |
| `streaming.cyr` | the real-time path, the allocation rule, **and the full FFI lifecycle** |
| `sequencing.cyr` | bouts, phrases, and a multi-voice chorus |

### Note — the runner is POSIX `sh`, deliberately

`scripts/run-examples.sh` is written for POSIX `sh`, not bash. CI invokes it as
`sh scripts/run-examples.sh`, which is **dash** on `ubuntu-latest`, where
`set -o pipefail` is an *error* rather than a no-op — the first version used it
(plus arrays and `shopt`) and failed CI at line 11 before running anything. It is
now free of bashisms and its shebang says `#!/bin/sh` so both invocations agree.
The trap is that `/bin/sh` on a typical dev box is a symlink to bash, so the
script passes locally and fails only in CI; there is a comment in the file saying
so. Both failure paths are verified: a build failure and a non-zero exit each
fail the gate, and the runner reports every failing example rather than stopping
at the first.

### The FFI surface has now been driven end to end

`streaming.cyr` runs `prani_ffi_voice_create` → `_stream_start` → `_stream_fill`
to completion → `_stream_is_finished` → both destroys, checking every return.
**That lifecycle is the condition the 2026-08-31 decision to drop consumer-green
from 2.0.8's gate rests on**, and nothing had ever driven it before.

It also proves the FFI is a thin adapter rather than a second implementation:
the FFI drain and the Cyrius-level drain produce **sample-for-sample identical
audio** — 0 of 2205 samples differ — for the same parameters.

### Fixed — an out-of-range species tag rendered audio and reported success

⭐ **Found by `error_handling.cyr`**, which is exactly the job a worked example is
supposed to do. 2.0.5 guarded the `voc` and `intent` tags for precisely this
reason — the oracle's enums made an out-of-range value unrepresentable, and the
port carries them as `i64` — and **missed `species`**. `species_params` falls
through to the FANTASY defaults for any unknown tag (deliberately, to preserve
totality), so:

```
crvoice_new(99)                              -> a valid voice, no error
crvoice_vocalize(that, HOWL, 44100.0, 0.05)  -> 2205 samples, no error
stream_new(that, HOWL, IDLE, 44100.0, 0.05)  -> a valid stream, no error
```

`crvoice_new` returns a pointer with no error channel, so the guard is at the
**point of use** — `crvoice_vocalize_with_intent` and `stream_new` — exactly as
[ADR-0006](docs/adr/0006-reject-non-finite-numeric-input.md) describes for the
builders. Pinned as **F13** in `tests/hardening.tcyr`, with controls at
`PRANI_SP_FANTASY` (the last valid tag) so a guard that rejected everything
could not pass.

### Filed — `stream_fill_buffer` allocates 8,800 bytes per call

Found by `streaming.cyr`. At 44100 Hz with 512-sample blocks that is 86
callbacks a second — **~757 KB/s retained for the life of the process**, on an
allocator that never frees; a one-minute creature loop retains ~45 MB. It
contradicts what `src/stream.cyr`'s own header advertises the module for:
*"suitable for real-time audio callbacks … (Wwise, FMOD, Godot, JACK)."*

Three per-call sources, all the **same pattern 2.0.3 already fixed twice** (F7,
F8). Filed as roadmap **2.0.9** rather than fixed here, to keep an examples
release from carrying a `stream.cyr` rework. It does not gate `rust-old/`'s
retirement — the oracle allocated per call too and simply had an allocator that
freed.

### Also noted

The examples surfaced two ergonomic gaps, both recorded rather than fixed
(adding public API is a minor bump, and 2.1.0 is where that belongs): there are
**no name helpers** for the species / vocalization / intent tags to match
`prani_err_name`, so every example and host log line hand-writes the same string
tables; and the **FFI has no `total_samples` getter**, so a host cannot learn the
length it should expect, and cannot account for an intent's `duration_scale`.

## [2.0.5] - Input-range validation, and a crash in the whole low-sample-rate band

Closes [ADR-0002](docs/adr/0002-deserializers-report-parse-failure.md)'s deferral
and audit findings **F10** and **F11**. Suite **1219 → 1894 assertions / 17
suites**, `cyrius audit` exit 0. **109 guards across 11 modules**, with the
accepted range of every numeric parameter written down in
[`docs/architecture/input-ranges.md`](docs/architecture/input-ranges.md) —
something the library never had.

### Fixed — CRITICAL: every sample rate in (1000, 7500] aborted the process

⭐ **The headline, and it was not prani's bug.** `crvoice_vocalize(voice, HOWL,
4000.0, …)` — an ordinary low sample rate, reachable from
`prani_ffi_stream_start` where a host passes the rate straight in — **killed the
process**. Bisected: 1000 rejected cleanly, **1001–7500 aborted**, 7501+ fine.

`crtract_new` was doing everything right. It checked `svara_tract_new`'s return
exactly as [ADR-0001](docs/adr/0001-check-svara-tract-constructor.md) requires,
and the return was a **valid pointer** — the tract simply could not survive its
own next call, for two independent reasons inside svara:

1. Every svara vowel target carries F5 = 3750 Hz, so at or below 7500 Hz svara
   fell back to a **single** 500 Hz formant and stored that one-element vec as
   the tract's record — then wrote five formants into it.
2. `svara_tract_new` stored two **unchecked naad error codes as filter
   pointers**; its fixed 600 Hz subglottal bandpass needs a rate above 1200 Hz.

Both are repaired in **svara 3.5.4**, found from here and fixed there rather than
worked around: a prani-side floor would have left the defect live for every other
svara consumer *and* rejected rates that actually work. prani bumps its pin
3.5.3 → 3.5.4 and adds an **F12** group to `tests/hardening.tcyr` that aborts on
the old tree.

**The working range widened.** 1201–7500 Hz never worked and now renders, via the
warn-then-continue path — which is the oracle's own semantics, so parity improves.

This is ADR-0001's lesson one level deeper. That ADR made the *constructor's*
return checkable; here a **later** call aborted instead of returning a code, and
no amount of checking at the constructor could see it coming.

### Fixed — HIGH: NaN was rendered as audio and returned as success

`f64_clamp(NaN, lo, hi)` is NaN — both comparisons are false — so a NaN passed
every clamp in the tree. The audit filed this as *"a silent empty buffer where an
error code belongs"*. Measured, the worst case is considerably worse:

> `crvoice_vocalize` on a **syringeal, stridulatory or vibratile** species
> returned a **full-length, all-NaN buffer as a success**. A host plays that.

It hid because the species you reach for first is the one that reports: on a
**laryngeal** species svara refuses the NaN f0 and it surfaces as an error.

Two of the audit's own notes were wrong and are corrected, both now pinned as
assertions in `tests/error.tcyr`: **infinities *are* bounded by `f64_clamp`** (so
the clamp-site hazard is NaN specifically), and **all three** non-finite values
convert to i64::MIN — not just NaN, and not saturating high.

### Fixed — F10: the chorus allocation, understated by eight orders of magnitude

The audit filed `base_samples + max_offset_samples * 2` as an i64 overflow at a
`timing_spread` around 1e14 seconds. Measured, a spread of **1e6 seconds** —
11.6 days, eight orders of magnitude below that — already demands over **700 GB**.
The reachable failure was arena exhaustion; a guard against the overflow alone
would have missed nearly the whole range. Separately, a NaN spread gave i64::MIN
and `i64::MIN * 2` **wraps to 0**, so the chorus quietly returned a
correct-length buffer — a silent *success*, not the silent-empty on file.

### Fixed — three more live crashes

- `crvoice_apply_nasal_antiformant`: `nasal_len = f64_to(len * nasal_fraction)`
  with no `min(len)` clamp — a fraction above 1.0 indexed past the buffer and
  terminated the process.
- `preset_from_json_str`: `str_data(bayan_json_v_str(…))` on a missing or
  non-string `name` dereferenced address 0.
- Five functions documented *"never fails"* became fallible, and **every in-tree
  call site was audited**: `crtract_synthesize_purr`'s two callers used the
  result as a pointer (`vec_len` on a negative code), and four `crvoice_apply_*`
  post-processing steps **discarded** their returns, silently leaving a buffer
  un-post-processed and returning it as success. All now propagate — their
  arguments cannot fail today, but "the caller already proved it" is exactly the
  assumption ADR-0001 was written about.

### Fixed — ADR-0002's deferral: deserializers reject nonsense that parses

All four hand-written codecs now range-check every field. A document with
`size_scale` 0 (it divides in `effective_f0`), a species tag out of range, or
`f0_min > f0_max` is rejected instead of producing a struct. prani's own
`to_json` output always satisfies the ranges — asserted as a control, because a
guard that rejected prani's own documents would be a worse bug than the one it
fixed.

### Filed upstream — `#derive(Serialize)`'s generated deserializer

Five of prani's codecs are **generated**, and the generated
`<Name>_from_json_str` opens with `load8(json + _p)` and no null guard, so
`X_from_json_str(0)` SIGSEGVs in any Cyrius project, and malformed input returns
the `memset`-zeroed struct as success — ADR-0002's exact defect, in codecs
ADR-0002 could not reach because they are emitted rather than written. Filed as a
cycc issue with a reproducer; not fixable from prani.

Found alongside it: **`cyrius test` reports `1 passed, 0 failed` for a `.tcyr`
whose process dies of SIGSEGV**, which is why these survived both the 2.0.3 sweep
and the 2.0.4 parity audit.

### Deferred — the builders, to 2.1.0

The 7 `crvoice_with_*` builders return `self` with no error channel, so rejecting
a NaN there is an API break. Roadmap 2.1.0 must already put a fallible return on
every constructor for the allocation contract, so the signature change happens
once, there. Until then a NaN smuggled in through a builder is caught at the
**point of use**. Written down rather than left to be rediscovered.

## [2.0.4] - Parity re-verification: the oracle's tests, not just its source

The port was verified module-by-module against `rust-old/`'s **source**. It had
never been verified against its **tests** — a different question, and the one
that decides whether 2.0.8 can safely remove the directory, because after that
parity is asserted by prani's own suites alone. This release asks it, closes what
it found, and writes down how to read the oracle once it is gone. Suite
**770 → 1200 assertions / 17 suites**; `cyrius audit` exits 0. **No file under
`src/` changed.**

### The result: zero behavioural defects

All **73** Rust `#[test]` blocks audited against the 17 Cyrius suites, one row
each, in [`docs/development/rust-test-parity.md`](docs/development/rust-test-parity.md).

| | |
|---|--:|
| ✅ Covered | 30 |
| 🟡 Partial | 30 |
| 🔴 Gap | 11 |
| ⬜ N/A (Rust-only language properties) | 2 |

The roadmap said a gap that turns out to be a real behavioural difference is a
defect and outranks everything else in the arc. **None was found.** Every
shortfall was a test gap: the port does the right thing, nothing asserted it.
Where reading both implementations was not conclusive, the verifying pass built a
throwaway probe and *measured* the port at the oracle's exact arguments.

### Fixed — the suite could not tell audio from silence

⭐ **The headline finding.** Every synthesis assertion in all 17 suites was
*non-error + exact length + all-finite*. **An all-zero buffer satisfied all
three.** No suite computed a peak, an RMS or an energy over a `crvoice_vocalize`
result — `f64_abs` appeared only inside tolerance comparisons on scalar helpers.
The oracle checks `max_amp > 0.001`, `near_energy > 10 * far_energy`,
Alarm-louder-than-Idle and shout-louder-than-whisper; none of it survived the
port. `t_max_abs` / `t_energy` helpers now exist and the oracle's own bars are
asserted.

This is the same shape as 2.0.3's lesson one level up: there, *a contract derived
from a dependency is a measurement*. Here — **an assertion that cannot fail is
not a test**, and 770 of them had been counted as if they were.

### Fixed — 6 of 13 species and 6 vocalizations never synthesized

Dog, Crow, Raptor, Crocodilian, Bee and Fantasy never reached `crvoice_vocalize`;
several never constructed a `CreatureTract` at all. SCREECH, RUMBLE, GROWL, BARK
and CHIRP were never synthesized, and the Cat+HOWL pair that drives the nasal
anti-formant and the cat formant-transition table was never built. All 13 species
now synthesize under the oracle's own selection ladder, asserted 13/13 on four
counters plus support-matrix cells so a wrong matrix cannot silently reroute a
species and still count as a success.

### Fixed — four serde roundtrips that could not fail, and eleven prefix compares

bayan's value accessors are null-safe by design (`bayan_json_v_int(0)` returns
0), so **a roundtrip asserted on a discriminant-0 value passes even if the derive
dropped the key entirely.** `CallBout` was round-tripped only with `HOWL` (= 0);
`VocalApparatus` only as Laryngeal (= 0), 1 of 5 variants. The oracle uses `Bark`
and `Dragon`. Now so does prani.

Chasing that, the systemic version: **all 11 idempotency checks were prefix
compares** — `memeq(json, json2, strlen(json))` — so a re-serialization that
*appended* a field passed. Each now asserts length equality first. This is the
same null-safe composition behind 2.0.3's HIGH finding
([ADR-0002](docs/adr/0002-deserializers-report-parse-failure.md)), surfacing in
the tests this time instead of the source.

### Fixed — edge inputs the oracle probed and the suites did not

Zero duration was never passed to any synthesis entry point. No stream was ever
drained in more than two `fill_buffer` calls, leaving the `t > 0.85`
release-boundary arm **dead in both `voice.cyr` and `stream.cyr`**. `fatigue` was
never read after more than one `record_call`, so an implementation that
**assigned instead of accumulated** would have passed. `emotion_update` was never
called on a non-default smoothing, so one that ignored the field would have
passed. `crvoice_with_jitter` and `crvoice_with_shimmer` had **zero test
callers**. No preset was ever synthesized; only 3 of 13 bout templates were even
constructed; every chorus mixed two bit-identical Wolves.

### Added — [ADR-0005](docs/adr/0005-serialized-tract-rebuilds-dsp-state.md): a serialized `CreatureTract` rebuilds svara's state

The one genuine oracle divergence in the tree that no ADR covered — it lived only
in source comments. The Rust derived `Serialize` over the whole struct, svara's
`VocalTract` included, and resumed seamlessly. svara hands prani an **opaque
handle with no state accessors**, so the port serializes eight fields and
*rebuilds* the svara tract and naad biquad from `params`. A tract serialized
mid-call resumes with **cleared filter memory**. `tests/tract.tcyr` now pins this
in both directions: the deserialized tract is bit-identical to a fresh one
carrying the restored scalars (proving rebuild), *and* differs from the
original's own continuation (proving the first assertion is not vacuous). Same
root cause as [ADR-0001](docs/adr/0001-check-svara-tract-constructor.md).

### Added — [ADR-0004](docs/adr/0004-cite-the-oracle-by-tag.md): cite the oracle by tag

2.0.8 removes `rust-old/` from the working tree, and a bare
`rust-old/src/x.rs:NN` in a comment stops resolving that day. The rule, in force
now: **a citation names a tag.** The path changed at 2.0.0 — the port *created*
`rust-old/` — so the recovery incantation differs by era, and both halves are
verified:

```sh
git show 2.0.3:rust-old/src/voice.rs   # port era (2.0.0+)
git show 1.1.0:src/voice.rs            # Rust era — the Rust WAS src/
```

### Fixed — two release-gate numbers were wrong

Both were measurements 2.0.8 depends on. The `rust-old/` citation count read
**176 across 45 files**; it had counted the vendored `lib/` bundles prani must
not modify (143 citations belonging to naad, goonj, svara and hisab) and
under-counted by exactly the 56 in `lib/naad.cyr`. The real figure is **83 across
42** maintained files. The Rust benchmark count read **10**; `criterion_group!`
registers **14**. Every number in the roadmap's facts table now carries the
command that reproduces it.

A third, found while correcting the second: `crvoice_vocalize
wolf_howl_0.05s@8k` **is not comparable** to the oracle's `wolf_howl_1s` — 400
samples against 44,100, at a sample rate that changes which svara path runs. 2.0.7
must add a matching row or drop wolf howl from the comparison.

### Fixed — CI ran three gates of five, and the contributor doc was still Rust

CI ran `deps` → `build` → `test`, so **fmt, lint, docs and bench were enforced
nowhere but contributors' machines**. It now runs `cyrius audit`, plus a **bundle
coherence** gate (`cyrius distlib` then `git diff --exit-code -- dist/`) — `dist/`
is a tracked build product that consumers build against, and it is deterministic,
so any diff is a stale commit.

`CONTRIBUTING.md` still told contributors to run `cargo fmt`, `cargo clippy` and
`cargo audit` and to install Rust 1.89+. It is now a Cyrius document.
`scripts/bench-history.sh` ran `cargo bench` into `benches/history.csv`, a path
that does not exist in this tree; it now drives `cyrius bench`, records the
measured timer floor alongside each figure, and refuses to write on a parse of
zero benchmarks.

### Decided — consumer-green does not gate `rust-old/`'s retirement

svara treated it as a hard gate. For prani it would block 2.0.8 indefinitely on
kiran/joshua ports that have no date. **Not a hard gate**, on one condition, now
a hard requirement on 2.0.6: `streaming.cyr` must drive the full FFI lifecycle
(create → start → fill to completion → is_finished → destroy). That lifecycle is
what consumer-green was standing in for, and an example runs in CI on every push,
which a consumer port does not.

### Note — one ledger row was wrong, and closing it caught that

The audit proposed asserting the cricket inter-chirp silence gap through a 0.2 s
`crvoice_vocalize` call. **That is impossible in the port and the oracle alike**:
`pos_in_chirp` is the index *within one synthesize call*, and vocalize renders in
882-sample blocks, so it never reaches `chirp_active` = 5880. The oracle's own
test passes on the per-syllable envelope, not the gap its comment names. The port
now carries the oracle's bar verbatim, and the silence arm is asserted where it
is reachable — a direct 12000-sample `crtract_synthesize_stridulatory` call, with
controls. A faithful port of an oracle quirk, and a reminder that a suggested fix
is a hypothesis until the code runs.

## [2.0.3] - The P(-1) sweep: one segfault on the primary API, and the contract that changed underneath it

Scaffold-hardening and security sweep of the whole tree, closing the one unmet
2.0.0 criterion the roadmap still carried (*"Security audit pass — deferred to a
work-loop cycle"*). **11 findings, 8 repaired, 3 accepted and written down**;
full report in [`docs/audit/2026-08-30-audit.md`](docs/audit/2026-08-30-audit.md),
three divergences from the oracle in [`docs/adr/`](docs/adr/). Suite **717 → 770
assertions / 17 suites**; `cyrius audit` **exits 0 for the first time** (fmt ·
lint · docs · tests · bench all clean). No behaviour change on any valid input:
every one of the 717 pre-existing parity assertions passes untouched.

### Fixed — CRITICAL: `crvoice_vocalize` segfaulted on any sample rate svara rejects

`rust-old`'s `VocalTract::new` returned `Self` — it could not fail, so
`CreatureTract::new` returns `Self` too. svara 3.x hardened that constructor into
a checkable negative code (svara's own ADR: *"`svara_tract_new` errors where Rust
panics"*) and it rejects **every sample rate at or below 1000 Hz**, plus negative
and non-finite ones. The port kept the oracle's shape and used the return value
as a pointer regardless:

```
crvoice_vocalize(wolf, HOWL, /*sample_rate*/ 10.0, /*duration*/ 1.0)   ->  exit 139
```

`svara_tract_set_formants_from_target(-1, target)` dereferenced `-1`. Reachable
from `crvoice_vocalize`, `crvoice_vocalize_with_intent`, the cat-purr path,
`stream_fill_buffer` and `crtract_from_json_str` — the entire public synthesis
surface. `crtract_new` now checks `svara_is_err` and returns a negative
`PRANI_ERR_*`; all five call sites propagate it
([ADR-0001](docs/adr/0001-check-svara-tract-constructor.md)).

⭐ **This is a third defect category, distinct from the two the port's parity bar
was built to catch.** It is not an inherited oracle defect and not a
transcription error: it is **a contract that changed underneath the port**. hisab
recorded the value-level form of this as *"a constant derived from a dependency
is a measurement, and it goes stale silently."* This is the same sentence one
level up — **a contract derived from a dependency is a measurement.** 2.0.2's
dependency bump checked every symbol prani calls for renames and signature
changes and found two; nothing in that check asks whether a return value still
*means* what it used to, and nothing in 717 parity assertions asks either,
because the oracle has no such failure to compare against.

### Fixed — HIGH: a JSON parse failure was indistinguishable from a successful parse

The four deserializers never looked at `bayan_json_v_parse_buf`'s result. bayan's
value accessors are all null-safe by design (`bayan_json_v_obj_get(0, k)` and
`bayan_json_v_int(0)` both return 0), so the two behaviours composed into a
silent one: **a malformed document produced a fully-formed struct with every
field 0, returned as a success.** The zeros are not inert — a `CreatureVoice`
with `size_scale` 0.0 makes `crvoice_effective_f0` divide by zero, and a
`SpeciesParams` with `f0_min == f0_max == 0` collapses every downstream clamp.

`crvoice_from_json_str`, `crtract_from_json_str`, `preset_from_json_str` and
`sequence_call_phrase_from_json_str` now return a negative `PRANI_ERR_*` on a
null or unparseable input. This **restores** the oracle's contract rather than
diverging from it — Rust's `serde_json::from_str` returns `Result`, and 2.0.1
restored the codecs without the failure half
([ADR-0002](docs/adr/0002-deserializers-report-parse-failure.md)). Field-range
validation is explicitly still out of scope and now written down as such.

### Fixed — HIGH: NoiseOnly synthesis dereferenced a filter that may be absent

`crtract_synthesize_noise` carried the comment *"noise_filter is always present
here"* while `crtract_new` populates that field only for `PRANI_APP_NOISE_ONLY`
and only `if (prani_is_err(nf) == 0)`. The invariant the comment asserted is one
the code beside it admits can be false. Now checked, returning
`PRANI_ERR_SYNTHESIS_FAILED`.

### Fixed — MEDIUM: a failed stream fill reported success

`rust-old/src/stream.rs:184` guards only the copy with `if let Ok(block)`, then
unconditionally advances `samples_rendered` and returns `to_render` — telling the
host it wrote N samples having written none, so the host plays whatever the
buffer held before (on a reused audio buffer, the previous block: an audible
repeat attributed to prani, with no error anywhere). Inherited by the port. A
failed fill now returns **0** and retires the stream, so a host draining
`while (stream_is_finished(s) == 0)` terminates instead of spinning on a failure
that repeats every call ([ADR-0003](docs/adr/0003-failed-fill-reports-zero-and-retires.md)).
A zero-length caller buffer also returns 0 immediately — it could never advance
the stream either. The same ADR records a divergence that had never been written
down: the port applies tilt and amplitude only when a block was produced, where
the oracle applies them to the stale buffer regardless.

### Fixed — MEDIUM: an RTPC setter allocated on every call, forever

`prani_ffi_voice_set_size` is a real-time parameter a host may call every frame,
per creature. It built a fresh `CreatureVoice` **and**, through `crvoice_new`, a
fresh `SpeciesParams`, copied five fields out and dropped the shell. Cyrius's
allocator is a bump arena that never frees, so **every call retained 168 bytes
permanently** — measured, not calculated: 1000 calls, `alloc_used()` before and
after, 168,000 bytes (`sizeof(CreatureVoice)` 40 + `sizeof(SpeciesParams)` 128).
At 60 fps across 100 creatures that is about 1 MB/second retained for the life of
the process. svara's own `streaming.cyr` states the rule this broke: *an
allocation inside an audio callback is not a leak that grows slowly, it is one
that ends the process.*

It now allocates **exactly 0 bytes**, asserted over 1000 calls in
`tests/hardening.tcyr`, and is proven field-for-field identical to the rebuild it
replaced. Making that possible is the one structural change in this release:
`species_params` is split into `species_params_into` (the 13-species table,
writing into a caller-owned struct) and a thin allocating wrapper, so both an
allocating and a non-allocating entry point are served by **one** copy of the
table.

⚠ The first attempt at this repair still measured **128 B/call** and the
assertion failed — `crvoice_reset_individual` had been written to call
`species_params()`, which allocates. Only the table split got it to 0. The
assertion was written before the repair was believed, which is the only reason
the shortfall was caught rather than shipped as "no longer allocates".

### Fixed — LOW: two allocation and loop-termination hardenings

- **One `SynthesisOptions` per synthesis call, not one per 20 ms block** (50 per
  second of audio, each retained forever). Nothing reads it after
  `crtract_synthesize` returns, so it is allocated once above the loop and
  re-armed per block.
- **`block_size` is floored at 1.** `(sample_rate * 0.02) as usize` is 0 below
  50 Hz and the oracle's loop (`voice.rs:227-231`) advances `rendered` by exactly
  that, so it never terminates. This is **defense in depth, not a fix for a live
  defect** — the CRITICAL guard above now rejects everything below ~1000 Hz
  before the loop is built, so nothing can reach it. Kept because a loop should
  not depend on a distant guard for termination, and asserted as arithmetic.

### Accepted, not repaired — written down rather than silently carried

- **Unchecked allocation (27 sites).** `alloc()` returns 0 on exhaustion and
  prani stores through it; `vec_push` returns -1 and no call site checks, so an
  exhausted arena silently truncates a buffer. The fix is an error return on
  every constructor in the library — a larger API decision than a repair release
  should make. On the roadmap; the two measured allocation reductions above
  attack the same problem from the other end.
- **`sequence_synthesize_chorus` length overflow** on a `timing_spread` around
  1e14 seconds, and **NaN propagation** through `f64_clamp` into svara. Both are
  bounded rather than open (`f64_to(NaN)` saturates to i64::MIN, so every
  affected loop bound is negative and simply does not run), and both belong with
  the general input-range validation deferred in ADR-0002.

### Changed — refactor

Two loop shapes had reached **three verbatim call sites each**, which is the bar
CLAUDE.md sets, so they were extracted into `dsp.cyr`:
`prani_vec_extend(dst, src)` and `prani_vec_push_zeros(v, n)`. Behaviour is
unchanged; `prani_vec_push_zeros` makes one shared edge explicit instead of
incidental — every call site derives its count from `f64_to(seconds * rate)`,
which saturates to i64::MIN on NaN, so a non-positive count must append nothing.

**Two duplications were found and deliberately left alone**, both at two
instances rather than three: `crtract_synthesize_stridulatory`'s bee branch is
byte-identical to `crtract_synthesize_vibratile`, and the `boundary_boost` block
is duplicated verbatim between `voice.cyr` and `stream.cyr`. Both are on the
roadmap for the third instance.

### Added — `tests/hardening.tcyr` (53 assertions)

One group per repaired finding, each written so it **fails on the 2.0.2 tree** —
the memory-safety groups crash the process there rather than reporting, which is
what makes them worth keeping. Every group carries controls (44100 and 8000 still
build a tract; valid JSON is still accepted; a 128-sample buffer still fills) so
it cannot pass vacuously by rejecting everything.

### Fixed — the lint and docs gates

- **7 lint warnings** in `src/vocalization.cyr:100-106` — the aligned
  `prani_intent_modifiers` dispatch table ran to 123 characters against a
  120-character limit. Rewrapped to block form; `cyrius lint` is clean tree-wide.
- **10 undocumented public functions** now documented: the four
  `*_from_json_str` deserializers (each stating its failure code), the three
  `sequence_*_new` constructors, the two `*_modifiers_make` builders, the five
  `prani_log_*` wrappers, and `main`. `cyrius audit`'s docs gate is complete, and
  with fmt and lint already clean **`cyrius audit` now exits 0** — it had exited
  1 on the docs gate since the port landed.

### Changed — the roadmap is open work only, in two arcs

A sweep for deferment language across the tree (source, tests, docs, CI, manifest
— `lib/` and `rust-old/` excluded) found **no TODO, FIXME or deferral marker in
any source file**; every open item lives in prose. Each was checked against the
tree rather than taken at its word, and three had gone stale:

- `port-audit.md` marked **sequence** 🟡 partial with *"synthesize parity test
  lands with voice/stream"*. It landed — `tests/sequence.tcyr` covers
  `CallBout::synthesize`, its error propagation, `CallPhrase::synthesize` and
  `synthesize_chorus`. Now ✅ 33, and **no partial or pending rows remain**.
- `port-audit.md`'s tract row still cited `filter_biquad_new(FILTER_BANDPASS,…)`,
  renamed by naad 2.2.0 and absorbed in 2.0.2.
- `port-audit.md`'s *"Deferred / follow-up work"* section listed two items that
  were both completed at port close-out (the distlib bundle, the 1.1.0 → 2.0.0
  bump).

One gap surfaced that nothing had recorded: **`CLAUDE.md` advertises
`docs/examples/` as "Runnable examples" and the directory holds a `.gitkeep` and
nothing else** — the same gap svara closed before its own oracle retirement.

[`roadmap.md`](docs/development/roadmap.md) is rewritten to carry **open work
only** — completed milestones are not restated there, they are in this file — and
**2.0.x is now one arc with one destination: retiring `rust-old/`** at 2.0.8,
under the preserve-first gate svara and goonj both used. **2.0.4 is a port parity
re-verification**, because the port was verified module-by-module against the
oracle's *source* and has never been checked against its *tests* — two different
questions, and svara found five real gaps when it asked the second one. Every gap
2.0.4 finds becomes a new 2.0.x item. The previously-listed items move back
behind it: input-range validation (2.0.5), runnable examples (2.0.6), benchmark
breadth and the Rust comparison (2.0.7). **2.x** keeps what changes the public
surface (the allocation-failure contract from audit F9) or is blocked outside
this repo (consumer-green), and four **watch items** carry the condition that
would schedule each.

The arc rests on facts measured rather than assumed: **73** Rust `#[test]` blocks
(72 of them in one `tests/integration.rs`) against 770 Cyrius assertions; **10**
Rust benchmarks against 4, and the Rust ten happen to cover every vocal apparatus
the Cyrius four miss; **176** `rust-old/…` citations across 45 tracked files that
become unresolvable on removal; and a recovery path that **splits by era** —
`git show 2.0.3:rust-old/src/voice.rs` works, but 1.1.0 and earlier predate the
port and carry the Rust at `src/` instead, so `git show 1.1.0:src/voice.rs` is
the form there. Both were run. `rust-old` still builds (`cargo fetch --locked`
resolves `svara 1.0.0` / `naad 1.0.0` / `hisab 1.2.0` from crates.io), which is
what makes 2.0.7's comparison possible — and is a window rather than a guarantee,
so 2.0.7 is the one item that must land before 2.0.8.

Every entry cites its source in the tree, and `state.md` now points at the
roadmap instead of duplicating it.

### Notes

- **No performance claim.** `dcblocker_process` 19 ns/sample, `prani_rng_next_f32`
  14 ns/sample, `emotion_evaluate` 69 ns/frame, `crvoice_vocalize` (wolf howl,
  0.05 s @ 8 kHz) 211 µs — every row within noise of 2.0.2. The optimization work
  in this release is measured in **bytes retained**, not nanoseconds, and both
  ends of each figure were measured with `alloc_used()`.
- ⚠ **The probe was wrong before the code was.** The first three bisect runs
  reported a hang, and the CRITICAL was very nearly filed as an infinite loop.
  Two instrument defects produced that: `${PIPESTATUS[0]}` is a bashism that
  expands to nothing under this project's zsh, so every *"exit=124 / timed out"*
  reading was fabricated by the harness rather than measured; and the probes
  printed through `print`, which was silently emitting nothing in that unit, so
  the bisect markers that would have located the fault never appeared. With `$?`
  captured directly and markers written through `test_group`'s raw syscall, the
  same input reported **exit 139 — SIGSEGV**, and the real defect was three
  frames further up than the one being chased. *Check the probe before believing
  the probe.*
- **What was checked and found clean** is recorded in the audit report rather
  than omitted: `vec_get` is bounds-checked and aborts loudly; both contour
  interpolations guard their divisor correctly (the guard class hisab has seen
  fail thirty times); the chorus modulo divisor is floored before use; the Doppler
  output length is bounded by the velocity clamp; every `prani_ffi_*` entry point
  null-checks its handle; and prani has no syscall, command-execution, path or
  network surface at all.
- `dist/prani.cyr` regenerated at v2.0.3.

## [2.0.2] - Toolchain + dependency catch-up

Maintenance release. Bumps the Cyrius pin **6.3.45 → 6.5.36** (124 toolchain
releases) and every pinned dependency to its current tag. **Two breaking upstream
renames** had to be absorbed; behaviour is otherwise unchanged. Suite **717
assertions / 16 suites**, all green — identical to 2.0.1, assertion for assertion.
`cyrius audit`: fmt gate clean, tests and bench green; it still exits 1 on the
pre-existing docs gate (10 undocumented public fns) and reports 7 pre-existing
lint warnings, neither of which this release touches — see Notes.

### Changed — toolchain

- **`[package].cyrius` `6.3.45` → `6.5.36`.**
- **`lib/` re-vendored from the 6.5.36 snapshot** (`cyrius lib sync` — the
  declared `[deps].stdlib` subset, 30 files rewritten). All 32 vendored stdlib
  files were then verified byte-identical to
  `~/.cyrius/versions/6.5.36/lib`, file by file, rather than assumed. This carries
  `lib/bayan.cyr` **1.0.4 → 1.5.2**, which is where the second rename below comes
  from.
- **112 lines reindented across 9 files** (7 in `src/`, 2 in `tests/`; 118
  changed lines total, of which 6 are the renames below). cyrius
  **6.5.28** fixed `cyrfmt`, which had never tracked parentheses: continuation
  lines inside an unclosed `(` were indented at `brace_depth * 4` regardless of
  nesting. Canonical is now 2 spaces per open-paren level. **Whitespace only** —
  `git diff -w` over `src/` and `tests/` is empty apart from the six rename lines
  below. (`cyrius fmt <file> --check` also accepts 4-space continuations; the
  `cyrius audit` fmt gate does not, which is what surfaced these.)

### Changed — dependencies

| Dep | Was | Now | |
|---|---|---|---|
| svara | 3.0.0 | **3.5.3** | glottal source · formant filter · vocal tract |
| naad | 2.1.0 | **2.2.2** | biquad (noise-only bandpass shaping) |
| hisab | 2.6.7 | **2.11.2** | `ease_in_out_smooth` (envelope curves) + transitive math |
| goonj | 2.0.0 | **2.0.4** | acoustics, referenced by the naad bundle |
| sakshi | 2.4.3 | **2.4.12** | structured logging (`prani_log_*`) |

svara 3.5.3 pins hisab 2.11.2 / naad 2.2.1 / goonj 2.0.4, and hisab 2.11.2 pins
sakshi 2.4.11 — so prani's set is coherent with what the bundles themselves pin.
Two deliberate steps past that: **naad 2.2.2** over svara's 2.2.1 (a pure bugfix
release — overlap-save streaming in `naad_convolution_process_block`; identical
dep pins, and prani touches no convolution path), and **sakshi 2.4.12** over
hisab's 2.4.11 (a lower-bound guard on `sakshi_span_enter`'s buffer). prani has
always pinned sakshi one ahead of the toolchain snapshot, so the
`./lib/ shadows version-pinned …` notice on every build is the normal state here,
not a new condition.

### Fixed — BREAKING upstream rename absorbed (`src/tract.cyr`)

naad **2.2.0** renamed all eight `FILTER_*` constants to `NAAD_FILTER_*` to clear
a flat-namespace collision with `nidhi`, which defines `FILTER_LOWPASS..NOTCH` at
identical values. prani calls exactly one of them:

- `FILTER_BANDPASS` → `NAAD_FILTER_BANDPASS` (the NoiseOnly pre-built bandpass,
  `crtract_new`)

The value is unchanged (2), so this is a compile-time break, not a behaviour
change — `tests/tract.tcyr`'s 52 assertions pass unchanged, the Snake hiss path
included.

**The other renames in that wave were checked and none reach prani.** Every
dependency symbol prani actually references was extracted from `src/` and
`tests/` and diffed against the new bundles: the naad/svara/hisab/goonj/sakshi
call set is `filter_biquad_new`, `filter_biquad_process_sample`,
`ease_in_out_smooth`, twelve `svara_*` (`svara_tract_new/_process_sample/_reset/
_set_formants_from_target/_synthesize`, `svara_glottal_new/_next_sample/
_set_breathiness/_set_jitter/_set_shimmer`, `svara_vowel_target_with_bandwidths`,
`svara_is_err`) and six `sakshi_*` — each of which resolves against the new
bundles with an unchanged signature. `ERR_* → NAAD_ERR_*` (naad 2.1.3) is inert
here because prani's codes are `PRANI_ERR_*`-prefixed and its one bare `ERR_NONE`
mention is a comment in `src/error.cyr`.

### Fixed — BREAKING stdlib rename absorbed (bayan 1.3.0, 4 call sites)

The re-vendored `lib/bayan.cyr` renames `bayan_json_v_parse_str` →
**`bayan_json_v_parse_buf`**, and the rename is load-bearing rather than
cosmetic: Cyrius routes a call `X(a, …)` to `X_str` whenever `a` is Str-typed at
the call site and `X_str` exists, so while the cstr+len form occupied that name,
every `bayan_json_v_parse(someStr)` in the ecosystem was silently rewritten into
a 1-arg call to a 2-arg function and returned 0 for valid JSON.

prani's four deserializers — `crtract_from_json_str`, `crvoice_from_json_str`,
`preset_from_json_str` and `sequence_call_phrase_from_json_str` — all called the
explicit 2-arg `(buf, len)` form, so **none of them was ever exposed to that
bug**; this is a pure rename. Signature and behaviour are identical, and the
serde roundtrip assertions in `tests/{tract,voice,preset,sequence}.tcyr` pass
unchanged.

### Fixed — two undefined functions the old bayan could not satisfy

Building at 2.0.1 emitted `undefined function 'bayan_f64_to_json'` and
`'bayan_f64_from_json'`. Neither name appears anywhere in prani's source or in
any dependency bundle, old or new — they are emitted by the compiler's
`#derive(Serialize)` expansion for f64 fields, and bayan **1.0.4 did not define
them**, so every build carried two dangling references. bayan 1.5.2 defines both.
The build is now warning-free apart from the standing `lib/` shadow notice.

This never reached prani's serde output: 2.0.1 types its f64 fields as `i64` bit
patterns precisely because the derive's float writer is lossy at 6 decimal
places, so the round-tripped values came from the integer path either way. The
dangling references were dead weight, not a wrong answer.

### Notes

- **Benchmarks re-run, no regression** (`tests/prani.bcyr`, x86_64 Linux, single
  core): `dcblocker_process` 20 ns/sample, `prani_rng_next_f32` 14 ns/sample,
  `emotion_evaluate` 70 ns/frame, `crvoice_vocalize` (Wolf howl, 0.05 s @ 8 kHz)
  214 µs/call. `emotion_evaluate` and the full synthesis path are ~16% and ~6%
  faster than the 2.0.0 figures; `docs/benchmarks.md` is updated. cyrius
  **6.5.19** taught `lib/bench.cyr` to measure the timer floor and subtract it
  from every sample — 1.31 µs on this host — so the harness now reports its own
  floor rather than the batch pattern merely amortizing it.
- **The svara behaviour changes between 3.0.0 and 3.5.3 do not move prani's
  output.** 3.5.0 collapsed the tract input delay line to two scalars, 3.3.0 gave
  the tract its own formant/nasal-place state, and 3.5.2 made
  `svara_formant_validate` test positively so NaN and ±inf are rejected — which
  closes the path through `svara_tract_set_formants_from_target`, one of prani's
  call sites. prani never passes NaN formants (species params are compile-time
  constants), and the svara-dependent suites assert structurally (non-error +
  exact length + all-finite + bit-identical determinism), by design for exactly
  this class of upstream change. All 717 assertions hold.
- **Two pre-existing conditions this release does not address**, both carried
  forward unchanged:
  - `cyrius lint` reports 7 line-length warnings in `src/vocalization.cyr`
    (lines 100-106, the aligned `prani_intent_modifiers` dispatch table). That
    file is untouched by this release. `cyrius lint` itself still exits 0.
  - `cyrius audit`'s docs gate reports 10 undocumented public fns — the four
    `*_from_json_str` deserializers, the three `sequence_*_new` constructors,
    the two `*_modifiers_make` builders and the `prani_log_*` wrappers. This
    release adds and removes no function and no doc comment (`git diff -w` over
    `src/` is the six rename lines, none of them a `fn` line), so the set is
    exactly the 2.0.1 set. It is what makes `cyrius audit` exit 1. Fixing it is
    a docs change, deliberately not bundled into a version bump.
- `dist/prani.cyr` regenerated at v2.0.2 (4,091 lines).

## [2.0.1] - Restore logging + serde

The 2.0.0 port **wrongly dropped logging and serde** — Cyrius has first-class
replacements for both, so this restores them to full parity.

### Added

- **Structured logging via sakshi** (`src/logging.cyr`, `prani_log_*`). All three
  Rust `tracing` sites are now wired for real (not dropped): `dsp` naad-error
  (`error!`), `tract` out-of-range-formant (`warn!`), and `voice` synthesis
  (`trace!` → `debug`). Always compiled in; runtime verbosity via `sakshi_set_level`.
- **serde restored on all 14 types** that had `#[derive(Serialize, Deserialize)]`
  — DcBlocker, Rng, IntentModifiers, EmotionState/Output, CallElement/Bout/Phrase,
  FatigueState/Modifiers, SpeciesParams, VoicePreset (incl. its display `name`),
  CreatureTract, CreatureVoice — via `#derive(Serialize)` + `bayan`. Each has a
  JSON `*_to_json` / `*_from_json_str` codec and a roundtrip test (**+90 assertions,
  717 total, all green**).
- **Lossless f64 serde**: f64 fields are typed `i64` so they serialize as their
  exact 64-bit bit patterns (bayan handles full 64-bit ints losslessly). The
  `#derive(Serialize)` float writer is only 6 decimal places (loses ~1 ULP on
  values like 0.7/0.15/0.05); the bit-pattern encoding roundtrips bit-exact.
  Restored `Rng` state roundtrips resume the identical stream.

### Notes

- `VoicePreset.name` (a `Cow<str>` display string) is serialized as a JSON string
  and restored — no longer stored as `0`.
- **Deviation (documented):** `CreatureTract` serde serializes the reconstructable
  state (params, rng, phase, dc_blocker, sample_rate) and rebuilds the opaque svara
  `VocalTract` + naad filter from params on deserialize — svara/naad have no Cyrius
  serialize surface. The species-determined response is bit-identical after rebuild;
  only a few samples of in-flight filter memory reset. `SynthStream` carries no
  serde (the Rust oracle never derived Serialize on it).

## [2.0.0] - Cyrius port

Complete rewrite from Rust to **Cyrius**. prani's Rust line shipped through 1.1.0;
the language port is a major break, so it lands as 2.0.0. The 3,527-line Rust
source is frozen at `rust-old/` as the parity oracle — every Cyrius module is
cross-checked against it function-for-function. Per-module ledger in
[`docs/development/port-audit.md`](docs/development/port-audit.md).

### Breaking

- **Language**: Rust crate → Cyrius library (`.cyr`). Consumers no longer add a
  `[dependencies] prani = …` Cargo entry; they include the `dist/prani.cyr`
  distlib bundle and resolve svara/naad/hisab/goonj/sakshi from their own
  `cyrius.cyml` (same pattern svara/naad use). No C ABI stability guarantee
  carried over — the `ffi` surface is re-expressed in Cyrius conventions.
- **Error handling**: the `PraniError` enum → integer codes (`PRANI_ERR_*`);
  fallible functions return `PRANI_ERR_NONE`/negative codes; `Result`/`Option`
  → code or sentinel returns (vec pointer on success, negative on error; `-1`
  for absent). `svara::error::SvaraError` mapping → `prani_from_svara`.
- **API shape**: methods → free functions (`CreatureVoice::vocalize` →
  `crvoice_vocalize`, etc.); enums → integer constants (`Species::Wolf` →
  `PRANI_SP_WOLF`); structs via `#derive(accessors)`. See the naming contract in
  the port audit.

### Changed

- Toolchain pinned via `cyrius.cyml [package].cyrius` (6.3.45). Build with
  `cyrius build src/main.cyr build/prani`; test a suite with
  `cyrius test tests/<mod>.tcyr`.
- **All 15 modules ported** (L0 → FFI): error, rng, dsp, spatial, vocalization,
  fatigue, emotion, sequence, species, bridge, tract, voice, preset, stream, ffi.
  (`math.rs`'s thin f32 transcendental wrappers fold into direct `f64_*` builtin
  + ganita calls; `lib.rs` carried no independent logic.)
- **f32 → f64** throughout (svara/naad/hisab are f64-only; widening is forced and
  improves precision). Test tolerances loosened where bit-exactness through
  svara's DSP isn't meaningful.
- **Dependencies**: svara (glottal/formant/vocal-tract), naad (biquad filters),
  hisab (`ease_in_out_smooth`), goonj, sakshi consumed as Cyrius distlib bundles.
  `thiserror`/`libm`/`criterion` dropped (`Vec` → stdlib `vec`; transcendentals
  via ganita builtins). serde and `tracing` were also dropped here — **incorrectly;
  restored in 2.0.1** via `#derive(Serialize)`+bayan and sakshi.

### Added

- **Parity test suites**: one `tests/<mod>.tcyr` per module — **627 assertions
  across 15 suites, all green** — cross-checked against the frozen Rust oracle
  (serde round-trip + Display-string tests dropped).
- **`dist/prani.cyr`** distlib bundle (15 modules, 3,760 lines, dependency-ordered,
  one flat namespace; cross-module symbol collisions audited to zero across every
  fn/struct/const). Consumers supply stdlib + svara + naad + hisab + goonj +
  sakshi. `src/main.cyr` smoke-builds and links the bundle (a Wolf howl end-to-end).
- **`resonance_seed` parity**: the 13 species seeds (an f32-bit-pattern hash in
  the oracle) are precomputed and stored, exactly matching Rust's
  `f32::to_bits`-based values (independently re-derived).
- **Hot-path benchmarks** (`cyrius bench tests/prani.bcyr`): DC blocker
  19 ns/sample, PCG32 15 ns/sample, `emotion_evaluate` 83 ns/frame, full Wolf-howl
  synthesis 227 µs (≈ 220× realtime, whole svara/naad stack). See
  [`docs/benchmarks.md`](docs/benchmarks.md).

### Removed

- `thiserror`, `libm`, `criterion` dependencies. (serde and `tracing` were also
  dropped here but that was a mistake — **restored in 2.0.1**; Cyrius provides
  both via `#derive(Serialize)`+bayan and sakshi.)
- `tracing-subscriber` dependency;
  the Cargo `std`/`naad-backend`/`logging`/`ffi` feature flags (Cyrius uses
  `cyrius.cyml` dep resolution instead).

## [1.1.0] - 2026-03-28

### Added

- `bridge` module: pure science-crate value conversions (body mass → size scale, temperature → f0 offset, threat level → intent, SPL → amplitude, wind → Doppler, f0 → species). No dependency on external science crates — consumers call bridge functions with primitive values
- `dsp` module: `DcBlocker` applied to all synthesis output paths (removes DC offset from asymmetric excitation sources); `map_naad_error` helper behind `naad-backend` feature gate
- Expanded `math.rs`: added `cos`, `exp`, `sqrt`, `powf` with std/libm dual paths (matching garjan pattern)
- naad dual code paths in `CreatureTract`: noise-only synthesis (snake) uses `naad::filter::BiquadFilter` when `naad-backend` is active, falling back to svara `FormantFilter` otherwise
- `#[must_use]` on `Species::params()`, `CallIntent::modifiers()`, `presets::all()` with descriptive messages
- `emotion` module: `EmotionState` valence/arousal model with smooth transitions. `evaluate()` maps 2D emotion space to vocalization selection, call intent, vocal effort, pitch scale, and breathiness. 9-region mapping (3×3 valence×arousal grid)
- `fatigue` module: `FatigueState` tracks vocal fatigue (pitch drift, breathiness increase, amplitude loss) and alarm habituation (unreinforced alarms lose intensity). Recovery during rest, reinforcement resets habituation
- `stream` module: `SynthStream` pull-based streaming synthesizer — yields audio blocks on demand via `fill_buffer()` or `next_block()` without full-buffer allocation. Suitable for real-time audio callbacks (Wwise, FMOD, Godot, JACK)
- `ffi` module (behind `ffi` feature gate): C FFI buffer-callback API with `extern "C"` functions — `prani_voice_create/destroy`, `prani_voice_set_effort/set_size/apply_lombard`, `prani_stream_start/fill/is_finished/destroy`. Species/vocalization/intent via integer indices
- Vocal effort parameter on `CreatureVoice` (0.0=whisper, 0.5=normal, 1.0=shout). Modulates amplitude (0.3–1.5×), spectral tilt (±3 dB/oct), and breathiness (U-shaped: breathy at extremes). Builder (`with_vocal_effort`) and real-time setter (`set_vocal_effort`)
- Lombard effect: `CreatureVoice::apply_lombard_effect(ambient_spl_db)` — involuntary vocal effort boost ~0.05 per 10 dB above 40 dB SPL baseline
- RTPC bridge functions: `pitch_scale_from_valence`, `vocal_effort_from_arousal`, `perturbation_from_urgency`, `lombard_effort_boost` — continuous parameter converters for game AI integration
- `ffi` feature flag (implies `std`)
- 22 new integration tests (72 total): vocal effort, emotion state, Lombard effect, fatigue/habituation, streaming, bridge functions, serde roundtrips for new types
- 3 new benchmarks (14 total): `wolf_howl_shout_1s`, `stream_wolf_howl_1s`, `emotion_evaluate`
- Send+Sync compile-time assertions for `EmotionState`, `EmotionOutput`, `FatigueState`, `FatigueModifiers`

### Fixed

- Removed `.unwrap()` in `FormantTransitionContour::at()` — replaced with safe match (zero-panic compliance)
- Fixed orphaned `#[inline]` attribute between `apply_am_pattern` and `vocalization_spectral_offset` doc comments
- `naad-backend` feature now implies `std` (matching garjan pattern — high-quality DSP requires stdlib)

### Changed

- DC blocker now applied to all 5 synthesis paths (laryngeal, syringeal, noise, stridulatory, vibratile, purr) — prevents DC offset accumulation from asymmetric excitation

---

## [Unreleased - pre-1.1.0]

### Added

- `naad` as optional dependency with `naad-backend` feature flag (default on), matching svara
- Non-stationary jitter/shimmer: perturbation scales with call urgency and position (stronger at boundaries and during alarm/distress)
- `Species::bout_template()`: species-specific default `CallBout` for all 13 species (e.g., dogs bark 5x at 0.25s intervals, wolves howl 3x with 2s gaps)
- 4 new voice presets: Bald Eagle, Raven, Field Cricket, American Alligator (11 total)
- Spectral envelope per vocalization: growls/rumbles darker (-2 dB/oct offset), screeches/hisses brighter (+1.5 to +2 dB/oct)
- Source-filter coupling for birds: F1 tracks toward f0 at 40% coupling strength, simulating syrinx-tract interaction
- 4 new tests: bout templates, spectral envelope, source-filter coupling, non-stationary perturbation (50 total)
- CI/CD pipeline: GitHub Actions workflows (ci.yml, release.yml) matching svara
- Makefile, rust-toolchain.toml, codecov.yml, scripts/bench-history.sh
- `spatial` module: `apply_distance_attenuation` (inverse-distance + atmospheric HF absorption), `apply_doppler_shift` (linear interpolation resampling)
- `sequence` module: `CallBout` (repeated calls with intervals), `CallPhrase` (ordered vocalization sequences), `synthesize_chorus` (multiple voices with timing spread)
- `preset` module: `VoicePreset` with 7 built-in presets (Alpha Wolf, Wolf Pup, House Cat, Kitten, Male Lion, Ancient Dragon, Young Dragon)
- `VocalApparatus::Vibratile`: new variant for bees (thoracic flight muscle vibration)
- `spectral_tilt` field on `SpeciesParams`: per-species dB/octave roll-off (lion: -6, bird: -1)
- Cat purr special-case synthesis: 25-30 Hz laryngeal muscle cycling with asymmetric waveform through vocal tract
- Formant transitions: dynamic formant changes during cat meow (nasal -> open -> closing) and wolf howl
- Cricket discrete pulse-train chirps: 3-5 pulse groups at ~30 Hz with inter-chirp silence
- Time-varying subharmonic amplitude for lion/dragon/crocodilian (peaks during middle of call)
- Deterministic chaos injection during peak intensity of roars (period-doubling roughness)
- Biphonation for canids: second independent pitch (~minor seventh) during wolf/dog howls
- Nasal resonance: anti-formant notch at ~250 Hz during nasal phases of cat meow and wolf howl
- AM patterns: bird trill rapid amplitude modulation at 20 Hz
- `#[must_use]` on `SpeciesParams`, `IntentModifiers`, `VoicePreset`
- Tracing warning when species formants fall out of svara's valid range
- 26 new integration tests (46 total), covering all new modules and features
- docs/architecture/overview.md with full data flow diagram
- docs/development/roadmap.md

### Changed

- Bee species now uses `VocalApparatus::Vibratile` (was `Stridulatory`)
- Bird species (Songbird, Crow, Raptor) have wider formant bandwidths for less defined resonances
- Crow breathiness increased (0.15 -> 0.18) for more realistic harsh/noisy calls
- Dragon fire-breath RNG seed derived from species params (was hardcoded 8888)
- Subharmonics now have time-varying envelope with chaos (was constant 0.3 amplitude sine)
- Removed unused f64 math module and unused RNG methods (poisson, next_f32_range, next_f32_unsigned)
- Removed `#[allow(dead_code)]` suppressions

### Performance

- New features add processing to the synthesis pipeline. Regressions are proportional to added complexity:
  - wolf_howl_1s: 1.29 -> 1.49 ms (+15%) — biphonation, nasal resonance, formant transitions, spectral tilt
  - wolf_alarm_howl_1s: 773 -> 1040 us (+35%) — same pipeline additions
  - lion_roar_1s: 1.64 -> 1.47 ms (-10%) — net improvement despite new subharmonic envelope + chaos
  - dragon_roar_1s: 1.52 -> 1.55 ms (+2%) — near-neutral
  - songbird_trill_500ms: 872 -> 802 us (-8%) — improved despite new AM pattern
  - snake_hiss_500ms: 519 -> 252 us (-51%) — improved (dead code removal, no new processing)
  - cricket_stridulate_300ms: 221 -> 235 us (+6%) — pulse-train replaces continuous AM

## [1.0.0] - 2026-03-27

### Added

- Initial scaffold of the prani crate
- `Species` enum: 13 species (Wolf, Dog, Cat, Lion, Songbird, Crow, Raptor, Snake, Crocodilian, Cricket, Bee, Dragon, Fantasy)
- `VocalApparatus` enum: Laryngeal, Syringeal, Stridulatory, NoiseOnly
- `SpeciesParams`: Per-species vocal parameters (f0 range, tract scale, breathiness, jitter, shimmer)
- `CreatureTract`: Species-specific vocal tract wrapping svara's VocalTract with apparatus-dependent synthesis
- `Vocalization` enum: 14 call types (Howl, Bark, Growl, Roar, Hiss, Chirp, Trill, Whine, Rumble, Purr, Yelp, Screech, Stridulate, Buzz)
- `CallIntent` enum: 7 behavioral intents (Alarm, Territorial, Mating, Distress, Idle, Threat, Social) with prosodic modifiers
- `CreatureVoice`: Species instance with individual variation (size, f0 offset, breathiness) and builder pattern
- `PraniError`: Error type with svara error conversion
- Integration tests: all species synthesize, intent modifies output, individual variation, serde roundtrips
- Criterion benchmarks: wolf howl, cat purr, cricket stridulate, dragon roar, snake hiss
- `no_std` support via `libm` + `alloc`
- Feature flags: `std` (default), `logging`, `full`
- Strict `deny.toml` matching hisab production patterns
- Send/Sync compile-time assertions on all public types
