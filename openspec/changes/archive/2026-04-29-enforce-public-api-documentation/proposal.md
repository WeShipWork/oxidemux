## Why

OxideMux exposes a growing `oxmux` public facade for headless proxy semantics and an `oxidemux` app-shell binary for contributor-facing platform integration, but CI does not currently fail when new public Rust APIs lack documentation. Issue #34 closes that quality gap now, before the workspace surface grows further and before downstream Rust consumers or contributors rely on undocumented routing, provider, protocol, configuration, management, usage, app-shell, and error contracts.

## What Changes

- Enable Rust public API documentation enforcement at all workspace crate roots, including the `oxmux` library facade and the `oxidemux` app-shell binary, so diagnostics apply through their module trees and CI fails on warning regressions.
- Backfill `//!` crate/module documentation and `///` public item documentation in every Rust source file that exposes public workspace API.
- Document public modules, structs, enums, variants, traits, functions, methods, associated functions, constants, fields, and facade re-exports with factual consumer-facing or contributor-facing semantics.
- Explicitly handle generated or intentionally internal-only surfaces with the smallest practical `#[allow(missing_docs)]` scopes, an in-source reason, and an audit entry, rather than silently bypassing enforcement.
- Preserve the existing `oxmux`/`oxidemux` boundary in documentation: `oxmux` docs describe headless core semantics, while `oxidemux` docs describe app-shell/platform adapter responsibilities for contributors without implying that proxy, routing, provider, quota, protocol, or management semantics moved into the binary crate.
- Keep the change limited to documentation enforcement and documentation backfill; do not add runtime behavior, dependencies, telemetry integration, provider SDK integration, or app-shell UI work.

## Capabilities

### New Capabilities

- `rust-public-api-documentation`: Defines workspace expectations for documented Rust public APIs, missing-docs enforcement, explicit internal/generated exceptions, and verification through `mise run ci` plus warnings-as-errors generated docs.

### Modified Capabilities

- None.

## Impact

- Affected code: all Rust source files that expose public workspace API, including `crates/oxmux/src/oxmux.rs`, public `oxmux` modules under `crates/oxmux/src/`, and `crates/oxidemux/src/main.rs` for app-shell crate documentation. Public `oxidemux` items are contributor-facing app-shell integration surfaces, not downstream ownership of `oxmux` core semantics.
- Affected specs: new `rust-public-api-documentation` capability only; existing proxy/runtime/provider/routing/protocol semantics are clarified in documentation but not changed.
- Public API impact: no behavioral API changes are intended, but public items gain consumer-facing documentation and undocumented public additions become CI-visible through `missing_docs` plus warnings-as-errors checks.
- Dependency impact: no new runtime dependencies; the implementation should not add GPUI, app-shell, provider SDK, OAuth UI, platform secret-store, telemetry, or documentation-generation dependencies to `oxmux`.
- Verification impact: `mise run ci`, workspace cargo checks, clippy with `-D warnings`, tests, formatting, and `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps` must pass after documentation enforcement is enabled. `mise run ci` must include the rustdoc generation task so local and CI verification share the same quality gate.
