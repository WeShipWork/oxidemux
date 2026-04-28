## ADDED Requirements

### Requirement: Core facade exposes minimal proxy engine path
The `oxmux` public facade SHALL expose the minimal proxy engine primitives needed for Rust consumers and tests to exercise a local OpenAI-compatible chat-completion smoke path through protocol, routing, provider execution, response mode handling, and structured errors without importing `oxidemux` or desktop-specific code.

#### Scenario: Rust consumer exercises minimal engine without app shell
- **WHEN** a Rust test or library consumer constructs the minimal proxy engine inputs through the `oxmux` facade
- **THEN** it can execute a deterministic mock-backed chat-completion request path without launching the `oxidemux` binary, opening a GPUI window, starting tray/menu lifecycle code, or contacting a real provider

#### Scenario: Core facade preserves structured errors
- **WHEN** minimal proxy engine routing, request validation, or provider execution fails
- **THEN** the returned core result includes structured `CoreError` data that consumers can match without parsing display strings or local HTTP response bodies

### Requirement: Core keeps proxy semantics independent from desktop shell
The `oxmux` core SHALL own the reusable minimal proxy request semantics for model extraction, canonical protocol request construction, routing decision, provider execution request construction, response mode handling, and deterministic local response serialization while `oxidemux` remains a consumer for future UI and lifecycle presentation.

#### Scenario: Desktop shell does not define proxy route semantics
- **WHEN** the minimal proxy engine path is implemented
- **THEN** route semantics, routing choices, provider execution outcomes, fallback reasons, and structured failures are represented in `oxmux` rather than duplicated in `oxidemux`

#### Scenario: Dependency boundary remains intact
- **WHEN** maintainers inspect `crates/oxmux/Cargo.toml` and run core dependency-boundary tests after adding the minimal proxy engine
- **THEN** `oxmux` remains free of GPUI, gpui-component, tray libraries, updater libraries, packaging tools, platform credential storage libraries, provider SDKs, OAuth UI libraries, and the `oxidemux` app crate

## MODIFIED Requirements

### Requirement: Minimal public facade for future core domains
The `oxmux` crate SHALL expose a small public facade that establishes ownership of proxy lifecycle, local health runtime, provider/auth, provider execution, routing, protocol translation, configuration, streaming, management/status, usage/quota, domain error primitives, and a minimal concrete proxy request smoke path without implementing full provider SDK integration, outbound provider calls, credential storage, full proxy request handling, or real streaming transport adapters in this change.

#### Scenario: Provider auth ownership is visible but not implemented
- **WHEN** maintainers inspect the `oxmux` public API or documentation
- **THEN** provider authentication and token refresh are identified as future core concerns without requiring OAuth UI, platform credential storage, or concrete provider clients in this phase

#### Scenario: Provider execution ownership exposes deterministic mock boundaries
- **WHEN** maintainers inspect the `oxmux` public API or documentation after adding provider execution primitives
- **THEN** provider execution is represented by trait, request, result, mock harness, and structured outcome primitives that can be used in deterministic tests without requiring real provider SDKs, HTTP clients, OAuth, platform credential storage, GPUI, or app-shell state

#### Scenario: Routing ownership exposes typed policy primitives
- **WHEN** maintainers inspect the `oxmux` public API or documentation after adding routing policy primitives
- **THEN** model aliases, account targeting, priority, fallback, exhausted states, degraded states, selection outcomes, skipped candidate metadata, and routing failure details are represented by typed public primitives and exercised by the minimal smoke route without requiring full proxy routing behavior, provider SDKs, outbound provider calls, GPUI, or app-shell state

#### Scenario: Protocol ownership exposes typed skeleton boundaries
- **WHEN** maintainers inspect the `oxmux` public API or documentation
- **THEN** OpenAI, Gemini, Claude, Codex, and provider-specific protocol translation are represented by typed request/response boundaries, typed protocol metadata, and deferred translation results while the minimal smoke route may construct an OpenAI canonical request without requiring full request translators, response translators, or outbound provider calls in this phase

#### Scenario: Streaming ownership exposes typed response primitives
- **WHEN** maintainers inspect the `oxmux` public API or documentation after adding streaming response primitives
- **THEN** response mode, complete responses, ordered stream events, in-sequence terminal events, stream completion, stream cancellation, stream errors, streaming failure details, and deterministic mock stream outcomes are represented by typed public primitives without requiring network transports, provider stream adapters, provider SDKs, outbound provider calls, GPUI, or app-shell state

#### Scenario: Management ownership includes local health runtime status
- **WHEN** maintainers inspect the `oxmux` public API or documentation
- **THEN** proxy lifecycle state, local health runtime status, provider listing, account health, usage, quota, and degraded service status are identified as core concerns while management endpoints beyond `/health` remain deferred

### Requirement: Core facade remains runtime-inert for this change
The `oxmux` management/lifecycle and provider execution facade SHALL remain usable without starting provider network transports, concrete provider clients, OAuth flows, token refresh, hot reload watchers, background proxy routing behavior, GPUI, or app-shell state, while also supporting explicit local health runtime startup, deterministic in-memory mock provider execution, and an explicitly configured minimal local proxy route for smoke testing.

#### Scenario: Tests use deterministic local runtime state
- **WHEN** core tests verify management snapshots, configuration validation, provider/account summaries, lifecycle states, usage/quota summaries, local health runtime behavior, and the minimal proxy smoke route
- **THEN** they use deterministic in-memory values, caller-supplied routing availability, mock provider execution, or loopback-only local listener state and do not require external network services, real credentials, provider accounts, a desktop app, or desktop platform APIs

#### Scenario: Tests use deterministic mock provider state
- **WHEN** core tests verify provider execution traits, mock provider outcomes, provider/account summary reflection, quota-limited outcomes, degraded outcomes, streaming-capable metadata, failed mock outcomes, or selected provider/account propagation through the minimal proxy route
- **THEN** they use deterministic in-memory mock providers and do not require external network services, real credentials, provider SDKs, OAuth flows, token refresh, raw secret storage, a desktop app, or desktop platform APIs

#### Scenario: Protocol ownership remains explicit
- **WHEN** provider capabilities, provider execution requests, or routing defaults reference OpenAI, Gemini, Claude, Codex, or provider-specific protocol families
- **THEN** the facade identifies those protocol families as typed metadata but does not translate full provider request or response surfaces in this change
