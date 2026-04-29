## Context

`oxmux` currently exposes a loopback-only local runtime with `GET /health` and a minimal `POST /v1/chat/completions` inference smoke route. The runtime parser is bounded and local-only, but its request representation only retains method, path, and body; local client authorization headers are not yet modeled. Management state exists as typed Rust snapshots, while management HTTP endpoints beyond `/health` were intentionally deferred.

Issue #18 adds the security and route-boundary contract needed before real provider adapters, CLI/IDE clients, and app-facing management controls depend on the local runtime. The boundary must authorize local clients without reusing or exposing provider credentials, and it must keep inference routes separate from management/status/control routes.

## Goals / Non-Goals

**Goals:**

- Represent local client authorization in `oxmux` as caller-owned access to the local proxy, not as provider authentication.
- Classify local runtime routes as health, inference, management/status/control, or unsupported before dispatch, with `/v0/management/*` reserved as the protected management namespace.
- Allow inference and management/status/control routes to use distinct authorization policies so future CLI/IDE clients can be granted only the access they need.
- Preserve loopback-only binding, bounded request parsing, stable `/health`, deterministic unsupported-path responses, and headless `oxmux` ownership.
- Add deterministic tests for valid, missing, and invalid authorization on inference and management/status/control paths.

**Non-Goals:**

- No remote management web panel or public network exposure by default.
- No OAuth login flow, token refresh, provider credential resolution, or platform secret storage.
- No Amp-specific URL rewriting, provider fallback, or new OpenAI-compatible endpoints beyond the existing minimal smoke route.
- No `oxidemux` GPUI, tray, notification, packaging, or desktop lifecycle work.
- No requirement to introduce Axum, Tower, or another HTTP framework for this change.

## Decisions

1. **Use explicit local route categories before dispatch.**
   - Decision: classify `GET /health` as health, `POST /v1/chat/completions` as inference, `/v0/management/*` as the protected management/status/control namespace, and all other paths as unsupported before invoking route behavior.
   - Rationale: route classification makes authorization decisions testable and prevents future management paths from being accidentally handled as inference or health requests.
   - Alternative considered: match method/path directly in `handle_connection` and add ad hoc checks. That keeps the current implementation small but makes future management route authorization harder to audit.

2. **Model local client authorization separately from provider credentials.**
   - Decision: add `oxmux` primitives for local client credentials, authorization policies, redacted credential metadata, and authorization outcomes without storing or displaying raw secrets.
   - Rationale: local API keys authorize access to the local proxy. Provider API keys authorize upstream provider calls. Mixing them would risk leaking provider credentials through local status surfaces or forwarding local client keys upstream.
   - Alternative considered: reuse provider `AuthState` or provider credential references. That would blur the product boundary and make management snapshots ambiguous.

3. **Prefer standard bearer-token semantics for local clients while keeping the core representation transport-neutral.**
   - Decision: parse `Authorization: Bearer <token>` for the initial HTTP runtime, reject missing, malformed, wrong-scheme, or wrong-token headers deterministically when a route policy requires authorization, but keep public primitives named around local client authorization rather than HTTP-only bearer auth.
   - Rationale: bearer tokens match OpenAI-compatible clients and common Rust proxy examples, while neutral naming leaves room for future Unix-socket, IPC, or desktop-mediated authorization.
   - Alternative considered: custom `x-api-key` only. It is common, but less compatible with OpenAI-style local clients.

4. **Keep `/health` stable and unauthenticated unless a later change explicitly reclassifies it.**
   - Decision: `/health` remains the smoke-test endpoint and does not become a protected management endpoint in this change.
   - Rationale: existing specs and tests rely on `/health` as a low-friction local runtime check. Future richer management/status/control routes can be protected without breaking smoke checks.
   - Alternative considered: protect all non-unsupported routes. That would be stricter, but it would change the established health contract unnecessarily.

5. **Use deterministic local tests instead of real network or provider calls.**
   - Decision: tests should exercise the current local runtime and mock provider execution path, including ensuring local client authorization is not exposed through provider credentials or status output.
   - Rationale: this preserves the headless core boundary and keeps CI independent from secrets, provider SDKs, OAuth, and external services.

6. **Make protection policy states explicit and fail-safe.**
   - Decision: model each protected scope as disabled or required. Disabled means the route does not require local client authorization. Required means a configured local credential must exist and the request must present a matching bearer token; if the credential is missing from configuration, the protected route fails closed with a deterministic unauthorized/configuration outcome rather than allowing access.
   - Rationale: explicit states avoid accidental open access when a maintainer enables protection but omits the credential.
   - Alternative considered: infer defaults from the presence or absence of a token. That is simpler but makes misconfiguration indistinguishable from intentionally disabled protection.

7. **Reserve a deterministic management boundary without creating a remote management API.**
   - Decision: `/v0/management/*` is classified and authorized in this change, but a valid authorized request returns a deterministic placeholder/boundary response unless a later OpenSpec change defines concrete management operations.
   - Rationale: issue #18 needs a testable management authorization boundary now, but the project explicitly does not want a remote management web panel or broad mutable API in this change.
   - Alternative considered: implement a real management status endpoint immediately. That would exceed the issue scope and could create API commitments before management operations are designed.

## Risks / Trade-offs

- **Risk: Local auth tokens could appear in debug output or errors.** → Mitigation: make secret-bearing values non-secret by design where possible, expose redacted metadata only, and add tests for debug/display/status surfaces.
- **Risk: Route categories overfit the current minimal runtime.** → Mitigation: define categories broadly enough for future CLI/IDE management clients while implementing only minimal route behavior now.
- **Risk: Management route tests require a route before real management HTTP APIs exist.** → Mitigation: add a deterministic placeholder management/status/control route classification and authorization response without claiming a full remote management API.
- **Risk: Bearer-only HTTP parsing could limit future clients.** → Mitigation: public primitives remain transport-neutral; bearer parsing is only the current local HTTP adapter behavior.
- **Risk: Required protection with missing configuration could accidentally permit access.** → Mitigation: fail closed and expose a structured configuration/unauthorized outcome without including secret values.

## Migration Plan

This is additive. Existing `/health` behavior and unsupported-path behavior remain stable. Existing `POST /v1/chat/completions` tests should be updated to include the configured valid local client authorization when inference auth is enabled, while tests can also cover disabled authorization for compatibility where useful.

Rollback is straightforward because the change should not introduce external services or persisted migrations: remove the local authorization configuration/primitives, restore the direct route dispatch behavior, and retain existing health/minimal proxy tests.

## Open Questions

- Should a later compatibility change add `x-api-key` parsing in addition to bearer authorization after the initial bearer-only HTTP adapter behavior lands?
