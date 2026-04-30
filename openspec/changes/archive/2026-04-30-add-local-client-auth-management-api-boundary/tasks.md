## 1. Local Client Authorization Primitives

- [x] 1.1 Add `oxmux` local client authorization types for disabled and required protection policies, protected route scopes, configured credentials, redacted metadata, and structured authorization outcomes.
- [x] 1.2 Ensure local client authorization secrets are not exposed through `Debug`, `Display`, status values, management snapshots, or structured errors.
- [x] 1.3 Re-export the local client authorization primitives through the public `oxmux` facade without adding GPUI, platform credential storage, OAuth, provider SDK, or app-shell dependencies.

## 2. Runtime Route Classification and Authorization

- [x] 2.1 Add explicit local route classification for `GET /health`, `POST /v1/chat/completions`, `/v0/management/*`, and unsupported requests before route dispatch.
- [x] 2.2 Extend bounded local HTTP parsing to retain only the header data needed for content length and `Authorization: Bearer <token>` local client authorization decisions.
- [x] 2.3 Apply inference authorization before `POST /v1/chat/completions` invokes routing or provider execution when inference protection is configured.
- [x] 2.4 Add deterministic protected `/v0/management/*` boundary responses for authorized and unauthorized management/status/control requests without implementing a full remote management API.
- [x] 2.5 Preserve stable unauthenticated `GET /health`, loopback-only binding, bounded request limits, and deterministic unsupported-path behavior.

## 3. Management and Error Surfaces

- [x] 3.1 Add management/status metadata that can report local route protection configuration and authorization health without exposing local client secrets.
- [x] 3.2 Add structured unauthorized local route errors or response codes that callers can match without parsing display strings.
- [x] 3.3 Verify local client authorization is never forwarded to mock provider execution or represented as a provider credential reference.

## 4. Tests and Verification

- [x] 4.1 Add runtime tests for authorized and unauthorized `POST /v1/chat/completions` requests.
- [x] 4.2 Add runtime tests for authorized and unauthorized `/v0/management/*` boundary requests.
- [x] 4.3 Add runtime tests proving inference authorization does not grant management/status/control access, and management authorization does not grant inference access.
- [x] 4.4 Add bearer parsing tests for missing, malformed, wrong-scheme, wrong-token, and missing-configured-credential cases.
- [x] 4.5 Add redaction tests for local client authorization configuration, errors, and management/status surfaces.
- [x] 4.6 Update public API documentation tests as needed for newly exported `oxmux` types.
- [x] 4.7 Run `openspec validate --changes add-local-client-auth-management-api-boundary`, `cargo test -p oxmux`, and `mise run ci`.
