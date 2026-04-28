## Context

`oxmux` already owns the headless core boundary, canonical protocol request/response skeletons, provider execution mocks, accepted routing policy primitives, and a placeholder `StreamingBoundary`. Issue #6 moves streaming from placeholder ownership to typed response contracts so future provider adapters, proxy routes, protocol translators, and tests can share one model for complete responses and chunk/SSE-style streams.

The accepted `oxmux-routing-policy` spec remains the source of truth for route selection, fallback, skipped candidates, and structured routing failures. This change must preserve those semantics while exposing streaming capability metadata clearly enough for future routing/proxy requests that require streaming-capable providers.

The implementation must stay inside `crates/oxmux` and use deterministic in-memory values. Real upstream streaming endpoints, HTTP proxy routes, provider SDK clients, OAuth/token refresh, GPUI, `oxidemux`, credential storage, and desktop lifecycle code remain outside this change.

## Goals / Non-Goals

**Goals:**

- Define Rust-native primitives for non-streaming responses, streaming events, stream completion, cancellation, and stream errors.
- Represent stream event order deterministically so tests can assert exact chunk/SSE-style sequences.
- Extend mock provider execution outcomes so mock providers can return either a complete response or a deterministic streaming response sequence.
- Keep streaming capability metadata compatible with accepted routing-policy selection and structured routing failure semantics.
- Surface streaming failures through structured `CoreError` data while keeping display text human-readable.
- Export streaming primitives from the `oxmux` public facade for direct Rust consumers and future `oxidemux` use.
- Cover streaming behavior with networkless `oxmux` tests.

**Non-Goals:**

- No outbound provider calls, provider SDKs, HTTP clients, SSE parser, WebSocket support, OAuth, token refresh, or credential storage.
- No local HTTP proxy route implementation or request/response transport plumbing beyond primitives needed by tests.
- No app-shell, GPUI, tray, updater, notification, or desktop lifecycle integration.
- No async runtime dependency solely for streaming; deterministic test sequences can be represented as owned values.
- No provider-specific streaming dialect translation for OpenAI, Gemini, Claude, Codex, or provider-specific APIs beyond typed protocol metadata carried on existing canonical responses/events.

## Decisions

1. **Model streaming as owned event sequences for this phase.**
   - Decision: define stream response primitives around ordered owned `StreamEvent` values rather than an async stream trait. Terminal states are represented in the same sequence as `StreamEvent::Terminal(StreamTerminalState)`, so validation can reject missing terminals, multiple terminals, and non-terminal events after a terminal.
   - Rationale: issue #6 needs deterministic tests and no real transport. Owned sequences avoid async runtime coupling and make event order easy to assert.
   - Alternative considered: expose `futures::Stream` from `oxmux`. That would force a runtime/dependency decision before real provider transports exist.

2. **Keep non-streaming and streaming responses under one response envelope.**
   - Decision: add a typed `ResponseMode` envelope that can hold either a complete `CanonicalProtocolResponse` or a deterministic `StreamingResponse` sequence.
   - Rationale: provider adapters and proxy routes need a single matchable result shape while preserving whether a request completed synchronously or streamed.
   - Alternative considered: keep `ProviderExecutionOutcome::StreamingCapable` as metadata only. That reports capability but cannot test actual event lifecycles.

3. **Make terminal stream states explicit values.**
   - Decision: stream sequences end with explicit completed, cancelled, or errored terminal states, and validation fails if a stream has no terminal event or has events after termination.
   - Rationale: callers should never infer cancellation/error from dropped values, short reads, or missing chunks.
   - Alternative considered: rely on `Result<Option<Event>>`-style iteration. That mirrors runtime transport but hides cancellation semantics in control flow.

4. **Separate delivered terminal states from operation failures.**
   - Decision: represent valid delivered cancellation and stream-error outcomes as typed stream terminal events, and reserve streaming-specific `CoreError` values for invalid sequences or failures that happen before a stream response exists.
   - Rationale: `oxidemux`, management surfaces, and Rust consumers need explicit terminal data for streams that emitted events, but invalid construction and pre-stream execution failures still need the core error boundary.
   - Alternative considered: return `Err(CoreError)` for all cancelled or errored streams. That would hide already-delivered stream events and recreate the silent dropped-stream ambiguity issue #6 is meant to prevent.

5. **Define minimum terminal data shapes now.**
   - Decision: define typed cancellation reasons with at least user-requested, client-disconnected, upstream-closed, timeout, and extensible other/code-message cases; define stream failure details with at least stable code and human-readable message fields.
   - Rationale: callers must match cancellation and error categories without parsing display text, while future provider-specific metadata can remain optional owned data rather than SDK types.
   - Alternative considered: store only opaque strings in terminal events. That would satisfy display needs but fail the structured matching requirement.

6. **Allow empty and metadata-only streams.**
   - Decision: validation requires exactly one terminal event and ordered non-terminal events, but it does not require a content chunk before completed, cancelled, or errored termination.
   - Rationale: future providers may emit only metadata or terminate before content; these are valid delivered stream lifecycles and should not be treated as invalid construction.
   - Alternative considered: require at least one content chunk for every valid stream. That would overfit early tests and reject plausible upstream behavior.

