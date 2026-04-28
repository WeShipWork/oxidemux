## Context

`oxmux` currently exposes the pieces needed for a proxy request but does not compose them. The local runtime binds a loopback HTTP listener and serves `GET /health`; protocol types can represent canonical OpenAI-style requests and responses; routing can select provider/account targets from explicit availability inputs; provider execution has a deterministic mock harness; response primitives distinguish complete and streaming responses.

Issue #15 is the first end-to-end core proxy slice. It should prove that `oxmux` can accept a local model request and move it through protocol, routing, provider execution, error handling, and response serialization without introducing real provider transports or app-shell behavior.

## Goals / Non-Goals

**Goals:**

- Add a minimal headless proxy engine seam in `oxmux` that can be exercised directly from Rust tests and from the local loopback route.
- Support exactly one smoke route: `POST /v1/chat/completions` with enough OpenAI-compatible request and response shape to validate model routing and complete-response serialization.
- Reuse existing `CanonicalProtocolRequest`, `RoutingPolicy`, `RoutingAvailabilitySnapshot`, `ProviderExecutionRequest`, `ProviderExecutor`, `MockProviderHarness`, and `ResponseMode` primitives, supplied through an explicit runtime/engine configuration seam.
- Return deterministic HTTP status, `Content-Type: application/json`, stable error codes, and JSON bodies for success, invalid request, provider failure, routing failure, unsupported response mode, oversized body, and unsupported path.
- Preserve `/health` behavior and loopback-only runtime constraints.
- Keep all behavior inside `crates/oxmux` and export any required seam through the `oxmux` facade.

**Non-Goals:**

- No real provider network calls, provider SDKs, OAuth flows, token refresh, raw credential storage, or platform secret-store adapters.
- No GPUI, tray/menu integration, updater, packaging, app-shell lifecycle, or `oxidemux` UI behavior.
- No full OpenAI compatibility beyond the smoke-route request/response subset.
- No streaming transport, SSE serialization, provider-prefixed route normalization, embeddings/completions/models routes, or management endpoints beyond existing health/status behavior.
- No provider protocol translation beyond preserving OpenAI canonical metadata and opaque payloads for this path.

## Decisions

1. **Add a small proxy engine seam instead of embedding business logic in the HTTP handler.**
   - Decision: introduce a headless engine/request path that coordinates request decoding, protocol envelope construction, routing selection, provider execution, and response encoding. The local runtime should dispatch to that seam rather than duplicating routing/provider behavior inline.
   - Rationale: Future CLI, tests, and desktop shell adapters need to reuse the same core semantics without depending on a running TCP listener.
   - Alternative considered: implement the entire route in `handle_connection`. That is faster initially but would make the route hard to test without I/O and would hide the core proxy contract behind transport details.

2. **Keep the OpenAI codec intentionally narrow.**
   - Decision: accept a bounded JSON body with a non-blank `model` field and enough `messages` shape to reject clearly invalid smoke requests; serialize a deterministic non-streaming `chat.completion` response body from the mock provider response.
   - Rationale: Issue #15 requires a smoke route, not full compatibility. A narrow codec makes the contract testable while avoiding premature support for tools, function calling, streaming chunks, multimodal content, or provider-specific extensions.
   - Alternative considered: treating the entire body as opaque and never parsing JSON. That would preserve existing protocol payload behavior but would not support invalid-request coverage for missing model or malformed JSON.

3. **Route by the requested model before provider execution.**
   - Decision: decode the OpenAI request model into `CanonicalProtocolRequest`, call `RoutingBoundary::select` with the configured policy and availability, and build `ProviderExecutionRequest` from the selected provider/account target. The engine/runtime configuration seam must accept the caller-supplied `RoutingPolicy`, `RoutingAvailabilitySnapshot`, and `ProviderExecutor` or deterministic mock registry rather than constructing hidden defaults.
   - Rationale: This proves that model aliases, fallback behavior, account targeting, and availability state remain core concerns owned by `oxmux`, while keeping desktop/provider adapters responsible only for supplying current state.
   - Alternative considered: passing a fixed mock provider directly from the route. That would satisfy a mock response but would not exercise the routing acceptance criterion.

