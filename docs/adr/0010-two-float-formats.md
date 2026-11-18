# 0010 — prani builds in either float format: f64, or f32

**Status**: Accepted
**Date**: 2026-11-17

## Context

Roadmap 2.3.0 was "f32 throughout". Its source was 2.0.7's comparison against the
Rust, which was f32, and ganita's f32 scalar tier, which made the f64 widening the
port was built on optional. Two facts framed it.

- **The bar is accuracy, not the Rust's bits.** The Rust (tag `2.0.3:rust-old/`)
  defined what prani does: its functions, behaviours and call patterns. It is not
  the reference for prani's numbers. Matching its f32 rounding was never the goal,
  and could not be done anyway, because its renders came from svara 1.0.0, and svara
  3.14.0 is f64 and a different engine (its ADR-0008, *The synthesis core stays f64*).
- **Both formats.** f32 is added, not substituted. A host that wants f32 gets it, a
  host that wants f64 keeps exactly what 2.2.1 gave, and prani's f32 code is there
  the day cyrius's f32 codegen gets faster.

What cyrius 6.7.18 allows, measured with probes on cycc 6.7.18:

- A parameter or local typed `f32` gets real single-precision instructions: `a * b`
  over typed f32 operands is one `mulss`, bit-exact against IEEE binary32. A compare
  over typed f32 is false against a NaN, as f64's builtins are.
- `f32` cannot be a function's return type. The interchange form is a word holding
  the binary32 bits in its low 32 bits, with the high 32 clear: what `f32_from` /
  `f32_to` and ganita's tier use.
- There is no type alias. A function-like macro is refused in an included file. So
  one source cannot be written once and typed twice by the compiler.
- `#inline` copies a single-expression body of at most two parameters into the
  caller. A body with a `var`, an `if` or a third parameter is not inlined.
- svara, naad, hisab and bayan are f64 in both builds.

## Decision

**A real is the build's float. It is an f64 unless the build defines `PRANI_F32`, and
then it is an f32.** A consumer chooses with `defines = ["PRANI_F32"]` in its
manifest's `[build]`, or `-D PRANI_F32`. Every prani value follows the choice: a sample,
a sample rate, a duration, a valence, a SpeciesParams field. svara's LOGGING define is
the precedent for a build-time option in a sibling's bundle.

### One source, two compiled forms

prani's source is written once, over the primitives in `src/real.cyr`: `pr_mul`,
`pr_lt`, `pr_from`, `pr_to64` and the rest. In the f32 build each primitive is an
`#inline` f32 operation, so `pr_mul(a, b)` is one `mulss`.

The f64 build does not compile that source. cycc 6.7.18 copies every argument of an
`#inline` replay into a slot of its own. Through the primitives the f64 build measured
**1.09× 2.2.1** across the 18 benchmarks (six alternating rounds, median of medians),
`emotion_evaluate` **1.30×** and the cricket **1.18×**. A compare tested with `== 1`
after a replay cost 2× a builtin compare. So `scripts/gen-f64.sh` writes
`src/f64/<module>.cyr` for each module that does arithmetic. That is the same text,
with each primitive spelled as the f64 builtin it stands for and each `#ifdef
PRANI_F32` block reduced to its `#else` arm. Each module is guarded so that exactly
one of the pair compiles. The generated files are committed, as `dist/` is, and
`gen-f64.sh --check` fails on a stale one.

**The f64 build is 2.2.1's arithmetic, operation for operation.** The whole-surface
sweep (716,010 lines: 404 renders, 26 documents, 953 hostile cases) is byte-identical
to tag 2.2.1, JSON included, and the benchmarks are back to 2.2.1's speed
(a median of medians of 1.001× 2.2.1 over six alternating rounds, every synthesis
benchmark within 0.992–1.019×).

### Where f32 has a choice, it takes the accurate one

The f32 build is measured against the f64 build, which is its reference.

1. **Transcendentals are computed in f64 and rounded once.** `pr_sin`, `pr_cos`,
   `pr_sqrt`, `pr_pow` and `pr_cbrt` widen, compute and narrow. sqrt is then correctly
   rounded, since 53 ≥ 2·24 + 2. The others are correctly rounded except within half an
   f64 ulp of an f32 tie.
