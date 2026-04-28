## Purpose

Define the `oxmux` headless Rust core crate boundary for direct library use
without desktop UI, platform lifecycle, or app-shell dependencies.

`oxmux` is the shared product engine for subscription-aware local proxying. It
owns reusable semantics for protocol compatibility, request rewriting, model
aliases, reasoning/thinking compatibility primitives, subscription-aware routing,
provider/account state, management snapshots, usage/quota state, and structured
errors. Desktop shells and platform adapters provide UI and OS integrations, but
they must not redefine those core semantics.
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
The `oxmux` crate SHALL expose a small public facade that establishes ownership of proxy lifecycle, local health runtime, provider/auth, provider execution, routing, protocol translation, configuration, streaming, management/status, usage/quota, and domain error primitives without implementing full provider SDK integration, outbound provider calls, credential storage, concrete proxy request handling, or real streaming transport adapters in this change.

#### Scenario: Provider auth ownership is visible but not implemented
- **WHEN** maintainers inspect the `oxmux` public API or documentation
- **THEN** provider authentication and token refresh are identified as future core concerns without requiring OAuth UI, platform credential storage, or concrete provider clients in this phase

#### Scenario: Provider execution ownership exposes deterministic mock boundaries
- **WHEN** maintainers inspect the `oxmux` public API or documentation after adding provider execution primitives
- **THEN** provider execution is represented by trait, request, result, mock harness, and structured outcome primitives that can be used in deterministic tests without requiring real provider SDKs, HTTP clients, OAuth, platform credential storage, GPUI, or app-shell state

#### Scenario: Routing ownership exposes typed policy primitives
- **WHEN** maintainers inspect the `oxmux` public API or documentation after adding routing policy primitives
- **THEN** model aliases, account targeting, priority, fallback, exhausted states, degraded states, selection outcomes, skipped candidate metadata, and routing failure details are represented by typed public primitives without requiring concrete proxy routing behavior, provider SDKs, outbound provider calls, GPUI, or app-shell state

#### Scenario: Protocol ownership exposes typed skeleton boundaries
- **WHEN** maintainers inspect the `oxmux` public API or documentation
- **THEN** OpenAI, Gemini, Claude, Codex, and provider-specific protocol translation are represented by typed request/response boundaries, typed protocol metadata, and deferred translation results without requiring request translators, response translators, or outbound provider calls in this phase

#### Scenario: Streaming ownership exposes typed response primitives
- **WHEN** maintainers inspect the `oxmux` public API or documentation after adding streaming response primitives
- **THEN** response mode, complete responses, ordered stream events, in-sequence terminal events, stream completion, stream cancellation, stream errors, streaming failure details, and deterministic mock stream outcomes are represented by typed public primitives without requiring network transports, provider stream adapters, provider SDKs, outbound provider calls, GPUI, or app-shell state

#### Scenario: Management ownership includes local health runtime status
- **WHEN** maintainers inspect the `oxmux` public API or documentation
- **THEN** proxy lifecycle state, local health runtime status, provider listing, account health, usage, quota, and degraded service status are identified as core concerns while management endpoints beyond `/health` remain deferred

### Requirement: Core owns subscription proxy semantics
The `oxmux` crate SHALL own the reusable subscription-aware proxy semantics needed to normalize local AI requests, represent model aliases, expose reasoning/thinking request compatibility primitives, accept app-supplied provider/account availability, route requests through deterministic policy, and return structured outcomes without depending on GPUI, tray/menu libraries, OAuth UI, platform credential storage, provider SDKs, or the `oxidemux` app shell.

#### Scenario: Thinking and model compatibility are core primitives
- **WHEN** future local proxy request handling accepts provider-shaped or OpenAI-compatible requests that include model aliases, reasoning budgets, or thinking-mode conventions
- **THEN** the normalization, typed metadata, and routing-relevant semantics are exposed through `oxmux` primitives while desktop-specific controls remain in `oxidemux`

#### Scenario: Subscription state is supplied without shell-owned routing
- **WHEN** future app or platform adapters know account auth health, quota pressure, subscription availability, degraded state, or user-selected provider/account preferences
- **THEN** they pass typed availability inputs or credential references into `oxmux`, and `oxmux` owns the route selection, fallback reason, and structured failure outcome

### Requirement: Core errors remain visible to consumers
The `oxmux` crate SHALL define or reserve a core error boundary so future fallible proxy, provider, configuration, routing, protocol, streaming, and management operations can propagate meaningful errors to library consumers and app layers, including structured routing and streaming failures that callers can match without parsing display text.

#### Scenario: Core error type is usable by the app shell
- **WHEN** `oxidemux` calls into the `oxmux` facade
- **THEN** fallible operations expose typed or structured errors that can be propagated or displayed rather than silently discarded

#### Scenario: Routing failures are matchable
- **WHEN** `oxmux` route selection fails because a route is missing, a target is missing, candidates are exhausted, only degraded candidates remain, or policy input is invalid
- **THEN** the returned `CoreError` includes structured routing failure data that Rust consumers can match without parsing display text

#### Scenario: Streaming failures are matchable
- **WHEN** `oxmux` streaming response validation fails because a stream sequence is invalid, or mock streaming execution fails before a stream response exists
- **THEN** the returned `CoreError` includes structured streaming failure data that Rust consumers can match without parsing display text

#### Scenario: Streaming capability remains metadata
- **WHEN** maintainers inspect provider summaries after adding streaming response primitives
- **THEN** streaming capability is represented as provider capability metadata independently from whether the current execution returned a complete or streaming response mode

#### Scenario: Delivered stream terminal states remain response data
- **WHEN** `oxmux` receives or constructs a valid stream response that terminates as cancelled or errored after zero or more events
- **THEN** the cancellation or stream error is represented as typed terminal response data rather than being converted into `Err(CoreError)` and losing the delivered event sequence

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
The `oxmux` management/lifecycle and provider execution facade SHALL remain usable without starting provider network transports, concrete provider clients, OAuth flows, token refresh, hot reload watchers, background proxy routing behavior, GPUI, or app-shell state, while also supporting an explicit local health runtime start operation for smoke testing and deterministic in-memory mock provider execution for provider boundary tests.

#### Scenario: Tests use deterministic local runtime state
- **WHEN** core tests verify management snapshots, configuration validation, provider/account summaries, lifecycle states, usage/quota summaries, and local health runtime behavior
- **THEN** they use deterministic in-memory values or loopback-only local listener state and do not require external network services, real credentials, provider accounts, a desktop app, or desktop platform APIs

#### Scenario: Tests use deterministic mock provider state
- **WHEN** core tests verify provider execution traits, mock provider outcomes, provider/account summary reflection, quota-limited outcomes, degraded outcomes, streaming-capable metadata, or failed mock outcomes
- **THEN** they use deterministic in-memory mock providers and do not require external network services, real credentials, provider SDKs, OAuth flows, token refresh, raw secret storage, a desktop app, or desktop platform APIs

#### Scenario: Protocol ownership remains explicit
- **WHEN** provider capabilities, provider execution requests, or routing defaults reference OpenAI, Gemini, Claude, Codex, or provider-specific protocol families
- **THEN** the facade identifies those protocol families as typed metadata but does not translate requests or responses in this change
