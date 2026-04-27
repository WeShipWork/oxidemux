## Context

Issue #4 asks for provider execution traits and an in-repo mock provider harness so `oxmux` can test provider execution boundaries without real credentials, network-backed providers, OAuth, or platform secret storage. The blocker, issue #3, is complete: `oxmux` now owns canonical protocol request/response envelopes, typed protocol metadata, and deferred translation results.

The current `oxmux` core already has the pieces this change should compose:

- `provider.rs` owns provider/account summaries, protocol family metadata, auth state metadata, quota state links, and degraded reasons.
- `protocol.rs` owns canonical request/response envelopes and the `ProtocolTranslator` trait pattern.
- `management.rs` owns management snapshots and core health states that app consumers can display without duplicating core state.
- `usage.rs` owns usage and quota value types.
- `errors.rs` owns structured `CoreError` variants exposed to app and library consumers.

This change should introduce the provider execution boundary without changing the project layering: `oxmux` remains a headless core crate, while real desktop UI, platform credential storage, OAuth, and provider SDK integration remain out of scope.

## Goals / Non-Goals

**Goals:**

- Define deterministic provider execution trait contracts in `oxmux` for explicitly selected provider/account boundaries.
- Provide an in-repo mock provider harness that can return success, degraded, quota-limited, streaming-capable, and failed outcomes.
- Reuse existing protocol, provider/account summary, management health, usage, quota, and error types.
- Expose provider execution primitives through the public `oxmux` facade for direct Rust consumers.
- Prove behavior with default tests that do not require credentials, network access, provider SDKs, OAuth, platform secret storage, GPUI, or `oxidemux`.

**Non-Goals:**

- No real provider HTTP calls, SDKs, OAuth browser flows, token refresh, keychain, secret-service, or raw secret storage.
- No routing policy, failover, account rotation, priority selection, model aliasing, or automatic provider selection beyond selecting one mock provider/account for a test.
- No streaming transport implementation; streaming-capable mocks only prove capability metadata and boundary shape.
- No protocol translation implementation; provider execution consumes canonical envelopes that already exist.
- No GPUI, app-shell state, desktop lifecycle, tray, updater, packaging, local management auth, or `oxidemux` dependency.

## Decisions

1. Keep provider execution primitives in `crates/oxmux/src/provider.rs`.
   - Rationale: provider identity, capabilities, account summaries, auth state, quota state, and degraded reasons already live there, so the execution trait can reuse those models directly.
   - Alternative considered: create a new provider execution module. Rejected for the initial boundary because the domain is still small and project guidance prefers existing files unless a new logical component clearly needs separation.

2. Model provider execution with explicit request/result types instead of passing raw strings or provider SDK types.
   - Rationale: deterministic tests need typed provider ID, optional account ID, canonical request, and structured outcome metadata. Raw strings would duplicate validation and make future app display weaker.
   - Alternative considered: expose only a closure-based mock API. Rejected because issue #4 asks for provider execution traits and downstream consumers need a stable trait boundary.

3. Use canonical protocol request/response envelopes as execution payloads.
   - Rationale: issue #3 established these shapes specifically for future proxy handling, and using them prevents provider execution from inventing wire-format-specific request models.
   - Alternative considered: add provider-specific request structs now. Rejected because full provider schema parity and translation are deferred.

4. Represent mock outcomes explicitly.
   - Rationale: tests must distinguish success, degraded, quota-limited, streaming-capable, and failed behavior without inferring state from ad hoc messages.
   - Alternative considered: use `Result<CanonicalProtocolResponse, CoreError>` alone. Rejected because degraded and streaming-capable outcomes are successful boundary states with extra provider/account metadata, not just errors.

5. Surface provider health through existing summaries and management types.
   - Rationale: `ProviderSummary`, `AccountSummary`, `QuotaState`, `DegradedReason`, `CoreHealthState`, and `ManagementSnapshot` already define app-visible state. Reusing them avoids app-shell copies and keeps UI concerns outside `oxmux`.
   - Alternative considered: create mock-only summary structs. Rejected because mock-only state would not exercise the production-facing management boundary.

6. Keep provider execution synchronous and dependency-free for this change.
   - Rationale: the mock harness is deterministic and in-memory. Introducing async traits, HTTP clients, or runtimes now would imply transport behavior that issue #4 explicitly excludes.
   - Alternative considered: define an async executor trait immediately for future real providers. Rejected for now because real network-backed execution belongs to later proxy engine/provider integration changes.

## Risks / Trade-offs

- [Risk] Synchronous traits may need adaptation when real network-backed providers arrive. → Mitigation: keep the request/result model transport-agnostic so a later async or runtime-specific adapter can wrap it without changing mock semantics.
- [Risk] Mock harness could grow into a routing engine. → Mitigation: require explicit provider/account selection and forbid priority, failover, rotation, and model alias behavior in this change.
- [Risk] Streaming-capable mocks may be mistaken for streaming transport support. → Mitigation: specify that streaming-capable only sets capability metadata and returns deterministic canonical responses.
- [Risk] Provider failures could become string-only errors. → Mitigation: add structured provider execution error data in `CoreError` and tests that match error variants rather than message text.
- [Risk] Summary reflection could duplicate existing management models. → Mitigation: require mocks to produce `ProviderSummary` and `AccountSummary` values that can be placed into `ManagementSnapshot` directly.

## Migration Plan

- Add provider execution request/result/outcome types, trait, and mock harness in `provider.rs`.
- Add any provider execution-specific `CoreError` variant needed for structured failures.
- Re-export new provider execution primitives from `oxmux.rs`.
- Add tests under `crates/oxmux/tests/` for deterministic mock outcomes, provider/account summary reflection, facade usability, and dependency isolation.
- Rollback is file-local: remove the provider execution types, mock harness, exports, tests, and new error variant if the boundary needs to be redesigned.

## Open Questions

- None for this initial boundary. Real provider transports, async execution, routing policy, credential access, streaming adapters, and quota refresh behavior are intentionally deferred to later issues.
