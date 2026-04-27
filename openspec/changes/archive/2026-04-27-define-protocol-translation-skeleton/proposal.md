## Why

Issue #3 needs oxmux to define stable protocol translation boundaries before proxy request handling grows provider-specific behavior. Establishing typed request/response shapes now keeps the reusable core deterministic and testable while explicitly deferring outbound provider calls and full translation parity.

## What Changes

- Add canonical protocol request and response structures in the `oxmux` core crate for future proxy handling.
- Add explicit typed metadata for provider protocol families covering OpenAI, Gemini, Claude, Codex, and provider-specific formats.
- Add translation interface boundaries that return structured errors or explicit placeholder results when behavior is intentionally deferred.
- Add deterministic tests for constructing and validating protocol request/response shapes.
- Keep provider network calls, OAuth, token refresh, credential storage, app-shell state, GPUI, tray, updater, and packaging out of scope.

## Capabilities

### New Capabilities

- `oxmux-protocol-translation`: Typed protocol request/response boundaries, provider protocol metadata, and deferred translation interfaces for the reusable oxmux core.

### Modified Capabilities

- `oxmux-core`: Public core API surface SHALL expose protocol translation skeleton types without introducing app-shell or outbound provider dependencies.

## Impact

- Affected crate: `crates/oxmux`.
- Likely affected modules: `crates/oxmux/src/protocol.rs`, `crates/oxmux/src/provider.rs`, `crates/oxmux/src/errors.rs`, and `crates/oxmux/src/oxmux.rs`.
- New or updated tests under `crates/oxmux/tests/` validate public API construction, validation, metadata mapping, and deferred translation results.
- No new provider SDK, HTTP client, OAuth, GPUI, or app-shell dependencies are expected.
