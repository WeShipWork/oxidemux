## 1. Provider Execution Contracts

- [x] 1.1 Add provider execution request, result, outcome, failure, and metadata types in `crates/oxmux/src/provider.rs` using `CanonicalProtocolRequest`, `CanonicalProtocolResponse`, `ProviderSummary`, `AccountSummary`, `QuotaState`, and `DegradedReason` where applicable.
- [x] 1.2 Add a provider execution trait boundary that executes an explicitly selected provider/account request and returns structured provider execution outcomes without provider SDK, HTTP, OAuth, credential storage, GPUI, or app-shell dependencies.
- [x] 1.3 Add structured provider execution error coverage to `CoreError` and its `Display` implementation so failed provider execution is matchable without parsing message text.

## 2. Mock Provider Harness

- [x] 2.1 Add an in-memory mock provider harness that accepts deterministic mock outcomes for success, degraded, quota-limited, streaming-capable, and failed execution paths.
- [x] 2.2 Ensure success and degraded mock outcomes return canonical protocol responses plus provider/account metadata without performing protocol translation or outbound network calls.
- [x] 2.3 Ensure quota-limited mock outcomes reflect quota state through existing `QuotaState` and `QuotaSummary`-compatible provider/account summary data.
- [x] 2.4 Ensure streaming-capable mock outcomes report `ProviderCapability.supports_streaming = true` without implementing streaming transport.

## 3. Facade and Summary Reflection

- [x] 3.1 Re-export provider execution contracts, mock harness types, and outcome types from `crates/oxmux/src/oxmux.rs` through the public facade.
- [x] 3.2 Add helpers or constructors only where needed to construct provider/account summaries from mock provider state while avoiding app-shell-specific summary copies.
- [x] 3.3 Verify mock provider health can be represented in `ManagementSnapshot` and `CoreHealthState` using existing provider/account summary, quota, warning, and error fields.

## 4. Tests

- [x] 4.1 Add `crates/oxmux/tests/provider_execution.rs` coverage for deterministic success, degraded, quota-limited, streaming-capable, and failed mock provider outcomes.
- [x] 4.2 Add public facade tests proving provider execution contracts and mock harness types are usable through `oxmux::...` imports by direct Rust consumers.
- [x] 4.3 Add or update dependency-boundary tests to ensure `oxmux` still excludes `oxidemux`, GPUI, provider SDK, HTTP client, OAuth, keyring, secret-service, and platform credential dependencies.
- [x] 4.4 Add regression assertions that provider execution does not perform protocol translation, routing policy selection, real provider calls, credential reads, or app-shell state access.

## 5. Verification

- [x] 5.1 Run `cargo fmt --all -- --check` and fix formatting issues.
- [x] 5.2 Run `cargo clippy --workspace --all-targets --all-features -- -D warnings` and fix lint issues.
- [x] 5.3 Run `cargo check --workspace --all-targets --all-features` and fix type errors.
- [x] 5.4 Run `cargo test --workspace --all-targets --all-features` and fix failing tests.
- [x] 5.5 Run `openspec status --change "define-provider-execution-mock-harness"` and confirm the change remains apply-ready.
