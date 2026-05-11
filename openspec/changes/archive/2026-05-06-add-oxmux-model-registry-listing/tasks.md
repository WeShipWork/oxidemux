## 1. Model Registry Contracts

- [x] 1.1 Add a focused `crates/oxmux/src/model_registry.rs` module with typed registry entry, listed model identity, provider-native target, alias/fork metadata, provider/account applicability, capability, disabled/degraded state, and listing filter primitives.
- [x] 1.2 Keep model registry primitives headless and dependency-light, avoiding `oxidemux`, GPUI, provider SDKs, OAuth, platform credential storage, provider scraping, remote updater jobs, HTTP clients, or outbound network calls.
- [x] 1.3 Export the model registry primitives from `crates/oxmux/src/oxmux.rs` for direct Rust consumers while preserving existing public facade behavior.
- [x] 1.4 Add public Rust documentation for all new public model registry types and constructors.

## 2. Static Registry Construction

- [x] 2.1 Implement deterministic registry construction from existing `RoutingPolicy`, `ModelAlias`, `ModelRoute`, `RoutingCandidate`, provider declarations, provider summaries, and protocol family metadata.
- [x] 2.2 Add construction helpers for validated file-backed configuration so routing defaults, routing default groups, provider identifiers, account identifiers, protocol families, routing eligibility, and streaming capability can produce static registry entries.
- [x] 2.3 Preserve user-facing model identifier, resolved model identifier, provider-native model identifier, alias metadata, and fork/candidate metadata without flattening them into one string.
- [x] 2.4 Represent disabled, routing-ineligible, degraded, and unknown-provider/account states as listing metadata without performing route selection or provider execution.

## 3. Listing and Future Route Semantics

- [x] 3.1 Add deterministic listing APIs for all configured entries, visible/routable entries, disabled entries, and degraded entries without mutating routing policy state.
- [x] 3.2 Define an OpenAI-compatible `/v1/models` serialization projection over typed registry entries without wiring a full HTTP route unless it remains a thin, explicitly scoped serializer.
- [x] 3.3 Ensure model listing does not imply live provider discovery, credential validation, quota checks, upstream availability checks, or provider network access.
- [x] 3.4 Keep routing selection in `RoutingBoundary::select`; registry construction and listing may consume routing metadata but must not duplicate route-selection outcomes.

## 4. Deterministic Test Coverage

- [x] 4.1 Add `crates/oxmux/tests/model_registry.rs` coverage for constructing static registry entries from in-memory routing/provider/protocol metadata.
- [x] 4.2 Add tests proving file-backed configuration can produce deterministic registry entries with provider/account applicability, routing eligibility, streaming support, disabled/degraded metadata, and stable listing order.
- [x] 4.3 Add tests proving aliases preserve requested and resolved model identifiers, and forked candidates preserve multiple provider/account targets under one listed model identity.
- [x] 4.4 Add tests proving listing filters distinguish all configured entries, visible/routable entries, disabled entries, and degraded entries without invoking route selection or provider execution.
- [x] 4.5 Add or update dependency-boundary tests to prove model registry primitives remain free of app-shell, GPUI, provider SDK, OAuth, credential storage, provider scraping, remote updater, and outbound network dependencies.

## 5. Verification

- [x] 5.1 Run `openspec validate add-oxmux-model-registry-listing --strict` and fix any OpenSpec validation issues.
- [x] 5.2 Run `cargo fmt --all --check` and fix formatting issues.
- [x] 5.3 Run `cargo test -p oxmux` and fix model registry/core test failures.
- [x] 5.4 Run `mise run ci` to verify workspace formatting, checking, clippy, tests, and documentation checks.
