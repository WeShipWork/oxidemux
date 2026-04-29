## ADDED Requirements

### Requirement: Deterministic local TOML configuration loading
The `oxmux` core SHALL load a single deterministic local TOML configuration file into typed configuration data for app-visible proxy settings, provider references, routing defaults, logging settings, usage collection settings, and auto-start intent without depending on `oxidemux`, GPUI, provider SDKs, OAuth UI, platform credential storage, remote configuration services, databases, or filesystem watchers.

The initial file format SHALL be TOML loaded from `.toml` paths or already-read TOML contents and SHALL reject unknown fields at every level. The top-level document SHALL contain `version = 1` and `[proxy]`; MAY contain `[observability]`, `[lifecycle]`, zero or more `[[providers]]` entries, zero or more nested `[[providers.accounts]]` entries, and zero or more `[[routing.defaults]]` entries; and SHALL NOT require or accept a separate `format` field. This change SHALL NOT define default path discovery; callers choose the path or provide already-read TOML contents.

Required fields SHALL be `version`, `proxy.listen-address`, and `proxy.port`. Optional supported fields SHALL receive these defaults before semantic validation: `observability.logging = "standard"`, `observability.usage-collection = true`, and `lifecycle.auto-start = "disabled"`. Provider entries require non-empty `id`, accepted `protocol-family`, and `routing-eligible`. Account entries require non-empty `id` and non-empty `credential-reference`. Routing defaults require non-empty `name`, non-empty `model`, `provider-id`, and `fallback-enabled`; `account-id` is optional when provider-only selection is supported by typed routing primitives.

The canonical minimal valid TOML shape is:

```toml
version = 1

[proxy]
listen-address = "127.0.0.1"
port = 0

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

Accepted `protocol-family` values SHALL match protocol-family metadata already represented by `oxmux` protocol/routing/provider primitives. Accepted `logging` values SHALL be `off`, `standard`, or `verbose`. Accepted `auto-start` values SHALL be `disabled` or `enabled`. `usage-collection` SHALL be a boolean. `port = 0` SHALL mean caller permits the local runtime to bind an available loopback port; non-zero ports SHALL be within the accepted TCP port range. `proxy.listen-address` SHALL be a loopback IP literal for this change, such as `127.0.0.1` or `::1`; wildcard, unspecified, multicast, link-local, and public interface addresses SHALL be rejected.

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
