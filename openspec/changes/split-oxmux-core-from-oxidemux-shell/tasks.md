## 1. Workspace Structure

- [x] 1.1 Convert the root `Cargo.toml` into a workspace manifest with `crates/oxmux` and `crates/oxidemux` members.
- [x] 1.2 Move the existing binary package metadata into `crates/oxidemux/Cargo.toml` while preserving package name, version, edition, rust-version, license, and description intent.
- [x] 1.3 Create `crates/oxmux/Cargo.toml` as a library package with Rust 1.95.0, edition 2024, dual-license metadata, and an explicit `[lib]` path.
- [x] 1.4 Remove or relocate the old root `src/main.rs` layout so source files live under the appropriate workspace member and no `mod.rs` paths are introduced.

## 2. `oxmux` Core Crate

- [x] 2.1 Add a minimal `oxmux` library root that exposes a small public facade for crate metadata or core identity without implementing provider, routing, protocol, streaming, or proxy runtime behavior.
- [x] 2.2 Add placeholder domain modules or public types only where needed to establish future ownership of provider/auth, routing, protocol translation, configuration, management/status, usage/quota, and error boundaries.
- [x] 2.3 Ensure `oxmux` has no GPUI, gpui-component, tray, updater, packaging, platform credential storage, or `oxidemux` dependencies.
- [x] 2.4 Add or update tests proving `oxmux` can be used directly by Rust code without launching the app binary, opening a window, starting IPC, or binding a local proxy server.

## 3. `oxidemux` App Shell

- [x] 3.1 Add the `oxidemux` binary entrypoint under `crates/oxidemux` and preserve the existing bootstrap behavior or equivalent package metadata output.
- [x] 3.2 Make `oxidemux` depend on `oxmux` through a workspace path dependency.
- [x] 3.3 Route shared metadata or core identity data through `oxmux` where appropriate so the app shell is demonstrably consuming the core crate.
- [x] 3.4 Keep GPUI, tray/background lifecycle, updater, packaging, IDE adapter, and platform credential storage implementation work out of this change.

## 4. Tests and Documentation

- [x] 4.1 Update `tests/bootstrap.rs` or replace it with workspace-appropriate tests that validate both `oxmux` and `oxidemux` package expectations.
- [x] 4.2 Add a dependency-boundary test, metadata assertion, or manifest check that helps prevent `oxmux` from depending on `oxidemux` or UI/platform crates.
- [x] 4.3 Update `README.md` to describe the workspace layout, current bootstrap status, `oxmux` versus `oxidemux` responsibilities, and deferred product capabilities.
- [x] 4.4 Update any development instructions that reference root-package paths so they remain accurate after the workspace split.

## 5. Tooling and Verification

- [x] 5.1 Confirm `mise.toml` tasks still run workspace-wide formatting, checking, clippy, and tests after the split.
- [x] 5.2 Confirm `.github/workflows/ci.yml` still runs formatting, clippy, check, and tests across the full workspace on Linux, macOS, and Windows.
- [x] 5.3 Run `cargo fmt --all -- --check` and fix formatting issues.
- [x] 5.4 Run `cargo clippy --all-targets --all-features -- -D warnings` and fix lint issues.
- [x] 5.5 Run `cargo check --all-targets --all-features` and fix compile issues.
- [x] 5.6 Run `cargo test --all-targets --all-features` and fix test failures.
- [x] 5.7 Run `mise run ci` to verify the repository task wrapper matches CI expectations.
