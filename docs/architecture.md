# OxideMux Architecture Intent

OxideMux is split into a shared product engine and a platform shell so that
provider access behavior works the same for subscription accounts, API-token
accounts, credit/quota-limited accounts, pay-as-you-go providers, Linux, macOS,
Windows, and headless test consumers.

```text
developer tool / local client
  -> local OxideMux endpoint
  -> oxmux protocol + request rewrite pipeline
  -> oxmux model alias + reasoning/thinking normalization
  -> oxmux provider access-aware routing policy
  -> provider/account execution seam
  -> compatible response stream or response body

platform shell
  -> displays auth, quota, cost/spend, routing, health, and lifecycle state
  -> supplies platform credential adapters and user choices
  -> starts/stops/backgrounds the shared runtime
```

## `oxmux` owns shared behavior

`oxmux` is the reusable Rust core. It owns behavior that must be deterministic,
testable, and available outside the desktop app:

- local proxy runtime contracts and lifecycle handles;
- protocol families and request/response translation boundaries;
- request rewrite primitives, compatibility shims, and provider-specific
  payload semantics;
- reasoning/thinking-mode normalization and token/budget semantics;
- model aliases and explicit provider/account targets;
- provider access-aware routing inputs, selection outcomes,
  degraded/exhausted/over-budget states, and structured routing errors;
- provider execution traits, mock harnesses, and account/capability summaries;
- usage/quota/cost summaries, management snapshots, and configuration state.

The core must define typed, dependency-free interfaces for auth/session
semantics and credential references. Implementations that require platform UI,
secret stores, browser/OAuth presentation, or provider SDKs must live in
`oxidemux` or app-owned adapter crates unless an accepted OpenSpec change
explicitly revises the boundary.

## `oxidemux` owns platform behavior

`oxidemux` is the app shell. It owns behavior that exists because users need a
native app on Linux, macOS, and Windows:

- GPUI windows, views, settings, dashboards, themes, and interaction flows;
- tray/menu-bar integration and platform-specific lifecycle policy;
- notifications, start-on-login, background behavior, packaging, and updater UX;
- platform credential storage adapters and browser/OAuth presentation flows;
- user-visible subscription onboarding, restore, auth repair, and account
  selection flows;
- presentation of core state: quota pressure, cost/spend pressure, account
  health, credential health, provider status, selected route, fallback reason,
  and next recovery action.

The shell must not duplicate routing, provider, quota, request rewrite,
thinking/reasoning, protocol, or management decision logic. It supplies user and
platform inputs to `oxmux`, invokes `oxmux` control surfaces, and renders outputs
from `oxmux`.

## Boundary test

Before adding behavior, ask:

1. Does this need to run in headless tests or from a non-GPUI Rust consumer?
   Put the semantics in `oxmux`.
2. Does this require a desktop OS, window, tray/menu, updater, notification, or
   platform credential store? Put the implementation in `oxidemux` or an
   app-owned adapter.
3. Does this affect provider access UX, auth/session behavior, credential
   behavior, cost/spend behavior, request rewriting, model aliases,
   thinking/reasoning behavior, routing, or crate boundaries?
   Update OpenSpec before implementation.

## Core state contract

`oxmux` owns the provider access UX semantics that callers need even without
the native app. Subscription accounts, API-token accounts, provider credits,
quota-limited plans, and pay-as-you-go providers must all flow through the same
typed core model. `oxmux` must expose enough snapshots, structured events, and
outcomes for headless tools to inspect and control provider state directly,
while `oxidemux` turns the same contracts into a visual, interactive,
platform-native control surface. Core contracts must expose, at minimum:

- proxy lifecycle and bound endpoint state;
- provider and account health, capability, degraded, and unavailable states;
- auth/session status, API-token credential-reference health, and access method
  metadata without raw secrets;
- usage, quota, credit, and cost/spend summaries with warning, exhausted, or
  over-budget states;
- routing decisions that include selected provider/account/model, skipped
  candidates, fallback reasons, budget/cost rationale, and recovery actions;
- compatibility outcomes for protocol translation, model alias resolution, and
  reasoning/thinking normalization;
- structured errors that distinguish invalid configuration, missing credentials,
  exhausted quota, depleted credits, over-budget spend, unavailable providers,
  and provider execution failures.

## Roadmap sequencing

The project should stay core-first until `oxmux` can accept, normalize, route,
execute, and return a minimal local proxy request with observable provider
access state. Current roadmap phases should be interpreted as:

| Phase | Primary owner | Purpose | Blocks |
| --- | --- | --- | --- |
| Phase 2 - Core proxy contracts | `oxmux` | protocol, model registry, thinking/reasoning, streaming, WebSocket and management contracts | provider execution and app status UX |
| Phase 3 - Provider state, quota, and spend | `oxmux` | credential boundary, account monitoring, quota/cost events, real provider adapter, quota- and spend-aware failover | settings, dashboards, recovery UX |
| Phase 4 - Desktop shell | `oxidemux` | GPUI status, settings, onboarding, provider configuration views | depends on stable core snapshots |
| Phase 5 - Desktop lifecycle and distribution | `oxidemux` | tray/background lifecycle, launch at login, analytics surfaces, packaging, updates | depends on usable runtime and shell state |

Desktop work may spike GPUI feasibility early, but production UX should not
encode routing, quota, auth, or rewrite semantics that are not already available
through `oxmux` contracts.

## Testing expectations

Changes to provider access UX, auth/session behavior, credential behavior,
cost/spend behavior, request rewriting, model aliases, reasoning/thinking
behavior, routing, provider selection, or crate boundaries require OpenSpec
coverage before implementation. Core behavior must include deterministic tests
for normal, quota-pressure, cost/spend-pressure, degraded/unavailable, and
provider-failure cases. App-shell behavior must include tests or documented
manual checks proving it renders core state and does not duplicate core decision
logic.

## Secrets, privacy, and adapters

`oxmux` may carry credential references, access-method metadata, auth state,
usage estimates, cost/spend summaries, and redacted diagnostics, but it must
never persist raw platform secrets or require a platform secret-store crate.

`oxidemux` and app-owned adapters own keychain, Secret Service, Windows
Credential Manager, browser/OAuth presentation, notifications, updater,
packaging, and OS lifecycle integrations. Telemetry or analytics must be based
on scrubbed typed events and must not include raw API keys, cookies, bearer
tokens, or provider session material.

## Enforcement and review

Every PR that touches `crates/oxmux`, provider access UX, auth/session
behavior, credential behavior, cost/spend behavior, request rewriting, model
aliases, reasoning/thinking behavior, routing, provider selection, or crate
boundaries must link the relevant OpenSpec change or explain why the change is
documentation-only. Reviews should verify:

- `oxmux` has no dependency on GPUI, gpui-component, tray libraries, updater
  libraries, packaging tools, platform credential storage libraries, provider
  SDKs, OAuth UI libraries, Tauri, Electron, or the `oxidemux` app crate;
- `oxidemux` consumes `oxmux` snapshots, events, traits, and errors rather than
  creating parallel routing, quota, cost/spend, provider, protocol, or rewrite
  models;
- reference-product features are adopted as product behavior with attribution in
  OpenSpec, not copied code, stack choices, or file structure;
- `mise run ci` and relevant targeted tests cover changed contracts.
