## Context

Issue #6 established the `oxmux` streaming data model: `ResponseMode`, ordered `StreamingResponse` events, terminal completion/cancellation/error states, `CancellationReason`, `StreamFailure`, and `CoreError::Streaming` for invalid or pre-stream failures. Issue #20 is the next layer: make those values robust enough for future local proxy/provider work by adding deterministic controls for keepalive, bootstrap retry, timeout metadata, cancellation propagation, and structured stream errors.

The external inspiration is specific. CLIProxyAPI treats streaming keepalives as explicit SSE comment heartbeats and permits bootstrap retry only before payload bytes are sent; once a stream has emitted payload, a later provider error is terminal stream data, not a safe retry candidate. VibeProxy validates the subscription-first desktop UX but delegates backend stream behavior to CLIProxyAPI-style core proxy logic. OxideMux should import the semantics, not the implementation: `oxmux` remains a deterministic headless Rust core, and `oxidemux` later presents the resulting state.

Current `oxmux` code supports deterministic in-memory streaming values and mock provider streaming outcomes. It does not yet have keepalive metadata, retry policy, timeout policy, cancellation context propagation, or live event-by-event HTTP/SSE streaming transport. The minimal proxy currently rejects `ResponseMode::Streaming`, which is acceptable for this change unless the implementation explicitly chooses to add a narrow deterministic serializer.

## Goals / Non-Goals

**Goals:**

- Define `oxmux`-owned streaming robustness policy for keepalive interval, bootstrap retry count, timeout behavior, and cancellation behavior.
- Represent keepalive, timeout, retry attempts, cancellation, and provider stream errors with typed/matchable `oxmux` data.
- Preserve the safe retry invariant: retry only before the first emitted event; any emitted event, including keepalive or timeout metadata/control events, commits the stream and forbids retry.
- Extend mock provider/test harness behavior so default tests cover idle keepalive, retry before first emitted event, cancellation, provider error after partial stream, and clean completion without network access.
- Surface stream robustness outcomes through core errors, response data, provider metadata, and management snapshots so app status can display them later without redefining semantics.
- Extend file-backed configuration validation so users can express stream robustness policy deterministically.

**Non-Goals:**

- No real upstream provider streaming endpoint in default tests.
- No WebSocket relay support.
- No provider-specific thinking/reasoning behavior.
- No GPUI or UI rendering.
- No provider SDK, OAuth/token refresh, credential storage, or live HTTP client dependency.
- No requirement to fully replace the current synchronous minimal proxy with production SSE/chunked streaming transport in this change.

## Decisions

### 1. Model stream controls as core policy plus ordered metadata

Add a small `oxmux` streaming robustness policy near the existing streaming contracts, with explicit fields for keepalive interval, bootstrap retry count, timeout behavior, and cancellation behavior. Keepalive, timeout, and committed retry observations SHALL be represented through typed helpers backed by reserved `StreamMetadata` names rather than raw provider transport frames or a new transport-specific event enum. Reserved metadata keys SHALL use the `oxmux.` namespace and include `oxmux.keepalive`, `oxmux.timeout`, `oxmux.retry_summary`, and `oxmux.retry_exhausted`. The constructors/validators SHALL prevent provider-supplied or caller-supplied generic metadata from using the reserved `oxmux.` namespace unless it is created through the typed robustness helpers.

Reserved metadata emitted into a delivered stream is limited to observations for the committed attempt. Failed pre-event attempts MAY be recorded in provider execution/management metadata, but they MUST NOT be emitted as stream events because emitting them would commit the stream and make further retry unsafe.

Rationale: Existing `StreamingResponse` already permits ordered metadata events and preserves insertion order. Reusing that model keeps tests deterministic and avoids introducing async timers or SSE dependencies into `oxmux`.

Alternative considered: add a transport-specific SSE event enum now. Rejected because the current specs require provider- and transport-agnostic headless semantics; SSE serialization can wrap these values later.

### 2. Keep retry-before-first-event as an execution policy, not a routing feature

Bootstrap retry belongs at the provider/stream execution seam. A stream attempt that fails before emitting any event may be retried within policy. The first emitted event is the commit boundary. Once any event is emitted, including keepalive metadata, timeout metadata, retry-summary metadata, content, or other control metadata, the attempt is committed and later failures must appear as terminal errored stream data. Timeout or cancellation observed before any event returns a structured pre-stream `CoreError::Streaming`; timeout or cancellation observed after any event remains terminal stream response data.

`bootstrap_retry_count` means additional attempts after the initial attempt. A value of `0` means no retry. Retry exhaustion reports the total attempted count and preserves the underlying provider/stream failure in structured data. A successful committed attempt may include final retry summary metadata, such as the number of failed pre-event attempts, but it MUST NOT include event data from failed attempts.

Rationale: This matches CLIProxyAPI's safe-before-first-byte behavior and prevents duplicate or corrupted client output. Routing may choose candidates before execution, but retrying a stream after partial data is an execution correctness problem, not route selection.

Alternative considered: reuse routing fallback for mid-stream provider errors. Rejected because fallback after emitted data cannot preserve a coherent single stream.

### 3. Preserve the existing error split

Keep valid delivered cancellations and terminal errors as `ResponseMode::Streaming` data. Use `Err(CoreError::Streaming { .. })` only for invalid stream construction, retry exhaustion before any event exists, or other failures that happen before a valid stream response exists. Configuration validation failures for streaming policy fields remain `CoreError::Configuration` with streaming field paths, not `CoreError::Streaming`.

Pre-event cancellation is a policy-controlled pre-stream outcome: if cancellation is observed before a valid stream response exists, execution returns structured `CoreError::Streaming` cancellation data; if cancellation is observed after any event has been emitted, execution returns terminal stream response data preserving event history.

