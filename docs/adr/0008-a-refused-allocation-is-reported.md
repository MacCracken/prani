# 0008 — A refused allocation is reported

**Status**: Accepted
**Date**: 2026-11-17

## Context

The 2.0.3 audit's **F9** was accepted rather than repaired, because the repair is
library-wide. prani allocates from the stdlib's bump arena, and that arena refuses
in three ways, none of which a host could handle through 2.1.2:

- `alloc` returns 0, and prani stored through it. Measured on 2.1.2, refusing
  `crvoice_new`'s first request killed the process with **SIGSEGV**.
- `vec_new` returns 0, and prani used it as a vec.
- `vec_push` exits the process when a grow is refused. Measured on 2.1.2:
  **`vec: alloc failed`**, exit 1. Only `vec_push_a` returns −1 instead.

The roadmap left the shape open: a negative `PRANI_ERR_ALLOC` from every
constructor, or a single checked-allocation helper that the constructors go
through. Since then the dependencies have picked one. naad 2.3.4 ([its
ADR-0010](https://github.com/MacCracken/naad/blob/2.3.5/docs/adr/0010-five-constructors-report-a-refused-allocation.md))
made the constructors a synth builds per voice return `NAAD_ERR_ALLOCATION`.
svara 3.6.4 passes that on from `svara_glottal_new`. Both draw a line: their
other refusals stop at the null page (naad ADR-0006, svara ADR-0009).

A game engine builds voices and streams while it runs, and it is the kind of host
that can act on a refused allocation, by dropping a creature's voice rather than
the process. naad's ADR-0010 extends a list on exactly that argument.

## Decision

**Every prani constructor and entry point that allocates reports a refused
allocation as `PRANI_ERR_ALLOC` (−6), for every allocation prani makes.** Both of
the roadmap's shapes are used, because they answer different questions:

- **The contract is the negative code.** `PRANI_ERR_ALLOC` is new. The oracle's
  allocator aborted, so its enum has no variant for it. It travels the same return
  value as every other error, so a caller already testing `prani_is_err` handles
  it unchanged.
- **The mechanism is one funnel.** `prani_alloc`, `prani_vec_new` and
  `prani_vec_push` in `src/error.cyr` are the only ways prani allocates.
  `scripts/check-alloc.sh` fails CI on any raw `alloc`, `vec_new` or `vec_push`
  elsewhere in `src/`. The helpers draw from prani's own Allocator vtable. It
  serves the same global arena through `alloc`, so where memory comes from does
  not change. What changes is that there is a seam: `tests/allocrefused.tcyr`
  redefines `prani_alloc_raw` and refuses prani's requests and nobody else's.

Within that:

1. **An error handle passes through.** Every function that returns a handle, vec or
   code returns an error it is handed instead of reading it: the `with_*`
   builders, the vocalize and render entry points, `stream_new`, the sequence
   renderers, `preset_build` and `crtract_new`. So
   `crvoice_with_size(crvoice_new(sp), x)` stays safe to chain, as prani's own
   presets do.
2. **The cheapest refusal is none.** Where prani allocated scratch it never needed
   to keep, it now allocates nothing:
   - species rows read into a 128-byte stack buffer (`species_supports_vocalization`,
     `stream_new`, `preset_from_json_str`);
   - the svara target built on the stack with `svara_vowel_target_set`
     (`crtract_new`, `crtract_set_formant_blend`), where svara allocated one
     unchecked per call;
   - the vocalize loop's blend storage hoisted out of the 20 ms block loop;
   - a cat-purr stream fill rendering into the stream's reused block.

   Output is byte-identical to 2.1.2 across the 716,010-line sweep.
3. **A stream records why it retired.** [ADR-0003](0003-failed-fill-reports-zero-and-retires.md)
   asked to be revisited if a transient failure mode appeared, and a refusal is
   the candidate. **Retirement stands.** The bump arena frees nothing, so a refusal
   is not something a later fill could wait out, and a drain loop must still
   terminate. What the fill's count cannot carry is the cause, so the stream keeps
   it: `stream_last_error` (and `prani_ffi_stream_last_error`) name the code, and
   `stream_next_block`, which has an error channel, returns it. Through 2.1.2,
   `stream_next_block` handed back the zero-filled block as rendered audio, both
   for a refusal and for a sample rate svara rejects.
4. **What a dependency reports, prani passes on.**
   - naad's bandpass refusal in `crtract_new` comes back as `NAAD_ERR_ALLOCATION`,
     and prani reports it as `PRANI_ERR_ALLOC`. It is no longer folded into the
     oracle's "this species has no filter".
   - svara's glottal source reports naad's refusals as `SVARA_ERR_COMPUTATION`,
     which reaches the caller as `PRANI_ERR_SVARA`. prani cannot tell that apart
     from svara's other computation errors, and does not guess.

### The boundary

prani reports what prani allocates. **A refusal inside a dependency's own
allocation behaves as that dependency decided.** svara's own fixed-size
allocations stop at the null page by svara's ADR-0009, and `svara_tract_new`'s
`vec_push` exits. The stdlib's `str_builder` (the JSON writers) and bayan's parser
(the JSON readers) allocate on their own terms. prani does not pre-check memory it
cannot see. The 2.2.0 changes do shrink the exposure: no svara target is
allocated per call now. One `svara_glottal_new` per synthesis block remains, and
it is svara's to report.

`VEC_CAP_MAX` is unchanged: a vec past 2^28 elements still ends the process
inside the stdlib. 2.0.14 already ranges every count before the first push
(`prani_buffer_count_ok`).

## Consequences

- **Positive**: through prani's own allocations, a host survives an exhausted
  arena. Every refusal comes back as a code, and none is a fault, an exit or a
  silently partial buffer. `tests/allocrefused.tcyr` refuses each of prani's
  requests in every public path, singly and from then on. That covers every
  species × every vocalization it supports, the stream, the FFI and the decoders.
  It requires exactly `PRANI_ERR_ALLOC` each time, with floors and
  one-past-the-end controls. Eight mutants, each removing one part of this
  contract, each fail it.
- **Positive**: less retained memory, measured against 2.1.2: a 1 s howl keeps
  21,848 B less, a cat-purr stream fill 8,088 B/call → 0, and
  `species_supports_vocalization` 128 B/call → 0. `tests/allocbudget.tcyr`
  pins all three.
- **Negative**: a public surface change, and the reason this is a minor release.
  Every constructor can now return a negative code, including those that could
  not fail before (`crvoice_new`, `dcblocker_new`, `prani_rng_new`,
  `species_params`, `crtract_options_*`, the sequence and preset constructors,
  `emotion_*`, `fatigue_*`). A consumer that used them unchecked must check them.
  prani has no consumers yet.
- **Negative**: a per-sample push is now a checked call through prani's
  Allocator. It is the same `vec_push_a` the stdlib's `vec_push` already wrapped,
  plus one comparison. Measured over six alternating rounds, every synthesis
  benchmark is 0.98–1.02× 2.1.2. `emotion_evaluate`, which allocates its result
  each frame, costs about 9 ns more (69 → 78 ns). `prani_alloc` calls
  `prani_alloc_raw` directly for that reason; through the vtable it was 16 ns.
- **Neutral**: the refused-allocation suite redefines `alloc` and
  `prani_alloc_raw`, so its build carries two `duplicate fn` warnings. svara's and
  naad's suites carry the same.

## Alternatives considered

- **Fail-stop, as naad's ADR-0006 first chose.** Simplest, and it leaves the host
  no choice. A game engine is the host that has one.
- **Check each call site against stdlib `alloc` directly, with no funnel.** That is
  the same contract with no seam. A test could then refuse only by redefining
  `alloc`, which also refuses svara's unchecked requests and ends the run. The
  proof would have to locate prani's requests by position in every path. svara's
  suite does that for one constructor; for prani's whole surface, it would be the
  test that rots.
- **A process-wide abort hook** (the stdlib's `_vec_die_hook`). It runs before the
  exit, but it cannot return control to the caller, so it only changes the last
  words.
- **Pre-sizing every buffer so pushes never grow.** Correct only while every count
  is right, and silent the day one is wrong. Checked pushes stay correct either
  way.
