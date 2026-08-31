# 0005 — A serialized `CreatureTract` restores prani's state and rebuilds svara's

**Status**: Accepted
**Date**: 2026-08-31

## Context

`2.0.3:rust-old/src/tract.rs:28-44` derives `Serialize, Deserialize` on the whole
struct:

```rust
pub struct CreatureTract {
    tract: VocalTract,                      // svara's — itself Serialize
    params: SpeciesParams,
    rng: Rng,
    sample_rate: f32,
    phase: f32,
    dc_blocker: DcBlocker,
    noise_filter: Option<naad::filter::BiquadFilter>,
}
```

Every field, including svara's `VocalTract` and naad's `BiquadFilter`, is *in*
the JSON. The oracle's own test asserts a whole-struct fixed point —
`assert_eq!(json, json2)` at `2.0.3:rust-old/tests/integration.rs:257` — which
cannot hold unless the nested DSP state round-trips exactly.

The port cannot do that. `svara_tract_new` hands back an **opaque handle**;
svara exposes no accessor for the filter histories behind it, and neither does
naad for the biquad. There is nothing for prani to read, so there is nothing for
it to write. This is the same root cause as
[ADR-0001](0001-check-svara-tract-constructor.md) — prani sits on the far side of
an interface that does not surface its own internals.

So `crtract_to_json` (`src/tract.cyr:171-192`) emits eight things — `sample_rate`,
`phase`, `rng_state`, `rng_inc`, `dcb_x_prev`, `dcb_y_prev`, `dcb_r`, `params` —
and `crtract_from_json_str` (`src/tract.cyr:200-219`) calls `crtract_new(params,
sample_rate)` to **rebuild** the svara tract and the naad filter from the
deserialized `SpeciesParams`, then writes the saved scalars over the rebuild.

The 2.0.4 parity audit ([`rust-test-parity.md`](../development/rust-test-parity.md))
is what promoted this from a source comment to an ADR. It was already described
at `src/tract.cyr:87-88` and `:195-197` and in a test comment, but it was the one
oracle divergence in the tree that no ADR covered — and it is the kind that a
consumer discovers at runtime rather than at compile time.

## Decision

**A serialized `CreatureTract` round-trips prani's own state exactly and
reconstructs the dependencies' DSP state from `params`.**

Specifically, and in this order:

1. `sample_rate`, `phase`, the `PrRng` state/inc, and the `DcBlocker`
   `x_prev`/`y_prev`/`r` round-trip as **exact f64 bit patterns**. Nothing is
   lost and nothing is approximated.
2. The svara `VocalTract` and the naad noise filter are **rebuilt** from the
   round-tripped `SpeciesParams`, not restored. Their *response* is bit-identical
   to the original's — it is a pure function of the params — but their **in-flight
   filter memory is cleared**.

The observable consequence, stated plainly because a consumer will hit it: a
tract serialized mid-call and restored **resumes with cleared filter memory**, so
its continuation differs from what the un-serialized original would have
produced. The oracle resumed seamlessly. The difference decays within a few
samples — it is filter history, not configuration — but it is not zero, and on a
noise-only species the naad bandpass is freshly built as well.

`tests/tract.tcyr` pins this rather than leaving it unstated: the deserialized
tract's continuation is asserted **bit-identical to a freshly built tract carrying
the same restored scalars** (proving the DSP side is rebuilt), and **different
from the original's own continuation** (proving the live memory genuinely does not
survive, so the first assertion is not vacuous).

## Consequences

- **Positive** — serialization works at all, and the part prani owns is exact
  rather than lossy. A saved voice reloads to the same species response, the same
  RNG stream, and the same phase, which is what a save-game or a network snapshot
  actually needs.
- **Positive** — the JSON is small and stable. It does not embed another
  library's internal layout, so a svara release that changes its filter
  representation cannot invalidate documents prani already wrote.
- **Negative** — **prani's serialization is not a perfect resume.** A host
  crossfading around a save point, or A/B-ing a restored tract against a live one,
  will measure a difference. That is now written down instead of surprising them.
- **Negative** — the port cannot reproduce the oracle's `json == json2`
  whole-struct fixed point over the *nested* state, because that state is not in
  the document. The port's idempotency assertion covers the eight fields it emits.
- **Neutral** — if svara ever exposes tract state accessors, this becomes a
  choice again rather than a constraint, and this ADR should be revisited. It is
  not worth asking svara for them on prani's behalf until a consumer reports the
  discontinuity as a real problem.

## Alternatives considered

- **Ask svara for state accessors and serialize the handle's contents.**
  Rejected *for now*, not on principle: it is a cross-repo API addition to solve a
  problem no consumer has yet reported, and it would couple prani's document
  format to svara's internal representation — the coupling the "Positive" note
  above says is worth avoiding. Revisit on a real report.
- **Re-run the tract through N samples of silence after restore** to warm the
  filters back up. Rejected: it fabricates state that was never measured, costs
  time on every load, and N is a guess. Clearing is honest; guessing is not.
- **Refuse to serialize `CreatureTract` at all** and expose only `SpeciesParams` +
  scalars, making the caller rebuild. Rejected: it moves the same rebuild into
  every consumer, and the oracle *did* have a `CreatureTract` codec — dropping it
  would be a parity regression rather than a documented divergence.
- **Serialize a zeroed placeholder for the svara/naad fields** so the document
  matches the oracle's shape. Rejected: a field that is always zero and always
  ignored is a lie in the wire format, and the first reader to trust it is the bug.
