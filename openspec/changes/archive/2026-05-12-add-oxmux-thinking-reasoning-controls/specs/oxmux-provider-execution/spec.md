## ADDED Requirements

### Requirement: Provider execution receives normalized reasoning metadata

The `oxmux` provider execution request boundary SHALL be able to receive normalized provider-neutral reasoning or thinking metadata and compatibility outcome data alongside the canonical protocol request without requiring concrete provider SDKs, provider-specific beta headers, OAuth flows, platform credential storage, GPUI, or outbound provider network calls in this change. Provider execution request metadata SHALL carry normalized reasoning intent plus the compatibility outcome evaluated for the already-selected provider/account/model target, while provider execution result metadata MAY echo the final outcome for diagnostics and future management surfacing.

#### Scenario: Execution request preserves reasoning intent

- **WHEN** a Rust consumer constructs a provider execution request for a selected provider/account with normalized reasoning intent
- **THEN** the request preserves that intent as typed `oxmux` metadata without forcing the mock provider or execution boundary to generate provider-specific payload rewrites

#### Scenario: Execution request preserves compatibility outcome

- **WHEN** reasoning compatibility has been evaluated for the selected provider/account/model target before provider execution
- **THEN** the provider execution request can carry the normalized reasoning intent and the typed supported, ignored, degraded, unsupported, or unknown compatibility outcome without requiring the provider executor to reselect a route or parse opaque payload fields

#### Scenario: Execution metadata reports ignored reasoning capability

- **WHEN** reasoning intent is ignored because the selected provider/account/model target cannot honor alias-derived or permissive reasoning controls
- **THEN** provider execution metadata can expose the ignored-capability outcome so callers and future management snapshots can explain the compatibility downgrade

#### Scenario: Execution metadata can report degraded or unknown reasoning capability

- **WHEN** reasoning intent is degraded or has unknown support for the selected provider/account/model target
- **THEN** provider execution metadata can expose typed degraded or unknown outcome data so callers and future management snapshots can explain the compatibility state without parsing display text

#### Scenario: Mock provider tests remain deterministic

- **WHEN** provider execution tests cover reasoning metadata propagation
- **THEN** they use deterministic in-memory mock providers and do not require real provider accounts, credentials, provider SDKs, outbound network calls, provider-specific beta headers, OAuth flows, token refresh, platform secret storage, GPUI, or `oxidemux` app-shell dependencies

### Requirement: Provider execution does not own reasoning payload rewrites

Provider execution in this change SHALL NOT implement concrete provider-specific reasoning payload rewrites, beta headers, or SDK request construction; those behaviors SHALL remain deferred to future provider adapter or protocol translator changes that consume the normalized reasoning metadata.

#### Scenario: Mock provider preserves canonical request envelope

- **WHEN** a mock provider executes a request containing reasoning metadata
- **THEN** it preserves the canonical request envelope and typed metadata without claiming that OpenAI, Claude, Gemini, Codex, or provider-specific reasoning payload translation occurred

#### Scenario: Provider execution does not parse opaque reasoning fields

- **WHEN** a provider execution request contains an opaque payload body with provider-specific reasoning-looking fields
- **THEN** provider execution does not parse those fields for reasoning intent or compatibility state and relies only on typed `oxmux` reasoning metadata supplied before execution
