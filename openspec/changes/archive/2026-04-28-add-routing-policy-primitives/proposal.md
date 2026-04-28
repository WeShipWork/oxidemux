## Why

Provider selection needs typed, deterministic routing policy primitives before real provider integrations land so downstream app and UI layers can rely on matchable outcomes instead of parsing strings or embedding routing rules in app-shell code.

## What Changes

- Add headless `oxmux` routing policy configuration for model aliases, priority order, fallback behavior, explicit provider/account targeting, and provider/account availability states.
- Expose typed routing selection results that identify the selected provider, account, resolved model, routing decision path, and skipped candidates.
- Surface routing failures through structured `CoreError` values for no-match, exhausted, degraded-only, missing target, and invalid policy cases.
- Add deterministic networkless tests covering model aliasing, priority order, fallback, explicit account targeting, and exhausted/degraded providers.
- Keep routing policy behavior independent of GPUI, `oxidemux`, real provider calls, quota fetching, payload rewrite DSLs, cloaking/obfuscation, and agent-client-specific routing tables.

## Capabilities

### New Capabilities

- `oxmux-routing-policy`: Typed routing policy configuration, provider/account availability inputs, selection outcomes, and routing failure contracts for deterministic headless provider selection.

### Modified Capabilities

- `oxmux-core`: Public facade and core error requirements expand from reserving routing ownership to exposing typed routing policy primitives and structured routing failures.

## Impact

- Affected crate: `crates/oxmux` only.
- Affected modules: `routing`, `errors`, public facade exports in `oxmux.rs`, and deterministic `oxmux` tests.
- API impact: new public Rust types for routing policy configuration, provider/account targets, availability state, selection results, skipped candidates, and routing failure details.
- Dependency impact: no new provider SDK, HTTP, OAuth, GPUI, app-shell, credential storage, or external service dependencies.
