## MODIFIED Requirements

### Requirement: Core exposes management snapshot
The system SHALL provide an `oxmux` management snapshot that represents app-visible core state and can reflect the minimal local health runtime without requiring a running desktop app, GPUI window, IPC process, external provider call, OAuth flow, routing engine, quota fetch, or platform credential storage.

#### Scenario: Snapshot can be constructed directly
- **WHEN** Rust code depends on `oxmux` and constructs the management snapshot from in-memory values
- **THEN** it can inspect core identity, lifecycle state, health state, configuration summary, provider/account summaries, usage/quota summaries, warnings, and errors without launching `oxidemux`

#### Scenario: Snapshot reports degraded state
- **WHEN** one or more provider accounts, configuration entries, or lifecycle checks are degraded
- **THEN** the management snapshot exposes structured degraded reasons that the app shell can display without reimplementing degradation logic

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
