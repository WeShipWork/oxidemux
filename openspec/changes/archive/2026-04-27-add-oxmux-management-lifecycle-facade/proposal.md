## Why

The workspace split established `oxmux` as a reusable core and `oxidemux` as its first app-shell consumer, but the core still exposes only identity and placeholder boundary types. The next useful step is to define the app-facing management, lifecycle, configuration, provider/account, and status contract that lets `oxidemux` consume real core state before GPUI, OAuth, full proxy routing, or provider execution are implemented.

## What Changes

- Add a small `oxmux` management and lifecycle facade for observing core status and expressing proxy lifecycle control intents.
- Define typed status snapshots for proxy lifecycle state, health, degraded/failed conditions, bound endpoint metadata, uptime, and visible warnings/errors.
- Define typed configuration snapshots and update intents for app-visible proxy settings such as listen address, auto-start intent, logging, usage collection, and routing defaults.
- Define provider/account summary types that can represent provider identity, capabilities, auth state, quota/status placeholders, and degraded/error reasons without implementing OAuth or real provider calls.
- Make `oxidemux` demonstrably consume the facade as the first real app-shell client, while still avoiding GPUI, tray, updater, packaging, platform credential storage, and provider execution work.
- Defer concrete HTTP proxy serving, OpenAI/Gemini/Claude/Codex protocol translation, streaming transports, OAuth flows, credential storage implementations, quota analytics, advanced routing/fallback, hot reload, and desktop UI widgets.

## Capabilities

### New Capabilities

- `oxmux-management-lifecycle`: Defines the app-facing `oxmux` management, lifecycle, status, configuration snapshot, provider/account summary, and usage/quota summary facade.

### Modified Capabilities

- `oxmux-core`: Promotes the previously reserved management/status, configuration, provider/auth, usage/quota, and lifecycle boundaries into a minimal typed public facade while keeping the core headless and dependency-light.
- `oxidemux-app-shell`: Requires the app shell to consume the new core management/lifecycle facade instead of owning or duplicating core status and control primitives.

## Impact

- Affects `crates/oxmux/src/management.rs`, `configuration.rs`, `provider.rs`, `usage.rs`, `errors.rs`, and the public facade in `oxmux.rs`.
- Affects `crates/oxidemux/src/main.rs` enough to prove the app shell reads core status or lifecycle information through `oxmux`.
- Adds or updates tests proving direct Rust use of the management/lifecycle facade and app-shell consumption without launching GPUI, opening a window, starting tray/background lifecycle, binding a local proxy server, or reaching external providers.
- Adds no new runtime dependencies unless a small dependency is justified by the design; `oxmux` must remain free of GPUI, app-shell, desktop lifecycle, updater, packaging, and platform credential storage dependencies.
