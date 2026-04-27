## 1. Public Facade Shape

- [x] 1.1 Define typed proxy lifecycle state and lifecycle control intent values in `oxmux` without binding sockets, spawning background runtime work, or calling providers.
- [x] 1.2 Define a management snapshot type that includes core identity, lifecycle state, health/degraded state, optional bound endpoint metadata, uptime metadata, warnings, and structured errors.
- [x] 1.3 Define structured core error variants for management snapshot, lifecycle intent, configuration validation, provider/account summary, and usage/quota summary failures.
- [x] 1.4 Re-export the minimal management/lifecycle facade from `crates/oxmux/src/oxmux.rs` without exposing app-shell-only implementation details.

## 2. Configuration, Provider, and Usage Types

- [x] 2.1 Define app-visible configuration snapshot and update intent types for listen address, port, auto-start intent, logging, usage collection, and routing defaults.
- [x] 2.2 Add validation for configuration update intents that returns structured `CoreError` values for invalid listen address, port, routing default, or provider reference data.
- [x] 2.3 Define provider summary, provider capability, protocol family, account summary, auth state, quota state, last-checked metadata, and degraded/error reason types.
- [x] 2.4 Define usage and quota summary types that distinguish zero, unknown, unavailable, and degraded states without requiring analytics persistence or network-backed quota fetching.

## 3. App Shell Consumption

- [x] 3.1 Update the `oxidemux` binary or app-shell integration code to read core management/status data from `oxmux` while preserving the existing bootstrap metadata behavior or equivalent output.
- [x] 3.2 Ensure `oxidemux` does not define duplicate proxy lifecycle, configuration, provider/account, usage/quota, or core error primitives when consuming the facade.
- [x] 3.3 Keep GPUI, tray/background lifecycle, updater, packaging, IDE adapter, OAuth UI, platform credential storage, and desktop secret-store implementation work out of this change.

## 4. Tests and Documentation

- [x] 4.1 Add direct `oxmux` tests that construct realistic management snapshots from deterministic in-memory values without launching `oxidemux`, binding a local proxy server, opening a window, or calling external providers.
- [x] 4.2 Add tests for lifecycle states and control intents proving they are inert descriptions in this change rather than runtime proxy operations.
- [x] 4.3 Add tests for configuration validation success and failure paths, including structured errors for invalid values.
- [x] 4.4 Add tests for provider/account summaries and usage/quota summaries, including unknown, unavailable, and degraded states.
- [x] 4.5 Add or update `oxidemux` tests proving the app shell consumes `oxmux` management/status facade data.
- [x] 4.6 Update README or developer-facing documentation only if public bootstrap behavior or workspace capability descriptions change.

## 5. Verification

- [x] 5.1 Confirm `oxmux` still has no GPUI, gpui-component, tray, updater, packaging, platform credential storage, or `oxidemux` dependencies.
- [x] 5.2 Run `cargo fmt --all -- --check` and fix formatting issues.
- [x] 5.3 Run `cargo clippy --all-targets --all-features -- -D warnings` and fix lint issues.
- [x] 5.4 Run `cargo check --all-targets --all-features` and fix compile issues.
- [x] 5.5 Run `cargo test --all-targets --all-features` and fix test failures.
- [x] 5.6 Run `mise run ci` to verify the repository task wrapper still matches CI expectations.
