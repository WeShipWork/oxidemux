## ADDED Requirements

### Requirement: File-backed configuration supplies model registry inputs
Validated file-backed configuration SHALL provide deterministic model registry inputs from provider declarations, provider account declarations, routing defaults, routing default groups, and routing policy data without requiring live provider model discovery, provider SDKs, OAuth flows, credential storage, provider scraping, remote model updater jobs, GPUI, `oxidemux`, or outbound provider network calls.

#### Scenario: Routing defaults produce model registry candidates
- **WHEN** a valid TOML configuration declares provider accounts and `[[routing.defaults]]` entries for one model
- **THEN** the validated configuration can supply deterministic model registry candidates for that model, preserving provider identifiers, optional account identifiers, routing default names, candidate order, and fallback metadata

#### Scenario: Provider declarations supply protocol and eligibility metadata
- **WHEN** a valid TOML configuration declares `[[providers]]` entries with `protocol-family` and `routing-eligible` values
- **THEN** model registry construction can attach provider family and routing eligibility metadata to entries derived from those providers

#### Scenario: File-backed registry construction preserves strict validation
- **WHEN** TOML configuration contains unknown fields or invalid provider/account/routing references while preparing model registry inputs
- **THEN** validation fails through existing structured configuration errors instead of deferring invalid registry state to listing time

#### Scenario: Credential references remain redacted
- **WHEN** model registry entries are built from provider account declarations that include credential references
- **THEN** registry metadata may identify account applicability but MUST NOT expose raw credential material or require credential lookup during registry construction
