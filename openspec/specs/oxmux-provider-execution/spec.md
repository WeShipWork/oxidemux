# oxmux-provider-execution Specification

## Purpose
TBD - created by archiving change define-provider-execution-mock-harness. Update Purpose after archive.
## Requirements
### Requirement: Provider execution trait boundary
The `oxmux` core SHALL expose a provider execution trait boundary that accepts an explicitly selected provider/account execution request using canonical protocol request data and returns a structured provider execution outcome without requiring provider SDKs, HTTP clients, OAuth flows, platform credential storage, GPUI, or `oxidemux` app-shell state.

#### Scenario: Execute explicit provider request
- **WHEN** a Rust consumer constructs a provider execution request with a provider identifier, optional account identifier, and `CanonicalProtocolRequest`
- **THEN** a provider executor can process the request through typed `oxmux` primitives without launching `oxidemux`, opening a window, starting IPC, performing protocol translation, contacting an external provider, or reading stored credentials

#### Scenario: Execution boundary returns structured failures
- **WHEN** a provider execution boundary cannot complete a request because the selected mock outcome is failed or invalid
- **THEN** `oxmux` returns a structured provider execution error that callers can match without parsing display text

### Requirement: Deterministic mock provider harness
The `oxmux` core SHALL provide an in-repo mock provider harness for tests that can deterministically return success, degraded, quota-limited, streaming-capable metadata, deterministic streaming response, and failed provider execution outcomes.

#### Scenario: Mock provider returns success
- **WHEN** a test configures a mock provider with a success outcome and executes a canonical request
- **THEN** the harness returns the configured canonical response and provider/account metadata deterministically without network access

#### Scenario: Mock provider returns degraded response
- **WHEN** a test configures a mock provider with a degraded outcome
- **THEN** the harness returns a deterministic execution result that includes a canonical response plus `DegradedReason` metadata for the affected provider or account

#### Scenario: Mock provider returns quota-limited response
- **WHEN** a test configures a mock provider with a quota-limited outcome
- **THEN** the harness returns deterministic provider/account summary data using existing `QuotaState` values rather than a mock-only quota model

#### Scenario: Mock provider reports streaming capability
- **WHEN** a test configures a mock provider as streaming-capable
- **THEN** provider capability metadata reports streaming support without requiring a real provider streaming endpoint or forcing the configured execution outcome to be a streaming response

#### Scenario: Mock provider returns deterministic stream events
- **WHEN** a test configures a mock provider with a deterministic streaming response outcome
- **THEN** the harness returns the configured `ResponseMode::Streaming` value containing ordered stream events and the terminal event without network access, provider SDKs, HTTP streaming, or app-shell state

#### Scenario: Streaming outcome implies streaming capability
- **WHEN** a test configures a mock provider with a deterministic streaming response outcome
- **THEN** provider capability metadata reports streaming support using existing `ProviderCapability` data

#### Scenario: Complete outcome can still report streaming capability
- **WHEN** a test configures a mock provider that supports streaming but returns a complete response for the current execution
- **THEN** provider capability metadata still reports streaming support while the returned response mode remains complete

#### Scenario: Mock provider returns failure
- **WHEN** a test configures a mock provider with a failed outcome
- **THEN** the harness returns a structured provider execution failure and can surface failed provider health through existing `oxmux` health and summary types

### Requirement: Provider/account summary reflection
The `oxmux` provider execution mock harness SHALL reflect mock provider health through existing provider, account, quota, degraded reason, and management snapshot types instead of introducing app-shell-specific or mock-only summary copies.

#### Scenario: Mock provider summary uses core provider models
- **WHEN** a mock provider is inspected after execution
- **THEN** its provider and account state is represented with `ProviderSummary`, `ProviderCapability`, `AccountSummary`, `AuthState`, `QuotaState`, `LastCheckedMetadata`, and `DegradedReason` as applicable