7. **Extend mock provider outcomes instead of adding provider transport.**
   - Decision: update `MockProviderOutcome` and `ProviderExecutionOutcome` so mocks can return deterministic streaming event sequences while provider summaries still report `supports_streaming`.
   - Rationale: the existing mock harness is the accepted networkless provider boundary and already reflects provider/account metadata into management summaries.
   - Alternative considered: create a separate streaming mock harness. That would duplicate provider/account metadata and weaken the provider execution contract.

8. **Treat provider streaming capability as metadata, not only an outcome.**
   - Decision: streaming outcomes imply `ProviderCapability.supports_streaming`, but mock providers also need a way to report streaming capability while returning a complete response.
   - Rationale: capability describes what a provider/account can do, while response mode describes what happened for one execution. Keeping them separate prevents tests and future routing/proxy consumers from conflating provider capability with a specific response.
   - Alternative considered: continue deriving streaming capability only from a streaming-capable outcome. That would keep the current mock shortcut but make capability metadata inaccurate for complete responses from streaming-capable providers.

9. **Treat routing-policy as an accepted dependency, not a placeholder.**
   - Decision: streaming work preserves the accepted `oxmux-routing-policy` contracts and only exposes streaming capability metadata through provider execution and future routing/proxy consumers. This change does not add a `require_streaming` field or streaming-specific selection branch to `RoutingPolicy`.
   - Rationale: routing selection, fallback, skipped-candidate metadata, and structured routing failures are already accepted core semantics; streaming should interoperate with them rather than redefine them.
   - Alternative considered: restating route selection behavior inside streaming response primitives. That would duplicate `oxmux-routing-policy` and risk divergent failure semantics.

10. **Make provider execution migration explicit.**
    - Decision: successful provider execution outcomes should carry `ResponseMode` so `Success`, `Degraded`, and `QuotaLimited` can represent complete responses now and streaming responses where applicable. Existing complete-response accessors should either remain for complete-only cases or be replaced with clearly named `response_mode()` / `complete_response()` APIs.
    - Rationale: existing outcomes assume every successful execution has a `CanonicalProtocolResponse`; streaming support needs one envelope without hiding response mode or forcing callers to parse variant names.
    - Alternative considered: add only a new top-level streaming outcome variant. That would be smaller but leaves degraded/quota-limited streaming behavior ambiguous.

## Risks / Trade-offs

- [Risk] Owned event sequences may not perfectly match future backpressure or async cancellation behavior. → Mitigation: keep this change focused on semantic contracts and leave transport adapters to wrap these primitives later.
- [Risk] Public stream event types may need more provider-specific metadata once real adapters land. → Mitigation: include protocol metadata and extensible reason/message fields while avoiding provider SDK types.
- [Risk] Terminal-state validation can feel strict for early tests. → Mitigation: strict validation prevents the exact silent cancellation and dropped-error cases called out by issue #6.
- [Risk] Separating provider capability metadata from response mode can add mock harness API surface. → Mitigation: keep the capability configuration small and reuse existing `ProviderCapability` summaries instead of adding mock-only summary types.
- [Risk] Provider execution API shape changes before real providers exist. → Mitigation: update deterministic mocks and facade exports now, before downstream adapters depend on metadata-only streaming capability.

## Migration Plan

1. Replace `StreamingBoundary` placeholder-only behavior in `crates/oxmux/src/streaming.rs` with typed `ResponseMode`, streaming response, event, terminal state, cancellation, and failure primitives.
2. Export streaming primitives from `crates/oxmux/src/oxmux.rs`.
3. Add streaming-specific structured error support in `crates/oxmux/src/errors.rs`.
4. Extend `ProviderExecutionOutcome` and `MockProviderOutcome` in `crates/oxmux/src/provider.rs` to support `ResponseMode` and deterministic streaming response sequences, while keeping streaming capability metadata configurable independently from response mode.
5. Add deterministic `crates/oxmux/tests/streaming_response.rs` and update `crates/oxmux/tests/provider_execution.rs` / direct facade tests for streaming exports, including empty-completed streams, metadata-only streams, event order preservation, cancellation reasons, terminal failures, and invalid sequences.
6. Add routing/streaming compatibility tests or assertions showing streaming-capable provider metadata remains available to accepted routing-policy consumers without adding streaming-specific routing selection behavior.
7. Run `cargo fmt --all --check`, `cargo test -p oxmux`, `mise run ci`, and `openspec validate add-streaming-response-abstraction --strict`.

Rollback is straightforward before real provider adapters depend on these types: remove the new streaming primitives, provider mock streaming outcome, error variants, and tests while retaining the placeholder `StreamingBoundary` ownership marker.

## Open Questions

- None for this proposal. Future provider adapter work can decide how async transports map live upstream events into these typed `oxmux` stream events.
