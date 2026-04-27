## ADDED Requirements

### Requirement: Core starts local health runtime
The system SHALL provide an `oxmux` local proxy health runtime that binds a configurable loopback HTTP endpoint from deterministic configuration without launching `oxidemux` or requiring external providers.

#### Scenario: Runtime binds configured loopback endpoint
- **WHEN** Rust code starts the `oxmux` local proxy health runtime with a valid localhost listen address and port
- **THEN** the runtime binds an HTTP listener on that endpoint and reports the actual bound endpoint through the core facade

#### Scenario: Runtime rejects non-local exposure
- **WHEN** runtime configuration requests a non-loopback listen address
- **THEN** `oxmux` returns a structured configuration or lifecycle error instead of binding a public network interface

### Requirement: Runtime exposes stable health endpoint
The system SHALL expose a stable health endpoint suitable for smoke testing the minimal local runtime without provider, routing, OAuth, quota, credential, GPUI, or app-shell dependencies.

#### Scenario: Health request succeeds
- **WHEN** a client sends `GET /health` to a running local health runtime
- **THEN** the runtime returns a successful HTTP response with stable content indicating the runtime is healthy

#### Scenario: Unknown path does not masquerade as health
- **WHEN** a client requests a path other than the supported health endpoint
- **THEN** the runtime returns a deterministic non-health response that does not report a healthy smoke-test result

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
The system SHALL keep the local health runtime independent from provider execution, OAuth, routing, protocol translation, quota collection, GPUI, tray/background lifecycle, updater, packaging, and platform credential storage.

#### Scenario: Health runtime runs without external services
- **WHEN** runtime tests start, query, and shut down the local health runtime
- **THEN** they complete without real provider accounts, network calls to upstream AI providers, OAuth flows, desktop APIs, GPUI windows, platform credential stores, or `oxidemux` process startup

#### Scenario: Core dependency boundary remains intact
- **WHEN** maintainers inspect `crates/oxmux/Cargo.toml` after adding the health runtime
- **THEN** `oxmux` still has no dependency on `oxidemux`, GPUI, gpui-component, tray libraries, updater libraries, packaging tools, provider SDKs, OAuth UI, or platform credential storage libraries
