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
| [0006](0006-reject-non-finite-numeric-input.md) | prani rejects non-finite numeric input the oracle accepted | Accepted |
| [0007](0007-the-corrected-pulse-at-the-oracles-level.md) | svara's corrected pulse, at the oracle's level | Accepted |
| [0008](0008-a-refused-allocation-is-reported.md) | A refused allocation is reported | Accepted |
| [0009](0009-renders-do-not-depend-on-how-they-are-cut.md) | A render does not depend on how it is cut | Accepted |
| [0010](0010-two-float-formats.md) | prani builds in either float format: f64, or f32 | Accepted |
| [0011](0011-a-pitch-past-nyquist-is-refused.md) | A pitch the sample rate cannot represent is refused | Accepted |
| [0012](0012-a-steady-wind-does-not-shift-the-pitch.md) | A steady wind does not shift the pitch | Accepted |
| [0013](0013-a-formant-past-nyquist-is-placed-in-band.md) | A formant past Nyquist is placed in band, not lost | Accepted |
| [0014](0014-a-render-does-not-depend-on-the-sample-rate.md) | A render's level does not depend on the sample rate | Accepted |

0001–0003 come out of the 2.0.3 P(-1) sweep
([`docs/audit/2026-08-30-audit.md`](../audit/2026-08-30-audit.md)). Each records a
place where the port's behaviour now differs from the Rust oracle — 0001 and 0003
because the oracle's own assumption stopped holding, 0002 because the port had
dropped half of a contract the oracle did have.

0004 is different in kind: it is not a behavioural divergence but the rule for
reading the oracle — in force since 2.0.8 removed it from the working tree.

0005 comes out of the 2.0.4 parity audit
([`docs/development/rust-test-parity.md`](../development/rust-test-parity.md)),
which found it as the one oracle divergence in the tree that no ADR covered — it
had been recorded only in source comments. Like 0001, its cause is that svara
exposes a handle whose internals prani cannot reach.

0006 comes out of 2.0.5's input-range validation, which is where ADR-0002's
deferred field-level checks and the audit's F10/F11 were finally settled. It is
the first divergence from the oracle that a dependency did not force.

0007 comes out of 2.1.2's dependency bump. svara 3.11.0 corrected its glottal pulse
and re-levelled it for human speech. prani keeps the corrected shape and puts it
back into the oracle's gain stage, so this is a divergence a dependency forced, as
with 0001, but the response is a calibration rather than a check.

0008 is roadmap 2.2.0's Lane B, the audit's F9 finally repaired. It is the first
public-surface change since the port: every constructor can now return
`PRANI_ERR_ALLOC`, a code the oracle's enum never had. It also revisits ADR-0003,
whose retirement rule it keeps while adding a way to ask why a stream retired.

0009 comes out of the 2.2.1 P(-1) sweep
([`docs/audit/2026-11-17-audit.md`](../audit/2026-11-17-audit.md)). It is the first
ADR to repair the oracle itself: six render defects that parity had carried over
faithfully, five of them making the audio depend on where a render was cut.

0010 is roadmap 2.3.0. It adds a second float format rather than replacing the first:
the f64 build stays 2.2.1's arithmetic, bit for bit, and an f32 build is one define
away. It also states the bar the f32 build is held to: accuracy against the f64 build,
not the Rust's f32 bits, since the Rust defined what prani does and not its numbers.

0011 and 0012 are roadmap 2.3.x, the two decisions the 2.2.1 sweep deferred, settled by
ADR-0010's bar of accuracy over the Rust's numbers. 0011 refuses a pitch at or past
Nyquist, which rendered silence or an alias as success. 0012 makes the wind's Doppler
term the physically correct 0: a steady wind does not change the pitch heard between a
source and a listener at rest.

0013 comes out of the 2.3 sweep
([`docs/audit/2026-11-17-2.3-audit.md`](../audit/2026-11-17-2.3-audit.md)). It extends
ADR-0009's placement of prani's estimated formants to the species' own: a formant past
Nyquist made svara refuse all five, and the Rust, and prani through 2.3.1, rendered
svara's default vowel as the species, and discarded each block's refusal.

0014 comes out of verifying 0013, and is older than 2.3. prani's levels are calibrated
at 44.1 kHz (ADR-0007), and four stages, two of them svara's, had a response fixed per
sample: a render at 8 kHz was 19 dB louder than the same call at 44.1 kHz. prani now
holds each to its 44.1 kHz response, through what svara exposes.
