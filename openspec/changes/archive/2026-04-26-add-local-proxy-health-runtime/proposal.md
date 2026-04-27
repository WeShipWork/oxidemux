## Why

`oxmux` now exposes typed management and lifecycle primitives, but there is no user-testable runtime behavior behind them. Adding a minimal localhost health server creates the first deterministic proxy runtime slice that can be smoke-tested without GPUI, provider integrations, OAuth, routing, or platform credential storage.

## What Changes

- Add an `oxmux` local proxy runtime that starts a minimal HTTP listener from deterministic configuration.
- Expose a stable health endpoint, such as `GET /health`, for smoke tests and app-shell status checks.
- Connect runtime startup, bind failure, shutdown, and health handling to typed lifecycle states: starting, running, failed, and stopped.
- Keep runtime configuration local and deterministic, including loopback listen address and port selection.
- Add tests for successful bind, bind failure, lifecycle status reporting, health response stability, and shutdown.
- Defer provider clients, OAuth, protocol translation, request routing, streaming, quota fetches, GPUI, tray/background lifecycle, updater, packaging, and platform credential storage.

## Capabilities

### New Capabilities

- `oxmux-local-proxy-runtime`: Defines the minimal headless runtime that binds a local HTTP listener, serves a stable health endpoint, reports lifecycle state, and shuts down cleanly without external providers or desktop dependencies.

### Modified Capabilities

- `oxmux-core`: Evolves the headless core from inert lifecycle descriptions to the first deterministic runtime behavior while keeping `oxmux` dependency-light and free of app-shell, GPUI, tray, updater, packaging, and credential-storage dependencies.
- `oxmux-management-lifecycle`: Requires lifecycle snapshots to reflect real local runtime transitions and bind errors instead of only directly constructed in-memory states.
- `oxidemux-app-shell`: Allows the app shell to exercise or report the new core runtime health path through `oxmux` without owning duplicate proxy runtime primitives.

## Impact

- Affects `crates/oxmux` runtime, configuration, lifecycle, error, and public facade code.
- Affects `crates/oxmux` tests for listener startup, bind failure, health response shape, lifecycle transitions, and shutdown behavior.
- May affect `crates/oxidemux` only enough to consume or smoke-check the core runtime facade while preserving the existing minimal app-shell boundary.
- Adds a minimal local HTTP serving approach only if justified by the design; no dependency may pull GPUI, desktop lifecycle, provider auth, platform storage, updater, or packaging concerns into `oxmux`.
