## 1. Routing Policy Contracts

- [x] 1.1 Replace the placeholder routing boundary with typed routing policy, model alias, provider/account target, candidate, fallback, and availability primitives in `crates/oxmux/src/routing.rs`.
- [x] 1.2 Add typed selection request, selection result, decision mode, skipped candidate, and failure-detail primitives in `crates/oxmux/src/routing.rs`.
- [x] 1.3 Implement deterministic model alias resolution that preserves both requested and resolved model identifiers in selection results.
- [x] 1.4 Implement deterministic candidate evaluation for explicit targets, priority order, fallback enabled, fallback disabled, exhausted candidates, unavailable candidates, and degraded candidates.
- [x] 1.5 Keep routing policy and per-selection availability state separate so policies remain reusable while exhausted/degraded inputs can vary per request.

## 2. Core Facade and Errors

- [x] 2.1 Export the new routing policy primitives from `crates/oxmux/src/oxmux.rs` for direct Rust consumers.
- [x] 2.2 Add routing-specific structured `CoreError` support for invalid policy, no route, missing explicit target, exhausted candidates, and degraded-only candidates.
- [x] 2.3 Ensure routing failure display text is human-readable while tests and consumers can still match structured error data without parsing strings.
- [x] 2.4 Preserve the `oxmux` dependency boundary by avoiding GPUI, `oxidemux`, provider SDK, HTTP, OAuth, token refresh, credential storage, and live quota-fetching dependencies.

## 3. Deterministic Test Coverage

- [x] 3.1 Add `crates/oxmux/tests/routing_policy.rs` coverage for model aliasing and typed requested/resolved model results.
- [x] 3.2 Add tests proving priority order selects the first available candidate and records skipped candidates when fallback occurs.
- [x] 3.3 Add tests proving explicit provider/account targeting wins over priority fallback when available and fails structurally when missing.
- [x] 3.4 Add tests for fallback-disabled failures, exhausted candidates, degraded candidates when allowed, and degraded-only failures when degraded routing is disallowed.
- [x] 3.5 Add tests for invalid policy failures that return structured `CoreError` values without panics or silent ignores.
- [x] 3.6 Update dependency-boundary tests if needed to prove routing policy primitives remain headless and networkless.

## 4. Verification

- [x] 4.1 Run `cargo fmt --all --check` and fix formatting issues.
- [x] 4.2 Run `cargo test -p oxmux` and fix routing/core test failures.
- [x] 4.3 Run `mise run ci` to verify workspace formatting, checking, clippy, and tests.
- [x] 4.4 Run `openspec validate add-routing-policy-primitives --strict` and fix any OpenSpec validation issues.
