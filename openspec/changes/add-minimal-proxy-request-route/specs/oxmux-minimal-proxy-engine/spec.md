## ADDED Requirements

### Requirement: Minimal OpenAI-compatible chat-completion proxy path
The `oxmux` core SHALL expose a minimal headless proxy engine path for loopback OpenAI-compatible chat-completion smoke tests that accepts a bounded `POST /v1/chat/completions` request, constructs a canonical OpenAI protocol request, selects a provider/account target through caller-supplied routing policy and availability, executes that selected target through the provider execution trait, and serializes a deterministic non-streaming OpenAI-shaped response without requiring `oxidemux`, GPUI, provider SDKs, OAuth flows, credential storage, or outbound provider network calls.

#### Scenario: Chat-completion route returns deterministic mock response
- **WHEN** a loopback client sends a valid minimal `POST /v1/chat/completions` request to an `oxmux` runtime configured with an available mock provider route
- **THEN** `oxmux` returns a successful HTTP `200` response with `Content-Type: application/json` and deterministic OpenAI-compatible chat-completion JSON derived from the mock provider response

#### Scenario: Route exercises core boundaries
- **WHEN** the minimal chat-completion route handles a valid request
- **THEN** the request path constructs `CanonicalProtocolRequest`, performs routing selection with `RoutingBoundary`, builds `ProviderExecutionRequest` from the selected provider/account target, executes a `ProviderExecutor`, consumes `ResponseMode::Complete`, and serializes the provider result through public `oxmux` primitives

#### Scenario: Route remains headless
- **WHEN** maintainers inspect or run the minimal proxy engine tests
- **THEN** they pass without `oxidemux` process startup, GPUI windows, tray/menu APIs, updater code, packaging code, platform credential storage, OAuth UI, token refresh, provider SDKs, real provider accounts, or outbound upstream provider requests

### Requirement: Deterministic minimal request validation
The `oxmux` minimal proxy engine SHALL reject malformed, oversized, or unsupported smoke-route requests with deterministic structured core failures and stable local HTTP `400` JSON responses rather than panicking, silently ignoring invalid input, or passing ambiguous request data to routing or provider execution.

#### Scenario: Malformed chat request fails before routing
- **WHEN** a loopback client sends `POST /v1/chat/completions` with malformed JSON, a missing model field, a blank model field, or an unsupported request body for the minimal smoke route
- **THEN** `oxmux` returns a deterministic invalid-request response with `Content-Type: application/json` and stable `error.code`, and does not call routing selection or provider execution

#### Scenario: Unsupported method or path does not masquerade as proxy success
- **WHEN** a loopback client sends a request whose method or path is outside the supported health endpoint and minimal chat-completion smoke route
- **THEN** `oxmux` returns a deterministic HTTP `404` JSON unsupported-path response that does not contain a chat-completion success body

### Requirement: Provider execution failures surface deterministically
The `oxmux` minimal proxy engine SHALL convert routing failures, unsupported response modes, and provider execution failures from the selected provider executor into deterministic local proxy failure responses while preserving structured `CoreError` details for Rust consumers.

#### Scenario: Mock provider failure maps to proxy failure response
- **WHEN** routing selects a mock provider whose configured execution outcome is failed
- **THEN** `oxmux` returns a deterministic provider-failure HTTP `502` JSON response with stable `error.code` and exposes matchable provider execution failure details through core error data

#### Scenario: Routing failure prevents provider execution
- **WHEN** no route, no available target, exhausted candidates, or disallowed degraded candidates prevent routing selection for the requested model
- **THEN** `oxmux` returns a deterministic HTTP `502` JSON proxy failure response without invoking provider execution for an unselected provider/account, preserving structured routing failure data including exhausted, degraded, skipped, or missing-target details when available

#### Scenario: Unsupported response mode maps to proxy failure
- **WHEN** the selected provider executor returns a response mode other than `ResponseMode::Complete` for the minimal non-streaming smoke route
- **THEN** `oxmux` returns a deterministic HTTP `502` JSON proxy failure response with stable `error.code` and matchable core error details rather than serializing a partial or streaming success body

### Requirement: Minimal proxy tests remain networkless and deterministic
Default `oxmux` tests for the minimal proxy engine SHALL use loopback-only local requests and deterministic in-memory mock providers, routing policies, availability inputs, and response payloads without calls to external or upstream provider networks.

#### Scenario: Required proxy cases are covered
- **WHEN** maintainers run default `oxmux` tests for the minimal proxy engine
- **THEN** tests cover successful chat-completion response, invalid request before routing/provider execution, selected provider/account propagation, provider execution failure, routing failure or degraded/exhausted fallback behavior, and unsupported path without external services

#### Scenario: Smoke route does not imply full OpenAI compatibility
- **WHEN** maintainers inspect the minimal proxy engine tests and specifications
- **THEN** the supported behavior is limited to the minimal non-streaming chat-completion smoke route and does not claim support for streaming, tools, function calling, multimodal input, embeddings, completions, models listing, provider-prefixed routes, or full OpenAI API compatibility
