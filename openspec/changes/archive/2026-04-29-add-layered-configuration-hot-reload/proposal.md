## Why

Issue #17 needs the next configuration step after single-file loading: `oxmux` can validate one user-owned TOML file today, but consumers still need a deterministic way to combine bundled defaults with user-owned overrides, publish one runtime view, and ignore reload events that do not change effective configuration.

Layered configuration belongs in the headless core because routing, provider declarations, management snapshots, and reload decisions must behave the same for desktop, CLI, and embedded Rust consumers before `oxidemux` adds platform-specific file watching or settings UI.

## What Changes

- Extend file-backed configuration from whole-document replacement into a layered configuration model with bundled defaults as the lowest-precedence layer and user-owned configuration as the highest-precedence layer.
- Define a merged runtime configuration view that validates before publish and continues to preserve the last valid active configuration on failed replacements.
- Add deterministic change fingerprinting so equivalent layer inputs produce an `Unchanged` reload outcome and do not trigger spurious reload notifications.
- Define reload hook outcomes for callers to distinguish unchanged, replaced, and rejected candidate configurations without requiring `oxmux` to own a filesystem watcher.
- Preserve user-owned provider/account declarations during config updates by defining collection merge rules instead of replacing whole provider declarations with bundled defaults.
- Surface layered source metadata, active fingerprint, reload outcome, and failed-load diagnostics through core state and management-visible snapshots without implying verified auth, quota, subscription health, provider availability, or credential usability.
- Keep cloud/database configuration backends, full GPUI settings editor behavior, provider OAuth credential persistence, remote model registry updates, and platform file watcher ownership out of scope.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `oxmux-file-configuration`: Adds layered configuration inputs, merge precedence, fingerprinting, and deterministic reload hook outcomes on top of existing TOML load/validate/replace semantics.
- `oxmux-core`: Exposes layered configuration and reload outcome types through the public headless facade while preserving the dependency boundary from `oxidemux` and desktop-specific services.
- `oxmux-management-lifecycle`: Extends management-visible configuration state with layered source metadata, active fingerprint, reload outcome, and failed replacement diagnostics.

## Impact

- Affected code: `crates/oxmux/src/configuration/file.rs`, `crates/oxmux/src/configuration/raw.rs`, `crates/oxmux/src/configuration/validation.rs`, `crates/oxmux/src/management.rs`, `crates/oxmux/src/oxmux.rs`, and deterministic `crates/oxmux/tests/file_configuration.rs` coverage.
- Affected specs: `oxmux-file-configuration`, `oxmux-core`, and `oxmux-management-lifecycle`.
- Public API impact: new headless configuration layer descriptors, fingerprint/reload outcome types, layered replacement hooks, and management snapshot metadata. Existing single-file loading and replacement APIs remain supported.
- Dependency impact: may add a lightweight hashing crate only if the standard library is insufficient for deterministic fingerprints; must not add GPUI, app-shell, watcher, provider SDK, OAuth UI, platform secret-store, remote storage, or database dependencies to `oxmux`.
- Downstream impact: `oxidemux` and future CLI/embedded consumers can pass already-read layer contents into `oxmux`, observe unchanged/replaced/rejected outcomes, and implement watcher/UI behavior outside the core.
