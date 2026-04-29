## ADDED Requirements

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
