# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability, please report it responsibly:

1. **Do not** open a public issue
2. Email security concerns to: security@agnos.org
3. Include a detailed description and reproduction steps
4. Allow 90 days for a fix before public disclosure

## Supported Versions

Only the current release is supported. Fixes land forward; nothing is backported.

| Version | Supported | |
|---------|-----------|---|
| 2.1.1 | Yes | current release; needs cyrius 6.7.2 or later |
| 2.1.0 | No | superseded; `crtract_set_formant_blend` accepts an infinite target bandwidth and then renders NaN, through svara 3.6.0 |
| 2.0.14 | No | superseded; the same bandwidth defect, through svara 3.5.6. It is the last release that builds below cyrius 6.7.2 |
| 2.0.6 – 2.0.13 | No | superseded; `crvoice_vocalize` aborts the process past 2^28 samples on any toolchain, and on an overflowing sample count when built with cyrius 6.6.8 or later, or for aarch64 |
| 2.0.0 – 2.0.5 | No | ships defects repaired in 2.0.3, 2.0.5 and 2.0.6 |
| 1.x | No | the pre-port Rust crate, frozen in git history and unmaintained |

The 2.0.x arc repaired reachable memory-safety and silent-wrong-answer defects
in shipped versions — two CRITICALs (a `crvoice_vocalize` segfault on any sample
rate svara rejects; a process abort for *every* rate in (1000, 7500]), an
all-NaN buffer returned as success, three further process aborts and two null
dereferences. Each is written up under its release in
[`CHANGELOG.md`](CHANGELOG.md). If you are pinned below 2.0.14, upgrade rather
than asking for a patch.