4. **Use mock providers as execution adapters for this route.**
   - Decision: route tests should supply deterministic `ProviderExecutor` implementations, primarily `MockProviderHarness`, and cover both success and `MockProviderOutcome::Failed` behavior.
   - Rationale: The issue explicitly excludes real providers and credentials while requiring provider execution trait coverage.
   - Alternative considered: add a special route-only mock response. That would bypass the provider execution trait and weaken the first core-engine path.

5. **Map core failures to stable smoke-route HTTP responses.**
   - Decision: unsupported path returns `404`, invalid or oversized request returns `400`, routing failure/provider execution failure/unsupported response mode returns `502`, and success returns `200`. All local HTTP responses for the proxy route should include `Content-Type: application/json`; error bodies should expose stable minimal `error.code` values while avoiding provider secrets or raw internal debug strings. Internally, the engine should keep structured `CoreError` values so Rust consumers and future UI code can inspect failures without parsing response text.
   - Rationale: The smoke route needs deterministic client-visible behavior while the core keeps matchable errors for subscription UX and recovery flows.
   - Alternative considered: return `500` for all engine failures. That is simpler but loses routing/provider distinction and undermines future troubleshooting UX.

6. **Preserve dependency restraint while allowing a justified JSON codec.**
   - Decision: keep the core free of provider SDKs and app dependencies. If implementation uses JSON parsing, constrain it to minimal request/response codec behavior and avoid async HTTP frameworks for this issue.
   - Rationale: Current `oxmux` has no dependencies and a hand-rolled loopback runtime. The main dependency risk is not JSON parsing itself; it is expanding into a full transport stack before the product contract is proven.
   - Alternative considered: add Axum/Hyper immediately. That would simplify HTTP parsing but materially changes the dependency/runtime boundary for a single smoke route.

## Risks / Trade-offs

- [Risk] Hand-rolled HTTP parsing grows beyond a smoke runtime. → Mitigation: bound request bytes, support only request line, required headers/body handling needed for `POST /v1/chat/completions`, and keep unsupported behavior deterministic.
- [Risk] Minimal OpenAI JSON compatibility may be mistaken for full API support. → Mitigation: name and document the codec as smoke-route-only, keep non-goals explicit, and test only the accepted subset.
- [Risk] Engine configuration could overfit mock tests. → Mitigation: define an explicit configuration seam accepting existing routing policy, availability, and trait-based executor inputs so future real adapters can supply the same core primitives without changing route semantics.
- [Risk] Provider failures lose structured detail when serialized to HTTP. → Mitigation: keep structured `CoreError` internally and serialize a stable minimal error JSON externally.
- [Risk] A new JSON dependency could weaken the dependency-boundary story. → Mitigation: update dependency-boundary tests to continue banning app-shell, provider SDK, OAuth, GPUI, and platform storage dependencies; justify any JSON-only dependency through the codec requirement.

## Migration Plan

1. Add the minimal proxy engine/request-route primitives inside `crates/oxmux`.
2. Extend local runtime request parsing and dispatch while preserving existing health endpoint behavior.
3. Add smoke-route codec and deterministic HTTP serialization for the supported OpenAI-compatible subset.
4. Add integration tests for route success, invalid request before routing/provider execution, selected provider/account propagation, provider failure, routing failure or degraded/exhausted fallback behavior, unsupported path, and dependency boundary preservation.
5. Run `cargo fmt --all --check`, `cargo test -p oxmux`, `mise run ci`, and `openspec validate add-minimal-proxy-request-route --strict`.

Rollback is straightforward before downstream code depends on the route: remove the new engine/codec exports and tests, and restore the runtime dispatcher to health-only behavior.

## Open Questions

- Whether to use a small JSON dependency for correctness or a deliberately tiny parser for the smoke subset should be decided during implementation based on the least complex way to reject malformed JSON and missing/blank model fields without expanding the transport stack.
