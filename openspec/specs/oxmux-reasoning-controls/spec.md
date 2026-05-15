# oxmux-reasoning-controls Specification

## Purpose

Define provider-neutral reasoning and thinking control primitives, validation, compatibility outcomes, diagnostics, and networkless test expectations owned by the `oxmux` headless core crate.

## Requirements
### Requirement: Provider-neutral reasoning intent
The `oxmux` core SHALL expose typed provider-neutral reasoning and thinking intent primitives that represent requested mode, source, effort, optional token budget, validation policy, and human-readable diagnostics without requiring `oxidemux`, GPUI, provider SDKs, OAuth flows, platform credential storage, provider-specific beta headers, payload rewrite DSLs, persisted TOML reasoning configuration, or outbound provider network calls.

For this change, explicit request metadata SHALL mean typed Rust `oxmux` metadata supplied by a core caller or deterministic test fixture. Extraction from OpenAI-compatible JSON fields, Claude payload fields, Gemini payload fields, Codex payload fields, or local HTTP route request bodies is deferred to a future protocol-translation or proxy-route change.

#### Scenario: Explicit reasoning intent is represented
- **WHEN** a Rust consumer constructs reasoning intent from explicit request metadata
- **THEN** the intent preserves the explicit source, mode, effort or token budget, and validation policy through typed `oxmux` values without parsing provider-specific payload rewrites

#### Scenario: Opaque provider payload is not parsed for reasoning intent
- **WHEN** a canonical protocol request contains an opaque OpenAI, Claude, Gemini, Codex, or provider-specific payload body with provider-specific reasoning-looking fields
- **THEN** this change does not parse those fields into reasoning intent unless the caller also supplies typed `oxmux` reasoning metadata

#### Scenario: Empty reasoning intent remains representable
- **WHEN** a request does not include reasoning or thinking controls
- **THEN** `oxmux` can represent the absence of requested reasoning behavior without inventing provider-specific defaults or mutating the request payload

#### Scenario: Empty reasoning intent does not require capability lookup
- **WHEN** a request does not include explicit, alias-derived, or default reasoning intent
- **THEN** reasoning compatibility handling preserves the absent intent without requiring provider/account/model reasoning capability metadata to be known

#### Scenario: Reasoning intent remains headless
- **WHEN** maintainers inspect or test reasoning intent primitives
- **THEN** the `oxmux` crate remains free of GPUI, app-shell, provider SDK, HTTP client, OAuth, platform credential storage, provider-specific beta header, and live provider discovery dependencies

### Requirement: Deterministic reasoning budget validation
The `oxmux` core SHALL validate provider-neutral reasoning effort and budget inputs deterministically and SHALL return structured `CoreError` data for invalid reasoning control values rather than panicking, silently clamping, or relying on provider-specific validation. Provider-neutral token budgets SHALL accept values from `1` through `200000` inclusive. Budget omission SHALL represent no explicit token budget, and `0` SHALL be invalid for provider-neutral token budgets. Provider-neutral effort and provider-neutral token budget controls SHALL be mutually exclusive in one normalized intent.

#### Scenario: Valid effort metadata is accepted
- **WHEN** a Rust consumer constructs reasoning intent with a supported provider-neutral effort level
- **THEN** validation succeeds and preserves that effort level as typed metadata

#### Scenario: Valid token budget metadata is accepted
- **WHEN** a Rust consumer constructs reasoning intent with a token budget from `1` through `200000` inclusive
- **THEN** validation succeeds and preserves that budget as typed metadata

#### Scenario: Invalid token budget fails structurally
- **WHEN** a Rust consumer constructs reasoning intent with `0`, a value greater than `200000`, or another invalid token budget
- **THEN** validation returns a structured `CoreError` identifying the reasoning budget field without contacting a provider or parsing display strings

#### Scenario: Effort and budget conflict fails structurally
- **WHEN** explicit request metadata supplies both provider-neutral effort and provider-neutral token budget controls for one normalized intent
- **THEN** validation returns a structured `CoreError` identifying the mutually exclusive fields instead of silently choosing one

#### Scenario: Ambiguous reasoning controls fail structurally
- **WHEN** explicit request metadata supplies conflicting reasoning controls that cannot be represented by one provider-neutral intent
- **THEN** validation returns a structured `CoreError` identifying the conflicting fields instead of silently choosing one

### Requirement: Alias-derived reasoning conventions
The `oxmux` core SHALL support narrow deterministic alias-derived reasoning or thinking conventions through typed alias metadata attached to in-memory alias definitions that can derive provider-neutral reasoning intent from configured model alias context while preserving the original requested model and resolved model separately. This change SHALL NOT parse suffix, bracket, or free-form model-name reasoning conventions unless a future OpenSpec change defines exact patterns and validation rules. This change SHALL NOT define persisted TOML alias reasoning configuration.

#### Scenario: Alias metadata derives reasoning intent
- **WHEN** an in-memory configured model alias declares supported typed reasoning metadata
- **THEN** `oxmux` derives typed reasoning intent with alias-derived source metadata while preserving the requested alias and resolved model identifier

#### Scenario: File-backed alias reasoning configuration is deferred
- **WHEN** a file-backed TOML configuration is loaded in this change
- **THEN** `oxmux` does not accept or require persisted reasoning metadata fields on aliases or routing defaults unless a future `oxmux-file-configuration` delta defines those fields

#### Scenario: Ordinary aliases are not reasoning controls
- **WHEN** a configured model alias has no supported reasoning convention
- **THEN** `oxmux` resolves the model alias without deriving reasoning intent or mutating the request payload

