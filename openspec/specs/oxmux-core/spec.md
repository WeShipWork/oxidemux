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
### Requirement: Core facade exposes minimal proxy engine path
The `oxmux` public facade SHALL expose the minimal proxy engine primitives needed for Rust consumers and tests to exercise a local OpenAI-compatible chat-completion smoke path through protocol, routing, provider execution, response mode handling, and structured errors without importing `oxidemux` or desktop-specific code.

#### Scenario: Rust consumer exercises minimal engine without app shell
- **WHEN** a Rust test or library consumer constructs the minimal proxy engine inputs through the `oxmux` facade
- **THEN** it can execute a deterministic mock-backed chat-completion request path without launching the `oxidemux` binary, opening a GPUI window, starting tray/menu lifecycle code, or contacting a real provider

#### Scenario: Core facade preserves structured errors
- **WHEN** minimal proxy engine routing, request validation, or provider execution fails
- **THEN** the returned core result includes structured `CoreError` data that consumers can match without parsing display strings or local HTTP response bodies

### Requirement: Core keeps proxy semantics independent from desktop shell
The `oxmux` core SHALL own the reusable minimal proxy request semantics for model extraction, canonical protocol request construction, routing decision, provider execution request construction, response mode handling, and deterministic local response serialization while `oxidemux` remains a consumer for future UI and lifecycle presentation.

#### Scenario: Desktop shell does not define proxy route semantics
- **WHEN** the minimal proxy engine path is implemented
- **THEN** route semantics, routing choices, provider execution outcomes, fallback reasons, and structured failures are represented in `oxmux` rather than duplicated in `oxidemux`

#### Scenario: Dependency boundary remains intact
- **WHEN** maintainers inspect `crates/oxmux/Cargo.toml` and run core dependency-boundary tests after adding the minimal proxy engine
- **THEN** `oxmux` remains free of GPUI, gpui-component, tray libraries, updater libraries, packaging tools, platform credential storage libraries, provider SDKs, OAuth UI libraries, and the `oxidemux` app crate

### Requirement: Headless core crate boundary
The system SHALL provide an `oxmux` Rust library crate that is usable without GPUI, desktop lifecycle code, tray integration, updater logic, packaging code, app-specific UI dependencies, provider execution dependencies, OAuth UI, or platform credential storage dependencies while owning reusable local proxy runtime behavior.

#### Scenario: Core builds without app dependencies
- **WHEN** workspace checks are run for the `oxmux` crate after adding the local health runtime
- **THEN** `oxmux` builds and tests without depending on GPUI, gpui-component, tray libraries, updater libraries, packaging libraries, platform credential storage libraries, provider SDKs, OAuth UI libraries, or the `oxidemux` app crate

#### Scenario: Core can be used by a Rust consumer
- **WHEN** a Rust test or example depends on `oxmux` directly
- **THEN** it can construct the public core facade and start, query, and shut down the minimal local health runtime without launching the `oxidemux` binary, opening a window, starting IPC, or contacting an external provider

### Requirement: Minimal public facade for future core domains
The `oxmux` crate SHALL expose a small public facade that establishes ownership of proxy lifecycle, local health runtime, local client authorization, provider/auth, provider execution, routing, protocol translation, configuration, streaming, management/status, usage/quota, domain error primitives, and a minimal concrete proxy request smoke path without implementing full provider SDK integration, outbound provider calls, credential storage, full proxy request handling, remote management panels, or real streaming transport adapters in this change.

#### Scenario: Provider auth ownership is visible but not implemented
- **WHEN** maintainers inspect the `oxmux` public API or documentation
- **THEN** provider authentication and token refresh are identified as future core concerns without requiring OAuth UI, platform credential storage, or concrete provider clients in this phase

#### Scenario: Local client authorization ownership is visible
- **WHEN** maintainers inspect the `oxmux` public API or documentation after adding local client authorization boundaries
- **THEN** local proxy client authorization, inference access, management/status/control access, redacted local client credential metadata, and structured unauthorized outcomes are represented as headless core concerns without requiring GPUI, desktop credential storage, OAuth UI, provider SDKs, or app-shell state