2. **An oscillator's time and phase stay f64.** The tract's syringeal, stridulatory and
   vibratile arms, and voice's trill AM, subharmonic and biphonation, evaluate
   `sin(w·n/sr)` at sample n. In f32 that fails twice over. n is inexact past 2^24 (6.3
   minutes at 44.1 kHz). Long before that, the phase outgrows f32: a 3 kHz tone at t =
   10 s is at 1.9e5 rad, where one f32 ulp is 0.016 rad, which is phase noise at −36 dB.
   The f32 build therefore takes the time from the integer index in f64, and the phase
   from the widened rate in f64, and rounds only the sine. `tests/tract.tcyr` renders a
   cricket and a bee 1000 s into their clock against the same at 0. Both formats match
   within 1e-6; with the f32 arithmetic instead, all three cases fail.

   *Amended in 2.3.2 (audit C1, B1, B4).* The rule covers every absolute position, not
   only an oscillator's. Three places 2.3.1 missed now follow it:
   - **Spatial's Doppler resampler** computes its read position, `i × ratio`, and its
     output length in f64. In f32 the fraction lost a bit per doubling of the buffer:
     a 10 s buffer was 9.2e-3 off, and past 190 s it was nearest-neighbour.
     `tests/spatial.tcyr` resamples 441,000 samples against an f64 reference.
   - **A phase accumulator** (the purr, the laryngeal subharmonic) adds in f64 for the
     whole call and is rounded once, when it is stored. In f32 each increment was
     re-rounded to the phase's grid, a frequency error of 1.7e-5 on the purr, so a
     stream drifted out of phase without bound (0.27 cycles in 10 minutes).
   - **Tests.** The syringeal dual source has a case in the 1000 s group. Voice's
     three arms share one helper, `crvoice_tone`, which `tests/voice.tcyr` holds to
     the true sine at 1000 s: 2.8e-8 in f32, 2.4e-10 in f64. Through 2.3.1 those four
     arms could be written in f32 time and every suite still passed.
3. **A random draw stays in [0, 1).** `prani_rng_next_f32` scales a 31-bit draw by
   2^-31. Rounding those 31 bits to f32's 24-bit significand, as the Rust did, sends the
   top 64 of them to 2^31, and the draw to exactly 1.0. That is once in 2^25 draws, or
   about 13 minutes of a 44.1 kHz noise source. `tests/rng.tcyr` solves for the state
   that produces the top value.

   *Corrected in 2.3.2 (audit C3).* 2.3.1 kept the 24 high bits and said each draw was
   within one ulp of the f64 build's. Truncating is exact only in 1.0's binade: of
   1,000,000 draws, 250,282 were more than an ulp off, by up to 499,712 ulps (all within
   2^-24 absolute). Since 2.3.2 the f32 build rounds the f64 draw once and clamps it to
   1 − 2^-24, the largest f32 below 1. Every draw is the f64 draw correctly rounded,
   except the top 64 values, which land on that clamp; `tests/rng.tcyr` checks 100,000.
4. **Each format truncates its own sample count.** A count is `trunc(duration ×
   sample_rate)` in the build's arithmetic, and a constant like 0.7 is a different
   number in each format. So a length can differ by a few samples. 0.1 s × 0.7 × 8 kHz
   is 559.99… in f64, which truncates to 559, and exactly 560.0 in f32. In the sweep 21
   of 404 renders differ in length, by 1 to 3 samples, and no render changed its status.
   Forcing equal lengths would need a second, f64 copy of every constant that reaches a
   count, and in cases like this one f32's product is the exact answer.

   *Amended in 2.3.2 (audit B2).* A count is bounded in integers, after the cast. Past
   2^24 a real cannot hold every count: the nasal anti-formant compared its count with
   `pr_from(len)`, which rounds 16,777,219 up to 16,777,220, and the f32 build read past
   the buffer and exited the process.
5. **A blend stays where the exact one is.** *Added in 2.3.2 (audit B5).* The emotion
   update `v·s + t·(1 − s)` can be rounded past its target once v has all but reached it,
   in either format, and a target on a zone bound then reads the zone beyond. The exact
   blend lies between v and t, so the result is held there, which can only move it
   toward the exact value. The f32 build blends in f64 and rounds once. The f64 build's
   bits change only where the result used to pass the target.

### Documents carry f64 bit patterns in both builds

prani's JSON writes each real as an integer bit pattern, which keeps non-finite values
and every bit. The f32 build writes the f64 bits of its values, which is exact, and
reads any document by rounding once to the nearest f32. For values f32 represents, the
two builds write the same bytes (`tests/real.tcyr`), and 2.2.x documents read unchanged.
The hand-written decoders widen on write and narrow on read. The nine derive-serialized
types holding reals get the same in the f32 build through a 64-bit shadow struct. Its
derived encoder and decoders do the work under the type's own names (`T_to_json`,
`T_from_json_str`, `T_from_json`).

