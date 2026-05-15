## Purpose

Define the canonical protocol request, response, metadata, and deferred
translation boundaries owned by the `oxmux` headless core crate.
## Requirements
### Requirement: Canonical protocol request and response shapes
The `oxmux` core SHALL expose canonical protocol request and response structures for future proxy handling and deterministic provider execution tests without performing provider translation or network transport.

#### Scenario: Construct canonical request shape
- **WHEN** a Rust consumer constructs a canonical protocol request with a supported protocol family, model identifier, and payload metadata
- **THEN** the request is represented by typed core structures without requiring provider SDKs, HTTP clients, or app-shell state

#### Scenario: Provider execution consumes canonical request shape
- **WHEN** a Rust consumer builds a provider execution request for a mock provider
- **THEN** the execution request uses the existing `CanonicalProtocolRequest` shape rather than a provider-specific SDK request or app-shell request copy

#### Scenario: Construct canonical response shape
- **WHEN** a Rust consumer constructs a canonical protocol response with protocol family, status metadata, and payload metadata
- **THEN** the response is represented by typed core structures without requiring provider SDKs, HTTP clients, or app-shell state

#### Scenario: Provider execution returns canonical response shape
- **WHEN** a mock provider returns a successful, degraded, quota-limited, or streaming-capable execution outcome
- **THEN** any response payload is represented with the existing `CanonicalProtocolResponse` shape while provider execution metadata carries health, quota, or capability state

### Requirement: Provider protocol family metadata
The `oxmux` core SHALL map OpenAI, Gemini, Claude, Codex, and provider-specific protocol formats to explicit typed metadata.

#### Scenario: Identify supported protocol family metadata
- **WHEN** a consumer inspects protocol metadata for OpenAI, Gemini, Claude, Codex, or a provider-specific format
- **THEN** the family is represented by an explicit typed value that can be matched deterministically

#### Scenario: Preserve provider-specific metadata
- **WHEN** a provider-specific protocol format is represented
- **THEN** the metadata preserves a typed provider-specific identifier without collapsing it into a free-form unknown state

### Requirement: Deferred translation interface results
The `oxmux` core SHALL define translation interfaces that return structured errors or explicit placeholder results when translation behavior is intentionally deferred, and provider execution SHALL NOT imply protocol translation has occurred unless a future translator explicitly performs it.

#### Scenario: Translation behavior is deferred
- **WHEN** a consumer invokes a translation boundary that has no concrete translator implementation yet
- **THEN** the result identifies the deferred translation behavior without panicking, silently succeeding, or making outbound provider calls

#### Scenario: Provider execution does not translate protocols
- **WHEN** a consumer executes a mock provider request with canonical protocol metadata
- **THEN** provider execution preserves and consumes the canonical protocol envelope without performing OpenAI, Gemini, Claude, Codex, or provider-specific translation behavior in this change

#### Scenario: Invalid protocol input is rejected
- **WHEN** a consumer attempts to create or validate a protocol boundary with invalid required metadata
- **THEN** validation returns a structured `CoreError` instead of constructing an ambiguous request or response shape

### Requirement: Deterministic protocol shape validation
The `oxmux` core SHALL provide deterministic construction and validation semantics for protocol request and response shapes.

#### Scenario: Equivalent protocol inputs produce equivalent shapes
- **WHEN** tests construct protocol request or response shapes from equivalent typed inputs
- **THEN** equality and debug representations are deterministic enough for regression tests to assert shape behavior

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

