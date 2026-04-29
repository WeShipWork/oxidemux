# Contributing

OxideMux uses OpenSpec for planned changes and mise for repeatable local and CI
verification. Keep pull requests focused on one purpose so reviewers can connect
the change, evidence, and verification result quickly.

## Setup

Install and trust the repository tools from `mise.toml`:

```bash
mise install
mise trust
```

Install hk hooks for local checks:

```bash
mise run hk-install
```

Run the same verification contract CI uses:

```bash
mise run ci
mise run security
mise run hk-check
```

`mise.toml` is the source of truth for workflow tools and verification tasks.
When Rust is upgraded, keep the `rust` pin in `mise.toml` aligned with
`rust-toolchain.toml`. Security tools are also invoked through mise: use
`mise run security` for the full dependency policy and vulnerability gate, or
`mise run deny` and `mise run audit` when you need a narrower check.

## Security and supply-chain policy

OxideMux keeps supply-chain exceptions explicit and reviewable. If
`cargo-deny` reports an advisory, license, duplicate dependency, registry, or
git-source exception that the project must accept, record the exception in
`deny.toml` with a reason instead of adding ad hoc CI shell flags or skipping
the mise task.

`cargo-vet` is not a required local or CI gate yet. Future blocking vet
adoption must first define audit ownership, exemption policy, and maintenance
expectations in OpenSpec or contributor documentation.

Release publishing workflows, crates.io token handling, and automated GitHub
release creation are intentionally out of scope until a later usable-product
milestone defines distribution readiness.

## Rust tool behavior configuration

Rust formatting and lint entrypoints remain `mise run fmt` and `mise run clippy`. This
change does not add `rustfmt.toml` or `clippy.toml` because the current repository
uses stable default behavior plus explicit mise commands; adding empty or
duplicative config files would create another source of truth without reducing
environment drift. Add either file only when a later change needs stable
project-wide behavior that cannot be expressed by the existing mise tasks.

## OpenSpec-first workflow

Create or update OpenSpec artifacts before implementing changes that affect:

- Runtime behavior.
- Public Rust APIs.
- Workflow policy.
- Provider or protocol semantics.
- Subscription UX, provider auth/session semantics, model aliases, request
  rewrite behavior, or reasoning/thinking compatibility.
- GPUI behavior.
- Cross-crate boundaries.

Use `openspec/changes/<change-name>/` for proposed changes and
`openspec/specs/<capability>/` for accepted capabilities. Include the relevant
OpenSpec path in the pull request.

No OpenSpec artifact is required for docs-only, typo-only, formatting-only,
dependency-free housekeeping, or clearly non-behavioral changes. In those cases,
add a short no-spec-required justification in the pull request template.

Use `docs/vision.md` and `docs/architecture.md` as the durable product-intent
guardrails. Changes that move behavior between `oxmux` and `oxidemux` must
explain how they preserve the shared core versus platform shell boundary.

## Pull request hygiene

- Keep each PR focused on one project or change.
- Use a clear, correctly capitalized, imperative PR title.
- Avoid conventional commit prefixes in PR titles.
- Follow `.github/PULL_REQUEST_TEMPLATE.md` for OpenSpec evidence, changelog
  expectations, verification checks, and release notes format.
