# development-workflow Specification

## Purpose
Define the repository development workflow contract for local verification, CI parity, OpenSpec evidence, pull request hygiene, and release-note expectations.
## Requirements
### Requirement: Mise defines the repository verification contract
The repository SHALL use mise-defined tasks as the canonical way to run development verification locally and in CI.

#### Scenario: Local verification uses mise
- **WHEN** a contributor reads the development workflow documentation
- **THEN** it tells them to install/trust tools with mise and run repository verification through `mise run ci`

#### Scenario: CI invokes mise verification
- **WHEN** GitHub Actions runs the standard repository quality checks
- **THEN** the workflow invokes the mise-defined CI task instead of maintaining an independent duplicate list of raw cargo verification commands

#### Scenario: Tool version pins stay visible
- **WHEN** maintainers update Rust or workflow tool versions
- **THEN** the contributor documentation identifies `mise.toml` as the workflow tool source of truth and keeps the Rust pin aligned with `rust-toolchain.toml`

### Requirement: Contributors follow a documented OpenSpec-first workflow
The repository SHALL document when contributors need OpenSpec artifacts before implementing code changes.

#### Scenario: Behavior changes require OpenSpec evidence
- **WHEN** a change affects runtime behavior, public Rust APIs, workflow policy, provider/protocol semantics, GPUI behavior, or cross-crate boundaries
- **THEN** contributor documentation requires an OpenSpec change or spec reference before implementation proceeds

#### Scenario: Non-behavior changes can justify omission
- **WHEN** a change is docs-only, typo-only, formatting-only, dependency-free housekeeping, or otherwise does not affect behavior or policy
- **THEN** contributor documentation allows the PR author to state why no OpenSpec artifact is required

### Requirement: Pull requests expose workflow evidence
The repository SHALL ask PR authors to provide OpenSpec, verification, and release-note evidence in the pull request template.

#### Scenario: PR template asks for OpenSpec evidence
- **WHEN** a contributor opens a pull request
- **THEN** the template includes an OpenSpec change/spec field or a no-spec-required justification field

#### Scenario: PR template preserves release-note hygiene
- **WHEN** a contributor fills out the pull request template
- **THEN** the template includes the required `Release Notes:` section format used by repository PR hygiene guidance

#### Scenario: PR template points to predictable checks
- **WHEN** a contributor fills out the pull request checklist
- **THEN** the checklist asks them to confirm the relevant mise-defined verification ran locally or explain why it was not applicable

### Requirement: CI guards OpenSpec evidence for code changes
The repository SHALL include lightweight CI validation that catches pull requests which modify guarded code or workflow paths without OpenSpec evidence or an explicit omission justification.

#### Scenario: Guarded code change lacks evidence
- **WHEN** a pull request changes guarded repository paths and its body lacks both an OpenSpec reference and a no-spec-required justification
- **THEN** CI reports a clear failure explaining how to add OpenSpec evidence or justify omission

#### Scenario: Pull request includes evidence
- **WHEN** a pull request changes guarded repository paths and its body links an OpenSpec change/spec or provides an accepted no-spec-required justification
- **THEN** the OpenSpec evidence validation passes

#### Scenario: Non-PR CI does not require PR body metadata
- **WHEN** CI runs for a push event without pull request metadata
- **THEN** OpenSpec evidence validation does not fail solely because no PR body exists

### Requirement: Mise defines repository security verification
The repository SHALL expose supply-chain and vulnerability checks through mise-defined tasks that can be run locally and from GitHub Actions.

#### Scenario: Local security verification uses mise
- **WHEN** a contributor reads the development workflow documentation
- **THEN** it tells them to run repository security verification through `mise run security` or documented granular mise tasks

#### Scenario: Security CI invokes mise verification
- **WHEN** GitHub Actions runs repository security checks
- **THEN** the workflow sets up mise tooling and invokes mise-defined security tasks instead of installing or running cargo security tools directly in workflow YAML

#### Scenario: Security workflow covers routine triggers
- **WHEN** maintainers inspect the security workflow triggers
- **THEN** the workflow runs for pull requests, pushes to `main`, and scheduled checks

#### Scenario: Security task preserves standard CI
- **WHEN** maintainers add security verification tasks
- **THEN** the existing `mise run ci` quality contract continues to run formatting, checking, clippy, tests, and documentation checks without being replaced by security-only checks

