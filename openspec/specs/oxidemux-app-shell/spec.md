## Purpose

Define the `oxidemux` app shell boundary as the desktop and integration consumer
of the reusable `oxmux` core crate.

## Requirements

### Requirement: App shell consumes core crate
The system SHALL provide an `oxidemux` app shell crate that depends on `oxmux` for reusable core behavior instead of owning proxy, provider, routing, protocol, configuration, management, usage, or error primitives itself.

#### Scenario: Dependency direction is app to core
- **WHEN** maintainers inspect workspace package manifests
- **THEN** `oxidemux` depends on `oxmux` and `oxmux` does not depend on `oxidemux`

#### Scenario: App preserves bootstrap behavior
- **WHEN** the `oxidemux` binary is run after the workspace split
- **THEN** it preserves the existing bootstrap behavior or equivalent package metadata output while routing reusable data through `oxmux` where appropriate

### Requirement: App shell owns desktop and integration concerns
The `oxidemux` crate SHALL be the owner of future GPUI UI, settings UX, dashboard surfaces, tray/background lifecycle, updater, packaging, platform credential storage integrations, IDE/coding-environment adapters, and local IPC or server surfaces where useful.

#### Scenario: GPUI remains app-owned
- **WHEN** future GPUI or gpui-component dependencies are evaluated
- **THEN** they are added only to the `oxidemux` app shell or a future app-owned crate, not to `oxmux`

#### Scenario: Desktop proxy control remains app-owned
- **WHEN** future proxy start/stop control, tray operation, or background lifecycle work is planned
- **THEN** the app shell owns the desktop control surface while delegating reusable proxy lifecycle primitives to `oxmux`

#### Scenario: Credential storage boundary remains explicit
- **WHEN** future secure credential storage work is planned
- **THEN** platform-specific storage implementations belong to `oxidemux` or app/platform adapters while reusable credential abstractions belong to `oxmux`

### Requirement: App shell separates UI-visible state from core behavior
The `oxidemux` crate SHALL adapt core state into user-visible UI or integration state without duplicating core routing, provider, quota, or protocol logic.

#### Scenario: Quota dashboard planning keeps logic in core
- **WHEN** future quota or usage dashboard work is planned
- **THEN** `oxidemux` owns presentation and interaction while `oxmux` owns reusable usage/quota primitives and status data

#### Scenario: Degraded service status is surfaced by app
- **WHEN** future provider or account degraded states are exposed by `oxmux`
- **THEN** `oxidemux` presents those states to users or integrations without reimplementing the degraded-state decision logic

### Requirement: Workspace verification includes app shell
The project verification commands SHALL include the `oxidemux` app shell crate in formatting, linting, checking, and testing.

#### Scenario: App shell is checked by default verification
- **WHEN** maintainers run the repository's documented cargo, mise, or CI verification commands
- **THEN** the `oxidemux` crate is included in fmt, clippy, check, and test coverage

### Requirement: App shell consumes management lifecycle facade
The `oxidemux` app shell SHALL consume `oxmux` management/lifecycle facade data for app-visible status instead of defining duplicate proxy lifecycle, configuration, provider/account, usage/quota, or error primitives.

#### Scenario: App shell reads core status
- **WHEN** the `oxidemux` binary or app-shell integration code needs current core status
- **THEN** it obtains that status through `oxmux` management/lifecycle facade types rather than app-owned duplicate structs

#### Scenario: App shell preserves bootstrap behavior through core status
- **WHEN** the `oxidemux` binary is run during this change
- **THEN** it preserves the existing metadata output or equivalent bootstrap behavior while proving it can read app-visible core status from `oxmux`

### Requirement: App shell keeps desktop concerns outside core
The `oxidemux` app shell SHALL remain responsible for future GPUI views, tray/background lifecycle, updater, packaging, settings UX, and platform credential storage while delegating reusable status/control primitives to `oxmux`.

#### Scenario: Future dashboard consumes core summaries
- **WHEN** future GPUI dashboard, provider, quota, settings, or logs work is planned
- **THEN** the app shell uses `oxmux` management, provider/account, configuration, usage/quota, and error summaries as its data contract instead of inventing app-only state models

#### Scenario: Platform storage remains app-owned
- **WHEN** future OAuth tokens, API keys, or desktop-protected secrets are needed
- **THEN** `oxidemux` or app/platform adapters own storage implementation while `oxmux` receives only credential state, references, or abstractions that remain UI-free
