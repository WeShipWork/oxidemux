## ADDED Requirements

### Requirement: Core facade includes management lifecycle primitives
The `oxmux` public facade SHALL expose the minimal management/lifecycle primitives needed by app and Rust consumers while preserving the headless core crate boundary.

#### Scenario: Facade exports management types
- **WHEN** Rust code imports the public `oxmux` facade
- **THEN** it can access the management snapshot, lifecycle state, lifecycle control intent, configuration snapshot, provider/account summary, usage/quota summary, and related error types without importing `oxidemux`

#### Scenario: Core remains dependency-light
- **WHEN** maintainers inspect `crates/oxmux/Cargo.toml` after adding management lifecycle primitives
- **THEN** the crate still does not depend on GPUI, gpui-component, tray libraries, updater libraries, packaging libraries, platform credential storage libraries, or the `oxidemux` app crate

### Requirement: Core facade remains runtime-inert for this change
The `oxmux` management/lifecycle facade SHALL be usable without starting network transports, protocol translators, provider clients, OAuth flows, token refresh, hot reload watchers, or background proxy runtime behavior.

#### Scenario: Tests use deterministic in-memory state
- **WHEN** core tests verify management snapshots, configuration validation, provider/account summaries, lifecycle states, and usage/quota summaries
- **THEN** they use deterministic in-memory values and do not require external network services, real credentials, a local proxy port, or desktop platform APIs

#### Scenario: Protocol ownership remains explicit
- **WHEN** provider capabilities or routing defaults reference OpenAI, Gemini, Claude, Codex, or provider-specific protocol families
- **THEN** the facade identifies those protocol families as typed metadata but does not translate requests or responses in this change
