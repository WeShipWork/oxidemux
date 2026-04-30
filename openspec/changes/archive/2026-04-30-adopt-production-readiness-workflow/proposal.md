## Why

OxideMux has a mise-first development workflow, but it does not yet expose repository-level security checks, supply-chain policy, or distribution-ready Cargo metadata through that workflow. Issue #35 asks us to adopt the production-readiness patterns proven in `langfuse-rs` while preserving `mise.toml` as the local and CI source of truth.

## What Changes

- Add repository security verification as a first-class mise-defined workflow, including local and GitHub Actions entrypoints.
- Add initial supply-chain policy for `cargo-deny`, covering vulnerability advisories, license allowances, duplicate dependencies, registries, and git sources.
- Add `cargo-audit` as a mise-managed vulnerability check.
- Defer blocking `cargo-vet` adoption until the dependency graph and audit/exemption policy are stable enough to maintain.
- Centralize shared Cargo package metadata in the workspace manifest with `[workspace.package]` and use workspace inheritance where appropriate.
- Add crate/package metadata needed before future distribution, such as repository, readme, documentation, keywords, categories, and package exclusions.
- Add explicit `clippy.toml` and `rustfmt.toml` only where they reduce environment drift and complement existing mise tasks.
- Update contributor documentation so local security checks are run through `mise run ...`, not raw cargo tool invocations.
- Exclude release publishing, release workflow scaffolding, crates.io token handling, and changelog-derived GitHub releases from this change until OxideMux is usable.

## Capabilities

### New Capabilities

<!-- No new product/runtime capabilities. -->

### Modified Capabilities

- `development-workflow`: Extend the repository workflow contract to include mise-defined security checks, supply-chain policy, Cargo metadata readiness, and explicit release automation deferral.

## Impact

- Affects repository workflow files such as `mise.toml`, `.github/workflows/`, `deny.toml`, optional tool config files, `Cargo.toml`, crate manifests, README, and contributor documentation.
- Does not change `oxmux` runtime behavior, `oxidemux` app behavior, public Rust API semantics, provider execution, routing, protocol translation, subscription UX, or GPUI behavior.
- Keeps `mise run ci` as the existing quality gate and adds security verification beside it rather than expanding release automation prematurely.
