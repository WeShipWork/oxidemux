## ADDED Requirements

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
