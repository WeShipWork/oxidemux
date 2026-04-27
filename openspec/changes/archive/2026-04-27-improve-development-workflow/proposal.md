## Why

OxideMux already has OpenSpec, mise, hk, Rust/GPUI guidance, and CI pieces, but the workflow entrypoints are split across README, AGENTS.md, `.opencode`, and GitHub files. We need one predictable development path where local checks and CI use the same mise-defined commands and code changes have OpenSpec evidence.

## What Changes

- Add a repository development workflow capability covering local setup, CI parity, OpenSpec requirements, contribution guidance, and PR review expectations.
- Establish mise as the universal entrypoint for installing tools and running repository checks locally and in CI.
- Add a root `CONTRIBUTING.md` that explains the expected change flow, including when OpenSpec artifacts are required.
- Update `.github/PULL_REQUEST_TEMPLATE.md` so PR authors link an OpenSpec change/spec or explicitly justify why no spec is needed.
- Add lightweight validation so guarded code changes cannot bypass the OpenSpec evidence requirement unintentionally.

## Capabilities

### New Capabilities

- `development-workflow`: Repository contribution, OpenSpec, mise, PR, and CI workflow requirements that keep local and CI outcomes predictable.

### Modified Capabilities

<!-- No existing product/runtime capability requirements change in this proposal. -->

## Impact

- Adds or updates repository governance documents such as `CONTRIBUTING.md` and `.github/PULL_REQUEST_TEMPLATE.md`.
- Updates CI workflow design so CI invokes mise-defined tasks instead of duplicating command sequences outside mise.
- May add a small validation script or CI job that checks PR metadata and changed paths for required OpenSpec evidence.
- Does not change `oxmux` runtime behavior, `oxidemux` app behavior, public Rust APIs, provider execution, protocol translation, or GPUI UI behavior.
