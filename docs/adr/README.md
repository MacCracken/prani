# Architecture Decision Records

Decisions about prani — what we chose, the context, and the consequences we accept. Use these when a future reader would reasonably ask *"why did we do it this way?"*

## Conventions

- **Filename**: `NNNN-kebab-case-title.md`, zero-padded to four digits. Never renumber.
- **One decision per ADR.** If a decision supersedes a prior one, add a new ADR and set the old one's status to `Superseded by NNNN`.
- **Status lifecycle**: `Proposed` → `Accepted` → (optionally) `Superseded` or `Deprecated`.
- Use [`template.md`](template.md) as the starting point.

## ADR vs. architecture note vs. guide

| Kind | Lives in | Answers |
|---|---|---|
| ADR | `docs/adr/` | *Why did we choose X over Y?* |
| Architecture note | `docs/architecture/` | *What non-obvious constraint is true about the code?* |
| Guide | `docs/guides/` | *How do I do X?* |

## Index

| ADR | Title | Status |
|---|---|---|
| [0001](0001-check-svara-tract-constructor.md) | `crtract_new` reports svara's rejection instead of assuming it cannot fail | Accepted |
| [0002](0002-deserializers-report-parse-failure.md) | Deserializers report a parse failure instead of returning a zero-filled struct | Accepted |
| [0003](0003-failed-fill-reports-zero-and-retires.md) | A failed stream fill reports 0 written and retires the stream | Accepted |

All three come out of the 2.0.3 P(-1) sweep
([`docs/audit/2026-08-30-audit.md`](../audit/2026-08-30-audit.md)). Each records a
place where the port's behaviour now differs from `rust-old/` — 0001 and 0003
because the oracle's own assumption stopped holding, 0002 because the port had
dropped half of a contract the oracle did have.
