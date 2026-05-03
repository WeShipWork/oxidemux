## Context

`oxmux` already owns typed protocol metadata, provider/account summaries, routing policy aliases, file-backed provider declarations, streaming capability metadata, management snapshots, and a minimal proxy route. Issue #26 asks for a model registry and listing contract so clients and the future `oxidemux` shell can discover configured models, aliases, disabled models, routing eligibility, and model metadata without duplicating these semantics in UI code.

The current model-related data is spread across `crates/oxmux/src/routing.rs`, `crates/oxmux/src/configuration/file.rs`, `crates/oxmux/src/provider.rs`, and `crates/oxmux/src/protocol.rs`. The new contract should assemble those inputs into a deterministic headless catalog while preserving the existing split: routing selects a target for a request, provider execution talks to selected providers, and the registry lists what the configured core can offer.

## Goals / Non-Goals

**Goals:**

- Define Rust-native model registry and model listing types for configured models, provider-native model targets, aliases, forks, provider/account applicability, routing eligibility, streaming capability, and disabled/degraded state.
- Build the first registry from deterministic `oxmux` inputs: validated file configuration, routing policy model routes, provider declarations, provider/account summaries, and explicit capability metadata.
- Preserve both user-facing names and provider-native model targets so future `/v1/models`, routing explanations, reasoning controls, and app-shell pickers can explain alias/fork behavior.
- Keep registry construction and tests networkless, provider-SDK-free, GPUI-free, OAuth-free, and independent from platform credential storage.
- Document the future `/v1/models` route as a serialization consumer of the typed registry, not as a separate source of model semantics.

**Non-Goals:**

- No live provider model discovery, provider-specific scraping, remote model updater, background refresh task, or network call.
- No real provider adapter behavior or provider SDK integration.
- No GPUI model picker, app-shell model registry, desktop settings UX, tray/menu behavior, updater, or platform credential storage.
- No broad request rewrite or reasoning/thinking control semantics; Issue #27 owns reasoning controls and may consume registry capability metadata later.
- No full HTTP `/v1/models` route unless a later task explicitly scopes it as thin serialization over the typed registry.

## Decisions

1. Add a focused `model_registry` core module instead of extending routing or provider files.
   - Rationale: registry entries combine routing, provider, protocol, and configuration concepts. Keeping them in a focused module avoids turning `routing.rs` or `provider.rs` into god-files and leaves route selection/provider execution responsibilities clear.
   - Alternative considered: add registry structs to `routing.rs`. Rejected because listing is catalog behavior, not route selection.
   - Alternative considered: add registry structs to `provider.rs`. Rejected because aliases and configured route groups are not provider execution state.

2. Treat the registry as configured catalog data, not live provider inventory.
   - Rationale: Issue #26 explicitly excludes remote updaters, provider scraping, and real provider calls. A deterministic catalog lets tests prove behavior and gives future UI/API surfaces stable data without requiring credentials or network availability.
   - Alternative considered: query providers during listing. Rejected because it would entangle model listing with auth/session, quota, credentials, and network failures before those phases are ready.

3. Represent aliases and forks explicitly instead of flattening them into model IDs.
   - Rationale: provider access UX needs to explain requested names, alias resolution, forked provider/account targets, and provider-native models. Flattening would hide why a model appears multiple times or why a route is disabled/degraded.
   - Alternative considered: expose only OpenAI-style `id` strings. Rejected because `/v1/models` compatibility is only one consumer; core and app-shell consumers need typed metadata.

4. Keep routing selection authoritative while the registry reports routing eligibility.
   - Rationale: existing `RoutingPolicy` already owns alias resolution, explicit provider/account targets, fallback behavior, and degraded/exhausted selection results. The registry should surface whether an entry is eligible or disabled/degraded, but actual per-request selection remains in routing.
   - Alternative considered: make registry perform route selection. Rejected because it would duplicate and drift from `oxmux-routing-policy`.

5. Define future `/v1/models` output as a projection of registry entries.
   - Rationale: OpenAI-compatible clients expect a model listing shape, but core typed data should remain the source of truth. A future route can serialize visible/routable entries without introducing endpoint-only semantics.
   - Alternative considered: implement `/v1/models` first and infer types from JSON. Rejected because it would bias core semantics toward one protocol family and make non-OpenAI providers harder to represent.

6. Treat OpenAI model listing as the minimum compatibility projection, not the full registry model.
   - Rationale: OpenAI-style listings are intentionally thin, while Anthropic and Gemini expose richer model metadata such as display names, token limits, capabilities, supported generation methods, and version stability. `oxmux` should keep richer provider-neutral metadata typed locally so future OpenAI, Claude, Gemini, and provider-specific projections can each serialize what they support.
   - Alternative considered: limit registry entries to OpenAI-style `id` and ownership fields. Rejected because it would erase provider access UX metadata needed for aliases, forks, disabled/degraded state, streaming support, and future reasoning capability checks.

## Risks / Trade-offs

- [Risk] Registry entries become too broad before real providers exist. → Mitigation: keep the first implementation deterministic and limited to configured/provider-supplied metadata already represented in core types.
- [Risk] Alias/fork terminology overlaps with routing policy language. → Mitigation: specs require preserving requested alias, resolved model, and provider-native model target separately, while route selection stays in routing.
- [Risk] Future `/v1/models` consumers expect live provider data. → Mitigation: document listing as configured catalog data and include unknown/unavailable states rather than pretending live discovery happened.
- [Risk] Capability flags drift from provider execution metadata. → Mitigation: derive streaming/protocol/auth/routing capability from provider declarations and summaries, and cover the mapping with tests.
- [Risk] Adding a new module increases facade surface. → Mitigation: keep types small, document public API, and verify dependency-boundary tests still exclude `oxidemux`, GPUI, provider SDKs, OAuth, and network clients.

## Migration Plan

- Add specs and tests before implementation.
- Implement model registry types and deterministic builders inside `crates/oxmux` without changing existing routing/proxy behavior.
- Export registry primitives through `crates/oxmux/src/oxmux.rs` once tests cover direct Rust consumer usage.
- Leave existing configuration and routing behavior compatible; no runtime migration or data migration is required.
- If implementation reveals a non-trivial `/v1/models` route requirement, defer route wiring to a follow-up OpenSpec change or a clearly scoped later task in this change.

## Open Questions

- Exact public type names should be chosen during implementation to align with existing `oxmux` naming style.
- The first implementation should decide whether disabled/degraded model state is represented as a single enum or as separate eligibility and reason fields, based on the smallest testable Rust API.