#### Scenario: Provider execution ownership exposes deterministic mock boundaries
- **WHEN** maintainers inspect the `oxmux` public API or documentation after adding provider execution primitives
- **THEN** provider execution is represented by trait, request, result, mock harness, and structured outcome primitives that can be used in deterministic tests without requiring real provider SDKs, HTTP clients, OAuth, platform credential storage, GPUI, or app-shell state

#### Scenario: Routing ownership exposes typed policy primitives
- **WHEN** maintainers inspect the `oxmux` public API or documentation after adding routing policy primitives
- **THEN** model aliases, account targeting, priority, fallback, exhausted states, degraded states, selection outcomes, skipped candidate metadata, and routing failure details are represented by typed public primitives and exercised by the minimal smoke route without requiring full proxy routing behavior, provider SDKs, outbound provider calls, GPUI, or app-shell state

#### Scenario: Protocol ownership exposes typed skeleton boundaries
- **WHEN** maintainers inspect the `oxmux` public API or documentation
- **THEN** OpenAI, Gemini, Claude, Codex, and provider-specific protocol translation are represented by typed request/response boundaries, typed protocol metadata, and deferred translation results while the minimal smoke route may construct an OpenAI canonical request without requiring full request translators, response translators, or outbound provider calls in this phase

#### Scenario: Streaming ownership exposes typed response primitives
- **WHEN** maintainers inspect the `oxmux` public API or documentation after adding streaming response primitives
- **THEN** response mode, complete responses, ordered stream events, in-sequence terminal events, stream completion, stream cancellation, stream errors, streaming failure details, and deterministic mock stream outcomes are represented by typed public primitives without requiring network transports, provider stream adapters, provider SDKs, outbound provider calls, GPUI, or app-shell state

#### Scenario: Management ownership includes local health runtime status
- **WHEN** maintainers inspect the `oxmux` public API or documentation
- **THEN** proxy lifecycle state, local health runtime status, provider listing, account health, usage, quota, degraded service status, and protected management/status/control route boundaries are identified as core concerns while full remote management panels remain deferred

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
The `oxmux` management/lifecycle and provider execution facade SHALL remain usable without starting provider network transports, concrete provider clients, OAuth flows, token refresh, hot reload watchers, background proxy routing behavior, GPUI, or app-shell state, while also supporting explicit local health runtime startup, deterministic in-memory mock provider execution, and an explicitly configured minimal local proxy route for smoke testing.

#### Scenario: Tests use deterministic local runtime state
- **WHEN** core tests verify management snapshots, configuration validation, provider/account summaries, lifecycle states, usage/quota summaries, local health runtime behavior, and the minimal proxy smoke route
- **THEN** they use deterministic in-memory values, caller-supplied routing availability, mock provider execution, or loopback-only local listener state and do not require external network services, real credentials, provider accounts, a desktop app, or desktop platform APIs

#### Scenario: Tests use deterministic mock provider state
- **WHEN** core tests verify provider execution traits, mock provider outcomes, provider/account summary reflection, quota-limited outcomes, degraded outcomes, streaming-capable metadata, failed mock outcomes, or selected provider/account propagation through the minimal proxy route
- **THEN** they use deterministic in-memory mock providers and do not require external network services, real credentials, provider SDKs, OAuth flows, token refresh, raw secret storage, a desktop app, or desktop platform APIs

#### Scenario: Protocol ownership remains explicit
- **WHEN** provider capabilities, provider execution requests, or routing defaults reference OpenAI, Gemini, Claude, Codex, or provider-specific protocol families
- **THEN** the facade identifies those protocol families as typed metadata but does not translate full provider request or response surfaces in this change

