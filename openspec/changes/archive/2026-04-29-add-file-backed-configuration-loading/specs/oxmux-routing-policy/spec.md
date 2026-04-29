## ADDED Requirements

### Requirement: Routing defaults validate against file-backed provider references
The `oxmux` core SHALL validate routing defaults loaded from file-backed configuration against declared provider/account references and typed routing policy primitives before publishing runtime configuration.

Provider identifiers SHALL be unique across the loaded file. Account identifiers SHALL be unique within their provider. Routing defaults SHALL be grouped by the tuple `(name, model)`. Multiple `[[routing.defaults]]` entries with the same `(name, model)` SHALL be treated as ordered candidates in file order. Entries with the same `name` and different `model` SHALL be distinct routing defaults. A duplicate candidate with the same `(name, model, provider-id, account-id)` tuple SHALL be invalid. A routing default SHALL reference a declared provider and MAY reference an account declared under that provider. Provider-only routing defaults SHALL remain valid only when existing routing policy primitives can represent provider-only selection for the referenced model. `fallback-enabled` SHALL apply to the candidate that declares it and SHALL only permit fallback to the next candidate in the same `(name, model)` group.

#### Scenario: Configured routing default resolves to declared candidates
- **WHEN** loaded TOML configuration declares providers/accounts and a routing default that references those declared identities
- **THEN** validation produces routing policy inputs that preserve deterministic candidate order and can be inspected or used by `oxmux` routing without stringly typed lookups outside identifier values

#### Scenario: Routing default with missing provider fails structurally
- **WHEN** loaded TOML configuration contains a routing default that references a provider or account not declared in the same configuration
- **THEN** validation returns a structured configuration error identifying the missing reference and does not publish the invalid routing default

#### Scenario: Duplicate provider or account identifiers fail structurally
- **WHEN** loaded TOML configuration declares duplicate provider identifiers or duplicate account identifiers within the same provider
- **THEN** validation returns a structured configuration error identifying the duplicate identity before publishing routing defaults

#### Scenario: Routing candidate order is preserved
- **WHEN** loaded TOML configuration declares multiple routing default candidates for the same `(name, model)` group in a specific order
- **THEN** validation produces routing policy inputs that preserve that deterministic file order within the group

#### Scenario: Duplicate routing candidate fails structurally
- **WHEN** loaded TOML configuration declares the same `(name, model, provider-id, account-id)` routing candidate more than once
- **THEN** validation returns a structured `InvalidRoutingDefault` error identifying the duplicate candidate before publishing routing defaults

#### Scenario: Routing default failure is not deferred to request routing
- **WHEN** a file-backed routing default cannot be represented by typed routing policy primitives
- **THEN** configuration validation fails before any proxy request is routed so app-visible state can show the configuration problem directly
