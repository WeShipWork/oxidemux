# OxideMux Architecture Intent

OxideMux is split into a shared product engine and a platform shell so that the
subscription proxy behaves the same on Linux, macOS, Windows, and headless test
consumers.

```text
developer tool / local client
  -> local OxideMux endpoint
  -> oxmux protocol + request rewrite pipeline
  -> oxmux model alias + reasoning/thinking normalization
  -> oxmux subscription-aware routing policy
  -> provider/account execution seam
  -> compatible response stream or response body

platform shell
  -> displays auth, quota, routing, health, and lifecycle state
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
- subscription-aware routing inputs, selection outcomes, degraded/exhausted
  states, and structured routing errors;
- provider execution traits, mock harnesses, and account/capability summaries;
- usage/quota summaries, management snapshots, and configuration state.

The core may describe auth/session semantics and credential references, but it
must not own OAuth UI, platform secret-store implementation, GPUI state, tray
libraries, packaging, updaters, or provider SDK dependencies unless an accepted
OpenSpec change explicitly revises the boundary.

## `oxidemux` owns platform behavior

`oxidemux` is the app shell. It owns behavior that exists because users need a
native app on Linux, macOS, and Windows:

- GPUI windows, views, settings, dashboards, themes, and interaction flows;
- tray/menu-bar integration and platform-specific lifecycle policy;
- notifications, start-on-login, background behavior, packaging, and updater UX;
- platform credential storage adapters and browser/OAuth presentation flows;
- user-visible subscription onboarding, restore, auth repair, and account
  selection flows;
- presentation of core state: quota pressure, account health, provider status,
  selected route, fallback reason, and next recovery action.

The shell must not duplicate routing, provider, quota, request rewrite,
thinking/reasoning, protocol, or management decision logic. It supplies inputs
to `oxmux` and renders outputs from `oxmux`.

## Boundary test

Before adding behavior, ask:

1. Does this need to run in headless tests or from a non-GPUI Rust consumer?
   Put the semantics in `oxmux`.
2. Does this require a desktop OS, window, tray/menu, updater, notification, or
   platform credential store? Put the implementation in `oxidemux` or an
   app-owned adapter.
3. Does this affect subscription UX, auth/session behavior, request rewriting,
   model aliases, thinking/reasoning behavior, routing, or crate boundaries?
   Update OpenSpec before implementation.
