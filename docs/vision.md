# OxideMux Vision

OxideMux exists because VibeProxy and zero-limit prove that developers want
subscription-aware local AI proxying, but they do not provide the cross-platform
Linux, macOS, and Windows experience this project needs. The product center is
subscription UX: users should understand account status, auth health,
quota/rate-limit pressure, provider availability, routing behavior, and recovery
paths without reading logs or hand-editing proxy state.

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

## Core product intent

- Subscription UX is the star of the application, not an add-on to a generic
  proxy.
- Local clients should keep using familiar OpenAI-compatible or provider-shaped
  APIs while OxideMux normalizes requests, maps aliases, applies reasoning or
  thinking options, routes across accounts/providers, and returns compatible
  responses.
- Provider availability, subscription state, quota pressure, degraded states,
  and fallback decisions must be represented as typed state that can be tested
  without a desktop shell.
- Auth redirects, cookies, credential references, and provider session health
  must be treated as user-facing product flows because broken auth is broken
  subscription UX.
- Desktop UI should explain what the core is doing: which account/provider was
  chosen, why fallback happened, what quota/auth state blocks a route, and what
  the user can do next.

## Crate responsibility rule

`oxmux` is the shared headless product engine. It owns provider protocols, auth
semantics, request rewriting, reasoning/thinking compatibility primitives,
model aliases, routing policy, management/status contracts, configuration,
usage/quota state, and testable decision logic.

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
