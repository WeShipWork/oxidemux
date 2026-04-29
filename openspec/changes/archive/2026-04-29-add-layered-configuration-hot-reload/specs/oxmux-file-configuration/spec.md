## ADDED Requirements

### Requirement: Layered configuration produces a validated runtime view
The `oxmux` core SHALL accept deterministic layered configuration inputs for bundled defaults and user-owned overrides, merge them into one runtime candidate, and validate the merged candidate before publishing active configuration state.

Bundled defaults SHALL have lower precedence than user-owned configuration. User-owned layers MAY omit fields supplied by bundled defaults, but the merged runtime candidate SHALL satisfy the same required-field, semantic validation, structured error, and credential redaction rules as file-backed configuration before publication. Existing single-file loading and replacement APIs SHALL remain available for consumers that do not opt into layered configuration.

#### Scenario: Bundled defaults fill missing user fields
- **WHEN** `oxmux` receives bundled defaults with required proxy settings and a user-owned layer that overrides only observability settings
- **THEN** the merged runtime configuration uses the bundled proxy settings, applies the user-owned observability settings, validates successfully, and publishes one active runtime view

#### Scenario: User layer overrides bundled scalar values
- **WHEN** bundled defaults and user-owned configuration both define a scalar setting such as proxy port, logging level, usage collection, or auto-start intent
- **THEN** the user-owned value wins in the merged runtime configuration before validation

#### Scenario: Invalid merged candidate is rejected
- **WHEN** the layered inputs merge into a candidate with an invalid listen address, port, routing default, provider reference, credential reference, logging setting, usage collection setting, or auto-start intent
- **THEN** `oxmux` returns structured configuration errors, preserves the last valid active configuration if present, and does not publish the invalid merged candidate

### Requirement: Layered merge preserves user-owned provider declarations
The `oxmux` core SHALL merge provider and account declarations using deterministic identity-based rules that prevent bundled defaults from overwriting user-owned declarations wholesale.

Provider declarations SHALL merge by provider identifier. Account declarations SHALL merge by provider identifier and account identifier. User-owned provider and account fields SHALL take precedence over bundled defaults for the same identity, while default-only providers and accounts SHALL remain available in the merged runtime view. Credential references SHALL remain opaque and SHALL NOT be echoed in management snapshots or structured errors. Arbitrary custom TOML settings SHALL remain unsupported in this change unless a later OpenSpec change defines a typed custom schema.

#### Scenario: User provider survives bundled defaults update
- **WHEN** bundled defaults define provider `default-openai` and user-owned configuration defines provider `local-lab`
- **THEN** the merged runtime configuration includes both providers without replacing the user-owned `local-lab` declaration

#### Scenario: User account overrides matching default account
- **WHEN** bundled defaults and user-owned configuration define the same provider and account identifiers
- **THEN** user-owned account fields take precedence for that identity while default-only provider/account declarations remain available

#### Scenario: Unknown custom TOML remains rejected
- **WHEN** a user-owned layer contains an unknown custom table or field that is not part of the accepted configuration schema
- **THEN** validation returns a structured unknown-field error rather than preserving arbitrary custom TOML

#### Scenario: Secret material remains outside merged configuration output
- **WHEN** a user-owned layer contains a raw token, API key, OAuth refresh token, cookie, or platform secret-store payload where only an opaque credential reference is allowed
- **THEN** validation fails with a structured credential-reference error without publishing or echoing the secret-like value

### Requirement: Configuration fingerprinting avoids spurious reloads
The `oxmux` core SHALL compute deterministic fingerprints for layered configuration candidates and SHALL use them to avoid publishing or notifying callers about unchanged effective configuration.

The reload decision fingerprint SHALL be derived from the normalized merged runtime configuration after successful parsing, merging, and validation. It SHALL NOT depend on filesystem modification time, watcher event count, process-local pointer identity, platform-specific file metadata, TOML comments, TOML whitespace, or TOML table ordering. When a candidate effective-runtime fingerprint matches the active fingerprint, the reload hook SHALL return an unchanged outcome and SHALL NOT replace active configuration or clear unrelated failure metadata unless explicitly required by the outcome contract.

