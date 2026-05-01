## 1. Streaming Robustness Contracts

- [x] 1.1 Add `oxmux` streaming robustness policy types for keepalive interval, bootstrap retry count, timeout behavior, and cancellation behavior with deterministic disabled defaults.
- [x] 1.2 Add validation for streaming robustness policy fields so invalid keepalive, retry, timeout, or cancellation values return structured `CoreError` data without panics or silent clamping; enforce duration range `1..=300000` milliseconds, retry count range `0..=10`, invalid explicit duration zero, invalid numeric TOML types, timeout/cancellation cross-field rules, and file-backed policy validation failures as `CoreError::Configuration`.
- [x] 1.3 Add typed helpers backed by reserved `oxmux.` metadata keys for keepalive, timeout, committed retry summary, and retry-exhaustion stream metadata while preserving existing stream order and terminal validation rules; reject provider/custom generic metadata that uses the reserved `oxmux.` namespace outside those helpers.
- [x] 1.4 Export the new robustness policy and metadata primitives through `crates/oxmux/src/oxmux.rs` for direct Rust consumers.

## 2. File Configuration

- [x] 2.1 Extend raw and validated file configuration types to parse a strict `[streaming]` TOML table with `keepalive-interval-ms`, `bootstrap-retry-count`, `timeout-ms`, and `cancellation` fields.
- [x] 2.2 Add semantic validation for streaming keepalive interval, bootstrap retry count, timeout behavior, and cancellation behavior with structured diagnostics using canonical `streaming.*` field paths.
- [x] 2.3 Update file configuration fixtures and tests for valid streaming policy, partial/omitted/default streaming policy, explicit zero duration rejection, negative/float/string/overflow numeric rejection, invalid cancellation enum values, `timeout-ms`/`cancellation` cross-field behavior, retry-count semantics, layered merge behavior, and unknown streaming fields.
- [x] 2.4 Ensure management configuration snapshots expose active streaming policy values for app/status consumers to inspect them.

## 3. Provider Execution and Mock Outcomes

- [x] 3.1 Extend mock provider execution support to model deterministic stream attempts, including pre-event failure, zero-retry failure, retry success, retry exhaustion, idle keepalive metadata, timeout, deterministic client-disconnect cancellation, pre-event cancellation, post-event cancellation, partial-content-then-error, and clean completion.
- [x] 3.2 Implement safe bootstrap retry coordination for deterministic mock streaming execution: retry only before any stream event is emitted, treat the first emitted event as the stream commit boundary, never retry after content or metadata has been emitted, and keep failed-attempt retry observations out of delivered stream events.
- [x] 3.3 Preserve the existing error split so pre-stream retry exhaustion returns `CoreError::Streaming`, while post-partial cancellation or error remains terminal stream response data.
- [x] 3.4 Keep provider streaming capability metadata independent from the current response mode and from robustness observations.

## 4. Management and Proxy Consumer Visibility

- [x] 4.1 Surface active streaming policy plus the latest streaming timeout, cancellation, retry exhaustion, and provider stream failure state through existing or focused `oxmux` management snapshot data; use latest-outcome, last-writer-wins semantics and defer aggregate history.
- [x] 4.2 Add tests proving management snapshots can expose streaming robustness warnings, degraded reasons, provider/account/routing context when known, deterministic latest-outcome replacement, and structured errors without app-shell-specific copies.
- [x] 4.3 Preserve current minimal proxy behavior that rejects unsupported streaming responses unless a narrow deterministic serializer is intentionally added and covered by tests.
- [x] 4.4 Ensure any proxy-facing structured error body continues to expose stable error codes for unsupported stream modes, pre-stream failures, and retry exhaustion.

## 5. Deterministic Test Coverage

- [x] 5.1 Add `crates/oxmux/tests/streaming_response.rs` coverage for reserved `oxmux.` metadata keys, rejection of spoofed provider/custom `oxmux.` metadata, keepalive metadata ordering, first-event commit behavior, timeout metadata before cancellation, committed retry summary metadata, pre-event cancellation failure data, zero retry budget, and robustness metadata with existing terminal validation.
- [x] 5.2 Add `crates/oxmux/tests/provider_execution.rs` coverage for retry before first emitted event, failed-attempt metadata not leaking into delivered streams, retry exhaustion before first event, deterministic client-disconnect cancellation outcomes without live transport dependencies, pre-event cancellation, cancellation propagation as terminal stream data, deterministic timeout, provider error after partial stream, and clean completion.
- [x] 5.3 Add configuration tests for TOML streaming policy parsing, defaults, concrete numeric range validation, explicit duration zero rejection, invalid numeric type rejection, timeout/cancellation cross-field rules, validation failures, and unknown-field rejection.
- [x] 5.4 Add direct facade tests proving the new streaming robustness policy, metadata, and failure types are public and matchable.
- [x] 5.5 Add or update runtime/proxy tests only where needed to prove visible structured error behavior, without requiring a real upstream provider streaming endpoint.

## 6. Documentation and Verification

- [x] 6.1 Update Rust public documentation for all new public streaming robustness, configuration, provider mock, and management types.
- [x] 6.2 Run `openspec validate streaming-robustness-controls --strict` and fix all spec validation issues.
- [x] 6.3 Run `cargo fmt --all --check`.
- [x] 6.4 Run `cargo test -p oxmux`.
- [x] 6.5 Run `mise run ci`.
