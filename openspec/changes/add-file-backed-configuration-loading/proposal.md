## Why

`oxmux` needs a deterministic, file-backed configuration contract before the app shell, management surfaces, and later layered reload work can depend on user-owned proxy settings. Issue #7 makes configuration loading the next core boundary because provider references, routing defaults, observability settings, and auto-start intent must be validated and visible through typed state instead of hand-wired test values.

## What Changes

- Add a headless `oxmux` file-configuration capability for loading a deterministic local configuration file.
- Choose TOML as the first supported on-disk format because it is deterministic, readable for local developer configuration, and maps cleanly to typed Rust structures.
- Validate loopback-only listen address, port, provider references, routing defaults, opaque credential references, logging settings, usage collection settings, and auto-start intent before producing runtime configuration.
- Surface invalid configuration as structured `CoreError` data with field path, invalid value category, and stable reason codes suitable for app-shell display without string parsing.
- Update management snapshots so loaded configuration can populate app-visible proxy settings and structured configuration warnings/errors.
- Define hot-reload hook points and replacement semantics while explicitly deferring default path discovery, full watcher behavior, layered config merging, GPUI settings UI, remote config storage, and secret persistence.

## Capabilities

### New Capabilities

- `oxmux-file-configuration`: Deterministic local configuration file loading, validation, runtime configuration construction, reload hook points, and structured configuration errors for the headless `oxmux` core.

### Modified Capabilities

- `oxmux-core`: Expose the file-backed configuration loader and validation types through the public core facade while preserving the headless dependency boundary.
- `oxmux-management-lifecycle`: Allow management snapshots to reflect loaded configuration summaries, configuration source metadata, and validation failures.
- `oxmux-routing-policy`: Require file-loaded routing defaults and provider references to validate against typed routing policy/provider identity primitives before runtime use.

## Impact

- Affected code: `crates/oxmux` configuration, error, routing, and management modules; public facade exports; deterministic core tests.
- Affected specs: new `oxmux-file-configuration` spec plus deltas for `oxmux-core`, `oxmux-management-lifecycle`, and `oxmux-routing-policy`.
- Dependencies: may add lightweight Rust parsing/serialization support for TOML if not already present; must not add GPUI, app-shell, provider SDK, OAuth UI, platform secret-store, watcher, remote storage, or database dependencies. File-loaded credential references remain opaque non-secret pointers and must not expose raw credentials through snapshots or errors.
- Downstream issues: unblocks layered configuration/hot reload foundation, management API boundary, model registry/listing, and future settings UX by establishing typed config contracts first.
