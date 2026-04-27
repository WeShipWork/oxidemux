## 1. Runtime API and Configuration

- [x] 1.1 Add an `oxmux` local health runtime configuration type that validates loopback listen address and port values without accepting public interface binds.
- [x] 1.2 Add structured `CoreError` coverage for local runtime configuration, bind failure, health serving, and shutdown failures.
- [x] 1.3 Define the public runtime handle/API needed to start, inspect, and shut down the local health runtime from Rust code without launching `oxidemux`.
- [x] 1.4 Re-export the minimal runtime configuration, runtime handle, and runtime status types through `crates/oxmux/src/oxmux.rs`.

## 2. Local Health Listener

- [x] 2.1 Implement listener startup so a valid loopback configuration binds a local HTTP endpoint and records the actual bound socket address.
- [x] 2.2 Implement `GET /health` with a stable success status and stable response content suitable for smoke testing.
- [x] 2.3 Implement deterministic handling for unsupported paths so they do not report a healthy smoke-test result.
- [x] 2.4 Implement explicit shutdown that stops accepting requests, releases the bound endpoint, and does not leave detached runtime work alive.

## 3. Lifecycle and Management Integration

- [x] 3.1 Wire runtime startup to typed lifecycle transitions from starting to running with bound endpoint metadata.
- [x] 3.2 Wire bind and runtime startup failures to a failed lifecycle state with inspectable structured error data.
- [x] 3.3 Wire shutdown to stopped lifecycle status and ensure status remains queryable after shutdown.
- [x] 3.4 Update management snapshot construction or runtime status accessors so app and library consumers can observe local health runtime status through existing facade concepts.

## 4. App Shell Boundary

- [x] 4.1 Add or update `oxidemux` app-shell integration tests only as needed to prove the app consumes the runtime through `oxmux` rather than owning listener, lifecycle, or health-response primitives.
- [x] 4.2 Preserve the existing minimal `oxidemux` bootstrap behavior unless a small smoke path is required by the implementation design.
- [x] 4.3 Confirm GPUI, tray/background lifecycle, updater, packaging, OAuth UI, provider SDKs, and platform credential storage remain outside `oxmux`.

## 5. Tests

- [x] 5.1 Add `oxmux` tests for successful loopback bind using deterministic configuration and inspection of the actual bound endpoint.
- [x] 5.2 Add `oxmux` tests for `GET /health` response status and stable response content.
- [x] 5.3 Add `oxmux` tests for unsupported path behavior.
- [x] 5.4 Add `oxmux` tests for bind failure using an unavailable or invalid endpoint and verify structured failed lifecycle status.
- [x] 5.5 Add `oxmux` tests for lifecycle status reporting across starting, running, failed, stopped, and shutdown paths.
- [x] 5.6 Add `oxmux` tests proving shutdown releases the listener and completes without external providers, OAuth, routing, GPUI, or platform credential storage.

## 6. Verification

- [x] 6.1 Confirm `crates/oxmux/Cargo.toml` has no `oxidemux`, GPUI, gpui-component, tray, updater, packaging, OAuth UI, provider SDK, or platform credential storage dependencies.
- [x] 6.2 Run `cargo fmt --all -- --check` and fix formatting issues.
- [x] 6.3 Run `cargo clippy --all-targets --all-features -- -D warnings` and fix lint issues.
- [x] 6.4 Run `cargo check --all-targets --all-features` and fix compile issues.
- [x] 6.5 Run `cargo test --all-targets --all-features` and fix test failures.
- [x] 6.6 Run `mise run ci` to verify the repository task wrapper still matches CI expectations.
