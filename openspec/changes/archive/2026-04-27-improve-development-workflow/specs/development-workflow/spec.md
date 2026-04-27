## ADDED Requirements

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
