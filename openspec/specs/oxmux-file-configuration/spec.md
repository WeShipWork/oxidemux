## Purpose

Define the `oxmux` headless core contract for deterministic file-backed local
configuration loading, semantic validation, runtime conversion, and
management-visible configuration state without desktop app, provider SDK,
credential storage, remote configuration, database, or watcher dependencies.

## Requirements

### Requirement: Deterministic local TOML configuration loading
The `oxmux` core SHALL load a single deterministic local TOML configuration file into typed configuration data for app-visible proxy settings, provider references, routing defaults, logging settings, usage collection settings, and auto-start intent without depending on `oxidemux`, GPUI, provider SDKs, OAuth UI, platform credential storage, remote configuration services, databases, or filesystem watchers.

The initial file format SHALL be TOML loaded from `.toml` paths or already-read TOML contents and SHALL reject unknown fields at every level. The top-level document SHALL contain `version = 1` and `[proxy]`; MAY contain `[observability]`, `[lifecycle]`, zero or more `[[providers]]` entries, zero or more nested `[[providers.accounts]]` entries, and zero or more `[[routing.defaults]]` entries; and SHALL NOT require or accept a separate `format` field. This change SHALL NOT define default path discovery; callers choose the path or provide already-read TOML contents.

Required fields SHALL be `version`, `proxy.listen-address`, and `proxy.port`. Optional supported fields SHALL receive these defaults before semantic validation: `observability.logging = "standard"`, `observability.usage-collection = true`, and `lifecycle.auto-start = "disabled"`. Provider entries require non-empty `id`, accepted `protocol-family`, and `routing-eligible`. Account entries require non-empty `id` and non-empty `credential-reference`. Routing defaults require non-empty `name`, non-empty `model`, `provider-id`, and `fallback-enabled`; `account-id` is optional when provider-only selection is supported by typed routing primitives.

The canonical minimal valid TOML shape is:

```toml
version = 1

[proxy]
listen-address = "127.0.0.1"
port = 8787

[[providers]]
id = "mock-openai"
protocol-family = "openai"
routing-eligible = true

[[providers.accounts]]
id = "default"
credential-reference = "mock-openai/default"

[[routing.defaults]]
name = "chat"
model = "gpt-4o-mini"
provider-id = "mock-openai"
account-id = "default"
fallback-enabled = true

[observability]
logging = "standard"
usage-collection = true

[lifecycle]
auto-start = "disabled"
```

Accepted `protocol-family` values SHALL match protocol-family metadata already represented by `oxmux` protocol/routing/provider primitives. Accepted `logging` values SHALL be `off`, `standard`, or `verbose`. Accepted `auto-start` values SHALL be `disabled` or `enabled`. `usage-collection` SHALL be a boolean. `proxy.port` SHALL be a non-zero TCP port within the accepted TCP port range; `port = 0` SHALL be rejected for file-backed configuration so user-owned files do not publish an unpredictable ephemeral bind port. `proxy.listen-address` SHALL be a loopback IP literal for this change, such as `127.0.0.1` or `::1`; wildcard, unspecified, multicast, link-local, and public interface addresses SHALL be rejected.

`credential-reference` SHALL be an opaque non-secret pointer that lets later credential adapters find account material outside this file-backed core contract. The TOML schema and examples SHALL NOT contain raw tokens, API keys, OAuth refresh tokens, cookies, or platform secret-store payloads. Management snapshots and structured errors SHALL expose whether a credential reference is present and valid, but SHALL NOT echo credential-reference values or secret-like account material.

#### Scenario: Valid TOML file loads into typed configuration
- **WHEN** a Rust consumer asks `oxmux` to load a syntactically valid local TOML configuration file containing listen settings, provider entries, routing defaults, logging settings, usage collection settings, and auto-start intent
- **THEN** `oxmux` returns typed configuration data that can be consumed by core runtime, routing, and management snapshot construction without requiring the desktop app or external services

#### Scenario: Unsupported format is rejected
- **WHEN** a Rust consumer asks `oxmux` to load a configuration file path that is not a `.toml` path for this change
- **THEN** `oxmux` returns a structured configuration error identifying the unsupported format without attempting best-effort parsing

#### Scenario: Unknown field is rejected
- **WHEN** a TOML configuration contains an unknown table or field at any supported level
- **THEN** `oxmux` returns a structured configuration error identifying the unknown field path without silently ignoring the value

