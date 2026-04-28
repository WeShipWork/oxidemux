## MODIFIED Requirements

### Requirement: Minimal public facade for future core domains
The `oxmux` crate SHALL expose a small public facade that establishes ownership of proxy lifecycle, local health runtime, provider/auth, provider execution, routing, protocol translation, configuration, streaming, management/status, usage/quota, and domain error primitives without implementing full provider SDK integration, outbound provider calls, credential storage, concrete proxy request handling, or real streaming transport adapters in this change.

#### Scenario: Provider auth ownership is visible but not implemented
- **WHEN** maintainers inspect the `oxmux` public API or documentation
- **THEN** provider authentication and token refresh are identified as future core concerns without requiring OAuth UI, platform credential storage, or concrete provider clients in this phase

#### Scenario: Provider execution ownership exposes deterministic mock boundaries
- **WHEN** maintainers inspect the `oxmux` public API or documentation after adding provider execution primitives
- **THEN** provider execution is represented by trait, request, result, mock harness, and structured outcome primitives that can be used in deterministic tests without requiring real provider SDKs, HTTP clients, OAuth, platform credential storage, GPUI, or app-shell state

#### Scenario: Routing ownership exposes typed policy primitives
- **WHEN** maintainers inspect the `oxmux` public API or documentation after adding routing policy primitives
- **THEN** model aliases, account targeting, priority, fallback, exhausted states, degraded states, selection outcomes, skipped candidate metadata, and routing failure details are represented by typed public primitives without requiring concrete proxy routing behavior, provider SDKs, outbound provider calls, GPUI, or app-shell state

#### Scenario: Protocol ownership exposes typed skeleton boundaries
- **WHEN** maintainers inspect the `oxmux` public API or documentation
- **THEN** OpenAI, Gemini, Claude, Codex, and provider-specific protocol translation are represented by typed request/response boundaries, typed protocol metadata, and deferred translation results without requiring request translators, response translators, or outbound provider calls in this phase

#### Scenario: Streaming ownership exposes typed response primitives
- **WHEN** maintainers inspect the `oxmux` public API or documentation after adding streaming response primitives
- **THEN** response mode, complete responses, ordered stream events, in-sequence terminal events, stream completion, stream cancellation, stream errors, streaming failure details, and deterministic mock stream outcomes are represented by typed public primitives without requiring network transports, provider stream adapters, provider SDKs, outbound provider calls, GPUI, or app-shell state

#### Scenario: Management ownership includes local health runtime status
- **WHEN** maintainers inspect the `oxmux` public API or documentation
- **THEN** proxy lifecycle state, local health runtime status, provider listing, account health, usage, quota, and degraded service status are identified as core concerns while management endpoints beyond `/health` remain deferred

### Requirement: Core errors remain visible to consumers
The `oxmux` crate SHALL define or reserve a core error boundary so future fallible proxy, provider, configuration, routing, protocol, streaming, and management operations can propagate meaningful errors to library consumers and app layers, including structured routing and streaming failures that callers can match without parsing display text.

#### Scenario: Core error type is usable by the app shell
- **WHEN** `oxidemux` calls into the `oxmux` facade
- **THEN** fallible operations expose typed or structured errors that can be propagated or displayed rather than silently discarded

#### Scenario: Routing failures are matchable
- **WHEN** `oxmux` route selection fails because a route is missing, a target is missing, candidates are exhausted, only degraded candidates remain, or policy input is invalid
- **THEN** the returned `CoreError` includes structured routing failure data that Rust consumers can match without parsing display text

#### Scenario: Streaming failures are matchable
- **WHEN** `oxmux` streaming response validation fails because a stream sequence is invalid, or mock streaming execution fails before a stream response exists
- **THEN** the returned `CoreError` includes structured streaming failure data that Rust consumers can match without parsing display text

#### Scenario: Streaming capability remains metadata
- **WHEN** maintainers inspect provider summaries after adding streaming response primitives
- **THEN** streaming capability is represented as provider capability metadata independently from whether the current execution returned a complete or streaming response mode

#### Scenario: Delivered stream terminal states remain response data
- **WHEN** `oxmux` receives or constructs a valid stream response that terminates as cancelled or errored after zero or more events
- **THEN** the cancellation or stream error is represented as typed terminal response data rather than being converted into `Err(CoreError)` and losing the delivered event sequence
