## MODIFIED Requirements

### Requirement: Core exposes management snapshot
The system SHALL provide an `oxmux` management snapshot that represents app-visible core state and can reflect the minimal local health runtime, deterministic mock provider execution health, and protected local management/status/control route boundary metadata without requiring a running desktop app, GPUI window, IPC process, external provider call, OAuth flow, routing engine, network-backed quota fetch, or platform credential storage.

#### Scenario: Snapshot can be constructed directly
- **WHEN** Rust code depends on `oxmux` and constructs the management snapshot from in-memory values
- **THEN** it can inspect core identity, lifecycle state, health state, configuration summary, provider/account summaries, usage/quota summaries, local management boundary metadata, warnings, and errors without launching `oxidemux`

#### Scenario: Snapshot reports degraded state
- **WHEN** one or more provider accounts, mock provider outcomes, configuration entries, local authorization checks, or lifecycle checks are degraded
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

#### Scenario: Snapshot does not expose local client secrets
- **WHEN** local client authorization is configured for inference or management/status/control access
- **THEN** the management snapshot can expose whether local route protection is configured and healthy without exposing raw local client authorization secrets
