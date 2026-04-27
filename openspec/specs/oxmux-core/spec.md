## Purpose

Define the `oxmux` headless Rust core crate boundary for direct library use
without desktop UI, platform lifecycle, or app-shell dependencies.

## Requirements

### Requirement: Headless core crate boundary
The system SHALL provide an `oxmux` Rust library crate that is usable without GPUI, desktop lifecycle code, tray integration, updater logic, packaging code, app-specific UI dependencies, provider execution dependencies, OAuth UI, or platform credential storage dependencies while owning reusable local proxy runtime behavior.

#### Scenario: Core builds without app dependencies
- **WHEN** workspace checks are run for the `oxmux` crate after adding the local health runtime
- **THEN** `oxmux` builds and tests without depending on GPUI, gpui-component, tray libraries, updater libraries, packaging libraries, platform credential storage libraries, provider SDKs, OAuth UI libraries, or the `oxidemux` app crate

#### Scenario: Core can be used by a Rust consumer
- **WHEN** a Rust test or example depends on `oxmux` directly
- **THEN** it can construct the public core facade and start, query, and shut down the minimal local health runtime without launching the `oxidemux` binary, opening a window, starting IPC, or contacting an external provider

### Requirement: Minimal public facade for future core domains
The `oxmux` crate SHALL expose a small public facade that establishes ownership of proxy lifecycle, local health runtime, provider/auth, routing, protocol translation, configuration, streaming, management/status, usage/quota, and domain error primitives without implementing full provider or proxy routing behavior in this change.

#### Scenario: Provider auth ownership is visible but not implemented
- **WHEN** maintainers inspect the `oxmux` public API or documentation
- **THEN** provider authentication and token refresh are identified as future core concerns without requiring OAuth UI, platform credential storage, or concrete provider clients in this phase

#### Scenario: Routing ownership is visible but not implemented
- **WHEN** maintainers inspect the `oxmux` public API or documentation
- **THEN** model aliases, account targeting, priority, round-robin, failover, exhausted, and degraded routing states are identified as future core concerns without requiring routing algorithm implementations in this phase

#### Scenario: Protocol ownership is visible but not implemented
- **WHEN** maintainers inspect the `oxmux` public API or documentation
- **THEN** OpenAI, Gemini, Claude, Codex, and provider-specific protocol translation are identified as future core concerns without requiring request or response translators in this phase

#### Scenario: Streaming ownership is visible but not implemented
- **WHEN** maintainers inspect the `oxmux` public API or documentation
- **THEN** streaming and non-streaming response support are identified as future core concerns without requiring network transports or provider stream adapters in this phase

#### Scenario: Management ownership includes local health runtime status
- **WHEN** maintainers inspect the `oxmux` public API or documentation
- **THEN** proxy lifecycle state, local health runtime status, provider listing, account health, usage, quota, and degraded service status are identified as core concerns while management endpoints beyond `/health` remain deferred

### Requirement: Core errors remain visible to consumers
The `oxmux` crate SHALL define or reserve a core error boundary so future fallible proxy, provider, configuration, routing, protocol, streaming, and management operations can propagate meaningful errors to library consumers and app layers.

#### Scenario: Core error type is usable by the app shell
- **WHEN** `oxidemux` calls into the `oxmux` facade
- **THEN** fallible operations expose typed or structured errors that can be propagated or displayed rather than silently discarded

### Requirement: Workspace verification includes core crate
The project verification commands SHALL include the `oxmux` crate in formatting, linting, checking, and testing.

#### Scenario: Core is checked by default verification
- **WHEN** maintainers run the repository's documented cargo, mise, or CI verification commands
- **THEN** the `oxmux` crate is included in fmt, clippy, check, and test coverage

### Requirement: Core facade includes management lifecycle primitives
The `oxmux` public facade SHALL expose the minimal management/lifecycle primitives needed by app and Rust consumers while preserving the headless core crate boundary.

#### Scenario: Facade exports management types
- **WHEN** Rust code imports the public `oxmux` facade
- **THEN** it can access the management snapshot, lifecycle state, lifecycle control intent, configuration snapshot, provider/account summary, usage/quota summary, and related error types without importing `oxidemux`

#### Scenario: Core remains dependency-light
- **WHEN** maintainers inspect `crates/oxmux/Cargo.toml` after adding management lifecycle primitives
- **THEN** the crate still does not depend on GPUI, gpui-component, tray libraries, updater libraries, packaging libraries, platform credential storage libraries, or the `oxidemux` app crate

### Requirement: Core facade remains runtime-inert for this change
The `oxmux` management/lifecycle facade SHALL remain usable without starting network transports, protocol translators, provider clients, OAuth flows, token refresh, hot reload watchers, or background proxy routing behavior, while also supporting an explicit local health runtime start operation for smoke testing.

#### Scenario: Tests use deterministic local runtime state
- **WHEN** core tests verify management snapshots, configuration validation, provider/account summaries, lifecycle states, usage/quota summaries, and local health runtime behavior
- **THEN** they use deterministic in-memory values or loopback-only local listener state and do not require external network services, real credentials, provider accounts, a desktop app, or desktop platform APIs

#### Scenario: Protocol ownership remains explicit
- **WHEN** provider capabilities or routing defaults reference OpenAI, Gemini, Claude, Codex, or provider-specific protocol families
- **THEN** the facade identifies those protocol families as typed metadata but does not translate requests or responses in this change
