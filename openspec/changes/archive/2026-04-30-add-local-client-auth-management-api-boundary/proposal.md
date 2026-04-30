## Why

The local proxy now has a loopback health endpoint and a minimal OpenAI-compatible inference route, but it does not yet define which local clients are allowed to call inference or future management/status/control routes. This change establishes the `oxmux` security and route-boundary contract before real provider adapters, account controls, and desktop/CLI management clients build on the runtime.

## What Changes

- Add local client authorization primitives to `oxmux` for representing a caller-owned local API key or equivalent authorization requirement without exposing provider credentials.
- Split local runtime route policy into inference and management/status/control categories so each category can require independent authorization decisions.
- Keep `GET /health` available as a stable local smoke endpoint while documenting how protected management/status/control routes differ from unauthenticated health checks.
- Add deterministic authorized and unauthorized request tests for inference and management/status/control paths.
- Preserve loopback-only defaults and avoid Amp-specific URL rewriting, remote management panels, OAuth flows, and provider fallback behavior.

## Capabilities

### New Capabilities

- `oxmux-local-client-auth`: Local client authorization primitives, redaction rules, and request authorization outcomes for loopback clients.

### Modified Capabilities

- `oxmux-core`: The public facade exposes local client authorization and route-boundary primitives as headless `oxmux` core concerns.
- `oxmux-local-proxy-runtime`: The local runtime distinguishes health, inference, and management/status/control route categories and applies configured authorization decisions.
- `oxmux-management-lifecycle`: Management/status/control access is represented separately from inference access so app and future CLI/IDE clients can reason about protected management surfaces.

## Impact

- Affected code: `crates/oxmux/src/local_proxy_runtime.rs`, `crates/oxmux/src/oxmux.rs`, likely a focused `oxmux` auth module, runtime tests, and public API documentation.
- Affected specs: `oxmux-core`, `oxmux-local-proxy-runtime`, `oxmux-management-lifecycle`, plus new `oxmux-local-client-auth` capability.
- No app-shell, GPUI, OAuth UI, platform credential storage, remote management web panel, provider fallback, or provider credential resolution work is included.
