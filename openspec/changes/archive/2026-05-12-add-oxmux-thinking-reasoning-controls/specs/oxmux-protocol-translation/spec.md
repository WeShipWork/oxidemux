## ADDED Requirements

### Requirement: Protocol requests carry reasoning metadata

Canonical protocol request boundaries in `oxmux` SHALL be able to carry normalized provider-neutral reasoning or thinking intent metadata independently from opaque protocol payload bodies so translators and provider execution seams do not need to parse provider-specific payload fragments to understand requested reasoning behavior. For this change, canonical protocol request metadata carries normalized intent and source diagnostics; selected-target compatibility outcomes are produced after routing and carried by provider execution metadata.

#### Scenario: Canonical request preserves normalized reasoning intent

- **WHEN** a Rust consumer constructs a canonical protocol request with normalized reasoning intent metadata
- **THEN** the request preserves that metadata separately from the opaque protocol payload and model identifier

#### Scenario: Canonical request does not parse opaque reasoning fields

- **WHEN** a Rust consumer constructs a canonical protocol request with an opaque provider-shaped payload that contains reasoning-looking fields
- **THEN** the canonical request preserves the opaque payload without deriving reasoning intent unless typed `oxmux` reasoning metadata is supplied separately

#### Scenario: Canonical request without reasoning remains valid

- **WHEN** a Rust consumer constructs a canonical protocol request without reasoning metadata
- **THEN** the request remains valid and does not imply provider-specific reasoning defaults or payload rewrites

### Requirement: Protocol translation preserves reasoning intent without concrete rewrites

Protocol translation boundaries in `oxmux` SHALL preserve reasoning/thinking intent metadata and may report deferred translation state without implementing provider-specific payload rewrites, provider-specific beta headers, provider SDK calls, or outbound network behavior in this change. Target-specific compatibility checks SHALL remain typed core outcomes and SHALL NOT require translators to mutate or drop reasoning metadata silently.

#### Scenario: Deferred translation preserves reasoning metadata

- **WHEN** a consumer invokes a protocol translation boundary with a canonical request that includes normalized reasoning intent
- **THEN** the deferred translation outcome preserves the existence of reasoning metadata without claiming a provider-specific rewrite occurred

#### Scenario: Unsupported translator capability is visible

- **WHEN** a target protocol or translator cannot honor normalized reasoning metadata under strict handling
- **THEN** `oxmux` returns structured compatibility or unsupported-capability data instead of silently dropping the metadata from the protocol request