Rationale: The existing `oxmux-core` spec explicitly requires delivered stream terminal states to remain response data so consumers do not lose event history.

Alternative considered: convert every stream failure to `Err(CoreError)`. Rejected because it erases partial stream context and recreates the silent dropped-stream ambiguity that #6 resolved.

### 4. Extend mock provider behavior before adding live transport

Implement deterministic mock outcomes for pre-event failure-then-success, pre-event retry exhaustion, keepalive-only idle windows, timeout, cancellation, partial-content-then-error, and clean completion. Timeout is modeled in this change as an explicit deterministic mock outcome or constructed stream observation, not as wall-clock enforcement. If needed, add a focused stream attempt plan type rather than immediately changing the public provider trait into a live async stream.

Rationale: Issue #20 explicitly excludes real upstream streaming endpoints in default tests. Mock-first semantics give the product contract and tests without forcing a transport architecture prematurely.

Alternative considered: introduce an async streaming trait and wire local proxy SSE transport immediately. Rejected for this proposal's first implementation path because it would make #20 much larger and pull in runtime/transport design before core semantics are stable.

### 5. Configuration stays strict and management-visible

Add file-backed streaming configuration under the existing strict TOML configuration model, preserving `deny_unknown_fields` behavior and structured validation failures. The canonical table is `[streaming]` with kebab-case fields:

```toml
[streaming]
keepalive-interval-ms = 15000
bootstrap-retry-count = 2
timeout-ms = 120000
cancellation = "client-disconnect"
```

All fields are optional. Omitted fields use deterministic disabled defaults: no keepalive, zero retries, no timeout policy, and `"disabled"` automatic cancellation. Duration fields use integer milliseconds. Supported duration values are `1..=300000` milliseconds. Supported `bootstrap-retry-count` values are `0..=10`, where `0` means no additional attempts. Explicit `0` for duration fields is invalid; omission disables the duration behavior. Negative numbers, floats, strings for numeric fields, and integer overflow are invalid. Supported cancellation values are `"disabled"`, `"client-disconnect"`, and `"timeout"`; `"timeout"` requires a configured `timeout-ms`, and configured `timeout-ms` with `cancellation = "disabled"` records timeout policy/metadata but does not automatically convert timeout into cancellation. In this change, `"client-disconnect"` enables only deterministic cancellation policy representation and mock execution outcomes; live request-context disconnect detection is deferred to a later streaming transport change. Future user-requested cancellation reasons can be represented in runtime response data without being enabled by file policy in this change. Invalid values fail with `CoreError::Configuration` and field paths such as `streaming.keepalive-interval-ms`.

Management snapshots expose the active streaming policy and the latest stream robustness outcome when supplied by provider/proxy execution. Outcome data includes timeout, cancellation, retry exhaustion, and post-partial stream errors, plus provider/account/routing context when known. Snapshot replacement is latest-outcome, last-writer-wins for this change and MUST avoid exposing raw provider secrets or transport frames. Aggregated stream outcome history is deferred.

Rationale: Subscription UX depends on visible recovery state. Invalid stream policy should fail as structured configuration data, and runtime stream failures should be visible to headless and app-shell consumers.

Alternative considered: keep stream policy code-only until live transport exists. Rejected because issue #20 acceptance requires configuration representation and app status visibility.

## Risks / Trade-offs

- [Risk] Adding full live streaming transport now could explode scope and entangle `oxmux` with async/SSE details. → Mitigation: define deterministic core policy and mock execution first; keep production SSE/chunked serialization as a later, explicit change unless implementation proves a tiny serializer is necessary.
- [Risk] Metadata-only keepalive events could be confused with provider metadata or spoofed by provider/custom metadata. → Mitigation: use stable constructor/helper names and enforce reserved `oxmux.` metadata keys in code and tests so generic metadata cannot use the reserved namespace.
- [Risk] Bootstrap retry semantics could accidentally retry after partial output. → Mitigation: model and test an explicit `emitted_event`/attempt-committed state and include post-partial error tests that assert no retry occurs.
- [Risk] Configuration fields may settle before transport needs are known. → Mitigation: keep policy minimal: interval, retry count, timeout, cancellation behavior; avoid provider-specific knobs.
- [Risk] Management snapshots may become cluttered with per-stream telemetry. → Mitigation: expose only active policy and latest structured state required for status surfaces in this change; defer aggregate history and detailed stream metrics.
- [Risk] Streaming robustness could be mistaken for live backpressure support. → Mitigation: explicitly defer transport backpressure and live client disconnect propagation until a later streaming transport change.

## Migration Plan

1. Add stream robustness policy and metadata helpers while preserving existing streaming constructors and tests.
2. Extend configuration parsing/validation with strict streaming policy fields and defaults.
3. Extend mock provider/test harness behavior for deterministic retry, cancellation, timeout, and post-partial error scenarios.
4. Surface structured stream robustness outcomes through existing core error and management paths.
5. Keep `ResponseMode::Streaming` rejection in the minimal proxy unless implementation explicitly adds a narrow deterministic serializer and tests it; existing unsupported streaming error codes must remain stable.

Rollback is straightforward before live provider adapters depend on the new types: remove the policy/config/mock outcome additions and their tests while retaining the existing #6 streaming primitives.

## Deferred Work

- Live SSE/chunked transport, async timer enforcement, backpressure, and real client disconnect propagation are deferred to a later streaming transport change.
- Provider-specific thinking/reasoning stream rewrites remain out of scope for this policy-only change.
- Detailed stream metrics and aggregate outcome history beyond active policy plus latest robustness outcome are deferred until status surfaces require them.