#### Scenario: Management snapshot can include mock provider health
- **WHEN** a management snapshot is constructed from mock provider summary data
- **THEN** it can expose provider health, account health, quota state, warnings, and errors through `ManagementSnapshot` and `CoreHealthState` without duplicating state in `oxidemux`

### Requirement: Default provider execution tests remain networkless
Default `oxmux` tests for provider execution SHALL use deterministic in-memory mocks and SHALL NOT require real provider accounts, credentials, provider SDKs, outbound provider network calls, upstream streaming endpoints, OAuth flows, token refresh, platform secret storage, GPUI, or `oxidemux` app-shell dependencies.

#### Scenario: Dependency boundary excludes provider integrations
- **WHEN** maintainers inspect or run default `oxmux` provider execution tests
- **THEN** those tests pass using in-repo mock providers and the `oxmux` crate remains free of provider SDK, HTTP client, OAuth, platform credential storage, GPUI, and `oxidemux` dependencies

#### Scenario: Streaming mock tests use in-memory events
- **WHEN** maintainers run provider execution tests for streaming mock outcomes
- **THEN** those tests assert deterministic in-memory stream events, terminal states, event order, validation reuse, and streaming capability metadata without contacting real streaming endpoints or depending on provider transport code

### Requirement: Mock providers support deterministic streaming robustness outcomes
The `oxmux` core SHALL extend in-repo mock provider behavior so tests can deterministically model pre-event stream failure, bootstrap retry success, retry exhaustion, idle keepalive, cancellation, post-partial provider error, and clean completion without a real upstream provider streaming endpoint.

#### Scenario: Mock provider can fail before first event then succeed
- **WHEN** a test configures a mock streaming provider to fail before emitting any event and then succeed within the configured bootstrap retry budget
- **THEN** provider execution returns a deterministic streaming response from the successful attempt and exposes final retry summary data without delivering stream events from the failed attempt

#### Scenario: Mock provider can exhaust bootstrap retries
- **WHEN** a test configures a mock streaming provider to fail before emitting any event for more attempts than the configured bootstrap retry budget
- **THEN** provider execution returns a structured pre-stream failure identifying retry exhaustion, total attempts, and the underlying provider stream failure

#### Scenario: Zero retry budget performs no retry
- **WHEN** a test configures `bootstrap_retry_count = 0` and the initial mock streaming attempt fails before emitting an event
- **THEN** provider execution returns the pre-stream failure after the initial attempt without running an additional attempt

#### Scenario: Mock provider can error after partial stream
- **WHEN** a test configures a mock streaming provider to emit content or metadata and then fail
- **THEN** provider execution returns a `ResponseMode::Streaming` value whose ordered events include the delivered data and an errored terminal state rather than retrying the request

#### Scenario: Mock provider can propagate cancellation
- **WHEN** a test configures a mock streaming provider to observe cancellation behavior
- **THEN** provider execution returns a deterministic streaming response with a cancelled terminal state and matchable cancellation reason

#### Scenario: Mock provider can model client disconnect deterministically
- **WHEN** a test configures a mock streaming provider to observe `client-disconnect` cancellation behavior
- **THEN** provider execution uses deterministic in-memory cancellation outcomes without requiring a live HTTP request context, socket close, or transport adapter

#### Scenario: Mock provider can model pre-event cancellation
- **WHEN** a test configures a mock streaming provider to observe cancellation before any stream event is emitted
- **THEN** provider execution returns structured pre-stream cancellation data rather than a delivered streaming response

#### Scenario: Mock provider can model deterministic timeout
- **WHEN** a test configures a mock streaming provider to time out deterministically before any event is emitted
- **THEN** provider execution returns structured pre-stream timeout failure data using in-memory mock outcomes rather than requiring wall-clock timers, HTTP streams, or async transport backpressure

#### Scenario: Mock provider can model committed timeout metadata
- **WHEN** a test configures a mock streaming provider to emit any event and then time out deterministically
- **THEN** provider execution returns terminal streaming response data preserving delivered event history and timeout metadata

