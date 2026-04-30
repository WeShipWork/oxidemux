# oxmux-local-client-auth Specification

## Purpose
TBD - created by archiving change add-local-client-auth-management-api-boundary. Update Purpose after archive.
## Requirements
### Requirement: Core represents local client authorization
The `oxmux` crate SHALL define local client authorization primitives for loopback clients that are separate from provider credentials and safe to expose through redacted metadata, structured outcomes, and tests.

#### Scenario: Local client credential is not a provider credential
- **WHEN** a local client credential is configured for access to the local proxy runtime
- **THEN** `oxmux` represents it as local client authorization state rather than as a provider API key, OAuth token, provider credential reference, or platform credential storage handle

#### Scenario: Authorization metadata is redacted
- **WHEN** local client authorization configuration, status, errors, debug output, or display text is inspected
- **THEN** raw local client secrets are not exposed and only redacted metadata or structured authorization state is visible

#### Scenario: Bearer authorization is accepted for protected HTTP routes
- **WHEN** a protected local HTTP route receives an `Authorization: Bearer <token>` header whose token matches the configured local client credential for that route scope
- **THEN** `oxmux` treats the request as locally authorized for that scope without representing the token as a provider credential

#### Scenario: Missing local authorization is structured
- **WHEN** a protected local route receives no local client authorization credential
- **THEN** `oxmux` returns a deterministic unauthorized outcome that callers can inspect without parsing display text

#### Scenario: Malformed local authorization is structured
- **WHEN** a protected local HTTP route receives an authorization header with a missing token, unsupported scheme, malformed bearer value, or otherwise invalid header shape
- **THEN** `oxmux` returns a deterministic unauthorized outcome that does not reveal the expected credential value

#### Scenario: Invalid local authorization is structured
- **WHEN** a protected local route receives an invalid local client authorization credential
- **THEN** `oxmux` returns a deterministic unauthorized outcome that does not reveal the expected credential value

### Requirement: Core defines fail-safe authorization policies
The `oxmux` local client authorization model SHALL define explicit policy states for disabled and required route protection, and required protection SHALL fail closed when no expected local credential is configured.

#### Scenario: Disabled protection does not require local authorization
- **WHEN** local authorization policy is disabled for a route scope
- **THEN** `oxmux` does not require an `Authorization` header for that scope while still preserving route classification and loopback-only runtime behavior

#### Scenario: Required protection accepts only matching credentials
- **WHEN** local authorization policy is required for a route scope
- **THEN** `oxmux` authorizes only requests with a matching local client credential for that scope and rejects missing, malformed, or mismatched credentials deterministically

#### Scenario: Missing configured credential fails closed
- **WHEN** local authorization policy is required for a route scope but no expected local credential is configured
- **THEN** `oxmux` rejects protected requests for that scope with a deterministic unauthorized or configuration outcome rather than allowing access

### Requirement: Core distinguishes route authorization scopes
The `oxmux` local client authorization model SHALL distinguish inference access from management/status/control access so future CLI, IDE, and app-shell clients can be authorized without Amp-specific coupling.

#### Scenario: Inference access can be authorized independently
- **WHEN** a local client is authorized for inference access but not management/status/control access
- **THEN** `oxmux` can allow protected inference routes while rejecting protected management/status/control routes with structured unauthorized responses

#### Scenario: Management access can be authorized independently
- **WHEN** a local client is authorized for management/status/control access but not inference access
- **THEN** `oxmux` can allow protected management/status/control routes while rejecting protected inference routes with structured unauthorized responses

#### Scenario: Authorization scopes remain client-generic
- **WHEN** maintainers inspect the local client authorization API
- **THEN** scope names and outcomes are generic to local proxy clients and do not mention Amp-specific URL rewriting, provider fallback, GPUI views, or desktop-only concepts