#### Scenario: Equivalent effective configuration produces unchanged outcome
- **WHEN** a caller submits layered configuration contents that parse, merge, and validate to the same normalized runtime configuration as the currently active configuration
- **THEN** `oxmux` returns an unchanged reload outcome and leaves active configuration state unchanged

#### Scenario: Syntactic-only TOML changes do not force reload
- **WHEN** a caller submits TOML with changed comments, whitespace, or table ordering but the same validated merged runtime configuration
- **THEN** `oxmux` returns an unchanged reload outcome

#### Scenario: Changed user layer produces replacement outcome
- **WHEN** a caller submits a user-owned layer whose merged runtime configuration changes the effective-runtime fingerprint and the merged candidate validates successfully
- **THEN** `oxmux` publishes the new active configuration and returns a replaced reload outcome with the new active fingerprint

#### Scenario: Watcher noise does not force reload
- **WHEN** an external watcher reports a file event but the already-read layer contents produce the same effective-runtime fingerprint as active configuration
- **THEN** `oxmux` treats the candidate as unchanged rather than publishing a spurious replacement

### Requirement: User-owned routing defaults replace bundled route list when present
The `oxmux` core SHALL treat `routing.defaults` as one ordered runtime list for layered merge purposes.

If a user-owned layer declares any routing defaults, that user-owned ordered route list SHALL replace bundled default routing defaults before validation. If the user-owned layer omits routing defaults, bundled default routing defaults SHALL remain in effect. The merged route list SHALL still validate provider/account references, fallback settings, candidate grouping, duplicate candidates, and route order according to existing file-backed configuration rules.

#### Scenario: User-owned route list replaces bundled routes
- **WHEN** bundled defaults define routing defaults and user-owned configuration defines at least one routing default
- **THEN** the merged runtime configuration uses the user-owned ordered routing defaults instead of appending bundled routes

#### Scenario: Bundled routes remain when user omits routes
- **WHEN** bundled defaults define routing defaults and user-owned configuration omits `routing.defaults`
- **THEN** the merged runtime configuration keeps the bundled routing defaults

### Requirement: Reload hooks expose deterministic outcomes without owning watchers
The `oxmux` core SHALL expose explicit layered reload hook points that accept already-read layer contents or caller-selected paths, validate before publish, and return deterministic outcomes for unchanged, replaced, and rejected candidates.

The core SHALL NOT start filesystem watchers, debounce tasks, background reload workers, GPUI notifications, platform path discovery, cloud sync, remote registry updates, database config services, provider SDK refreshes, OAuth credential persistence, or platform secret-store operations as part of layered configuration reload. Those behaviors belong to app-shell or caller-owned adapters that invoke the headless reload hooks.

#### Scenario: Successful layered replacement reports replacement
- **WHEN** a valid layered candidate differs from the active fingerprint
- **THEN** `oxmux` publishes the merged runtime configuration and returns a replaced reload outcome suitable for caller-owned notification or management refresh

#### Scenario: Rejected layered replacement preserves active state
- **WHEN** a layered candidate fails parsing, merging, or validation after a previous valid configuration exists
- **THEN** `oxmux` returns a rejected reload outcome with failed candidate diagnostics and the current active fingerprint while keeping the previous active configuration available

#### Scenario: Initial rejected layered load leaves active state absent
- **WHEN** the first layered candidate fails before any active configuration has been published
- **THEN** `oxmux` exposes failed-load metadata and does not synthesize active configuration from unrelated defaults

#### Scenario: Core does not own filesystem watching
- **WHEN** maintainers inspect or test layered configuration reload behavior
- **THEN** `oxmux` exposes deterministic hook points and outcomes but no filesystem watcher, debounce timer, background task, GPUI UI, or platform credential integration
