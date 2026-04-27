## MODIFIED Requirements

### Requirement: Core exposes management snapshot
The system SHALL provide an `oxmux` management snapshot that represents app-visible core state and can reflect the minimal local health runtime and deterministic mock provider execution health without requiring a running desktop app, GPUI window, IPC process, external provider call, OAuth flow, routing engine, network-backed quota fetch, or platform credential storage.

#### Scenario: Snapshot can be constructed directly
- **WHEN** Rust code depends on `oxmux` and constructs the management snapshot from in-memory values
- **THEN** it can inspect core identity, lifecycle state, health state, configuration summary, provider/account summaries, usage/quota summaries, warnings, and errors without launching `oxidemux`

#### Scenario: Snapshot reports degraded state
- **WHEN** one or more provider accounts, mock provider outcomes, configuration entries, or lifecycle checks are degraded
- **THEN** the management snapshot exposes structured degraded reasons that the app shell can display without reimplementing degradation logic

#### Scenario: Snapshot reflects failed mock provider state
- **WHEN** a deterministic mock provider execution outcome is failed
- **THEN** provider/account summaries and snapshot health data can expose the failed state through existing `ProviderSummary`, `AccountSummary`, `CoreHealthState`, warnings, and structured `CoreError` values without app-shell-specific copies

#### Scenario: Snapshot reflects quota-limited mock provider state
- **WHEN** a deterministic mock provider execution outcome is quota-limited
- **THEN** provider/account summaries and snapshot quota data can expose that state through existing `QuotaState` and `QuotaSummary` values without adding a mock-only quota model

#### Scenario: Snapshot reflects local runtime status
- **WHEN** the minimal local health runtime starts, fails to bind, runs, or shuts down
- **THEN** the management snapshot can expose the corresponding lifecycle state, bound endpoint metadata when available, and structured error data when startup fails

### Requirement: Core exposes provider and account summaries
The system SHALL define provider and account summary types for app-visible provider status and deterministic mock provider execution status without implementing OAuth, token refresh, concrete provider clients, outbound provider calls, provider SDKs, or platform credential storage.

#### Scenario: Provider capabilities are visible
- **WHEN** `oxidemux` or another Rust consumer reads provider summaries from `oxmux`
- **THEN** each provider can expose typed identity and capability metadata such as supported protocol family, streaming support, auth method category, and routing eligibility

#### Scenario: Mock streaming capability is visible without streaming transport
- **WHEN** a mock provider is configured as streaming-capable
- **THEN** its provider capability metadata can indicate streaming support without requiring streaming adapters, network transports, or provider SDKs

#### Scenario: Account auth and quota placeholders are visible
- **WHEN** `oxidemux` or another Rust consumer reads account summaries from `oxmux`
- **THEN** each account can expose auth state, optional quota/status placeholder data, last-checked metadata, and degraded/error reasons without exposing stored secrets

#### Scenario: Credential storage remains outside core implementation
- **WHEN** a provider account needs a real OAuth token, API key, or platform-protected secret
- **THEN** this change only represents the credential state or reference and does not add keychain, secret-service, OAuth UI, or desktop storage dependencies to `oxmux`
