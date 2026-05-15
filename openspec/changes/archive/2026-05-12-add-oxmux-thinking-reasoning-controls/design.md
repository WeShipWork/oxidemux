## Context

`oxmux` already exposes typed protocol envelopes, model aliases, routing policy decisions, provider execution requests, provider/account capability summaries, model registry metadata, and structured errors. Issue #27 adds the missing reasoning/thinking normalization layer between local client request interpretation and provider execution so reference-product behavior such as thinking-mode aliases and reasoning budgets can be represented without putting provider-specific rewrite logic in `oxidemux`.

The current core has no first-class place to distinguish an explicit client reasoning request from a model alias convention, no provider-neutral representation for effort or token budget, and no structured outcome for providers that cannot honor the requested behavior. This change should add those semantics as deterministic headless primitives before concrete provider adapters or app-shell controls exist.

## Goals / Non-Goals

**Goals:**

- Define provider-neutral reasoning/thinking intent types with source metadata, effort level, optional token budget, mode, and validation outcomes.
- Normalize explicit typed Rust request metadata and narrow typed alias-derived conventions into typed `oxmux` metadata before route execution.
- Expose compatibility outcomes that distinguish supported, ignored, degraded, and unsupported reasoning behavior without silently mutating payloads.
- Carry normalized reasoning metadata through protocol/provider execution boundaries so future translators can apply provider-specific payload rewrites at the correct seam.
- Keep all tests deterministic, networkless, provider-SDK-free, GPUI-free, OAuth-free, and independent from platform credential storage.

**Non-Goals:**

- No broad payload rewrite DSL or arbitrary JSON transformation system.
- No provider-specific beta headers, SDK payload generation, Anthropic/OpenAI/Gemini concrete rewrite implementation, or live provider calls.
- No extraction of reasoning controls from OpenAI-compatible JSON, Claude payloads, Gemini payloads, or HTTP route bodies; explicit request metadata is supplied through typed Rust core inputs in this change.
- No GPUI controls, settings UX, desktop picker behavior, tray/menu behavior, or app-shell-owned reasoning model.
- No route-selection changes beyond carrying/inspecting normalized metadata and reporting compatibility outcomes.
- No real provider capability discovery, credential validation, quota checks, or account network probes.

## Decisions

1. Add a focused `reasoning` core module rather than extending `protocol.rs`, `routing.rs`, or `provider.rs` with all reasoning types.
   - Rationale: reasoning controls intersect protocol translation, aliases, provider capabilities, and execution metadata. A dedicated module keeps existing files focused and avoids turning protocol or routing into god-files.
   - Alternative considered: put reasoning structs in `protocol.rs`. Rejected because alias parsing and provider capability compatibility are broader than protocol envelopes.
   - Alternative considered: put reasoning structs in `routing.rs`. Rejected because route selection should not own provider-specific request semantics.

2. Represent reasoning intent separately from compatibility outcome.
   - Rationale: callers need to know what was requested and what `oxmux` decided can happen. `ReasoningIntent` can remain stable while `ReasoningCompatibilityOutcome` records supported, ignored, degraded, or unsupported behavior.
   - Capability metadata declares target support, unknown state, and limits; compatibility outcomes record what happened for one selected request and may include supported, ignored, degraded, unsupported, or unknown outcomes.
   - Alternative considered: store only a final provider rewrite plan. Rejected because provider-specific rewrites are out of scope and would obscure unsupported/ignored semantics.

3. Treat explicit and alias-derived reasoning sources differently for unsupported providers.
   - Rationale: explicit client metadata is user intent and should fail visibly when strict or unsupported. Alias/default-derived intent may be compatibility sugar and can be ignored with metadata when configured permissively.
   - Proposed default: explicit unsupported intent returns a structured error unless a permissive unsupported policy is supplied; alias/default-derived unsupported intent may produce ignored-capability metadata.
   - Alternative considered: always ignore unsupported reasoning. Rejected because silent downgrades hide broken provider access UX.
   - Alternative considered: always error on unsupported reasoning. Rejected because alias compatibility could make otherwise usable models fail unexpectedly.

4. Use typed alias metadata as the first alias-derived reasoning convention.
   - Rationale: CLIProxyAPI validates model-name budget conventions, while codex-proxy and Pipelex validate structured configuration/capability metadata. For OxideMux, typed alias metadata best preserves requested and resolved model identity without ambiguous string parsing or a broad rewrite DSL.
   - For this change, typed alias metadata is attached to in-memory alias definitions and consumed by core normalization APIs. Persisted TOML alias reasoning configuration is deferred.
   - Suffix or bracket parsing is deferred to a later change unless it scopes exact patterns, precedence, validation, and ordinary-alias ambiguity cases.
   - Alternative considered: parse every provider-specific model naming convention. Rejected because it would become provider scraping/rewrite behavior before adapters are scoped.

5. Let explicit request reasoning controls override alias-derived controls.
   - Rationale: explicit request metadata represents the current caller's intent. Alias metadata is compatibility/defaulting sugar. If both sources are present and differ, `oxmux` preserves the explicit intent and records typed ignored-alias diagnostic metadata rather than silently merging controls.
   - Matching explicit and alias-derived controls may be coalesced into a single normalized intent while preserving source diagnostics.
   - Alternative considered: fail any explicit/alias conflict structurally. Rejected because it makes configured aliases unexpectedly brittle for clients that already send explicit reasoning controls.
   - Alternative considered: let alias metadata win. Rejected because it hides explicit caller intent.

