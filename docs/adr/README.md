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
| [0004](0004-cite-the-oracle-by-tag.md) | Cite the Rust oracle by tag, never by working-tree path | Accepted |
| [0005](0005-serialized-tract-rebuilds-dsp-state.md) | A serialized `CreatureTract` restores prani's state and rebuilds svara's | Accepted |

0001–0003 come out of the 2.0.3 P(-1) sweep
([`docs/audit/2026-08-30-audit.md`](../audit/2026-08-30-audit.md)). Each records a
place where the port's behaviour now differs from the Rust oracle — 0001 and 0003
because the oracle's own assumption stopped holding, 0002 because the port had
dropped half of a contract the oracle did have.

0004 is different in kind: it is not a behavioural divergence but the rule for
reading the oracle after roadmap 2.0.8 removes it from the working tree.

0005 comes out of the 2.0.4 parity audit
([`docs/development/rust-test-parity.md`](../development/rust-test-parity.md)),
which found it as the one oracle divergence in the tree that no ADR covered — it
had been recorded only in source comments. Like 0001, its cause is that svara
exposes a handle whose internals prani cannot reach.