### Requirement: Provider streaming robustness remains metadata-driven
Provider execution SHALL keep streaming capability and robustness observations as typed metadata rather than provider-specific SDK values or transport-specific frames.

#### Scenario: Retry and timeout metadata are provider-neutral
- **WHEN** provider execution records bootstrap retry attempts, timeout metadata, or cancellation state
- **THEN** the data is represented through `oxmux` owned types and provider/account summaries without importing provider SDK error types

#### Scenario: Failed-attempt retry observations are not stream events
- **WHEN** provider execution records failed pre-event retry attempts
- **THEN** those observations remain provider execution or management metadata until a successful stream attempt is committed
- **AND** failed-attempt metadata does not appear in the ordered delivered stream sequence

#### Scenario: Streaming capability remains independent from response mode
- **WHEN** a provider advertises streaming capability or a robustness test configures a streaming outcome
- **THEN** provider capability metadata remains inspectable independently from whether the current execution returned a complete or streaming response

### Requirement: Provider execution receives normalized reasoning metadata
The `oxmux` provider execution request boundary SHALL be able to receive normalized provider-neutral reasoning or thinking metadata and compatibility outcome data alongside the canonical protocol request without requiring concrete provider SDKs, provider-specific beta headers, OAuth flows, platform credential storage, GPUI, or outbound provider network calls in this change. Provider execution request metadata SHALL carry normalized reasoning intent plus the compatibility outcome evaluated for the already-selected provider/account/model target, while provider execution result metadata MAY echo the final outcome for diagnostics and future management surfacing.

#### Scenario: Execution request preserves reasoning intent
- **WHEN** a Rust consumer constructs a provider execution request for a selected provider/account with normalized reasoning intent
- **THEN** the request preserves that intent as typed `oxmux` metadata without forcing the mock provider or execution boundary to generate provider-specific payload rewrites

#### Scenario: Execution request preserves compatibility outcome
- **WHEN** reasoning compatibility has been evaluated for the selected provider/account/model target before provider execution
- **THEN** the provider execution request can carry the normalized reasoning intent and the typed supported, ignored, degraded, unsupported, or unknown compatibility outcome without requiring the provider executor to reselect a route or parse opaque payload fields

#### Scenario: Execution metadata reports ignored reasoning capability
- **WHEN** reasoning intent is ignored because the selected provider/account/model target cannot honor alias-derived or permissive reasoning controls
- **THEN** provider execution metadata can expose the ignored-capability outcome so callers and future management snapshots can explain the compatibility downgrade

#### Scenario: Execution metadata can report degraded or unknown reasoning capability
- **WHEN** reasoning intent is degraded or has unknown support for the selected provider/account/model target
- **THEN** provider execution metadata can expose typed degraded or unknown outcome data so callers and future management snapshots can explain the compatibility state without parsing display text

#### Scenario: Mock provider tests remain deterministic
- **WHEN** provider execution tests cover reasoning metadata propagation
- **THEN** they use deterministic in-memory mock providers and do not require real provider accounts, credentials, provider SDKs, outbound network calls, provider-specific beta headers, OAuth flows, token refresh, platform secret storage, GPUI, or `oxidemux` app-shell dependencies

### Requirement: Provider execution does not own reasoning payload rewrites
Provider execution in this change SHALL NOT implement concrete provider-specific reasoning payload rewrites, beta headers, or SDK request construction; those behaviors SHALL remain deferred to future provider adapter or protocol translator changes that consume the normalized reasoning metadata.

#### Scenario: Mock provider preserves canonical request envelope
- **WHEN** a mock provider executes a request containing reasoning metadata
- **THEN** it preserves the canonical request envelope and typed metadata without claiming that OpenAI, Claude, Gemini, Codex, or provider-specific reasoning payload translation occurred

#### Scenario: Provider execution does not parse opaque reasoning fields
- **WHEN** a provider execution request contains an opaque payload body with provider-specific reasoning-looking fields
- **THEN** provider execution does not parse those fields for reasoning intent or compatibility state and relies only on typed `oxmux` reasoning metadata supplied before execution
