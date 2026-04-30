## Context

OxideMux already treats `mise.toml` as the canonical local and CI task graph. The current workflow runs formatting, checking, clippy, tests, rustdoc, hk checks, and OpenSpec PR evidence validation through mise-backed GitHub Actions. Issue #35 expands the same repository workflow layer to cover production-readiness patterns from `langfuse-rs`: security checks, supply-chain policy, and package metadata.

The important constraint is that `langfuse-rs` is a reference, not a template. Its security and release workflows prove the desired shape, but OxideMux must keep tool installation and command selection centralized in mise. Release publishing and release workflow scaffolding are intentionally deferred until OxideMux has usable product behavior and a clear distribution milestone.

## Goals / Non-Goals

**Goals:**

- Add a mise-defined security workflow that can run locally and in GitHub Actions.
- Add initial `cargo-deny` policy and `cargo-audit` vulnerability checks.
- Keep `cargo-vet` evaluated but non-blocking until audits and exemptions are maintainable.
- Centralize shared Cargo package metadata without changing package behavior.
- Prepare crate manifests for future distribution metadata review.
- Add explicit Rust tool behavior configuration only when it reduces drift from mise-managed checks.
- Document the new security workflow for contributors.

**Non-Goals:**

- No release publishing workflow.
- No crates.io publishing, token handling, or release automation.
- No changelog-derived GitHub release creation.
- No changes to `oxmux` runtime semantics, `oxidemux` app-shell behavior, provider execution, routing, protocol translation, subscription UX, or GPUI behavior.
- No blocking `cargo-vet` gate unless a later proposal defines audit ownership and exemption maintenance.

## Decisions

### Decision: Make mise the security command boundary

Security tools should be installed or pinned through `mise.toml` where practical, and GitHub Actions should call mise tasks such as `mise run security`, `mise run audit`, or `mise run deny`. Workflow YAML must not install `cargo-deny` or `cargo-audit` directly with raw cargo commands, and it must not run the underlying cargo security tools directly when a mise task exists.

Alternative considered: copy `langfuse-rs` workflow steps that install cargo tools directly in Actions. Rejected because OxideMux already specifies mise as the workflow source of truth, and duplicated setup would drift from local verification.

### Decision: Add `cargo-deny` and `cargo-audit` first

`cargo-deny` gives an immediate policy surface for licenses, duplicate crates, advisories, registries, and git sources. `cargo-audit` gives a focused vulnerability check that is easy for contributors to understand and run locally.

Alternative considered: start with `cargo-vet` as the primary supply-chain gate. Rejected for this change because `cargo-vet` requires an audit/exemption maintenance model that is premature for the current small dependency graph.

### Decision: Treat `cargo-vet` as deferred evaluation

This change may add documentation or a task placeholder for evaluating `cargo-vet`, but it must not make vetting a required CI gate unless the dependency graph, exemption policy, and ownership model are established.

Alternative considered: add a generated `supply-chain/config.toml` immediately. Rejected unless the implementation can prove it is maintainable and non-blocking; otherwise it creates security theater and future churn.

### Decision: Centralize shared metadata with workspace inheritance

The root workspace manifest should own shared fields such as edition, rust-version, license, repository, homepage, and authors where applicable. Crate manifests should inherit shared metadata and keep crate-specific metadata local.

Alternative considered: leave metadata duplicated in crate manifests. Rejected because duplicated package metadata drifts before distribution and makes future publishing review harder.

### Decision: Defer release automation entirely

Issue #35 originally mentioned release workflow scaffolding, but OxideMux is not usable yet. Release automation would introduce publishing assumptions before crate ownership, binary packaging, versioning, and product readiness are settled.

Alternative considered: add a no-publish release workflow that only runs CI and extracts changelog notes. Rejected for this proposal because even scaffolded release automation invites premature maintenance and should be revisited at a usable-product milestone.

## Risks / Trade-offs

- `cargo-deny` license policy could block acceptable transitive dependencies. Mitigation: start with an explicit, reviewable allow list aligned with current dependencies and document how exceptions are reviewed.
- Duplicate dependency checks can be noisy while dependencies are still evolving. Mitigation: use warning levels where appropriate and reserve hard failures for policy violations that are clearly actionable.
- `cargo-audit` can fail on advisories without available fixes. Mitigation: document how ignored advisories must be justified in policy rather than silently bypassed.
- Workspace metadata inheritance can accidentally change package metadata. Mitigation: verify `cargo metadata`/manifest behavior and keep crate-specific fields local when inheritance would alter package identity.
- Adding tool config files can create another source of truth. Mitigation: only add `clippy.toml` or `rustfmt.toml` when they express stable behavior not already captured by mise commands.

## Migration Plan

1. Add the development-workflow spec updates for security verification, supply-chain policy, metadata readiness, and release automation deferral.
2. Add mise tool pins and security tasks.
3. Add `deny.toml` and wire `cargo-deny`/`cargo-audit` through mise.
4. Add a dedicated GitHub security workflow that sets up mise and runs the mise security task.
5. Centralize workspace package metadata and update crate manifests.
6. Update README/CONTRIBUTING/PR guidance for the new security checks and release deferral.
7. Verify `mise run ci`, `mise run hk-check`, and the new security task.

Rollback is straightforward because this change is repository tooling and metadata only: remove the new workflow/config/tasks and restore manifest metadata if any check creates unacceptable churn.

## Open Questions

- Which exact `cargo-deny` license allow list should be used for the current dependency graph?
- Should duplicate dependencies be warnings or hard failures at first adoption?
- Which mise backend is most reliable for pinning `cargo-deny` and `cargo-audit` in local and CI environments while still keeping workflow YAML limited to mise setup plus `mise run ...` invocations?
