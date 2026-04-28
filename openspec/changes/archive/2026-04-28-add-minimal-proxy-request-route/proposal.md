## Why

`oxmux` has separate health-runtime, protocol, routing, provider-execution, and response primitives, but it cannot yet accept a model request and prove those boundaries work together through a local proxy path. Issue #15 adds the first minimal headless proxy engine slice so future provider and desktop work can build on a tested core request flow instead of app-shell-only assumptions.

## What Changes

- Add a loopback-only `oxmux` proxy request route suitable for OpenAI-compatible chat-completion smoke tests.
- Compose the route through canonical protocol request construction, caller-supplied routing policy and availability inputs, selected provider/account propagation, provider execution, error handling, and deterministic response serialization.
- Use deterministic in-repo mock providers for success and failure coverage without real provider network calls.
- Extend local runtime request handling beyond `/health` only enough to support a bounded `POST /v1/chat/completions` smoke route and deterministic unsupported-path responses.
- Add `oxmux` tests for success, invalid request, provider failure, fallback/degraded routing failures, and unsupported path behavior without external provider network calls.
- Keep the engine headless and independent of GPUI, tray/menu integration, updater, packaging, OAuth UI, token refresh, raw credential storage, provider SDKs, and `oxidemux` app-shell lifecycle.

## Capabilities

### New Capabilities

- `oxmux-minimal-proxy-engine`: Minimal headless proxy request path that accepts an OpenAI-compatible chat-completion smoke request, routes it through core protocol/routing/provider seams, and serializes deterministic responses or structured errors.

### Modified Capabilities

- `oxmux-core`: Public core requirements expand from owning separate protocol, routing, provider, response, and local runtime primitives to exposing an end-to-end minimal proxy engine path through the headless facade.
- `oxmux-local-proxy-runtime`: Runtime requirements expand from health-only loopback smoke testing to include a bounded local proxy request route while preserving loopback-only and dependency-boundary constraints.

## Impact

- Affected crate: `crates/oxmux` only.
- Affected modules: local runtime request handling, protocol request/response helpers, routing/provider engine coordination, core errors, response serialization, and public facade exports.
- Affected tests: new or expanded `oxmux` integration tests for the minimal proxy route and engine path.
- API impact: new public Rust primitives may be added for the minimal proxy engine request/response seam so tests and future shells can exercise the route without starting `oxidemux`.
- Dependency impact: no GPUI, app-shell, tray, updater, packaging, OAuth UI, credential storage, provider SDK, or real upstream provider dependency. Any JSON parsing dependency must remain isolated to core request/response codec behavior and justified by smoke-route validation needs.
