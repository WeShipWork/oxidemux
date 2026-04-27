## 1. Protocol Data Model

- [x] 1.1 Expand `crates/oxmux/src/protocol.rs` with canonical request and response structures for future proxy handling.
- [x] 1.2 Add typed protocol metadata for OpenAI, Gemini, Claude, Codex, and provider-specific formats.
- [x] 1.3 Add validation constructors or methods that reject invalid required protocol metadata with `CoreError`.

## 2. Translation Boundary

- [x] 2.1 Define translation boundary traits or functions that accept typed protocol requests and responses without making outbound provider calls.
- [x] 2.2 Return structured deferred results or errors for intentionally unimplemented translation behavior.
- [x] 2.3 Ensure deferred translation behavior cannot be mistaken for successful provider translation.

## 3. Public Core Facade

- [x] 3.1 Re-export protocol request, response, metadata, and translation boundary types from `crates/oxmux/src/oxmux.rs`.
- [x] 3.2 Keep `crates/oxmux` dependency-light by avoiding provider SDK, HTTP client, OAuth, GPUI, tray, updater, and app-shell dependencies.
- [x] 3.3 Update `CoreError` only if protocol validation or deferred translation needs a new structured error variant.

## 4. Tests

- [x] 4.1 Add oxmux tests proving deterministic construction of canonical request and response shapes.
- [x] 4.2 Add oxmux tests proving typed protocol metadata maps OpenAI, Gemini, Claude, Codex, and provider-specific formats explicitly.
- [x] 4.3 Add oxmux tests proving invalid metadata returns structured errors or explicit deferred results instead of panics or silent success.
- [x] 4.4 Add public facade tests proving downstream Rust consumers can use the protocol skeleton through `oxmux` exports.

## 5. Verification

- [x] 5.1 Run `cargo fmt --all -- --check` from the repository root.
- [x] 5.2 Run `cargo clippy --workspace --all-targets -- -D warnings` from the repository root.
- [x] 5.3 Run `cargo check --workspace` from the repository root.
- [x] 5.4 Run `cargo test --workspace` from the repository root.
