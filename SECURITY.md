# Security policy

## Supported versions

ATSGate has no release yet. Until v1.0, only the latest release (once one exists) and `main` are supported.

## Reporting a vulnerability

Please do **not** open a public issue. Report privately through GitHub's
[private vulnerability reporting](https://github.com/Avarok-Cybersecurity/atsgate/security/advisories/new)
for this repository. Include the affected version or commit, a description, reproduction steps and the impact
you expect. We acknowledge reports within 3 business days and aim to ship a fix or mitigation within 30 days,
coordinating disclosure with you.

## Scope

ATSGate parses untrusted documents. Particularly relevant:

- a document that makes extraction panic, hang, or allocate without bound (the extractors are pure functions
  over bytes and must fail with an error instead);
- a document that hides text from the integrity checks (white, tiny, zero-width, off-page or metadata-only
  text that is not flagged);
- anything that lets an input make the CLI or the action read or write outside the paths it was given, fetch
  a URL it was not given, or run a command.

## Design

The parsing crates perform no I/O and contain no `unsafe` code (`unsafe_code = "forbid"` workspace-wide).
Dependencies are license- and advisory-checked by `cargo deny` in CI, and every GitHub Action is pinned by
full commit SHA.
