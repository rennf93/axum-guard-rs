# AGENTS.md
Guidance for AI agents (including Claude Code) working in this repository.

## Project Overview

axum-guard-rs is the reserved namespace for the axum adapter of the Guard ecosystem. It will wire the [guard-core-rs](https://github.com/rennf93/guard-core-rs) detection engine into axum as application-layer security middleware.

**It is currently a scaffold with no implementation.** Verify this before trusting any other description:

- `src/lib.rs` is the stock 14-line `cargo new` scaffold: one `add(left, right)` function and one trivial unit test. There is no middleware, no guard code, no placeholder types.
- `Cargo.toml` declares version 0.0.1, edition 2024, MIT, and **no dependencies at all** (not even axum or tower).
- No workflow runs any cargo command. The five workflows under `.github/` are label, greeting, stale, and issue-summary automation only.
- `README.md` status line: "Reserved namespace. Implementation pending."

- **Repository**: https://github.com/rennf93/axum-guard-rs
- **Language**: Rust, edition 2024 (requires Rust 1.85 or newer)
- **License**: MIT
- **Version**: 0.0.1 (pre-release, not published to crates.io)
- **Status**: scaffold, implementation pending

## Ecosystem Position

```
guard-core (Python)      <- Reference implementation and spec owner (specs/01-14, spec 4.0.2)
├── guard-core-rs        <- Rust detection engine (pre-1.0: CPU-bound pipeline + conformance harness)
│   ├── tower-guard-rs   <- Sibling adapter (scaffold; generic tower::Layer/Service foundation)
│   ├── axum-guard-rs    <- This repo: axum adapter (scaffold)
│   ├── actix-guard-rs   <- Sibling adapter (scaffold)
│   └── rocket-guard-rs  <- Sibling adapter (scaffold)
└── fastapi-guard, flaskapi-guard, djapi-guard, tornadoapi-guard  <- Python adapters
```

The engine crate stays framework-free (no I/O, no tokio, no framework types). This repository is the opposite side of that boundary: framework glue only. No security logic belongs here; it belongs in guard-core-rs.

## Status

Scaffold, implementation pending. Concretely, what does not exist:

- No dependency on axum, tower, or guard-core-rs
- No layer, middleware, or service implementation, no configuration type, no response mapping
- No tests beyond the stock `cargo new` stub
- No CI that compiles, tests, or lints the crate
- No published release (version 0.0.1 is a placeholder)

Do not describe this crate as functional, integrated, or published in docs, issues, or PRs.

## Intended Integration

Roadmap, not reality. The intended design, consistent with `specs/impl/rs.md` in the reference repo and the engine's current API:

1. **Dependencies**: `guard-core-rs` (facade crate, re-exports `compiler`, `preprocessor`, `semantic`) plus `axum` (and/or `tower`). Versions to be chosen when implementation starts.
2. **Middleware**: axum is tower-based, so the natural shape is either a `tower::Layer`/`Service` pair or an `axum::middleware::from_fn` handler that wraps the rest of the router. tower-guard-rs is planned as the generic tower foundation; if it lands first, this crate becomes a thin axum-specific convenience over it.
3. **Per request**: extract method, path, headers, client IP, and body; call the CPU-bound engine functions synchronously (the engine has no I/O and no tokio dependency, so it can run inside the async handler without spawning); short-circuit with a 403 response when the engine returns a threat verdict.
4. **Out of scope for now**: rate limiting state, Redis, IP intelligence, logging, and event dispatch. Those are later sections of the reference spec and are not part of guard-core-rs at 0.0.1. Do not pull them into the engine.
5. **Configuration**: no config surface exists yet (section 02 of the reference spec is not ported). Design it only when guard-core-rs provides one.

Honesty constraint: guard-core-rs at 0.0.1 implements preprocessing, semantic analysis, and pattern compilation, and lacks the 4.x pattern-table scan stage. Any integration built today is partial. Say so in design notes.

## Development Commands

No Makefile and no CI. Commands that work on the scaffold as it stands:

```bash
cargo build
cargo test
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
```

The repository's first-contribution checklist (text inside `.github/workflows/greetings.yml`) cites `cargo fmt --check`, `cargo clippy --all-features --all-targets -- -D warnings`, `cargo test --all-features`, and `RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps` as the expected bar. No workflow enforces them today; treat them as the target gate once implementation starts.

## Technology Stack

- **Rust**, edition 2024. No `rust-toolchain.toml`; any recent stable toolchain (1.85+) builds the scaffold.
- **Dependencies**: none today. Planned: `guard-core-rs` and `axum` (tower-compatible by construction).
- **Tooling**: no rustfmt.toml, clippy.toml, deny.toml, or pre-commit config yet.
- **Automation**: 5 workflows (greetings, labeler, stale, summary, sync-labels) plus `.github/labeler.yml` and `.github/labels.yml`. None of them compile code.

## Related Projects

- [guard-core-rs](https://github.com/rennf93/guard-core-rs): the engine this adapter will wire in (pre-1.0, work in progress).
- Sibling adapters: [tower-guard-rs](https://github.com/rennf93/tower-guard-rs) (generic tower foundation), [actix-guard-rs](https://github.com/rennf93/actix-guard-rs), [rocket-guard-rs](https://github.com/rennf93/rocket-guard-rs).
- [guard-core](https://github.com/rennf93/guard-core): Python reference implementation and spec owner (spec 4.0.2).
- [fastapi-guard](https://github.com/rennf93/fastapi-guard): the most mature adapter in the ecosystem, a useful reference for feature coverage.
