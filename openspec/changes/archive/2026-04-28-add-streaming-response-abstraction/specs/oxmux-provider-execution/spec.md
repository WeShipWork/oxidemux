## MODIFIED Requirements

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

### Requirement: Default provider execution tests remain networkless
Default `oxmux` tests for provider execution SHALL use deterministic in-memory mocks and SHALL NOT require real provider accounts, credentials, provider SDKs, outbound provider network calls, upstream streaming endpoints, OAuth flows, token refresh, platform secret storage, GPUI, or `oxidemux` app-shell dependencies.

#### Scenario: Dependency boundary excludes provider integrations
- **WHEN** maintainers inspect or run default `oxmux` provider execution tests
- **THEN** those tests pass using in-repo mock providers and the `oxmux` crate remains free of provider SDK, HTTP client, OAuth, platform credential storage, GPUI, and `oxidemux` dependencies

#### Scenario: Streaming mock tests use in-memory events
- **WHEN** maintainers run provider execution tests for streaming mock outcomes
- **THEN** those tests assert deterministic in-memory stream events, terminal states, event order, validation reuse, and streaming capability metadata without contacting real streaming endpoints or depending on provider transport code
