## Why

`oxmux` needs a provider execution boundary that can be tested without real credentials, provider SDKs, OAuth flows, network calls, or platform secret storage. Issue #4 can proceed now that the protocol translation skeleton from issue #3 is complete, because provider execution can reuse the canonical protocol request and response shapes instead of inventing provider wire formats.

## What Changes

- Add provider execution trait contracts to the headless `oxmux` core so callers can execute an explicitly selected provider/account boundary in deterministic tests.
- Add an in-repo mock provider harness with explicit success, degraded, quota-limited, streaming-capable, and failed outcomes.
- Reuse existing `oxmux` protocol, provider/account summary, management health, usage, and quota types so provider state is not copied into app-shell-specific models.
- Keep default tests credentialless and networkless while proving provider execution, summary reflection, and structured error behavior through the public `oxmux` facade.
- Preserve scope boundaries: no real provider SDKs, HTTP clients, OAuth browser flows, token refresh, raw secret storage, routing policy implementation, GPUI, or `oxidemux` app-shell dependency.

## Capabilities

### New Capabilities

- `oxmux-provider-execution`: Defines the headless provider execution trait, deterministic mock provider harness, mock outcome taxonomy, and provider/account summary reflection behavior.

### Modified Capabilities

- `oxmux-core`: Expose provider execution primitives through the public core facade while keeping `oxmux` dependency-light and app-shell-free.
- `oxmux-management-lifecycle`: Allow management snapshots and provider/account summaries to reflect mock provider health, quota, degraded, and failed states without duplicating app-shell state.
- `oxmux-protocol-translation`: Clarify that provider execution consumes existing canonical protocol request and response envelopes while protocol translation behavior remains deferred.

## Impact

- Affected core files: `crates/oxmux/src/provider.rs`, `crates/oxmux/src/errors.rs`, and `crates/oxmux/src/oxmux.rs`.
- Affected tests: new or updated `crates/oxmux/tests/` coverage for deterministic mock execution, provider/account summary reflection, facade exports, and dependency boundaries.
- Affected specs: new `oxmux-provider-execution` capability plus deltas for core facade, management lifecycle, and protocol translation contracts.
- Dependencies: no new runtime, UI, provider SDK, OAuth, HTTP, keyring, secret-service, GPUI, or app-shell dependencies.
