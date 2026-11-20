# Contributing to ATSGate

Thanks for helping. ATSGate is split by one rule: the library crates under `crates/` are **pure**
(bytes and values in, values out, no file system, network, clock or environment access) and
`atsgate-cli` is the only crate that performs I/O. Keep that boundary when you add code.

## Setup

The Rust toolchain is pinned by `rust-toolchain.toml`; rustup installs it on first use.

## Checks (all run in CI)

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check
cargo run -p xtask -- citations --check
cargo run -p xtask -- fixtures --check
cargo run -p xtask -- schemas --check
```

## Rules

- **Every claim needs a citation.** Add the source to [CITATIONS.md](CITATIONS.md) with an evidence
  grade, after opening it. Never edit the README's generated Citations section by hand; run
  `cargo run -p xtask -- citations --write`.
- **No implicit defaults.** Thresholds and options are passed explicitly by the caller. A named
  constant is fine when its doc comment says why the value was chosen.
- **Fail fast.** Return an error rather than guessing; never swallow one.
- **Tests use the real code.** Fixtures are real documents committed under `fixtures/`; a fake needs
  a comment saying why.
- **Generated files are never edited by hand.** Change the generator, then run the task with
  `--write`: `fixtures` for `fixtures/`, `schemas` for `schemas/`.
- **No test-only branches in production code.**
- **Files stay at or under 250 lines**; split by responsibility.
- **Dependencies** must pass `deny.toml` (no GPL or AGPL; permissive licenses only).

## Pull requests

Keep them focused and describe the user-visible change. By contributing you agree that your
contribution is dual-licensed under MIT OR Apache-2.0, as described in the README.
