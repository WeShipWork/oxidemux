## Context

`crates/oxmux/src/protocol.rs` currently exposes only a placeholder `ProtocolBoundary`, while issue #3 needs typed request/response boundaries for OpenAI, Gemini, Claude, Codex, and provider-specific formats. The core crate already owns provider metadata in `provider.rs`, errors in `errors.rs`, and public facade exports in `oxmux.rs`; this change should extend those boundaries without adding network clients or app-shell behavior.

## Goals / Non-Goals

**Goals:**

- Define canonical protocol request and response structures that are deterministic to construct and validate.
- Model provider protocol families as explicit typed metadata, including OpenAI, Gemini, Claude, Codex, and provider-specific variants.
- Introduce translation interface boundaries that can return structured errors or explicit deferred placeholder results.
- Re-export the new core types from the `oxmux` public facade and test them through the crate boundary.

**Non-Goals:**

- No outbound provider HTTP calls, SDK integrations, OAuth, token refresh, or credential storage.
- No full request/response parity for OpenAI, Gemini, Claude, Codex, or provider-specific protocols.
- No GPUI, tray, updater, packaging, app-shell state model, or management endpoint changes.
- No routing algorithm, streaming adapter, or provider selection behavior beyond typed protocol metadata.

## Decisions

1. Keep protocol skeleton types in `crates/oxmux/src/protocol.rs`.
   - Rationale: `protocol.rs` is already the explicit extension point for protocol ownership, so expanding it avoids scattering protocol concerns across provider or routing modules.
   - Alternative considered: create a new protocol subdirectory. Rejected for now because the skeleton is small and project guidance prefers existing files unless a new logical component is large enough to justify splitting.

2. Represent canonical request/response payloads with typed metadata plus opaque body placeholders.
   - Rationale: future translators need stable envelopes, but issue #3 explicitly defers full provider translation and outbound calls.
   - Alternative considered: embed full provider-specific schemas immediately. Rejected because it would exceed the issue scope and create premature parity promises.

3. Add explicit deferred translation results instead of panics or silent no-ops.
   - Rationale: callers should be able to distinguish invalid inputs, unsupported protocol families, and intentionally deferred behavior.
   - Alternative considered: return `Option` for deferred behavior. Rejected because `Option` loses error semantics and makes future UI feedback weaker.

4. Keep provider family metadata aligned with `provider.rs` and facade exports in `oxmux.rs`.
   - Rationale: provider/account summaries already expose protocol family concepts; the new protocol skeleton should be discoverable through the same public core API.

## Risks / Trade-offs

- [Risk] Skeleton types become too generic for future translators. → Mitigation: include typed family, format, direction, and validation metadata while keeping provider payload details opaque.
- [Risk] Deferred results could look like successful translations. → Mitigation: use explicit result variants or structured errors that name the deferred behavior.
- [Risk] Core API expands before implementation is complete. → Mitigation: tests document deterministic construction and validation only; non-goals prevent provider call behavior from entering this change.

## Migration Plan

- Add types and validation in `protocol.rs` without changing existing runtime behavior.
- Re-export new protocol skeleton types from `oxmux.rs`.
- Add tests under `crates/oxmux/tests/` that compile against the public facade and validate deterministic shapes.
- Rollback is file-local: remove new protocol types, exports, tests, and any new `CoreError` variants if needed.

## Open Questions

- None for this skeleton. Full provider schema parity, streaming details, and outbound call execution are intentionally deferred to later changes.
