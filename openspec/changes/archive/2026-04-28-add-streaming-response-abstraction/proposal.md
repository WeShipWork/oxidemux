## Why

Future provider adapters and local proxy routes need one typed `oxmux` contract for non-streaming responses, chunk/SSE-style streaming events, completion, cancellation, and stream errors before real provider streaming endpoints are added.

Issue #6 requires this now so provider execution mocks, protocol adapters, and routing/proxy work can test streaming behavior deterministically without introducing network transports, provider SDKs, GPUI, or app-shell dependencies.

## What Changes

- Replace the placeholder streaming boundary with typed headless `oxmux` response primitives for complete non-streaming responses and ordered streaming events, using one `ResponseMode` envelope for complete versus streaming delivery.
- Add explicit terminal stream states for completed, cancelled, and errored streams so callers never infer cancellation or failure from a dropped iterator/task.
- Extend provider execution mocks so tests can emit deterministic chunk/SSE-style event sequences and terminal events without contacting real upstream providers.
- Surface invalid stream construction and pre-stream execution failures through structured `CoreError` values, while valid delivered cancelled or errored streams remain typed terminal response data that consumers can match without parsing display text.
- Add deterministic networkless `oxmux` tests for non-streaming responses, stream chunks, stream completion, stream cancellation, stream errors, and mock-provider event ordering.
- Preserve the accepted `oxmux-routing-policy` semantics without adding streaming-specific route selection in this change, while making streaming capability metadata available to routing/proxy consumers that need to inspect streaming-capable providers.
- Keep HTTP proxy routes, real upstream streaming endpoints, provider SDK integration, OAuth/token refresh, GPUI, tray, credential storage, and app-shell behavior outside this change.

## Capabilities

### New Capabilities

- `oxmux-streaming-response`: Typed non-streaming and streaming response primitives, deterministic stream event ordering, explicit stream terminal states, and streaming failure contracts.

### Modified Capabilities

- `oxmux-core`: Public facade and core error requirements expand from reserving streaming ownership to exposing typed streaming response primitives and structured streaming failures.
- `oxmux-provider-execution`: Mock provider execution requirements expand from reporting streaming capability metadata to producing deterministic streaming event sequences in tests.

## Impact

- Affected crate: `crates/oxmux` only.
- Affected modules: `streaming`, `provider`, `errors`, public facade exports in `oxmux.rs`, and deterministic `oxmux` tests.
- API impact: new public Rust types for `ResponseMode`, stream events with terminal events in-order, terminal state, cancellation reason, stream failure details, and mock-provider streaming outcomes. Provider streaming capability metadata becomes configurable independently from the delivered response mode.
- Dependency impact: no new provider SDK, HTTP client, async runtime, OAuth, GPUI, app-shell, credential storage, or external service dependencies.
