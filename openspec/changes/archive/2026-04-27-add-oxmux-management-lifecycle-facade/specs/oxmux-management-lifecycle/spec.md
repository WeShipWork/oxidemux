## ADDED Requirements

### Requirement: Core exposes management snapshot
The system SHALL provide an `oxmux` management snapshot that represents app-visible core state without requiring a running desktop app, GPUI window, IPC process, local proxy bind, or external provider call.

#### Scenario: Snapshot can be constructed directly
- **WHEN** Rust code depends on `oxmux` and constructs the management snapshot from in-memory values
- **THEN** it can inspect core identity, lifecycle state, health state, configuration summary, provider/account summaries, usage/quota summaries, warnings, and errors without launching `oxidemux`

#### Scenario: Snapshot reports degraded state
- **WHEN** one or more provider accounts, configuration entries, or lifecycle checks are degraded
- **THEN** the management snapshot exposes structured degraded reasons that the app shell can display without reimplementing degradation logic

### Requirement: Core exposes proxy lifecycle state and control intents
The system SHALL define typed proxy lifecycle states and control intents for start, stop, restart, and status refresh operations without implementing real HTTP proxy startup in this change.

#### Scenario: Lifecycle state is visible to app shell
- **WHEN** `oxidemux` asks `oxmux` for current lifecycle status
- **THEN** it receives a typed state such as stopped, starting, running, degraded, failed, or stopping with any relevant bound endpoint, uptime, and last-error metadata

#### Scenario: Control intent does not bind a proxy server
- **WHEN** Rust code constructs or validates a start, stop, restart, or refresh-status intent in this change
- **THEN** `oxmux` does not bind a socket, spawn a background proxy server, open a window, or call an external provider

### Requirement: Core exposes app-visible configuration snapshots
The system SHALL define typed configuration snapshots and update intents for app-visible proxy settings while deferring file persistence and hot reload.

#### Scenario: Configuration includes proxy and observability settings
- **WHEN** the app shell reads the core configuration snapshot
- **THEN** it can inspect listen address, port, auto-start intent, logging setting, usage collection setting, and routing default names through typed fields

#### Scenario: Invalid configuration surfaces structured errors
- **WHEN** a configuration update intent contains invalid listen address, port, routing default, or provider reference data
- **THEN** `oxmux` returns a structured core error instead of silently accepting or discarding the invalid value

### Requirement: Core exposes provider and account summaries
The system SHALL define provider and account summary types for app-visible provider status without implementing OAuth, token refresh, concrete provider clients, or platform credential storage.

#### Scenario: Provider capabilities are visible
- **WHEN** `oxidemux` reads provider summaries from `oxmux`
- **THEN** each provider can expose typed identity and capability metadata such as supported protocol family, streaming support, auth method category, and routing eligibility

#### Scenario: Account auth and quota placeholders are visible
- **WHEN** `oxidemux` reads account summaries from `oxmux`
- **THEN** each account can expose auth state, optional quota/status placeholder data, last-checked metadata, and degraded/error reasons without exposing stored secrets

#### Scenario: Credential storage remains outside core implementation
- **WHEN** a provider account needs a real OAuth token, API key, or platform-protected secret
- **THEN** this change only represents the credential state or reference and does not add keychain, secret-service, OAuth UI, or desktop storage dependencies to `oxmux`

### Requirement: Core exposes usage and quota summaries for dashboards
The system SHALL define usage and quota summary types that can power future dashboard cards without implementing analytics persistence or network-backed quota fetching.

#### Scenario: Usage summary can be empty but typed
- **WHEN** no requests have been proxied yet
- **THEN** `oxmux` can still return a typed usage summary indicating zero or unknown request, token, model, provider, and account totals

#### Scenario: Quota summary distinguishes unknown and degraded states
- **WHEN** quota data has not been fetched or cannot be trusted
- **THEN** `oxmux` represents unknown, unavailable, or degraded quota state explicitly instead of overloading numeric counters

### Requirement: Core errors cover management and lifecycle failures
The system SHALL expose structured errors for management snapshot, lifecycle intent, configuration validation, provider/account summary, and usage/quota summary failures.

#### Scenario: App shell can display core lifecycle errors
- **WHEN** a lifecycle or management operation fails in `oxmux`
- **THEN** `oxidemux` can receive a typed error with enough category and message data to display or log it without string parsing internal implementation details
