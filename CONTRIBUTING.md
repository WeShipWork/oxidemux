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
mise run hk-check
```

`mise.toml` is the source of truth for workflow tools and verification tasks.
When Rust is upgraded, keep the `rust` pin in `mise.toml` aligned with
`rust-toolchain.toml`.

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
