## MODIFIED Requirements

### Requirement: Minimal public facade for future core domains
The `oxmux` crate SHALL expose a small public facade that establishes ownership of proxy lifecycle, local health runtime, local client authorization, provider/auth, provider execution, routing, protocol translation, configuration, streaming, management/status, usage/quota, domain error primitives, and a minimal concrete proxy request smoke path without implementing full provider SDK integration, outbound provider calls, credential storage, full proxy request handling, remote management panels, or real streaming transport adapters in this change.

#### Scenario: Provider auth ownership is visible but not implemented
- **WHEN** maintainers inspect the `oxmux` public API or documentation
- **THEN** provider authentication and token refresh are identified as future core concerns without requiring OAuth UI, platform credential storage, or concrete provider clients in this phase

#### Scenario: Local client authorization ownership is visible
- **WHEN** maintainers inspect the `oxmux` public API or documentation after adding local client authorization boundaries
- **THEN** local proxy client authorization, inference access, management/status/control access, redacted local client credential metadata, and structured unauthorized outcomes are represented as headless core concerns without requiring GPUI, desktop credential storage, OAuth UI, provider SDKs, or app-shell state

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
- **THEN** proxy lifecycle state, local health runtime status, provider listing, account health, usage, quota, degraded service status, and protected management/status/control route boundaries are identified as core concerns while full remote management panels remain deferred
