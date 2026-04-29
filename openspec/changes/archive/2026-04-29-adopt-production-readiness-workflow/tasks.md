## 1. Mise-managed security tooling

- [x] 1.1 Add mise-managed tool pins or installation strategy for `cargo-deny` and `cargo-audit`.
- [x] 1.2 Add granular mise tasks for dependency policy and vulnerability checks.
- [x] 1.3 Add a top-level `mise run security` task that runs the granular security checks.
- [x] 1.4 Verify the new security tasks do not replace or weaken the existing `mise run ci` task.
- [x] 1.5 Verify GitHub workflow YAML does not install or run `cargo-deny`, `cargo-audit`, or other cargo security tools directly when a mise task exists.

## 2. Supply-chain policy

- [x] 2.1 Add an initial `deny.toml` covering advisories, license allow list, duplicate dependency handling, registries, and git sources.
- [x] 2.2 Run the cargo-deny task and tune only explicit, reviewable policy exceptions required by the current dependency graph.
- [x] 2.3 Run the cargo-audit task and document any unavoidable advisory exceptions in policy rather than CI shell commands.
- [x] 2.4 Document that blocking `cargo-vet` adoption is deferred pending audit ownership and exemption policy.
- [x] 2.5 Verify `cargo-vet` is absent from required local and CI security paths or explicitly non-blocking.

## 3. GitHub security workflow

- [x] 3.1 Add a dedicated GitHub Actions security workflow for pull requests, pushes to `main`, and scheduled checks.
- [x] 3.2 Configure the workflow to set up mise and invoke `mise run security` instead of duplicating raw cargo security commands.
- [x] 3.3 Keep workflow permissions minimal and read-only unless a later change requires elevated permissions.

## 4. Cargo metadata readiness

- [x] 4.1 Add shared `[workspace.package]` metadata to the root `Cargo.toml` where inheritance preserves current package behavior.
- [x] 4.2 Update `crates/oxmux/Cargo.toml` to inherit shared metadata and keep crate-specific package metadata local.
- [x] 4.3 Update `crates/oxidemux/Cargo.toml` to inherit shared metadata and keep crate-specific package metadata local.
- [x] 4.4 Add distribution-readiness metadata such as repository, readme, documentation, keywords, categories, and package exclusions where appropriate.
- [x] 4.5 Verify manifest metadata changes preserve package names, crate targets, readme/documentation links, included files, package-specific descriptions, runtime behavior, and crate boundary responsibilities without requiring release publishing.

## 5. Tool behavior configuration

- [x] 5.1 Evaluate whether `clippy.toml` reduces drift beyond the existing `mise run clippy` command.
- [x] 5.2 Add `clippy.toml` only if it captures stable project-wide lint behavior.
- [x] 5.3 Evaluate whether `rustfmt.toml` reduces drift beyond the existing `mise run fmt` command.
- [x] 5.4 Add `rustfmt.toml` only if it captures stable project-wide formatting behavior.

## 6. Documentation and scope guardrails

- [x] 6.1 Update README development instructions to include the new mise security task.
- [x] 6.2 Update `CONTRIBUTING.md` to explain security verification, supply-chain policy exceptions, and deferred release automation.
- [x] 6.3 Update the pull request template if needed so contributors can report security verification results.
- [x] 6.4 Ensure documentation states that release publishing workflows, crates.io token handling, and automated GitHub releases remain out of scope until a later usable-product milestone.

## 7. Verification

- [x] 7.1 Run `openspec validate adopt-production-readiness-workflow --strict`.
- [x] 7.2 Run `mise run security` or the new granular security task set.
- [x] 7.3 Run `mise run ci`.
- [x] 7.4 Run `mise run hk-check`.
- [x] 7.5 Confirm no release workflow or publishing credential requirement was introduced.
- [x] 7.6 Confirm `cargo-vet` is not required for local or CI success in this change.
