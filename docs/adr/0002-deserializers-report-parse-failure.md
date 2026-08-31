# 0002 — Deserializers report a parse failure instead of returning a zero-filled struct

**Status**: Accepted
**Date**: 2026-08-30

## Context

prani's four deserializers — `crvoice_from_json_str`, `crtract_from_json_str`,
`preset_from_json_str`, `sequence_call_phrase_from_json_str` — each opened with:

```
var root = bayan_json_v_parse_buf(json, strlen(json));
```

and never looked at `root`. `bayan_json_v_parse_buf` returns 0 on a parse
failure, and every bayan value accessor is null-safe by design:
`bayan_json_v_obj_get(0, key)` returns 0, and `bayan_json_v_int(0)` returns 0.

The two behaviours compose into a silent one. A malformed document produced a
**fully-formed struct with every field 0** and returned it as a success. Nothing
in the signature or the return value distinguished it from a real parse. The
zeros are not inert, either: a `CreatureVoice` with `size_scale` = 0.0 makes
`crvoice_effective_f0` divide by zero, and a `SpeciesParams` with
`f0_min == f0_max == 0` collapses every downstream clamp.

The oracle did not have this problem. Rust's `serde_json::from_str` returns
`Result`, and the 2.0.0 port dropped serde entirely; 2.0.1 restored the codecs
via `#derive(Serialize)` + bayan but did not restore the failure half of the
contract.

## Decision

Each of the four checks both the null input and the parse result, and returns a
negative `PRANI_ERR_*` on failure:

| Function | Code on failure |
|---|---|
| `crvoice_from_json_str` | `PRANI_ERR_INVALID_SPECIES` |
| `crtract_from_json_str` | `PRANI_ERR_INVALID_TRACT` |
| `preset_from_json_str` | `PRANI_ERR_INVALID_SPECIES` |
| `sequence_call_phrase_from_json_str` | `PRANI_ERR_INVALID_VOCALIZATION` |

This **restores** the oracle's contract rather than diverging from it: `Result`
becomes "pointer or negative code", which is what every other fallible function
in the port already returns.

Field-level validation is explicitly **out of scope**. A document that parses but
carries nonsense values still produces a struct; only a failure to parse is
reported. Ranging every field is a larger decision about whether prani validates
its own serialized output, and it is on the roadmap rather than smuggled in here.

## Consequences

- **Positive** — a caller can tell a failure from a success, which it could not
  before. The division-by-zero path through `effective_f0` is no longer reachable
  from malformed input.
- **Negative** — callers must now check `prani_is_err` on a deserializer's
  return. Any consumer that fed these untrusted input was previously getting
  silent garbage, so there is no correct existing behaviour to break.
- **Neutral** — the guard is at the parse boundary only, so a well-formed
  document with out-of-range values is still accepted. That gap is now written
  down instead of being an accident.

## Alternatives considered

- **Return a struct populated with species defaults on failure.** Rejected for
  the same reason as clamping in ADR-0001: it is a fabricated plausible answer.
- **Validate every field's range as well.** Deferred, not rejected — see the
  roadmap. Doing it here would have mixed a much larger behavioural change into a
  repair release, and the field ranges are not all documented.