### Requirement: Core facade exposes file-backed configuration loading
The `oxmux` public facade SHALL expose file-backed configuration loading, validation, replacement hook points, opaque credential-reference validation, and structured configuration error types needed for Rust consumers and tests to exercise deterministic local TOML configuration without importing `oxidemux` or desktop-specific code. The facade SHALL NOT resolve credential references into secrets, authenticate accounts, contact providers, or require platform secret-store dependencies in this change.

#### Scenario: Rust consumer loads configuration without app shell
- **WHEN** a Rust test or library consumer loads a valid local TOML configuration through the `oxmux` facade
- **THEN** it can obtain validated configuration and management-visible state without launching the `oxidemux` binary, opening a GPUI window, starting tray/menu lifecycle code, reading platform secrets, or contacting a real provider

#### Scenario: Core facade preserves structured configuration errors
- **WHEN** file parsing, validation, provider reference validation, routing default validation, or configuration replacement fails
- **THEN** the returned core result includes structured configuration error data that consumers can match without parsing display strings

#### Scenario: Dependency boundary remains intact for file configuration
- **WHEN** maintainers inspect `crates/oxmux/Cargo.toml` and run core dependency-boundary tests after adding file-backed configuration loading
- **THEN** `oxmux` remains free of GPUI, gpui-component, tray libraries, updater libraries, packaging tools, platform credential storage libraries, provider SDKs, OAuth UI libraries, remote configuration clients, database clients, and the `oxidemux` app crate

### Requirement: Core facade exposes layered configuration reload primitives
The `oxmux` public facade SHALL expose the minimal layered configuration primitives needed by Rust, CLI, and app-shell consumers while preserving the headless core crate boundary.

The facade SHALL include types for configuration layer kind, layer source metadata, layered input, configuration fingerprint, validated layered configuration state, and reload outcome. The facade SHALL expose layered load/replacement hooks through the configuration boundary without requiring `oxidemux`, GPUI, IPC, filesystem watcher services, provider SDKs, OAuth UI, platform secret stores, remote storage, or database dependencies.

#### Scenario: Rust consumer reloads layered config without app shell
- **WHEN** Rust code imports the public `oxmux` facade and provides already-read bundled-default and user-owned configuration layer contents
- **THEN** it can request a layered reload and receive unchanged, replaced, or rejected outcome data without launching `oxidemux`

#### Scenario: Dependency boundary remains intact for layered configuration
- **WHEN** `oxmux` is checked or tested after layered configuration primitives are added
- **THEN** the core crate builds without GPUI, app-shell, watcher, provider SDK, OAuth UI, platform secret-store, remote config, or database dependencies

#### Scenario: Existing single-file facade remains available
- **WHEN** Rust code uses existing single-file configuration loading and replacement APIs
- **THEN** those APIs remain available and keep their current whole-document validation semantics

### Requirement: Core reload outcomes are matchable by consumers
The `oxmux` core SHALL represent layered configuration reload results with matchable outcome data instead of display-string parsing or UI callbacks.

Reload outcomes SHALL distinguish at least unchanged, replaced, and rejected candidates. Unchanged outcomes SHALL expose the active effective-runtime fingerprint that matched the candidate. Replaced outcomes SHALL expose the new active effective-runtime fingerprint and management-visible source metadata. Rejected outcomes SHALL expose structured parse, merge, or validation diagnostics, candidate source summaries, the previous active fingerprint when present, and a candidate fingerprint only when one could be computed without pretending the rejected candidate is active. Unchanged outcomes SHALL allow callers to skip proxy restarts, management refresh notifications, and UI reload banners.

#### Scenario: Consumer skips work on unchanged outcome
- **WHEN** a layered reload hook returns an unchanged outcome
- **THEN** a Rust, CLI, or app-shell consumer can determine that no proxy restart, management snapshot refresh, or user notification is required solely from typed outcome data

#### Scenario: Consumer reports rejected outcome
- **WHEN** a layered reload hook returns a rejected outcome
- **THEN** a Rust, CLI, or app-shell consumer can display structured candidate diagnostics while keeping the previous active runtime state visible

