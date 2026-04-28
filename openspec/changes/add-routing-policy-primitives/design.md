## Context

`oxmux` already owns the headless core boundary, provider execution mocks, protocol metadata, configuration snapshots, management summaries, and a placeholder `RoutingBoundary`. Issue #5 moves routing from placeholder ownership to typed policy primitives so later configuration, proxy execution, credential selection, quota-aware failover, model listing, and reasoning-budget work can depend on stable routing contracts.

The implementation must remain inside `crates/oxmux` and use deterministic typed inputs. Provider integrations, quota fetching, concrete proxy routing, GPUI, `oxidemux`, credential storage, and app-shell lifecycle code remain outside this change.

## Goals / Non-Goals

**Goals:**

- Define Rust-native routing policy types for model aliases, priority order, fallback behavior, explicit provider/account targeting, and provider/account availability.
- Provide a deterministic selection function that returns typed success or structured `CoreError` routing failures.
- Represent degraded and exhausted provider/account states from caller-supplied inputs without fetching live quota or health data.
- Export the routing policy primitives from the `oxmux` public facade for direct Rust consumers.
- Cover routing behavior with networkless tests in the `oxmux` crate.

**Non-Goals:**

- No outbound provider calls, provider SDKs, HTTP clients, OAuth, token refresh, or credential storage.
- No payload rewrite DSL, cloaking/obfuscation, prompt mutation, or agent-client-specific routing tables.
- No app-shell, GPUI, tray, updater, or desktop lifecycle integration.
- No automatic quota polling or live health discovery; exhausted/degraded state is explicit input supplied by mocks, configuration, or future management layers.

## Decisions

1. **Keep policy and runtime state separate.**
   - Decision: `RoutingPolicy` describes aliases, priority candidates, fallback behavior, and optional explicit targets; a separate availability snapshot describes provider/account state for a selection attempt.
   - Rationale: Policies should be reusable and serializable later, while availability can change per request without mutating the policy.
   - Alternative considered: storing availability directly on policy entries. That would blur configuration with runtime state and make deterministic tests less precise.

2. **Return an auditable selection outcome instead of only a provider/account pair.**
   - Decision: selection returns a typed outcome with selected provider, optional account, requested model, resolved model, decision mode, and skipped candidate reasons.
   - Rationale: Future UI and management endpoints need explainability without parsing logs or display strings.
   - Alternative considered: returning only the chosen target. That would be simpler but would hide fallback and degraded/exhausted reasoning from callers.

3. **Use structured `CoreError` variants for terminal routing failures.**
   - Decision: add routing-specific structured failure data to `CoreError` for invalid policy, missing target, no route, exhausted candidates, and degraded-only candidates when degraded fallback is disallowed.
   - Rationale: Downstream code must match errors directly and present meaningful feedback without string parsing.
   - Alternative considered: defining a routing-only error type and converting later. That fragments the core error boundary and makes app-shell propagation inconsistent.

4. **Model aliases resolve before candidate selection.**
   - Decision: the requested model is first resolved through alias rules, then provider/account candidates are evaluated in deterministic priority order.
   - Rationale: Alias behavior must be testable independently and should not vary by provider integration state.
   - Alternative considered: provider-specific alias resolution during execution. That would defer core routing decisions to provider implementations and weaken the headless core contract.

5. **Fallback policy is explicit.**
   - Decision: routing only tries lower-priority candidates when fallback is enabled for the policy or selection request; degraded candidates are selectable only when explicitly allowed.
   - Rationale: This prevents surprising provider/account changes and keeps degraded service behavior visible to users.
   - Alternative considered: always fall back to any non-exhausted candidate. That maximizes availability but can violate user intent and account targeting expectations.

## Risks / Trade-offs

- [Risk] Routing primitives overfit early mock scenarios before real provider integrations exist. → Mitigation: keep types small, headless, and focused on selection inputs/results rather than provider transport details.
- [Risk] Explicit degraded handling adds more states for callers. → Mitigation: return skipped-candidate metadata and structured failures so callers can explain why no healthy route was selected.
- [Risk] Public API may need refinement when file-backed configuration arrives. → Mitigation: isolate policy data models in `routing` and export through the facade so future configuration loading can construct the same types.
- [Risk] Core facade growth could accidentally pull app dependencies. → Mitigation: keep implementation in `crates/oxmux`, avoid new non-core dependencies, and extend dependency-boundary tests.

## Migration Plan

1. Add routing policy, target, availability, outcome, and failure primitives in `crates/oxmux/src/routing.rs`.
2. Export new routing types from `crates/oxmux/src/oxmux.rs`.
3. Add routing-specific `CoreError` support in `crates/oxmux/src/errors.rs`.
4. Add deterministic `crates/oxmux/tests/routing_policy.rs` coverage for the issue acceptance criteria.
5. Run `cargo fmt`, `cargo test -p oxmux`, and workspace CI tasks to confirm the core remains dependency-light.

Rollback is straightforward before downstream issues depend on these types: remove the new routing exports, error variants, and tests while leaving the placeholder `RoutingBoundary` intact.

## Open Questions

- None for this proposal. Later provider integration work can refine how live provider health populates the explicit availability inputs without changing the core selection contract.
