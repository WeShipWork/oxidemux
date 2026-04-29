## Purpose

Define typed, deterministic routing policy primitives owned by the `oxmux`
headless core crate for model aliases, provider/account targeting, fallback
selection, availability-aware outcomes, and structured routing failures.
## Requirements
### Requirement: Typed routing policy configuration
The `oxmux` core SHALL expose typed routing policy configuration primitives for model aliases, ordered provider/account candidates, fallback behavior, explicit provider/account targeting, and caller-supplied provider/account availability states without depending on GPUI, `oxidemux`, provider SDKs, HTTP clients, OAuth flows, credential storage, live quota fetching, or outbound provider network calls.

#### Scenario: Policy config represents model aliasing
- **WHEN** a Rust consumer configures a routing policy with a model alias from a public model name to a canonical provider model name
- **THEN** the policy can be constructed and inspected through typed `oxmux` primitives without parsing strings outside the model identifier values themselves

#### Scenario: Policy config represents ordered candidates
- **WHEN** a Rust consumer configures multiple provider/account candidates for a resolved model
- **THEN** the policy preserves deterministic priority order for selection tests and future proxy routing behavior

#### Scenario: Policy config represents explicit targeting
- **WHEN** a Rust consumer configures an explicit provider target with or without an explicit account target
- **THEN** the routing policy can represent that target separately from priority fallback candidates

#### Scenario: Policy config remains headless
- **WHEN** maintainers inspect or test routing policy configuration primitives
- **THEN** the `oxmux` crate remains free of GPUI, app-shell, provider SDK, HTTP, OAuth, token refresh, credential storage, and live quota-fetching dependencies

### Requirement: Deterministic route selection outcomes
The `oxmux` core SHALL provide deterministic routing selection behavior that resolves model aliases before selecting from explicit targets or ordered candidates and returns typed selection results containing the requested model, resolved model, selected provider, selected account when present, decision mode, and skipped candidate metadata.

#### Scenario: Model alias resolves before selection
- **WHEN** a request asks for a model alias that maps to a canonical model with available candidates
- **THEN** route selection returns a typed result containing both the requested alias and the resolved canonical model

#### Scenario: Priority order selects first available candidate
- **WHEN** multiple candidates are configured and the highest-priority candidate is available
- **THEN** route selection returns the highest-priority provider/account candidate without evaluating lower-priority candidates as selected

#### Scenario: Fallback skips unavailable candidate
- **WHEN** fallback is enabled and the highest-priority candidate is exhausted or unavailable
- **THEN** route selection skips that candidate with typed skip metadata and selects the next available candidate in deterministic order

#### Scenario: Fallback disabled fails before lower-priority selection
- **WHEN** fallback is disabled and the highest-priority candidate is exhausted or unavailable
- **THEN** route selection returns a structured `CoreError` instead of selecting a lower-priority candidate

#### Scenario: Explicit account targeting wins over fallback candidates
- **WHEN** a request includes an explicit provider/account target and that target is available
- **THEN** route selection returns the explicit target rather than a different priority or fallback candidate

#### Scenario: Degraded provider selection requires permission
- **WHEN** only degraded candidates remain and degraded routing is explicitly allowed
- **THEN** route selection may return a degraded candidate with typed metadata indicating that degraded state influenced the decision

### Requirement: Structured routing failures
Routing failures in `oxmux` SHALL surface as structured `CoreError` values with matchable routing failure details rather than display strings.

#### Scenario: Missing explicit target fails structurally
- **WHEN** a request targets a provider/account pair that is not present in the routing availability input
- **THEN** route selection returns a structured `CoreError` describing the missing target without falling back silently

#### Scenario: Exhausted candidates fail structurally
- **WHEN** every candidate for the resolved model is exhausted
- **THEN** route selection returns a structured `CoreError` describing exhausted routing candidates

#### Scenario: Degraded-only candidates fail when degraded routing is disallowed
- **WHEN** every remaining candidate is degraded and degraded routing is not allowed
- **THEN** route selection returns a structured `CoreError` describing degraded-only routing candidates

#### Scenario: No route for model fails structurally
- **WHEN** a request asks for a model with no alias mapping and no candidates
- **THEN** route selection returns a structured `CoreError` describing the missing route for that model

#### Scenario: Invalid policy fails structurally
- **WHEN** a routing policy contains invalid data such as an empty model identifier, duplicate ambiguous aliases, or an explicit target that cannot be represented as a provider/account target
- **THEN** route selection or policy validation returns a structured `CoreError` describing the invalid policy field without panicking or silently ignoring the invalid input

### Requirement: Routing tests remain networkless
Default `oxmux` routing policy tests SHALL use deterministic in-memory policies and availability inputs, and SHALL NOT require real provider accounts, credentials, provider SDKs, outbound provider network calls, OAuth flows, token refresh, platform secret storage, GPUI, or `oxidemux` app-shell dependencies.

#### Scenario: Tests cover required routing modes
- **WHEN** maintainers run default `oxmux` tests for routing policy primitives
- **THEN** deterministic tests cover model aliasing, priority order, fallback enabled, fallback disabled, explicit account targeting, exhausted providers, degraded providers, and invalid policy failures without external services

#### Scenario: Routing tests preserve core dependency boundary
- **WHEN** maintainers inspect or run routing policy tests
- **THEN** the tests use only `oxmux` core primitives and do not depend on GPUI, `oxidemux`, provider SDKs, HTTP clients, OAuth flows, credential storage, or live quota fetching

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
