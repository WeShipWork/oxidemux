## 1. Streaming Response Contracts

- [x] 1.1 Replace the placeholder streaming boundary with typed `ResponseMode`, complete-response, streaming-response, stream-event, in-sequence terminal-event, cancellation-reason, and streaming-failure primitives in `crates/oxmux/src/streaming.rs`.
- [x] 1.2 Add public constructor and validation for deterministic stream event sequences so streams require exactly one terminal event represented in the event sequence and reject missing terminals, multiple terminals, and non-terminal events after termination with structured `CoreError` values.
- [x] 1.3 Ensure validation preserves insertion order and does not sort, deduplicate, synthesize event identity, or require content chunks for empty-completed or metadata-only streams.
- [x] 1.4 Keep streaming primitives owned and headless, without adding provider SDK, HTTP client, async runtime transport, OAuth, credential storage, GPUI, `oxidemux`, or live upstream dependencies.

## 2. Core Facade and Errors

- [x] 2.1 Export the new streaming response primitives from `crates/oxmux/src/oxmux.rs` for direct Rust consumers and future app-shell use.
- [x] 2.2 Add streaming-specific structured `CoreError` support for invalid stream sequences and failures that happen before a stream response exists, while preserving valid cancellation and stream-error terminal states as typed response data.
- [x] 2.3 Ensure streaming failure display text is human-readable while tests and consumers can still match structured error data without parsing strings.
- [x] 2.4 Define minimum typed cancellation reasons and stream terminal failure fields, including stable failure code and human-readable message, without provider SDK types.
- [x] 2.5 Preserve existing routing, protocol, management, provider, and usage public APIs unless a change is required to thread `ResponseMode` through provider execution.

## 3. Mock Provider Streaming Outcomes

- [x] 3.1 Extend `ProviderExecutionOutcome` and `MockProviderOutcome` in `crates/oxmux/src/provider.rs` so mocks can return `ResponseMode::Complete` or deterministic `ResponseMode::Streaming` event sequences.
- [x] 3.2 Ensure provider capability metadata reports `supports_streaming` independently from response mode: streaming mock outcomes imply support, and complete mock outcomes can still be configured as streaming-capable without real upstream streaming transport.
- [x] 3.3 Validate streaming mock outcomes through the same streaming response validation path used by direct streaming primitives.
- [x] 3.4 Keep provider/account summary reflection using existing `ProviderSummary`, `ProviderCapability`, `AccountSummary`, `QuotaState`, `AuthState`, `LastCheckedMetadata`, and `DegradedReason` types without mock-only duplicates.
- [x] 3.5 Update or replace complete-response helper APIs so callers can match response mode directly and request complete responses only when present.

## 4. Deterministic Test Coverage

- [x] 4.1 Add `crates/oxmux/tests/streaming_response.rs` coverage for complete responses, ordered stream chunks, completed streams, empty-completed streams, metadata-only streams, cancelled streams, errored streams, event order preservation, constructor validation, and invalid stream sequence validation.
- [x] 4.2 Update `crates/oxmux/tests/provider_execution.rs` to cover deterministic streaming mock outcomes, event order, terminal states, validation reuse, complete-response outcomes from streaming-capable providers, and streaming capability metadata without real provider endpoints.
- [x] 4.3 Add routing/streaming compatibility coverage showing streaming-capable provider metadata remains available to accepted routing-policy consumers without introducing `require_streaming` or other new routing selection behavior in this change.
- [x] 4.4 Update direct-use or facade coverage so public `oxmux` exports include the new streaming primitives and remain usable without launching `oxidemux`.
- [x] 4.5 Update dependency-boundary tests if needed to prove streaming response primitives remain headless and networkless, including no new async runtime, futures-stream, SSE, WebSocket, HTTP client, provider SDK, OAuth, GPUI, `oxidemux`, or credential-storage dependencies.

## 5. Verification

- [x] 5.1 Run `cargo fmt --all --check` and fix formatting issues.
- [x] 5.2 Run `cargo test -p oxmux` and fix streaming/provider/core test failures.
- [x] 5.3 Run `mise run ci` to verify workspace formatting, checking, clippy, and tests.
- [x] 5.4 Run `openspec validate add-streaming-response-abstraction --strict` and fix any OpenSpec validation issues.
