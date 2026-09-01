# 0006 — prani rejects non-finite numeric input the oracle accepted

**Status**: Accepted
**Date**: 2026-08-31

## Context

[ADR-0002](0002-deserializers-report-parse-failure.md) closed half a contract and
said so: *"Field-level validation is explicitly out of scope … Ranging every
field is a larger decision about whether prani validates its own serialized
output, and it is roadmap 2.0.5."* The 2.0.3 audit deferred **F10** and **F11**
to the same place. This is that decision.

A survey of all **328** numeric parameters on prani's public surface measured
what actually happens today. Two facts govern everything below, and both correct
what the audit had recorded:

```
f64_clamp(NaN,  lo, hi) -> NaN     NaN passes every clamp in the tree
f64_clamp(+inf, lo, hi) -> hi      infinities ARE bounded by clamp
f64_to(NaN) == f64_to(+inf) == f64_to(-inf) == i64::MIN
```

So the hazard at a clamp site is **NaN specifically**, not non-finiteness in
general; and **all three** non-finite values convert to i64::MIN, not just NaN,
which is why a loop bound derived from one goes negative and the symptom is a
silently *empty* buffer rather than a crash.

The oracle has the same holes. Rust's `f32::clamp` is documented to return NaN
for a NaN input, so `2.0.3:rust-old/src/voice.rs`'s `offset.clamp(-range, range)`
passes NaN through exactly as the port did. **This is an inherited defect class,
not a port defect** — which is precisely why it needs an ADR: fixing it is a
deliberate divergence from "matches what Rust did".

What made it worth diverging is what the measurements found on the way. The worst
case is not the audit's "silent empty buffer":

> `crvoice_vocalize` on a **syringeal, stridulatory or vibratile** species with a
> NaN reachable from any builder returned a **full-length, all-NaN buffer as a
> success**. A host plays that.

It hid because the species you would reach for first is the one that reports: on
a **laryngeal** species svara refuses the NaN f0 and it surfaces as an error.

## Decision

**Every public entry point that already returns "value or negative code" rejects
non-finite and out-of-range numeric input with a `PRANI_ERR_*`.** 109 guards
across 11 modules; the range for each parameter is written down in
[`../architecture/input-ranges.md`](../architecture/input-ranges.md).

Four rules bound it:

1. **No new error code.** The oracle's five variants already carry the range
   meaning by category — `PraniError::InvalidTract` is documented as *"a vocal
   tract parameter is out of valid range"* — so guards map onto them. A sixth
   code would diverge from the oracle's enum for no gain.
2. **No signature changes.** The 7 `crvoice_with_*` builders return `self` with
   no error channel; making them fallible is an API break every consumer must
   handle, and roadmap **2.1.0** must already put a fallible return on every
   constructor for the allocation contract. The signature change happens once,
   there. Until then a NaN smuggled in through a builder is caught at the
   **point of use** by the fallible function that consumes it.
3. **Zero stays valid.** The oracle's `test_zero_duration_synthesis` asserts a
   zero duration returns an **empty buffer, not an error**. Guards use
   `prani_is_non_negative` where zero is legal and `prani_is_positive` only where
   the value is a divisor or a rate.
4. **Reject, never clamp.** Deserializers reject out-of-range fields rather than
   clamping them to something plausible. prani's own `*_to_json` output always
   satisfies the ranges, so this only ever rejects documents prani did not write
   — asserted as a control in `tests/hardening.tcyr`.

**F10 is bounded by allocation, not by overflow.** The audit filed the chorus
length as an i64 overflow at a `timing_spread` around 1e14 seconds. Measured, a
spread of **1e6 seconds** — eight orders of magnitude below that — already demands
over **700 GB**. The reachable failure was arena exhaustion; a guard against the
overflow alone would have missed nearly the whole range. Separately, a NaN spread
gave i64::MIN and `i64::MIN * 2` **wraps to 0**, so the chorus quietly returned a
correct-length buffer — a silent *success*, not the silent-empty the audit
described.

## Consequences

- **Positive** — the all-NaN-buffer-as-success path is gone from every species,
  and a host that asks for nonsense is told so instead of being handed audio.
- **Positive** — the accepted range of every numeric parameter is now written
  down, which it never was. A consumer can read the contract instead of probing.
- **Negative** — **this is a behavioural divergence from the oracle on invalid
  input**, and it is the first one that is not forced by a dependency (0001, 0003
  and 0005 all are). A caller that fed prani NaN and got a buffer back now gets an
  error code. There is no correct existing behaviour to break — the buffer was
  NaN — but it is a change, and a consumer that ignored return codes will notice.
- **Negative** — five functions that were documented "never fails" now can
  (`crtract_synthesize_purr`, `crtract_apply_spectral_tilt`, and the three
  `crvoice_apply_*` post-processing steps). Every in-tree call site was audited
  and now propagates. Their arguments are all derived from already-validated
  params, so none can fail today — they propagate anyway, because "the caller
  already proved it" is exactly the assumption ADR-0001 was written about.
- **Neutral** — the builders remain a hole until 2.1.0, closed at the point of
  use rather than at the source. Written down in `input-ranges.md` rather than
  left to be rediscovered.

## Alternatives considered

- **Clamp out-of-range input instead of rejecting it**, matching what the
  builders do. Rejected on ADR-0001's own reasoning: it fabricates a plausible
  answer for a caller who asked for something else, and hides their bug. The
  builders clamp because that is the oracle's documented behaviour for *finite*
  input; NaN is not finite input.
- **Make the builders fallible now**, a literal reading of "every public entry
  point". Rejected as sequencing, not principle: it changes 7 signatures and
  redoes work 2.1.0 must do anyway to the same functions.
- **Add a `PRANI_ERR_INVALID_PARAM`.** Rejected — see rule 1. It reads better at
  the call site and costs a permanent divergence from the oracle's enum.
- **Sanitise NaN to the clamp's low bound.** Rejected: silently ignoring what the
  caller asked for is the same hidden-bug behaviour, one step quieter.
