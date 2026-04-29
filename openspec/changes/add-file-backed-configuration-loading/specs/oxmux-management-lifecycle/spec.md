## ADDED Requirements

### Requirement: Management snapshot reflects file-backed configuration
The `oxmux` management snapshot SHALL reflect the currently active validated file-backed configuration, including configuration source metadata, listen address, port, auto-start intent, logging setting, usage collection setting, routing default names, provider/account reference summaries, warnings, and structured validation errors from failed replacement attempts.

The active configuration portion of the management snapshot SHALL always reflect the last successfully validated configuration. File-loaded provider/account summaries SHALL represent configured declarations and references only; they SHALL NOT imply verified auth health, subscription health, quota availability, provider availability, or credential usability without separate core auth/provider state. Failed replacement details SHALL be exposed separately as last configuration load failure metadata and SHALL NOT overwrite active listen settings, routing defaults, provider/account summaries, logging setting, usage collection setting, or auto-start intent. If there has never been a successful file-backed load, failed-load metadata MAY be visible while active file-backed configuration remains absent. A successful replacement SHALL clear previous failed-load metadata.

#### Scenario: Snapshot updates after valid file configuration load
- **WHEN** a valid local TOML configuration is loaded and applied through `oxmux`
- **THEN** the management snapshot exposes the loaded source metadata and app-visible configuration fields without duplicating validation logic in `oxidemux`

#### Scenario: Snapshot keeps last valid state after failed replacement
- **WHEN** a configuration replacement attempt fails because of invalid listen settings, port, routing defaults, provider references, logging settings, usage collection settings, or auto-start intent after a valid configuration was active
- **THEN** the management snapshot preserves the last valid active configuration and exposes structured validation errors for the failed attempt

#### Scenario: Snapshot represents initial load failure without synthetic health
- **WHEN** the first file-backed configuration load fails before any active file-backed configuration exists
- **THEN** the management snapshot can expose failed-load metadata without synthesizing active listen settings, provider/account health, quota health, or routing defaults from unrelated defaults

#### Scenario: Configured accounts remain auth-unverified
- **WHEN** management snapshot exposes provider/account summaries derived from file-backed configuration
- **THEN** those summaries identify configured declarations without marking auth state, subscription health, quota pressure, provider availability, or credential usability as verified

#### Scenario: Successful replacement clears failed replacement details
- **WHEN** a valid configuration replacement succeeds after a previous failed replacement attempt
- **THEN** the management snapshot reflects the new active configuration and no longer exposes the previous failed replacement as current load-failure metadata

#### Scenario: Auto-start remains intent only
- **WHEN** management snapshot exposes auto-start intent loaded from file-backed configuration
- **THEN** it represents the user's desired lifecycle setting as typed core state without registering OS launch services, mutating login items, starting tray code, or persisting platform lifecycle settings
