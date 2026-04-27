## 1. Contributor documentation

- [x] 1.1 Add root `CONTRIBUTING.md` that documents setup with `mise install` and `mise trust`, local verification with `mise run ci`, hk hook setup through `mise run hk-install`, and the expected OpenSpec-first change flow.
- [x] 1.2 Document when OpenSpec artifacts are required, including behavior changes, public Rust API changes, workflow policy changes, provider/protocol semantics, GPUI behavior, and cross-crate boundaries.
- [x] 1.3 Document allowed no-spec-required cases, including docs-only, typo-only, formatting-only, and clearly non-behavioral changes that include a short PR justification.
- [x] 1.4 Include PR hygiene guidance in `CONTRIBUTING.md`, including one-purpose PRs, imperative PR titles, local verification expectations, and the repository `Release Notes:` format.

## 2. PR template and OpenSpec evidence

- [x] 2.1 Update `.github/PULL_REQUEST_TEMPLATE.md` with an OpenSpec change/spec field and an explicit no-spec-required justification path.
- [x] 2.2 Update `.github/PULL_REQUEST_TEMPLATE.md` to preserve the required `Release Notes:` section format as the final section.
- [x] 2.3 Add checklist items for `mise run ci`, relevant hk checks, tests, and focused single-purpose PR scope.

## 3. Mise-driven CI parity

- [x] 3.1 Update `.github/workflows/ci.yml` so standard quality checks install/use mise and run `mise run ci` instead of duplicating raw cargo commands outside mise.
- [x] 3.2 Preserve cross-platform verification while routing the shared verification command through mise-managed tools.
- [x] 3.3 Add a CI step for `mise run hk-check` or an equivalent mise-defined hook check if it is reliable on CI runners.
- [x] 3.4 Document in `CONTRIBUTING.md` that CI and local verification intentionally share the same mise task graph.

## 4. Lightweight OpenSpec validation

- [x] 4.1 Add a small validation script under `scripts/ci/` that inspects pull request metadata and changed guarded paths for an OpenSpec link or a no-spec-required justification.
- [x] 4.2 Wire the validation script into GitHub Actions for pull request events without failing push events solely because no PR body exists.
- [x] 4.3 Keep validation messages actionable by explaining how to add an `openspec/changes/...` or `openspec/specs/...` reference, or how to justify omission.

## 5. Verification

- [x] 5.1 Run `openspec status --change "improve-development-workflow"` and confirm the change is apply-ready.
- [x] 5.2 Run `mise run ci` and confirm formatting, checking, clippy, and tests pass through the mise task graph.
- [x] 5.3 Run the OpenSpec evidence validation script against representative PR-like inputs covering code change with evidence, code change with no justification, no-spec-required justification, and push/non-PR metadata.
- [x] 5.4 Review changed workflow documentation for consistency with `README.md`, `AGENTS.md`, `mise.toml`, and `.github/workflows/ci.yml`.