### Requirement: Supply-chain policy is explicit and reviewable
The repository SHALL include an initial supply-chain policy for Rust dependencies that covers vulnerability advisories, license allowances, duplicate dependency handling, registries, and git sources.

#### Scenario: Cargo deny policy is present
- **WHEN** a contributor runs the documented supply-chain policy check
- **THEN** the check evaluates a repository `deny.toml` policy for advisories, licenses, duplicate crates, registries, and git sources

#### Scenario: Unknown dependency sources are rejected
- **WHEN** dependency resolution includes an unapproved registry or git source
- **THEN** the supply-chain policy check fails with a clear source policy violation

#### Scenario: Policy exceptions stay visible
- **WHEN** a dependency requires a license, advisory, duplicate, registry, or git-source exception
- **THEN** the exception is recorded in the repository policy instead of being hidden in ad hoc CI commands

### Requirement: Cargo vet remains deferred until maintainable
The repository SHALL NOT require `cargo-vet` as a blocking local or CI gate until a later change defines audit ownership, exemption policy, and maintenance expectations.

#### Scenario: Cargo vet is not required by initial security verification
- **WHEN** a contributor runs the initial documented security verification for this change
- **THEN** it does not fail solely because `cargo-vet` audits or exemptions have not been initialized

#### Scenario: Cargo vet is non-blocking in CI
- **WHEN** GitHub Actions runs the initial security workflow for this change
- **THEN** `cargo-vet` is absent from the required path or is explicitly configured as non-blocking

#### Scenario: Future cargo vet adoption requires policy
- **WHEN** maintainers decide to make `cargo-vet` blocking
- **THEN** they first define the audit ownership and exemption policy in OpenSpec or contributor documentation

### Requirement: Cargo workspace metadata is centralized for future distribution
The repository SHALL centralize shared Cargo package metadata in the workspace manifest and use workspace inheritance where it does not change crate behavior.

#### Scenario: Shared package metadata is inherited
- **WHEN** maintainers inspect workspace and crate manifests
- **THEN** shared metadata such as edition, rust-version, license, repository, homepage, and authors is defined once in `[workspace.package]` where appropriate and inherited by workspace crates

#### Scenario: Crate-specific metadata remains local
- **WHEN** a crate needs package-specific metadata such as name, description, readme, documentation, keywords, categories, or exclusions
- **THEN** that metadata remains in the crate manifest or explicitly inherits only fields that preserve the crate's package identity

#### Scenario: Metadata changes do not alter runtime behavior
- **WHEN** workspace package metadata is centralized
- **THEN** `oxmux` and `oxidemux` runtime behavior, public API semantics, and crate boundary responsibilities remain unchanged

#### Scenario: Metadata changes preserve package identity
- **WHEN** workspace package metadata and crate metadata are updated
- **THEN** manifest validation confirms package names, crate targets, readme/documentation links, included files, and package-specific descriptions remain intentional without requiring release publishing

### Requirement: Rust tool configuration remains subordinate to mise tasks
The repository SHALL add explicit Rust tool configuration files only when they complement mise-defined checks and reduce environment drift.

#### Scenario: Clippy configuration complements mise
- **WHEN** maintainers add or update `clippy.toml`
- **THEN** the configuration captures stable project-wide lint behavior without replacing `mise run clippy` as the contributor and CI entrypoint

#### Scenario: Rustfmt configuration complements mise
- **WHEN** maintainers add or update `rustfmt.toml`
- **THEN** the configuration captures stable project-wide formatting behavior without replacing `mise run fmt` as the contributor and CI entrypoint

#### Scenario: Tool configuration is omitted when redundant
- **WHEN** explicit Rust tool configuration would only duplicate existing defaults or mise task commands
- **THEN** the implementation omits that configuration and documents why no file was added

### Requirement: Release automation is deferred until usable product readiness
The repository SHALL defer release publishing workflows, crates.io token handling, and automated GitHub release creation until OxideMux has a usable product milestone and a separate release proposal.

#### Scenario: Production-readiness workflow excludes publishing
- **WHEN** this change is implemented
- **THEN** it does not add a release publishing workflow, crates.io publishing steps, or required publishing credentials

#### Scenario: Release readiness is documented as deferred
- **WHEN** contributors read workflow documentation after this change
- **THEN** it explains that release automation is intentionally deferred until a later usable-product milestone