6. Treat provider-neutral effort and provider-neutral token budget as mutually exclusive controls.
   - Rationale: Gemini rejects simultaneous level and budget controls for some model families, Pipelex enforces mutual exclusivity, and OpenAI-style effort controls are not the same abstraction as Anthropic/Gemini token budgets. A single normalized intent should contain either effort or budget, not both.
   - The core token-budget range for this change is `1..=200000`; absence means no explicit token budget. `0` is invalid in the provider-neutral budget type so disabling reasoning is represented by mode/effort policy, not by overloading budget.
   - Provider-specific ranges, dynamic budget sentinels, and provider-specific disable values remain adapter compatibility concerns for later translator changes.

7. Represent unknown reasoning capability as a first-class compatibility outcome.
   - Rationale: static configuration and early registry entries may not know whether a provider/account/model supports reasoning. Strict explicit intent should fail visibly on unknown support, while permissive alias/default intent may produce ignored unknown-capability metadata.
   - Alternative considered: always fail unknown. Rejected because it would make static or partially configured providers unusable until every capability is known.
   - Alternative considered: always ignore unknown. Rejected because it would silently downgrade explicit caller intent.

8. Keep persisted TOML reasoning configuration out of this change.
   - Rationale: issue #27 is about core typed semantics before provider rewrites and UI controls. In-memory typed alias metadata and capability metadata are enough to satisfy the core contract without expanding the file configuration schema prematurely.
   - File-backed reasoning alias/capability configuration requires a later `oxmux-file-configuration` OpenSpec delta if persisted user settings are needed.

9. Carry reasoning metadata through canonical protocol and provider execution boundaries as typed core data.
   - Rationale: future protocol translators need access to normalized intent, but provider execution should not parse opaque JSON payloads or model aliases itself. Metadata propagation keeps responsibilities clear.
   - Canonical protocol request metadata carries normalized reasoning intent and diagnostics independently from the opaque payload. Routing preserves requested/resolved model and selected target metadata without evaluating provider-specific rewrites. Compatibility evaluation consumes the already-selected target and produces a typed compatibility outcome. Provider execution request metadata carries the intent and outcome, and provider execution result metadata may echo the final outcome for diagnostics and future management surfacing.
   - Alternative considered: embed reasoning only in `ProtocolPayloadBody::Json`. Rejected because it requires payload parsing and hides compatibility outcomes from routing/provider summaries.

10. Model provider reasoning support as capability metadata owned by `oxmux` summaries and/or registry entries.

- Rationale: compatibility checks need to know whether a provider/account/model target supports reasoning, token budgets, effort levels, or thinking mode. This should be static/deterministic in this change and can later be fed by real adapters.
- Alternative considered: wait for live provider adapters to decide. Rejected because UI and routing need a stable core contract first.

11. Keep reasoning compatibility advisory to the selected route in this change.

- Rationale: provider access-aware routing already has deterministic target selection semantics. Reasoning compatibility should explain whether the selected target can honor the normalized intent, but it should not introduce hidden fallback behavior before routing policy explicitly supports capability-aware selection.
- If a lower-priority candidate supports reasoning but the selected candidate does not, this change evaluates compatibility against the selected candidate and returns supported, ignored, degraded, unsupported, or unknown outcome according to handling policy.
- Alternative considered: make reasoning support influence fallback immediately. Rejected because it would mix route selection and request compatibility without a routing-policy OpenSpec that defines priority, quota, degraded state, and capability trade-offs.

12. Preserve model identity layers in reasoning diagnostics.

- Rationale: provider access UX needs to explain what the caller requested, what alias resolved to, and which provider/account/model target was selected. Reasoning diagnostics should carry requested model alias, resolved model identifier, selected provider/account target, provider-native model identifier when known, reasoning source, and typed failure/outcome data without requiring display-string parsing.
- Alternative considered: only record the resolved model. Rejected because it loses alias-derived source context and makes future UI/recovery explanations weaker.

13. Defer management snapshot fields while preserving a source for future surfacing.

- Rationale: issue #27 is about core normalization and compatibility contracts. Provider execution metadata should expose ignored, degraded, unsupported, and unknown reasoning outcomes so a later `oxmux-management-lifecycle` change can surface warnings and recovery hints without re-deriving compatibility state.
- Alternative considered: add management snapshot fields in this change. Rejected to keep the scope focused and avoid designing UI/status policy before the core outcome model is implemented.

## Risks / Trade-offs

- [Risk] Reasoning controls become too provider-specific too early. → Mitigation: use provider-neutral intent/outcome types and defer concrete payload rewrites to later adapter changes.
- [Risk] Alias parsing becomes ambiguous with ordinary model names. → Mitigation: keep conventions narrow, documented, and test-backed; preserve requested and resolved names separately.
- [Risk] Unsupported defaults surprise clients. → Mitigation: distinguish explicit versus alias/default sources and expose structured ignored/unsupported metadata.
- [Risk] Capability metadata duplicates model registry/provider metadata. → Mitigation: define reasoning support as reusable core capability data and derive registry/provider surfaces from the same types where possible.
- [Risk] New public API surface grows too large. → Mitigation: keep types focused, export only stable primitives, and add rustdoc-covered direct-use tests.

## Migration Plan

- Add OpenSpec requirements and deterministic tests before implementation.
- Implement a focused `reasoning` module with validation and compatibility outcome primitives.
- Wire metadata into existing protocol/provider execution request shapes with backwards-compatible constructors or additive builder methods.
- Export reasoning primitives through the `oxmux` facade once direct Rust consumer tests pass.
- Leave existing routing, minimal proxy, and provider mock behavior compatible when no reasoning intent is supplied.
- Rollback is straightforward before archive: remove the additive module/metadata and tests; no persisted data migration is required.

## Open Questions

- Exact public type names should be chosen during implementation to align with existing `oxmux` naming style.
