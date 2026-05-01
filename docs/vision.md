# OxideMux Vision

OxideMux exists because VibeProxy and zero-limit prove that developers want
subscription-aware local AI proxying, but this project also needs to support
regular provider API tokens, credits, and pay-as-you-go billing with first-class
Linux, macOS, and Windows UX. The product center is provider access UX: users
should understand account status, auth health, quota/rate-limit pressure,
cost/spend pressure, provider availability, routing behavior, and recovery paths
without reading logs or hand-editing proxy state.

OxideMux should feel like a native always-on utility. The tray/menu experience
must have platform-appropriate implementations on Linux, macOS, and Windows, but
those shells adapt shared behavior rather than redefining it.

## Product references

- CLIProxyAPI validates the headless proxy/API boundary: provider auth,
  protocol compatibility, routing, model aliases, fallback, streaming, and
  management endpoints need reusable contracts.
- zero-limit validates the desktop control surface: quota visibility,
  proxy lifecycle controls, tray/background operation, themes, and updates are
  real product requirements.
- VibeProxy validates the subscription-first local proxy UX: auth/session flows,
  model aliasing, request compatibility shims, provider-specific routing, and
  reasoning or thinking-mode request rewriting are core product behavior, not
  optional polish.

OxideMux is an independent Rust implementation of those product lessons with a
clearer shared-core boundary and first-class cross-platform support.

## Reference adoption criteria

Adopt reference-product behavior only when it strengthens the OxideMux product
model without importing the reference implementation shape:

1. The behavior must be expressible as typed Rust semantics in `oxmux` or as a
   native platform-shell concern in `oxidemux`.
2. The OpenSpec change must name which reference validates the behavior and why
   it belongs in the current phase.
3. Core behavior must have deterministic tests that do not require real provider
   credentials, desktop UI, or network calls.
4. Desktop behavior must render or adapt core state instead of reimplementing
   routing, quota, auth, protocol, or rewrite decisions.
5. Electron, Tauri, React, SwiftUI, AppKit, Go services, and upstream code
   structure are non-goals. OxideMux copies product lessons, not code or stack.


## Core product intent

- Provider access UX is the star of the application, not an add-on to a generic
  proxy. Subscription accounts, API-token accounts, credits, quotas, and
  pay-as-you-go spend should all be represented explicitly.
- Local clients should keep using familiar OpenAI-compatible or provider-shaped
  APIs while OxideMux normalizes requests, maps aliases, applies reasoning or
  thinking options, routes across accounts/providers, and returns compatible
  responses.
- Provider availability, access method, subscription state, API-token state,
  credit balance, quota pressure, cost/spend pressure, degraded states, and
  fallback decisions must be represented as typed state that can be tested
  without a desktop shell.
- Auth redirects, cookies, credential references, and provider session health
  must be treated as user-facing product flows because broken auth is broken
  subscription UX.
- `oxmux` should explain what the core is doing through typed state and
  structured outcomes: which account/provider was chosen, why fallback happened,
  what quota, spend, credential, or auth state blocks a route, and what the
  caller or user can do next.
  `oxidemux` should make that experience visual, interactive, and
  platform-native.

## Crate responsibility rule

`oxmux` is the shared headless product engine. It owns provider protocols, auth
semantics, request rewriting, reasoning/thinking compatibility primitives,
model aliases, routing policy, management/status contracts, configuration,
usage/quota/cost state, and testable decision logic.

`oxidemux` is the platform shell. It owns GPUI views, tray/menu integration,
windows, notifications, packaging, updater behavior, platform credential storage
adapters, and OS-specific lifecycle UX.

Use this rule when placing new behavior:

> If behavior must work in CLI/headless tests, put it in `oxmux`. If behavior
> exists only because a desktop OS, GPUI, tray, packaging, or platform storage
> exists, put it in `oxidemux` or an app-owned adapter.

## What future agents must preserve

- Do not reduce VibeProxy or zero-limit to vague inspiration. Treat them as
  validated product references with an unresolved Linux/cross-platform gap.
- Do not move subscription-aware routing, request rewrite, model alias,
  thinking/reasoning, or protocol compatibility behavior into the desktop shell
  just because the UI exposes it.
- Do not start with tray, packaging, themes, or updater work if the core cannot
  yet accept, normalize, route, execute, and return a minimal proxy request.
- Do not make `oxmux` depend on GPUI, tray libraries, provider SDKs, platform
  secret stores, or OAuth UI. The core can define typed semantics and seams;
  shell/platform adapters provide concrete OS integrations.

## Glossary

The older phrase **Subscription UX** is a subset of provider access UX: it covers
subscription-backed access, while OxideMux also supports regular API-token,
credit, quota, and spend-tracked providers.


- **Provider access UX**: the end-to-end experience of using subscription
  accounts, API-token accounts, provider credits, quota-limited plans, and
  pay-as-you-go providers through OxideMux. It includes the core semantics that
  determine and explain account health, auth state, credential state, quota
  pressure, cost/spend pressure, provider availability, routing choices,
  fallback reasons, and recovery actions. `oxmux` must expose this information
  without `oxidemux`; `oxidemux` makes it visual, interactive, and
  platform-native.
- **Provider/account state**: typed core data describing provider availability,
  account identity, access method, auth/session health, credential health, quota
  status, credit or spend status, capabilities, degraded conditions, and
  recovery hints.
- **Model alias**: a user- or provider-facing model name that `oxmux` resolves to
  an explicit provider, account, and provider-native model target before
  execution.
- **Compatibility shim**: deterministic request or response normalization that
  lets OpenAI-compatible or provider-shaped clients keep working while OxideMux
  routes through provider accounts with subscriptions, API tokens, credits,
  quotas, or pay-as-you-go billing.
- **Reasoning/thinking normalization**: deterministic handling of reasoning
  budgets, thinking-mode options, and provider-specific payload conventions so
  routing and execution receive explicit typed intent.
- **Provider/account execution seam**: the trait-based boundary where `oxmux`
  hands a normalized request to a provider/account adapter and receives a
  compatible response, stream, or structured failure.