#### Scenario: Missing file is structured
- **WHEN** a Rust consumer asks `oxmux` to load a local configuration file path that does not exist or cannot be read
- **THEN** `oxmux` returns a structured configuration error identifying the source path and read failure category without panicking or silently falling back to unrelated defaults

### Requirement: Configuration validation returns structured field errors
The `oxmux` core SHALL validate loaded configuration before producing runtime configuration and SHALL return structured `CoreError` data with stable configuration reason codes, field paths, and invalid value categories for invalid listen address, port, provider references, routing defaults, logging settings, usage collection settings, and auto-start intent.

Configuration errors SHALL expose a matchable kind selected from `ReadFailed`, `ParseFailed`, `UnsupportedFormat`, `MissingRequiredField`, `UnknownField`, `InvalidVersion`, `InvalidListenAddress`, `InvalidPort`, `DuplicateProviderId`, `DuplicateAccountId`, `InvalidProviderProtocolFamily`, `InvalidCredentialReference`, `UnknownProviderReference`, `UnknownAccountReference`, `InvalidRoutingDefault`, `InvalidLoggingSetting`, `InvalidUsageCollectionSetting`, and `InvalidAutoStartIntent`. Field paths SHALL use dotted kebab-case paths with zero-based array indexes such as `proxy.listen-address`, `providers[0].id`, `providers[0].protocol-family`, `providers[0].accounts[0].id`, `providers[0].accounts[0].credential-reference`, `routing.defaults[0].provider-id`, `observability.logging`, and `lifecycle.auto-start`. Invalid value categories SHALL distinguish missing, malformed, unsupported, duplicate, unknown-reference, secret-like, and out-of-range values. Validation SHALL return at least one structured error for an invalid document and MAY aggregate multiple field errors; aggregated errors SHALL be ordered deterministically by document order and then field path.

#### Scenario: Invalid listen address is rejected
- **WHEN** loaded configuration contains a listen address that cannot be represented as a loopback local bind address for the supported runtime scope
- **THEN** validation fails with a structured configuration error whose field path identifies the listen address field

#### Scenario: Public bind address is rejected
- **WHEN** loaded configuration contains a wildcard, unspecified, multicast, link-local, or public listen address
- **THEN** validation fails with `InvalidListenAddress` before any runtime can expose the local subscription proxy beyond loopback

#### Scenario: Invalid port is rejected
- **WHEN** loaded configuration contains a port value outside the accepted TCP port range or otherwise unusable by the local runtime configuration model
- **THEN** validation fails with a structured configuration error whose field path identifies the port field

#### Scenario: Unknown provider reference is rejected
- **WHEN** a routing default, provider account reference, or app-visible setting references a provider identifier that is not declared in the same loaded configuration
- **THEN** validation fails with a structured configuration error identifying the unknown provider reference and the field that referenced it

#### Scenario: Invalid provider protocol family is rejected
- **WHEN** loaded configuration contains a provider protocol family that is not accepted by existing typed `oxmux` protocol/routing/provider primitives
- **THEN** validation fails with `InvalidProviderProtocolFamily` and a field path identifying the invalid provider protocol-family field

#### Scenario: Invalid credential reference is rejected without leaking secrets
- **WHEN** loaded configuration contains a missing, empty, malformed, or secret-like credential reference for a provider account
- **THEN** validation fails with `InvalidCredentialReference` and a field path identifying the credential-reference field without echoing the invalid credential-reference value

#### Scenario: Invalid routing default is rejected
- **WHEN** loaded configuration names a routing default that cannot be represented by typed routing policy primitives or references no valid provider/account candidate
- **THEN** validation fails with a structured configuration error identifying the invalid routing default rather than deferring the failure to request routing

#### Scenario: Invalid observability settings are rejected
- **WHEN** loaded configuration contains unsupported logging or usage collection values
- **THEN** loading fails with structured configuration errors identifying the invalid observability fields without reconfiguring runtime logging or analytics side effects

#### Scenario: Invalid auto-start intent is rejected
- **WHEN** loaded configuration contains an unsupported auto-start intent value
- **THEN** validation fails with a structured configuration error while leaving platform launch-at-login persistence out of `oxmux`

### Requirement: Validated configuration updates management-visible state
The `oxmux` core SHALL use successfully validated file configuration to update app-visible management snapshot fields for configuration source metadata, listen settings, routing default names, provider/account summaries derived from references, logging setting, usage collection setting, auto-start intent, and configuration warnings without requiring GPUI, IPC, provider network calls, real credentials, or desktop lifecycle code.

