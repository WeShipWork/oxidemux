## Purpose

Define the `oxmux` local proxy health runtime used for loopback-only smoke testing of the reusable core without desktop, provider, routing, or app-shell dependencies.

## Requirements

### Requirement: Runtime dispatches minimal proxy route
The `oxmux` local runtime SHALL preserve the stable health endpoint while also dispatching a bounded loopback `POST /v1/chat/completions` request to the minimal proxy engine path when configured with deterministic core proxy inputs.

#### Scenario: Health endpoint remains stable
- **WHEN** a client sends `GET /health` to a running local runtime after the minimal proxy route is added
- **THEN** the runtime still returns the stable health response defined for local health smoke testing

#### Scenario: Chat-completion route is dispatched on loopback runtime
- **WHEN** a client sends a valid minimal `POST /v1/chat/completions` request to a running loopback runtime configured for mock-backed proxy execution
- **THEN** the runtime dispatches the request to the `oxmux` minimal proxy engine and returns the serialized engine response over the local HTTP connection

#### Scenario: Runtime rejects unsupported local requests deterministically
- **WHEN** a client sends a local request whose method or path is neither the health endpoint nor the supported minimal chat-completion route
- **THEN** the runtime returns a deterministic unsupported-path response without reporting health success or proxy execution success

### Requirement: Runtime request parsing is bounded and local-only
The `oxmux` local runtime SHALL parse only the bounded local HTTP request data needed for the health endpoint and minimal chat-completion smoke route, and SHALL reject malformed or oversized requests with deterministic failures instead of panicking or reading unbounded input.

#### Scenario: Malformed local proxy request is rejected
- **WHEN** a loopback client sends malformed request-line, header, body, or content data for the minimal chat-completion route
- **THEN** the runtime returns a deterministic invalid-request response and keeps the listener usable for later valid requests

#### Scenario: Runtime remains loopback-only
- **WHEN** runtime configuration requests a non-loopback listen address after the minimal proxy route is added
- **THEN** `oxmux` still returns a structured local runtime configuration error instead of binding a public network interface

#### Scenario: Runtime avoids desktop and provider-network dependencies
- **WHEN** maintainers inspect or run local runtime tests for the minimal proxy route
- **THEN** the runtime remains independent from GPUI, tray/background lifecycle, updater, packaging, OAuth UI, token refresh, raw credential storage, provider SDKs, real provider accounts, and outbound provider network calls


### Requirement: Core starts local health runtime
The system SHALL provide an `oxmux` local proxy health runtime that binds a configurable loopback HTTP endpoint from deterministic configuration without launching `oxidemux` or requiring external providers.

#### Scenario: Runtime binds configured loopback endpoint
- **WHEN** Rust code starts the `oxmux` local proxy health runtime with a valid localhost listen address and port
- **THEN** the runtime binds an HTTP listener on that endpoint and reports the actual bound endpoint through the core facade

#### Scenario: Runtime rejects non-local exposure
- **WHEN** runtime configuration requests a non-loopback listen address
- **THEN** `oxmux` returns a structured configuration or lifecycle error instead of binding a public network interface

### Requirement: Runtime exposes stable health endpoint
The system SHALL expose a stable health endpoint suitable for smoke testing the minimal local runtime, and adding the minimal proxy route SHALL NOT change the health response contract or make health checks depend on provider, routing, OAuth, quota, credential, GPUI, or app-shell state.

#### Scenario: Health request succeeds
- **WHEN** a client sends `GET /health` to a running local runtime after the minimal proxy route is added
- **THEN** the runtime returns the same successful HTTP response with stable content indicating the runtime is healthy

#### Scenario: Unknown path does not masquerade as health
- **WHEN** a client requests a path other than the supported health endpoint or minimal chat-completion smoke route
- **THEN** the runtime returns a deterministic non-health response that does not report a healthy smoke-test result or proxy execution success

### Requirement: Runtime reports lifecycle transitions
The system SHALL report local runtime startup, running, failure, shutdown, and stopped status through typed `oxmux` lifecycle facade states.

#### Scenario: Successful startup reaches running
- **WHEN** runtime startup begins and the listener binds successfully
- **THEN** lifecycle status transitions from starting to running and includes the bound endpoint metadata

#### Scenario: Bind failure reaches failed
- **WHEN** runtime startup cannot bind the configured endpoint because the address is invalid or the port is unavailable
- **THEN** lifecycle status becomes failed with a structured error that app and library consumers can inspect without parsing logs

#### Scenario: Shutdown reaches stopped
- **WHEN** a running local health runtime is shut down through the `oxmux` runtime handle
- **THEN** the listener stops accepting requests, releases the endpoint, and reports stopped lifecycle status

### Requirement: Runtime remains provider and desktop independent
The system SHALL keep the local runtime independent from real provider transports, OAuth, quota collection side effects, GPUI, tray/background lifecycle, updater, packaging, and platform credential storage while allowing the configured minimal proxy route to invoke core routing and deterministic provider execution traits supplied by `oxmux` tests or future adapters.

#### Scenario: Health runtime runs without external services
- **WHEN** runtime tests start, query, and shut down the local health runtime
- **THEN** they complete without real provider accounts, network calls to upstream AI providers, OAuth flows, desktop APIs, GPUI windows, platform credential stores, or `oxidemux` process startup

#### Scenario: Core dependency boundary remains intact
- **WHEN** maintainers inspect `crates/oxmux/Cargo.toml` after adding the health runtime
- **THEN** `oxmux` still has no dependency on `oxidemux`, GPUI, gpui-component, tray libraries, updater libraries, packaging tools, provider SDKs, OAuth UI, platform credential storage libraries, or outbound provider HTTP client stacks required by real provider transports
