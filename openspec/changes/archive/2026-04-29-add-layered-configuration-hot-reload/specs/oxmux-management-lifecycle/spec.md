## ADDED Requirements

### Requirement: Management snapshot reflects layered configuration metadata
The `oxmux` management snapshot SHALL expose management-visible layered configuration metadata for successfully published layered configuration without leaking raw credentials or implying provider/account health.

Management-visible layered metadata SHALL include the active configuration fingerprint, deterministic layer source summaries, the merged runtime configuration fields already exposed by file-backed configuration, and the most recent reload outcome when available. Provider and account summaries derived from layered configuration SHALL remain declaration/reference state only; auth health, quota pressure, subscription health, provider availability, and credential usability SHALL remain unknown or unverified unless supplied by separate core state.

#### Scenario: Snapshot includes active layered fingerprint
- **WHEN** a layered configuration candidate validates and becomes active
- **THEN** the management snapshot exposes the active fingerprint and layer source summaries alongside the merged runtime configuration fields

#### Scenario: Snapshot keeps provider accounts auth-unverified
- **WHEN** layered configuration declares providers, accounts, and opaque credential references
- **THEN** management-visible provider/account summaries identify configured declarations without marking auth health, quota state, subscription health, provider availability, or credential usability as healthy or verified

#### Scenario: Snapshot omits raw credential references
- **WHEN** management-visible layered metadata is built from user-owned configuration containing opaque credential references
- **THEN** the snapshot indicates credential-reference presence or validity without echoing raw credential-reference values or secret-like material

### Requirement: Management snapshot preserves reload diagnostics separately from active state
The `oxmux` management snapshot SHALL expose failed layered reload diagnostics separately from the last valid active configuration.

Rejected layered candidates SHALL record candidate source metadata, candidate fingerprint when available, previous active fingerprint when present, and structured parse, merge, or validation errors. Rejected candidates SHALL NOT replace active configuration fields, provider summaries, usage/quota summaries, or active fingerprint. A later successful replacement SHALL clear prior failed layered reload diagnostics unless the reload outcome contract explicitly keeps historical diagnostics elsewhere.

#### Scenario: Failed layered reload preserves last valid snapshot
- **WHEN** a layered reload candidate fails after a previous valid layered configuration was active
- **THEN** the management snapshot continues to expose the previous active configuration and active fingerprint while separately exposing the rejected candidate diagnostics

#### Scenario: Initial layered reload failure has no synthetic active config
- **WHEN** the initial layered reload candidate fails before any valid configuration has been published
- **THEN** the management snapshot exposes failed-load metadata without synthesizing active layered configuration from unrelated defaults

#### Scenario: Successful layered reload clears previous failure
- **WHEN** a rejected layered reload is followed by a valid changed layered reload
- **THEN** the management snapshot exposes the new active configuration and fingerprint and clears the previous failed layered reload diagnostics
