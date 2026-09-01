# 0001 — `crtract_new` reports svara's rejection instead of assuming it cannot fail

**Status**: Accepted
**Date**: 2026-08-30

## Context

`2.0.3:rust-old/src/tract.rs` opens `CreatureTract::new` with:

```rust
pub fn new(params: &SpeciesParams, sample_rate: f32) -> Self {
    let mut tract = VocalTract::new(sample_rate);
```

`VocalTract::new` returned `Self`. It could not fail, so `CreatureTract::new`
could not fail either, and it returns `Self` rather than `Result`.

That contract did not survive the dependency. svara 3.x hardened the constructor
— svara's own changelog lists it as one of three deliberate divergences,
"`svara_tract_new` errors where Rust panics" — so `svara_tract_new` now returns a
**negative error code** rather than panicking, and it rejects every sample rate
at or below 1000 Hz along with negative and non-finite ones (measured:
`0, 1, 10, 49, 50, 100, 999, 1000` all return `-1`; `4000` and above return a
pointer).

The port kept the oracle's shape and used the return value as a pointer
unconditionally. `svara_tract_set_formants_from_target(-1, target)` then
dereferenced `-1`. **This is reachable from the primary public API** —
`crvoice_vocalize`, `crvoice_vocalize_with_intent`, the cat-purr path,
`stream_fill_buffer` and `crtract_from_json_str` all reach it — and the observed
result is SIGSEGV, not an error:

```
crvoice_vocalize(wolf, HOWL, /*sample_rate*/ 10.0, /*duration*/ 1.0)  ->  exit 139
```

The decision is forced because there is no valid tract to return and no way to
continue: prani cannot synthesize without one.

## Decision

`crtract_new` checks `svara_is_err` and returns a negative `PRANI_ERR_*` (via
`prani_from_svara`) instead of a `CreatureTract` pointer. Every call site checks
`prani_is_err` and propagates. This is a **signature-level divergence from the
oracle**: the Rust returns `Self`, the port returns "pointer or negative code",
which is the convention `crtract_synthesize` and `crvoice_vocalize` already use.

Two smaller decisions follow the same root — the oracle's assumptions no longer
hold — and are recorded here rather than as separate ADRs, because neither is a
choice once the above is made:

- **`block_size` is floored at 1.** `(sample_rate * 0.02) as usize` is 0 below
  50 Hz, and the oracle's loop (`2.0.3:rust-old/src/voice.rs:227-231`) advances
  `rendered` by exactly that, so it never terminates. This is **defense in
  depth, not a fix for a live defect**: the guard above rejects everything below
  ~1000 Hz before the loop is reached, so no input can now drive `block_size` to
  0. It is kept because the loop should not depend on a distant guard for
  termination.
- **`stream_fill_buffer` does not cache an error code as its tract.** The field
  is the `Option::get_or_insert_with` slot; storing `-5` in it would make the
  next call see a non-zero tract and use the code as a pointer.

## Consequences

- **Positive** — the crash is gone, and the failure is now a checkable code on a
  library whose whole error convention is checkable codes. Callers that already
  check `prani_is_err` on `crvoice_vocalize` need no change at all.
- **Negative** — `crtract_new` is no longer infallible, so every present and
  future call site must check it. Consumers calling `crtract_new` directly (it is
  public) must add a check; before this change they would have crashed instead,
  so the migration is strictly an improvement, but it is a source change.
- **Neutral** — prani now depends on svara's rejection threshold without
  restating it. That is deliberate: hardcoding "1000 Hz" here would be a
  constant derived from a dependency, which goes stale silently. The threshold is
  recorded in `tests/hardening.tcyr` as a *measurement*, with controls at 8000
  and 44100 that fail if svara ever widens or narrows the range.

## Alternatives considered

- **Validate `sample_rate` in prani before calling svara.** Rejected: it would
  duplicate svara's rule, and the duplicate is the thing that goes stale. prani
  would also have to guess the threshold, which is not documented as API.
- **Clamp the sample rate to something svara accepts.** Rejected — it fabricates
  a plausible answer for a caller who asked for something else, which is the
  exact defect class this sweep was looking for. A host that asks for 10 Hz has a
  bug, and silently giving it 4000 Hz audio hides that bug.
- **Keep returning a pointer and log the error.** Rejected: there is no pointer
  to return.
