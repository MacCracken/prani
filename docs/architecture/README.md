# Architecture notes

Non-obvious constraints, quirks, and invariants that a reader cannot derive from the code alone. Numbered chronologically — never renumber.

Not decisions (those live in [`../adr/`](../adr/)) and not guides (those live in [`../guides/`](../guides/)). An item here describes *how the world is*, not *what we chose* or *how to do something*.

## Items

_Empty. Add a numbered entry (`001-kebab-case-title.md`) the first time the code has a non-obvious invariant a reader can't derive. Do not write entries for decisions — those are ADRs._

- [`input-ranges.md`](input-ranges.md) — every public entry point's accepted
  numeric range and the `PRANI_ERR_*` it returns outside it (milestone 2.0.5,
  [ADR-0006](../adr/0006-reject-non-finite-numeric-input.md)).
