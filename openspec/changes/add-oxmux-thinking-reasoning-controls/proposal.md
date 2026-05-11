## Why

Issue #27 needs `oxmux` to normalize provider-neutral thinking and reasoning intent before provider adapters, model aliases, routing, and app-shell controls depend on provider-specific payload conventions. Existing protocol, routing, model registry, and provider metadata establish where requests are parsed and selected, but there is not yet a typed core contract for reasoning effort, token budgets, alias-derived thinking modes, or unsupported-capability outcomes.

Without this contract, future provider adapters or `oxidemux` UI would be tempted to hard-code Anthropic/OpenAI/Gemini-style rewrite behavior in the wrong layer. The core needs deterministic, testable reasoning intent semantics that can be carried through request normalization and compatibility reporting without real provider calls.

## What Changes

- Add provider-neutral `oxmux` thinking/reasoning budget controls for explicit request metadata and model-alias conventions.
- Define typed reasoning intent, budget/effort validation, source metadata, compatibility outcomes, and unsupported/degraded handling before provider-specific translators apply payload rewrites.
- Define how protocol translators and provider execution boundaries receive normalized reasoning intent and compatibility outcomes as typed metadata rather than by parsing provider-specific payload fragments.
- Define deterministic typed alias metadata attached to in-memory alias definitions that can derive reasoning intent from configured aliases without suffix/bracket/free-form model-name parsing in this change and without moving model alias parsing into `oxidemux` or provider adapters.
- Preserve scope boundaries: no broad payload rewrite DSL, provider-specific beta headers, real provider network calls, provider SDK integration, suffix/bracket/free-form alias parsing, persisted TOML reasoning configuration, GPUI controls, app-shell rewrite logic, or UI configuration in this change.

## Reference Product Evidence

- VibeProxy validates subscription-first local proxy UX where auth, routing, model aliases, and extended thinking behavior are visible product concerns rather than hidden adapter details.
- CLIProxyAPI validates compatibility behavior for model-name thinking conventions and provider-specific rewrite seams; OxideMux adopts the product need while deferring suffix/bracket/free-form parsing and concrete provider rewrites.
- zero-limit validates user-visible provider/model/quota status as a desktop lifecycle concern that should consume typed core state instead of redefining core routing or reasoning semantics.
- Supporting technical context from Pipelex reinforces the separation between request-level reasoning parameters and provider support checks; it is not treated as a product reference for OxideMux.

## Capabilities

### New Capabilities

- `oxmux-reasoning-controls`: Defines provider-neutral thinking/reasoning intent, budget and effort validation, alias-derived reasoning conventions, capability compatibility outcomes, and structured unsupported/degraded behavior for the reusable core.

### Modified Capabilities

- `oxmux-core`: Public core API surface SHALL expose reasoning/thinking control primitives without introducing app-shell, provider SDK, outbound network, OAuth, credential storage, or UI dependencies.
- `oxmux-protocol-translation`: Protocol request boundaries SHALL carry normalized reasoning/thinking intent as typed metadata instead of requiring provider-specific payload rewrites in routing.
- `oxmux-routing-policy`: Routing and alias primitives SHALL supply model alias context that can derive reasoning intent without making route selection responsible for provider-specific rewrite behavior.
- `oxmux-model-registry`: Model registry metadata SHALL remain consumable by future reasoning controls for alias and capability checks without duplicating model alias parsing in app-shell code.
- `oxmux-provider-execution`: Provider execution request primitives SHALL be able to receive normalized reasoning/thinking intent and unsupported-capability metadata without performing real provider network calls in this change.

## Impact

- Affected crate: `crates/oxmux`.
- Affected specs: new `oxmux-reasoning-controls` plus focused deltas to `oxmux-core`, `oxmux-protocol-translation`, `oxmux-routing-policy`, `oxmux-model-registry`, and `oxmux-provider-execution`.
- Affected tests: new deterministic reasoning-control tests plus targeted protocol/routing/provider-execution tests for explicit typed metadata, typed alias-derived metadata, validation failures, capability compatibility, explicit-over-alias precedence, unknown capability, and structured unsupported outcomes.
- Deferred work: concrete provider-specific payload rewrites, beta headers, real provider adapter behavior, extraction from OpenAI/Gemini/Claude HTTP JSON payload fields, suffix/bracket/free-form alias parsing, persisted TOML reasoning configuration, management snapshot surfacing, UI controls, settings UX, and live provider/network validation.