#### Scenario: Invalid alias metadata fails structurally
- **WHEN** configured typed alias reasoning metadata contains an invalid effort, mode, or budget
- **THEN** `oxmux` returns structured validation data identifying the alias-derived reasoning control problem before provider execution

#### Scenario: Explicit reasoning controls override alias metadata
- **WHEN** a request supplies explicit reasoning metadata and the selected alias also declares different typed reasoning metadata
- **THEN** `oxmux` preserves the explicit reasoning intent and records typed ignored-alias diagnostic metadata without merging the two controls or letting the alias override caller intent

### Requirement: Reasoning capability compatibility outcomes
The `oxmux` core SHALL compare normalized reasoning intent against provider, account, model, or registry capability metadata and SHALL return typed compatibility outcomes that distinguish supported, ignored, degraded, unsupported, and unknown reasoning behavior. Reasoning capability metadata SHALL declare target support, unknown state, and limits; reasoning compatibility outcome metadata SHALL describe the result for one selected request. Capability metadata SHALL NOT use ignored as a target capability state, because ignored is an outcome of applying handling policy to a request.

When capability metadata exists at multiple layers, model-candidate capability metadata SHALL take precedence over account capability metadata, account capability metadata SHALL take precedence over provider capability metadata, and absence of applicable metadata SHALL be represented as unknown rather than silently treated as unsupported.

#### Scenario: Supported reasoning intent is forwarded
- **WHEN** a selected provider/account/model target declares support for the normalized reasoning intent
- **THEN** compatibility evaluation returns a supported outcome and preserves the normalized intent for protocol translation or provider execution metadata

#### Scenario: Capability and outcome remain distinct
- **WHEN** a selected provider/account/model target declares reasoning support, partial support, unsupported state, or unknown state
- **THEN** `oxmux` records that declaration as capability metadata and separately records supported, ignored, degraded, unsupported, or unknown compatibility outcome data for the current request

#### Scenario: More specific capability metadata wins
- **WHEN** provider, account, and model-candidate reasoning capability metadata disagree for the same selected route
- **THEN** compatibility evaluation uses model-candidate metadata before account metadata and account metadata before provider metadata, preserving typed diagnostics that identify the metadata layer used

#### Scenario: Explicit unsupported intent fails visibly
- **WHEN** explicit request metadata asks for reasoning behavior that the selected provider/account/model target cannot honor under strict handling
- **THEN** `oxmux` returns a structured unsupported-capability error rather than silently removing or rewriting the reasoning controls

#### Scenario: Alias-derived unsupported intent can be ignored with metadata
- **WHEN** alias-derived or default reasoning intent cannot be honored and permissive handling is configured
- **THEN** `oxmux` returns ignored-capability metadata that identifies the requested reasoning behavior and the unsupported capability without mutating provider payloads silently

#### Scenario: Degraded reasoning support is visible
- **WHEN** a provider/account/model target can only partially honor the normalized reasoning intent
- **THEN** compatibility evaluation returns a degraded outcome with typed reasons that callers can inspect without parsing display text

#### Scenario: Unknown explicit reasoning support fails visibly under strict handling
- **WHEN** explicit request metadata asks for reasoning behavior and the selected provider/account/model target has unknown reasoning capability under strict handling
- **THEN** `oxmux` returns a structured unknown-capability error rather than silently removing or rewriting the reasoning controls

#### Scenario: Unknown alias-derived reasoning support can be ignored with metadata
- **WHEN** alias-derived or default reasoning intent targets a provider/account/model with unknown reasoning capability and permissive handling is configured
- **THEN** `oxmux` returns ignored unknown-capability metadata identifying the requested reasoning behavior and the unknown capability without mutating provider payloads silently

### Requirement: Reasoning diagnostics preserve model identity layers
Reasoning normalization, validation, and compatibility diagnostics SHALL preserve model identity layers and selected target context needed to explain provider access behavior without parsing display text.

#### Scenario: Reasoning diagnostics identify requested and resolved models
- **WHEN** reasoning intent is derived from explicit metadata, alias metadata, or compatibility evaluation
- **THEN** diagnostics can identify the requested model alias when present, resolved model identifier, reasoning source, and ignored or conflicting source metadata without flattening those values into one model string

#### Scenario: Reasoning compatibility diagnostics identify selected target
- **WHEN** reasoning compatibility evaluation returns supported, ignored, degraded, unsupported, or unknown outcome data
- **THEN** the outcome can identify the selected provider, selected account when present, resolved model identifier, provider-native model identifier when known, and typed capability gap or degradation reason without requiring provider-specific payload parsing

### Requirement: Reasoning tests remain networkless
Default `oxmux` reasoning-control tests SHALL use deterministic in-memory inputs and SHALL NOT require real provider accounts, credentials, provider SDKs, outbound provider network calls, provider-specific beta headers, OAuth flows, token refresh, platform secret storage, GPUI, or `oxidemux` app-shell dependencies.

#### Scenario: Tests cover explicit and alias-derived reasoning controls
- **WHEN** maintainers run default `oxmux` reasoning-control tests
- **THEN** deterministic tests cover explicit metadata, typed alias-derived metadata, absent intent, invalid budgets, effort/budget mutual exclusivity, explicit-over-alias precedence, supported capability, unsupported strict behavior, ignored permissive behavior, degraded compatibility, and unknown capability without external services

#### Scenario: Tests preserve core dependency boundary
- **WHEN** maintainers inspect or run reasoning-control tests
- **THEN** those tests use only `oxmux` core primitives and do not depend on GPUI, `oxidemux`, provider SDKs, HTTP clients, OAuth flows, credential storage, or live provider discovery