Active management configuration fields SHALL always reflect the last successfully validated configuration. A file-loaded provider or account summary SHALL represent declaration/reference state only; auth health, quota pressure, subscription health, provider availability, and credential usability SHALL remain unknown or unverified unless supplied by separate core auth/provider state. Failed replacement errors SHALL be exposed separately as last configuration load failure metadata, including candidate source metadata when available, and SHALL be cleared by the next successful replacement. Configuration warnings MAY be empty for this change unless implementation defines a concrete warning case covered by tests.

#### Scenario: Management snapshot reflects valid file configuration
- **WHEN** a valid local TOML configuration is loaded and applied to the core configuration state
- **THEN** the management snapshot exposes the configuration source, listen settings, routing defaults, provider/account reference summaries, logging setting, usage collection setting, and auto-start intent through typed fields

#### Scenario: File-loaded accounts remain auth-unverified
- **WHEN** a valid local TOML configuration declares providers, accounts, and credential references
- **THEN** management-visible provider/account summaries identify configured declarations without marking auth health, quota state, subscription health, provider availability, or credential usability as healthy or verified

#### Scenario: Validation failure is visible without replacing active configuration
- **WHEN** an attempted configuration replacement fails validation
- **THEN** the management snapshot can expose structured validation errors for the failed attempt while preserving the last successfully validated active configuration

### Requirement: Configuration replacement hook validates before publish
The `oxmux` core SHALL expose deterministic configuration replacement hook points that validate candidate file contents or a candidate local path before publishing new active configuration, and SHALL preserve the previous valid configuration when validation fails.

Replacement hook points SHALL perform whole-document replacement only. They SHALL NOT merge layers, compute fingerprints, debounce writes, watch files, own background reload tasks, discover default paths, or combine bundled defaults with user-owned overrides in this change. When an initial load or replacement fails before any active configuration exists, the hook SHALL return structured errors and leave active file-backed configuration absent while preserving failed-load metadata for management-visible diagnostics.

#### Scenario: Valid replacement publishes new snapshot
- **WHEN** a candidate TOML configuration passes parsing and validation through the replacement hook
- **THEN** `oxmux` publishes the new active configuration and management-visible snapshot data in a single deterministic replacement step

#### Scenario: Invalid replacement preserves previous snapshot
- **WHEN** a candidate TOML configuration fails parsing or validation through the replacement hook after a previous valid configuration exists
- **THEN** `oxmux` returns structured validation errors and keeps the previous active configuration and management snapshot available to callers

#### Scenario: Initial invalid load leaves active configuration absent
- **WHEN** a candidate TOML configuration fails parsing or validation before any valid file-backed configuration has been published
- **THEN** `oxmux` returns structured validation errors, exposes failed-load metadata, and does not synthesize an active file-backed configuration from unrelated defaults

#### Scenario: Hook points do not start a watcher
- **WHEN** maintainers inspect or test file-backed configuration loading in this change
- **THEN** `oxmux` exposes explicit load/validate/replace seams but does not start filesystem watchers, debounce tasks, background reload workers, remote sync, GPUI settings UI, platform launch-at-login persistence, or credential storage

#### Scenario: Successful replacement clears failed-load metadata
- **WHEN** a failed replacement is followed by a valid replacement
- **THEN** `oxmux` publishes the valid configuration and clears the previous failed-load metadata from management-visible state

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

### Requirement: File configuration represents streaming robustness policy
The `oxmux` core SHALL load and validate file-backed streaming robustness configuration for keepalive interval, bootstrap retry count, timeout behavior, and cancellation behavior without depending on `oxidemux`, GPUI, provider SDKs, OAuth UI, credential storage, remote configuration services, databases, filesystem watchers, or live upstream streams.

The canonical TOML table SHALL be `[streaming]`. Supported fields SHALL be `keepalive-interval-ms`, `bootstrap-retry-count`, `timeout-ms`, and `cancellation`. Duration fields use integer milliseconds and SHALL accept values from `1` through `300000` inclusive. Explicit `0` for duration fields SHALL be invalid; omission disables the behavior. `bootstrap-retry-count` is the number of additional attempts after the initial attempt and SHALL accept values from `0` through `10` inclusive. Negative numbers, floats, strings for numeric fields, and integer overflow SHALL be invalid. Supported cancellation values SHALL be `"disabled"`, `"client-disconnect"`, and `"timeout"`. `cancellation = "timeout"` SHALL require a configured `timeout-ms`; `timeout-ms` with `cancellation = "disabled"` SHALL record timeout policy/metadata without automatically converting timeout into cancellation. `cancellation = "client-disconnect"` SHALL configure deterministic core policy and mock outcomes only; live request-context disconnect propagation is deferred to a later transport change.

