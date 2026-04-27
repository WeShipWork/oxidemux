## ADDED Requirements

### Requirement: Provider execution trait boundary
The `oxmux` core SHALL expose a provider execution trait boundary that accepts an explicitly selected provider/account execution request using canonical protocol request data and returns a structured provider execution outcome without requiring provider SDKs, HTTP clients, OAuth flows, platform credential storage, GPUI, or `oxidemux` app-shell state.

#### Scenario: Execute explicit provider request
- **WHEN** a Rust consumer constructs a provider execution request with a provider identifier, optional account identifier, and `CanonicalProtocolRequest`
- **THEN** a provider executor can process the request through typed `oxmux` primitives without launching `oxidemux`, opening a window, starting IPC, performing protocol translation, contacting an external provider, or reading stored credentials

#### Scenario: Execution boundary returns structured failures
- **WHEN** a provider execution boundary cannot complete a request because the selected mock outcome is failed or invalid
- **THEN** `oxmux` returns a structured provider execution error that callers can match without parsing display text

### Requirement: Deterministic mock provider harness
The `oxmux` core SHALL provide an in-repo mock provider harness for tests that can deterministically return success, degraded, quota-limited, streaming-capable, and failed provider execution outcomes.

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
- **THEN** provider capability metadata reports streaming support while execution still returns deterministic in-memory canonical response data and does not implement streaming transport

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
Default `oxmux` tests for provider execution SHALL use deterministic in-memory mocks and SHALL NOT require real provider accounts, credentials, provider SDKs, outbound provider network calls, OAuth flows, token refresh, platform secret storage, GPUI, or `oxidemux` app-shell dependencies.

#### Scenario: Dependency boundary excludes provider integrations
- **WHEN** maintainers inspect or run default `oxmux` provider execution tests
- **THEN** those tests pass using in-repo mock providers and the `oxmux` crate remains free of provider SDK, HTTP client, OAuth, platform credential storage, GPUI, and `oxidemux` dependencies
