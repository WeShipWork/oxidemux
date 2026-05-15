## ADDED Requirements

### Requirement: Core facade exposes reasoning controls

The `oxmux` public facade SHALL expose provider-neutral reasoning and thinking control primitives, validation outcomes, compatibility outcomes, and structured errors needed by Rust consumers and tests without importing `oxidemux` or desktop-specific code.

#### Scenario: Rust consumer imports reasoning primitives

- **WHEN** Rust code imports the public `oxmux` facade after reasoning controls are added
- **THEN** it can construct and inspect reasoning intent, reasoning source, reasoning effort or budget, compatibility outcome, and unsupported-capability data without importing app-shell, provider SDK, OAuth, or platform credential storage types

#### Scenario: Rust consumer supplies explicit reasoning metadata

- **WHEN** Rust code imports the public `oxmux` facade after reasoning controls are added
- **THEN** it can supply explicit typed reasoning metadata through core primitives without requiring HTTP route parsing, provider-shaped JSON parsing, app-shell state, or provider SDK request types

#### Scenario: Core dependency boundary remains intact for reasoning controls

- **WHEN** maintainers inspect `crates/oxmux/Cargo.toml` and run core dependency-boundary tests after adding reasoning controls
- **THEN** `oxmux` remains free of GPUI, gpui-component, tray libraries, updater libraries, packaging tools, platform credential storage libraries, provider SDKs, OAuth UI libraries, provider-specific beta header clients, outbound network clients, and the `oxidemux` app crate

### Requirement: Core errors include reasoning failures

The `oxmux` core SHALL represent invalid reasoning controls and unsupported reasoning capability outcomes as structured `CoreError` data that callers can match without parsing display text.

#### Scenario: Invalid reasoning input is matchable

- **WHEN** reasoning intent validation fails because a budget, effort, mode, source, or alias convention is invalid
- **THEN** the returned `CoreError` includes structured reasoning failure data with the affected field and stable failure code

#### Scenario: Unsupported reasoning capability is matchable

- **WHEN** a selected provider/account/model target cannot honor strict explicit reasoning intent
- **THEN** the returned `CoreError` includes structured unsupported-capability data that identifies the requested reasoning behavior and the target capability gap
