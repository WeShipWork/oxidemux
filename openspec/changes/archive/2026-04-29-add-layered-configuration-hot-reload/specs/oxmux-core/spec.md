## ADDED Requirements

### Requirement: Core facade exposes layered configuration reload primitives
The `oxmux` public facade SHALL expose the minimal layered configuration primitives needed by Rust, CLI, and app-shell consumers while preserving the headless core crate boundary.

The facade SHALL include types for configuration layer kind, layer source metadata, layered input, configuration fingerprint, validated layered configuration state, and reload outcome. The facade SHALL expose layered load/replacement hooks through the configuration boundary without requiring `oxidemux`, GPUI, IPC, filesystem watcher services, provider SDKs, OAuth UI, platform secret stores, remote storage, or database dependencies.

#### Scenario: Rust consumer reloads layered config without app shell
- **WHEN** Rust code imports the public `oxmux` facade and provides already-read bundled-default and user-owned configuration layer contents
- **THEN** it can request a layered reload and receive unchanged, replaced, or rejected outcome data without launching `oxidemux`

#### Scenario: Dependency boundary remains intact for layered configuration
- **WHEN** `oxmux` is checked or tested after layered configuration primitives are added
- **THEN** the core crate builds without GPUI, app-shell, watcher, provider SDK, OAuth UI, platform secret-store, remote config, or database dependencies

#### Scenario: Existing single-file facade remains available
- **WHEN** Rust code uses existing single-file configuration loading and replacement APIs
- **THEN** those APIs remain available and keep their current whole-document validation semantics

### Requirement: Core reload outcomes are matchable by consumers
The `oxmux` core SHALL represent layered configuration reload results with matchable outcome data instead of display-string parsing or UI callbacks.

Reload outcomes SHALL distinguish at least unchanged, replaced, and rejected candidates. Unchanged outcomes SHALL expose the active effective-runtime fingerprint that matched the candidate. Replaced outcomes SHALL expose the new active effective-runtime fingerprint and management-visible source metadata. Rejected outcomes SHALL expose structured parse, merge, or validation diagnostics, candidate source summaries, the previous active fingerprint when present, and a candidate fingerprint only when one could be computed without pretending the rejected candidate is active. Unchanged outcomes SHALL allow callers to skip proxy restarts, management refresh notifications, and UI reload banners.

#### Scenario: Consumer skips work on unchanged outcome
- **WHEN** a layered reload hook returns an unchanged outcome
- **THEN** a Rust, CLI, or app-shell consumer can determine that no proxy restart, management snapshot refresh, or user notification is required solely from typed outcome data

#### Scenario: Consumer reports rejected outcome
- **WHEN** a layered reload hook returns a rejected outcome
- **THEN** a Rust, CLI, or app-shell consumer can display structured candidate diagnostics while keeping the previous active runtime state visible