*Amended in 2.3.2 (audit B3).* A maximum a builder clamps to is the f32 at or below the
f64 maximum. Rounded to nearest, the f32 0.05 and 0.1 are above the decimals, so a
voice at the jitter or shimmer ceiling, written by the f32 build, was refused by the f64
build's range check. An f0 offset clamped to the range of a custom voice whose bounds
are not integers likewise came out above the exact range. Those three ceilings now round
toward zero. The decoders' bounds stay at nearest, so each build reads the other's
documents.

### Tests hold f64 exactly and f32 to a measured accuracy

Every suite runs in both formats. An expected real is stated as the f64 value it should
be. `assert_real(actual, expected, ulps, msg)` (`tests/realt.cyr`) is bit equality in
the f64 build. In the f32 build it allows `ulps` units in the last place of f32, where
`ulps` is the measured error of that computation, recorded beside it: 0 or 1 for one
rounding, up to 6 for 30 accumulated calls, 10 where 0.15 − 0.14 cancels. Where a format
decides an input or an answer itself, such as the next representable count past a limit
or a truncated length, `t_per(a64, a32)` states both. *Since 2.3.2 (audit C5)* a NaN
fails `assert_real` in both builds, and in f32 so does a word with any of its high 32
bits set, which through 2.3.1 was measured as the real in its low half.

**Measured f32 accuracy against the f64 build**, from the sweep:

- every render keeps its status, and none is non-finite;
- vocalize's 127 renders have a median relative sample error of 9.7e-7, a largest of
  1.2e-4 of peak, and RMS unchanged to 0.0000 dB;
- the 15 pinned layer levels (`tests/layers.tcyr`) are within a relative 6.4e-7;
- a single computation is 0 to 1 ulp from the f64 value, rounded.

## Consequences

- **Positive.** A consumer gets f32 or f64 from one bundle, with the f64 build exactly
  2.2.1's. When cyrius's f32 codegen improves, prani's f32 build gets it without a
  rewrite. The tests are stated as accuracy, so they say what each result must be, not
  what one build happened to produce. Error codes and reals cannot be confused in the
  f32 build, because every f32 real is a non-negative word.
- **Negative.** Two builds to test: CI runs every suite, the examples and the fuzz
  harness in both. `src/f64/` is generated and must be regenerated after any source
  change (`sh scripts/gen-f64.sh`; CI checks it). In the f32 build every svara call
  widens its inputs and narrows its output, per sample on the render path
  (the f32 build measures 1.085× the f64 build, `emotion_evaluate` 1.30×: the same
profile the primitives cost the f64 build, so the cost is the `#inline` copy, not f32
arithmetic, and it goes when cycc stops copying). A request past 2^24 samples is a count f32 cannot hold exactly. A
  derive-serialized struct's accessors in the f32 build are accessors-only, and the
  64-bit shadow is a second struct per type.
- **Neutral.** The generated twins and the primitive layer exist because cyrius has no
  type alias and `#inline` copies its arguments. If either changes, the twins can go.
  The consumer floor is measured per format. It is cyrius 6.7.2 for both: a from-scratch consumer builds there and its output is byte-identical to 6.7.18's. That held only after cycc 6.7.2's miscompile of `!` on a float builtin's result was kept out of the f64 build (a negated compare is written `== 0`; see `src/real.cyr`).
  *Since 2.3.2 (audit A1–A4)* `gen-f64.sh` refuses a source it cannot read exactly: it
  recognises a directive by its first word, fails on an unbalanced or unknown one, and
  leaves strings alone. It checks that `[lib].modules` lists each source with its twin,
  and lints the code the f32 build compiles for f64 arithmetic outside an f32 arm. Through
  2.3.1 a comment after `#else` dropped the f64 arm's last line and every gate passed.

## Alternatives considered

- **Close the lane and stay f64, as svara and naad did.** Rejected: prani is to have
  both formats.
- **Replace f64 with f32.** Rejected for the same reason. f64 stays the default and
  the reference.
- **One compiled form through the primitives in both builds.** Simplest, but measured
  1.09× on the default build (`emotion_evaluate` 1.30×), and the default build must not
  pay for the option.
- **A hand-maintained f32 copy of each module.** Two copies to keep in step by hand. The
  generator's output is checked instead.
- **Bit parity with the Rust's f32.** Not the bar: accuracy is, and the Rust's renders
  came from another svara.
- **Decimal JSON (derive's float writer).** Readable, but derive writes NaN and infinity
  as `null` and decodes `null` as 0, and a 2.2.x document's integer bit patterns would
  silently read as huge numbers.
- **Equal lengths in both formats**, by counting in f64 from f64 constants. That needs
  an f64 copy of every constant reaching a count, and it would make the f32 build
  truncate 0.1 × 0.7 × 8000 to 559 when its own arithmetic gives the exact 560.