Omitted `[streaming]` configuration SHALL produce deterministic disabled defaults: no keepalive interval, `bootstrap-retry-count = 0`, no timeout, and `cancellation = "disabled"`. Unknown streaming fields or nested tables SHALL be rejected under the existing strict configuration model.

#### Scenario: Streaming policy can be configured from TOML
- **WHEN** a Rust consumer loads a syntactically valid local TOML configuration containing supported streaming robustness fields
- **THEN** `oxmux` returns typed configuration data that represents keepalive interval, bootstrap retry count, timeout behavior, and cancellation behavior for core runtime consumers

#### Scenario: Streaming policy accepts canonical field names
- **WHEN** a Rust consumer loads TOML containing `[streaming]` with `keepalive-interval-ms`, `bootstrap-retry-count`, `timeout-ms`, and `cancellation`
- **THEN** `oxmux` maps those fields into the typed streaming robustness policy without requiring app-shell configuration types

#### Scenario: Streaming policy defaults are explicit
- **WHEN** a TOML configuration omits streaming robustness settings
- **THEN** `oxmux` produces deterministic default streaming policy values rather than relying on transport-specific implicit defaults

#### Scenario: Streaming policy fields can be partially omitted
- **WHEN** a TOML configuration includes `[streaming]` with only some supported fields
- **THEN** `oxmux` applies disabled defaults for omitted fields and validates supplied fields normally

#### Scenario: Invalid streaming configuration is structured
- **WHEN** loaded configuration contains an unsupported keepalive interval, bootstrap retry count, timeout value, or cancellation behavior
- **THEN** validation fails with structured configuration error data identifying the streaming field path and invalid value category
- **AND** field paths use the canonical names, such as `streaming.keepalive-interval-ms`, `streaming.bootstrap-retry-count`, `streaming.timeout-ms`, and `streaming.cancellation`

#### Scenario: Streaming numeric ranges are enforced
- **WHEN** loaded configuration contains `0`, a negative number, a float, a string, or an overflow-sized integer for `streaming.keepalive-interval-ms` or `streaming.timeout-ms`
- **THEN** validation fails with structured configuration error data for the affected duration field
- **AND** omission remains the only way to disable a duration behavior

#### Scenario: Streaming retry count range is enforced
- **WHEN** loaded configuration contains a negative number, a number greater than `10`, a float, a string, or an overflow-sized integer for `streaming.bootstrap-retry-count`
- **THEN** validation fails with structured configuration error data for `streaming.bootstrap-retry-count`
- **AND** `streaming.bootstrap-retry-count = 0` remains valid and means no additional attempts

#### Scenario: Timeout cancellation requires timeout policy
- **WHEN** loaded configuration sets `streaming.cancellation = "timeout"` without `streaming.timeout-ms`
- **THEN** validation fails with structured configuration error data identifying the missing timeout policy relationship

#### Scenario: Timeout policy can be metadata-only
- **WHEN** loaded configuration sets `streaming.timeout-ms` and leaves cancellation omitted or sets `streaming.cancellation = "disabled"`
- **THEN** validation succeeds and represents timeout metadata policy without automatic timeout-driven cancellation

#### Scenario: Invalid streaming configuration stays in configuration error taxonomy
- **WHEN** loaded configuration contains an unsupported streaming policy value
- **THEN** validation returns `CoreError::Configuration` data rather than `CoreError::Streaming`

#### Scenario: Bootstrap retry count distinguishes retries from attempts
- **WHEN** a TOML configuration sets `streaming.bootstrap-retry-count = 2`
- **THEN** the resulting policy allows at most two additional bootstrap attempts after the initial streaming attempt

#### Scenario: Unknown streaming configuration fields remain rejected
- **WHEN** a TOML configuration contains an unknown streaming robustness table or field
- **THEN** `oxmux` returns a structured unknown-field configuration error rather than silently ignoring the value

#### Scenario: Layered configuration preserves streaming validation
- **WHEN** file-backed configuration layers or fixtures merge streaming policy values
- **THEN** the merged result validates the same canonical field paths, defaults, unknown-field rejection, and structured error categories as a single TOML file
