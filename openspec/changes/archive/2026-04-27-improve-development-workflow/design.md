## Context

The repository already has the pieces of a predictable workflow: `mise.toml` pins Rust, hk, and other development tools; `mise run ci` aggregates formatting, checking, clippy, and tests; README documents local setup; AGENTS.md records Rust/GPUI and PR hygiene; OpenSpec changes record proposal/design/spec/task history; and CI runs equivalent cargo checks.

The problem is that those pieces are not yet a single contract. CI duplicates cargo commands instead of running the mise task graph, contribution rules are not centralized in `CONTRIBUTING.md`, and PRs do not require OpenSpec evidence. That makes outcomes harder to predict as the repository grows beyond the current Rust bootstrap.

## Goals / Non-Goals

**Goals:**

- Make mise the canonical entrypoint for repository tool installation and verification, both locally and in GitHub Actions.
- Document one contributor path from idea to OpenSpec change to implementation to PR.
- Require code-changing PRs to link an OpenSpec change/spec or provide an explicit no-spec-required justification.
- Keep the first enforcement pass lightweight, reviewable, and easy to adjust.

**Non-Goals:**

- Change `oxmux` or `oxidemux` runtime behavior.
- Introduce a heavyweight policy engine or branch-protection-specific configuration in this change.
- Add product requirements for provider execution, protocol translation, local proxy runtime, or GPUI UI behavior.

## Decisions

### Decision: Use mise as the single verification contract

CI should install/use mise and invoke mise tasks rather than duplicating raw cargo commands. The repository already has `mise.toml` with `rust = "1.95.0"`, hk, and a `ci` task; using it in CI makes local and remote checks share one source of truth.

Alternative considered: keep CI as raw cargo commands and document that developers should run mise locally. Rejected because duplicated command lists drift over time and hide non-Rust tool pins from CI.

### Decision: Keep Rust toolchain pin aligned with mise

The existing `rust-toolchain.toml` and `mise.toml` both pin Rust 1.95.0. This change should preserve that alignment and document that Rust upgrades update both pins together.

Alternative considered: remove `rust-toolchain.toml` and rely only on mise. Rejected for now because Rust tooling, editors, and contributors without mise still understand `rust-toolchain.toml`.

### Decision: Put contributor workflow in root `CONTRIBUTING.md`

README can stay product-facing and quick-start oriented. `CONTRIBUTING.md` should become the durable contributor contract: setup with mise, expected checks, OpenSpec flow, PR hygiene, and verification expectations.

Alternative considered: expand README only. Rejected because workflow policy would compete with project overview content and be harder to discover from GitHub's standard contribution entrypoint.

### Decision: Enforce OpenSpec evidence through PR template plus lightweight CI

The PR template should require an OpenSpec link or an explicit no-spec-needed justification. CI should check PR metadata and guarded changed paths to prevent accidental bypass. The check should focus on clear repository code paths first and produce helpful messages.

Alternative considered: rely on reviewer discipline only. Rejected because the goal is predictable workflow outcomes, and a lightweight automated reminder catches missing evidence earlier.

## Risks / Trade-offs

- **[Risk] CI mise setup adds installation time** → Mitigation: use the official mise GitHub Action with caching and keep `mise run ci` focused on existing checks.
- **[Risk] Spec enforcement blocks trivial changes** → Mitigation: allow an explicit no-spec-required PR justification for docs-only, typo-only, formatting-only, and clearly non-behavioral changes.
- **[Risk] `latest` tool pins can reduce predictability** → Mitigation: document that shared workflow tools should move toward explicit versions or a committed lockfile when reproducibility requires it.
- **[Risk] Validation script becomes too clever** → Mitigation: start with path/body checks and clear messages; avoid parsing Rust API semantics in the first iteration.

## Migration Plan

1. Add contributor documentation.
2. Update PR template to expose the OpenSpec expectations at authoring time.
3. Change CI to install/use mise and run the mise-defined verification task.
4. Add lightweight OpenSpec evidence validation for pull requests.
5. Run local mise checks and OpenSpec status validation before implementation is considered complete.

Rollback is simple: revert the documentation, PR template, CI, and validation script changes. No runtime data or user-facing product behavior is migrated.

## Open Questions

- Should this repository commit `mise.lock` now, or first replace `latest` tool pins with explicit versions for hk-adjacent tooling?
