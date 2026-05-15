## ADDED Requirements

### Requirement: Typed model registry entries
The `oxmux` core SHALL expose typed model registry entries that represent configured model identifiers, provider-native model targets, aliases, forks, provider family, provider/account applicability, routing eligibility, streaming support, disabled state, degraded state, and human-readable reasons without requiring `oxidemux`, GPUI, provider SDKs, OAuth flows, platform credential storage, provider scraping, remote model updater jobs, or outbound provider network calls.

#### Scenario: Registry entry preserves model identity layers
- **WHEN** a Rust consumer inspects a model registry entry built from configured routing and provider metadata
- **THEN** the entry exposes the user-facing listed model identifier separately from the provider-native model identifier and any alias or fork metadata

#### Scenario: Registry entry exposes provider access metadata
- **WHEN** a registry entry is associated with a provider and account declaration
- **THEN** the entry exposes provider family, provider identifier, optional account identifier, routing eligibility, streaming support, disabled or degraded state, and reasons without exposing raw credentials or requiring live provider access

#### Scenario: Registry remains headless
- **WHEN** maintainers inspect or test model registry primitives
- **THEN** the `oxmux` crate remains free of `oxidemux`, GPUI, provider SDK, OAuth, platform credential storage, provider scraping, remote updater, and outbound network dependencies

### Requirement: Deterministic model registry listing
The `oxmux` core SHALL provide deterministic model registry listing behavior that can return all configured entries, visible/routable entries, disabled entries, and degraded entries from in-memory core inputs without performing routing selection or provider execution.

#### Scenario: Listing returns configured entries deterministically
- **WHEN** a Rust consumer builds a registry from the same validated configuration, routing policy, provider summaries, and capability metadata twice
- **THEN** `oxmux` returns the same ordered model registry entries both times

#### Scenario: Listing distinguishes visible and disabled entries
- **WHEN** configured model metadata includes disabled or routing-ineligible entries
- **THEN** model listing preserves those entries with disabled or ineligible state instead of silently dropping them from the full registry

#### Scenario: Listing does not perform route selection
- **WHEN** a Rust consumer lists registry entries for a model that has multiple candidate providers or accounts
- **THEN** the listing reports the candidate entries and eligibility metadata without selecting a runtime route or mutating fallback state

### Requirement: Alias and fork metadata preservation
The `oxmux` core SHALL preserve alias and fork metadata so model listing consumers can explain how a requested model name maps to one or more provider/account/model targets.

#### Scenario: Alias listing preserves requested and resolved names
- **WHEN** a routing policy maps a requested model alias to a resolved model identifier
- **THEN** the model registry entry exposes both the alias and the resolved model identifier without requiring callers to parse a flattened model string

#### Scenario: Fork listing preserves multiple targets
- **WHEN** one listed model can route to multiple provider or account candidates
- **THEN** the model registry exposes those provider/account targets as distinct candidate metadata while preserving the shared listed model identity

#### Scenario: Alias metadata supports future reasoning controls
- **WHEN** future reasoning or thinking controls inspect registry metadata
- **THEN** they can consume typed alias and capability metadata without implementing model alias parsing in `oxidemux` or provider-specific adapters

### Requirement: Future model listing route serialization
The `oxmux` core SHALL define future `/v1/models` serialization semantics as a projection of typed model registry entries rather than a separate endpoint-owned model catalog.

#### Scenario: OpenAI-compatible model listing uses registry data
- **WHEN** a future local proxy route serializes an OpenAI-compatible `/v1/models` response
- **THEN** the route derives listed model identifiers and metadata from typed model registry entries

#### Scenario: OpenAI-compatible projection remains minimal
- **WHEN** a future local proxy route serializes an OpenAI-compatible `/v1/models` response
- **THEN** the projection can emit OpenAI-compatible model identifiers without dropping richer typed registry metadata needed by non-OpenAI protocol families and core consumers

#### Scenario: Serialization omits live discovery assumptions
- **WHEN** a model registry entry was built from static configuration or unknown provider/account state
- **THEN** future route serialization does not imply that upstream provider discovery, credential validation, quota checks, or live network calls occurred

### Requirement: Model registry tests remain networkless
Default `oxmux` model registry tests SHALL use deterministic in-memory or file-backed fixtures and SHALL NOT require real providers, real credentials, OAuth flows, provider SDKs, GPUI, `oxidemux`, platform credential storage, provider scraping, remote updater jobs, or outbound provider network calls.

#### Scenario: Tests cover static config-backed registry data
- **WHEN** `cargo test -p oxmux` runs model registry tests
- **THEN** tests cover static config-backed registry construction, provider/account metadata, routing eligibility, streaming capability, disabled or degraded state, and listing order without network access

#### Scenario: Tests cover alias and fork behavior
- **WHEN** `cargo test -p oxmux` runs model registry tests
- **THEN** tests cover alias preservation, resolved model identity, and multiple provider/account fork metadata without invoking route selection or provider execution
