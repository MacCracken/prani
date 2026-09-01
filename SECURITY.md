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
| 2.0.11 | Yes | current release |
| 2.0.6 – 2.0.10 | No | superseded; no known unrepaired defect |
| 2.0.0 – 2.0.5 | No | ships defects repaired in 2.0.3, 2.0.5 and 2.0.6 |
| 1.x | No | the pre-port Rust crate, frozen in git history and unmaintained |

The 2.0.x arc repaired reachable memory-safety and silent-wrong-answer defects
in shipped versions — two CRITICALs (a `crvoice_vocalize` segfault on any sample
rate svara rejects; a process abort for *every* rate in (1000, 7500]), an
all-NaN buffer returned as success, three further process aborts and two null
dereferences. Each is written up under its release in
[`CHANGELOG.md`](CHANGELOG.md). If you are pinned below 2.0.11, upgrade rather
than asking for a patch.
