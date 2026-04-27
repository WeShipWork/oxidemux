## MODIFIED Requirements

### Requirement: Minimal public facade for future core domains
The `oxmux` crate SHALL expose a small public facade that establishes ownership of proxy lifecycle, local health runtime, provider/auth, provider execution, routing, protocol translation, configuration, streaming, management/status, usage/quota, and domain error primitives without implementing full provider SDK integration, outbound provider calls, credential storage, or proxy routing behavior in this change.

#### Scenario: Provider auth ownership is visible but not implemented
- **WHEN** maintainers inspect the `oxmux` public API or documentation
- **THEN** provider authentication and token refresh are identified as future core concerns without requiring OAuth UI, platform credential storage, or concrete provider clients in this phase

#### Scenario: Provider execution ownership exposes deterministic mock boundaries
- **WHEN** maintainers inspect the `oxmux` public API or documentation after adding provider execution primitives
- **THEN** provider execution is represented by trait, request, result, mock harness, and structured outcome primitives that can be used in deterministic tests without requiring real provider SDKs, HTTP clients, OAuth, platform credential storage, GPUI, or app-shell state

#### Scenario: Routing ownership is visible but not implemented
- **WHEN** maintainers inspect the `oxmux` public API or documentation
- **THEN** model aliases, account targeting, priority, round-robin, failover, exhausted, and degraded routing states are identified as future core concerns without requiring routing algorithm implementations in this phase

#### Scenario: Protocol ownership exposes typed skeleton boundaries
- **WHEN** maintainers inspect the `oxmux` public API or documentation
- **THEN** OpenAI, Gemini, Claude, Codex, and provider-specific protocol translation are represented by typed request/response boundaries, typed protocol metadata, and deferred translation results without requiring request translators, response translators, or outbound provider calls in this phase

#### Scenario: Streaming ownership is visible but not implemented
- **WHEN** maintainers inspect the `oxmux` public API or documentation
- **THEN** streaming and non-streaming response support are identified as future core concerns without requiring network transports or provider stream adapters in this phase

#### Scenario: Management ownership includes local health runtime status
- **WHEN** maintainers inspect the `oxmux` public API or documentation
- **THEN** proxy lifecycle state, local health runtime status, provider listing, account health, usage, quota, and degraded service status are identified as core concerns while management endpoints beyond `/health` remain deferred

### Requirement: Core facade remains runtime-inert for this change
The `oxmux` management/lifecycle and provider execution facade SHALL remain usable without starting provider network transports, concrete provider clients, OAuth flows, token refresh, hot reload watchers, background proxy routing behavior, GPUI, or app-shell state, while also supporting an explicit local health runtime start operation for smoke testing and deterministic in-memory mock provider execution for provider boundary tests.

#### Scenario: Tests use deterministic local runtime state
- **WHEN** core tests verify management snapshots, configuration validation, provider/account summaries, lifecycle states, usage/quota summaries, and local health runtime behavior
- **THEN** they use deterministic in-memory values or loopback-only local listener state and do not require external network services, real credentials, provider accounts, a desktop app, or desktop platform APIs

#### Scenario: Tests use deterministic mock provider state
- **WHEN** core tests verify provider execution traits, mock provider outcomes, provider/account summary reflection, quota-limited outcomes, degraded outcomes, streaming-capable metadata, or failed mock outcomes
- **THEN** they use deterministic in-memory mock providers and do not require external network services, real credentials, provider SDKs, OAuth flows, token refresh, raw secret storage, a desktop app, or desktop platform APIs

#### Scenario: Protocol ownership remains explicit
- **WHEN** provider capabilities, provider execution requests, or routing defaults reference OpenAI, Gemini, Claude, Codex, or provider-specific protocol families
- **THEN** the facade identifies those protocol families as typed metadata but does not translate requests or responses in this change
