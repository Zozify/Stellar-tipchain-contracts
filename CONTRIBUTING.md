# Contributing to Stellar-tipchain-contracts

## Branching Strategy

- `main` — stable, deployable code only
- `feat/<name>` — new features
- `fix/<name>` — bug fixes
- `chore/<name>` — tooling, deps, docs

## Coding Standards

- Rust edition 2021, `#![no_std]` for contract crates
- Run `cargo fmt` and `cargo clippy -- -D warnings` before committing
- Keep contract functions minimal; move helpers to separate modules if needed

## Test Requirements

Every PR must include unit tests covering:
- The happy path for any new function
- At least one failure/panic case per function that can panic

Run tests with:
```
cargo test -p tipjar
```

## Pull Request Checklist

- [ ] `cargo fmt` applied
- [ ] `cargo clippy` passes with no warnings
- [ ] `cargo test -p tipjar` passes
- [ ] Contract builds to WASM: `cargo build -p tipjar --target wasm32v1-none --release`
- [ ] PR description explains *what* and *why*
- [ ] New storage keys documented in README if added
