## MODIFIED Requirements

### Requirement: App shell consumes management lifecycle facade
The `oxidemux` app shell SHALL consume `oxmux` management/lifecycle facade data for app-visible status and the minimal local health runtime instead of defining duplicate proxy lifecycle, configuration, provider/account, usage/quota, runtime, or error primitives.

#### Scenario: App shell reads core status
- **WHEN** the `oxidemux` binary or app-shell integration code needs current core status
- **THEN** it obtains that status through `oxmux` management/lifecycle facade types rather than app-owned duplicate structs

#### Scenario: App shell preserves bootstrap behavior through core status
- **WHEN** the `oxidemux` binary is run during this change
- **THEN** it preserves the existing metadata output or equivalent bootstrap behavior while proving it can read app-visible core status from `oxmux`

#### Scenario: App shell can smoke-check health runtime through core
- **WHEN** app-shell tests or integration code exercise the minimal local health runtime
- **THEN** they start, query, and stop it through `oxmux` APIs without moving listener ownership, lifecycle logic, or health response definitions into `oxidemux`

### Requirement: App shell keeps desktop concerns outside core
The `oxidemux` app shell SHALL remain responsible for future GPUI views, tray/background lifecycle, updater, packaging, settings UX, platform credential storage, and desktop-managed lifecycle policy while delegating reusable status, control, and minimal local health runtime behavior to `oxmux`.

#### Scenario: Future dashboard consumes core summaries
- **WHEN** future GPUI dashboard, provider, quota, settings, logs, or health-runtime status work is planned
- **THEN** the app shell uses `oxmux` management, provider/account, configuration, usage/quota, lifecycle, runtime, and error summaries as its data contract instead of inventing app-only state models

#### Scenario: Platform storage remains app-owned
- **WHEN** future OAuth tokens, API keys, or desktop-protected secrets are needed
- **THEN** `oxidemux` or app/platform adapters own storage implementation while `oxmux` receives only credential state, references, or abstractions that remain UI-free

#### Scenario: Desktop lifecycle remains app-owned
- **WHEN** future tray, background start-on-login, menu-bar, updater, packaging, or GPUI status controls are added
- **THEN** those concerns are implemented in `oxidemux` or app-owned adapters while `oxmux` continues to expose headless runtime handles and lifecycle state
