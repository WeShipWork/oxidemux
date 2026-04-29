## Purpose

Define the app-facing `oxmux` management, lifecycle, configuration, provider/account, and usage/quota facade that can be consumed without launching the desktop app or starting provider-backed proxy routing behavior.
## Requirements
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

### Requirement: Core exposes proxy lifecycle state and control intents
The system SHALL define typed proxy lifecycle states and control intents for start, stop, restart, and status refresh operations, and SHALL use those states to report the minimal local health runtime lifecycle without implementing provider-backed proxy routing in this change.

#### Scenario: Lifecycle state is visible to app shell
- **WHEN** `oxidemux` asks `oxmux` for current lifecycle status
- **THEN** it receives a typed state such as stopped, starting, running, degraded, failed, or stopping with any relevant bound endpoint, uptime, and last-error metadata

#### Scenario: Control intent can start local health runtime only
- **WHEN** Rust code invokes a start intent for this change
- **THEN** `oxmux` may bind the minimal local health listener but does not start protocol translation, provider clients, routing, OAuth flows, GPUI, tray lifecycle, or external network calls

#### Scenario: Bind failures are structured lifecycle failures
- **WHEN** the local health runtime cannot bind its configured endpoint
- **THEN** lifecycle status becomes failed with structured error details that app and library consumers can display or log without string parsing

#### Scenario: Stop intent shuts down local runtime
- **WHEN** Rust code invokes a stop intent against a running local health runtime
- **THEN** `oxmux` shuts down the listener and reports stopped lifecycle status without leaving detached runtime work alive

### Requirement: Core exposes app-visible configuration snapshots
The system SHALL define typed configuration snapshots and update intents for app-visible proxy settings, including deterministic local health runtime listen configuration, while deferring file persistence and hot reload.

#### Scenario: Configuration includes proxy and observability settings
- **WHEN** the app shell reads the core configuration snapshot
- **THEN** it can inspect listen address, port, auto-start intent, logging setting, usage collection setting, and routing default names through typed fields

#### Scenario: Invalid configuration surfaces structured errors
- **WHEN** a configuration update intent contains invalid listen address, port, routing default, provider reference data, or non-local health runtime bind data
- **THEN** `oxmux` returns a structured core error instead of silently accepting or discarding the invalid value

#### Scenario: Runtime configuration remains deterministic
- **WHEN** tests or app-shell smoke checks construct local health runtime configuration
- **THEN** they can use explicit loopback listen settings and inspect the bound endpoint selected by the runtime

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

### Requirement: Management snapshot reflects file-backed configuration
The `oxmux` management snapshot SHALL reflect the currently active validated file-backed configuration, including configuration source metadata, listen address, port, auto-start intent, logging setting, usage collection setting, routing default names, provider/account reference summaries, warnings, and structured validation errors from failed replacement attempts.

The active configuration portion of the management snapshot SHALL always reflect the last successfully validated configuration. File-loaded provider/account summaries SHALL represent configured declarations and references only; they SHALL NOT imply verified auth health, subscription health, quota availability, provider availability, or credential usability without separate core auth/provider state. Failed replacement details SHALL be exposed separately as last configuration load failure metadata and SHALL NOT overwrite active listen settings, routing defaults, provider/account summaries, logging setting, usage collection setting, or auto-start intent. If there has never been a successful file-backed load, failed-load metadata MAY be visible while active file-backed configuration remains absent. A successful replacement SHALL clear previous failed-load metadata.

#### Scenario: Snapshot updates after valid file configuration load
- **WHEN** a valid local TOML configuration is loaded and applied through `oxmux`
- **THEN** the management snapshot exposes the loaded source metadata and app-visible configuration fields without duplicating validation logic in `oxidemux`

#### Scenario: Snapshot keeps last valid state after failed replacement
- **WHEN** a configuration replacement attempt fails because of invalid listen settings, port, routing defaults, provider references, logging settings, usage collection settings, or auto-start intent after a valid configuration was active
- **THEN** the management snapshot preserves the last valid active configuration and exposes structured validation errors for the failed attempt

#### Scenario: Snapshot represents initial load failure without synthetic health
- **WHEN** the first file-backed configuration load fails before any active file-backed configuration exists
- **THEN** the management snapshot can expose failed-load metadata without synthesizing active listen settings, provider/account health, quota health, or routing defaults from unrelated defaults

#### Scenario: Configured accounts remain auth-unverified
- **WHEN** management snapshot exposes provider/account summaries derived from file-backed configuration
- **THEN** those summaries identify configured declarations without marking auth state, subscription health, quota pressure, provider availability, or credential usability as verified

#### Scenario: Successful replacement clears failed replacement details
- **WHEN** a valid configuration replacement succeeds after a previous failed replacement attempt
- **THEN** the management snapshot reflects the new active configuration and no longer exposes the previous failed replacement as current load-failure metadata

#### Scenario: Auto-start remains intent only
- **WHEN** management snapshot exposes auto-start intent loaded from file-backed configuration
- **THEN** it represents the user's desired lifecycle setting as typed core state without registering OS launch services, mutating login items, starting tray code, or persisting platform lifecycle settings
