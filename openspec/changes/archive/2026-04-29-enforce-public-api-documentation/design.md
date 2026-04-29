## Context

Issue #34 asks OxideMux to follow the pattern already used by `../langfuse-rs`: enable `#![warn(missing_docs)]` at Rust crate roots while CI treats warnings as errors, then backfill meaningful public API documentation throughout the crate module trees. OxideMux currently has a small two-crate workspace: `oxmux` is the headless reusable library facade, and `oxidemux` is the app-shell binary consumer of `oxmux` that contributors use as the platform integration entrypoint. `oxidemux` adapts and presents `oxmux` headless core state for platform integration; its docs must not imply it owns proxy, routing, provider, quota, protocol, or management semantics.

The documentation burden is concentrated in `oxmux`. Its crate root is `crates/oxmux/src/oxmux.rs`, and it re-exports public modules for configuration, errors, local proxy runtime, management, minimal proxy, protocol, provider execution, routing, streaming, and usage/quota state. Documentation must be added in the source files that define those public items, not only in the crate root. `oxidemux` currently has a much smaller binary root at `crates/oxidemux/src/main.rs`, but enabling missing-docs there keeps contributor-facing app-shell responsibilities explicit as the binary grows. A dry diagnostic check with missing docs promoted to errors shows the `oxmux` work is cross-cutting: public fields, enum variants, structs, enums, functions, methods, associated functions, traits, constants, modules, and the crate itself all need coverage.

## Goals / Non-Goals

**Goals:**

- Enable missing-docs enforcement on every Rust crate root in the workspace so it applies to public items throughout each crate's module tree.
- Add factual crate, module, function, method, associated function, and item documentation for public API surfaces that Rust consumers, app-shell contributors, or tests import or match against.
- Preserve existing behavior and public API shapes while making core semantics discoverable through rustdoc.
- Document the `oxmux`/`oxidemux` boundary in public docs: headless core semantics stay in `oxmux`; platform shell concerns stay in `oxidemux`.
- Handle generated or intentionally internal-only surfaces with explicit narrow allowances instead of broad suppression.
- Verify the result through the existing `mise run ci` path, with rustdoc generation wired into that task, using `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps` so documentation warnings fail locally and in CI.

**Non-Goals:**

- No runtime behavior changes, proxy pipeline changes, configuration schema changes, or app-shell UI work.
- No new dependencies, documentation generators, provider SDKs, telemetry SDKs, OAuth libraries, platform secret-store adapters, or GPUI changes.
- No public API renames or removals solely to reduce documentation volume.
- No conversion from `warn(missing_docs)` to `deny(missing_docs)` in source unless a later change decides the project wants source-level deny semantics; CI already treats warnings as errors.
- No blanket workspace-wide `allow(missing_docs)` or silent exclusion of public API surfaces.

## Decisions

1. **Enable enforcement at every workspace crate root.**
   - Rationale: `oxmux` is the reusable public facade consumed by tests, the app shell, and future Rust consumers, while `oxidemux` is the contributor-facing binary entrypoint for app-shell responsibilities. Applying the same lint posture to both crate roots keeps the workspace policy simple and prevents undocumented app-shell public additions as that crate grows.
   - Alternative considered: enable missing docs only in CI through `RUSTFLAGS`. Rejected because crate-level `#![warn(missing_docs)]` makes the policy visible in source and mirrors the `langfuse-rs` reference.

2. **Keep `oxidemux` docs focused on app-shell responsibilities.**
   - Rationale: enforcing docs in the binary is useful for contributors, but those docs must not imply that proxy, routing, provider, quota, protocol, or management semantics are owned by the app shell.
   - Alternative considered: document only `oxmux` and leave the binary outside this pass. Rejected because it would leave contributor-facing app-shell public items without the same documentation quality gate.

3. **Use product/spec language as the source for public docs.**
   - Rationale: `docs/vision.md`, `docs/architecture.md`, and `openspec/specs/oxmux-core/spec.md` already define the crate boundary and product semantics. Public rustdoc should summarize those contracts rather than inventing new meaning.
   - Alternative considered: write mechanically generated one-line docs from item names. Rejected because issue #34 requires meaningful docs that clarify subscription-aware routing, provider/account state, protocol compatibility, management snapshots, and shell boundaries.

4. **Document the facade and high-risk semantic types before low-risk helpers.**
   - Rationale: `CoreError`, routing failures, provider execution, management snapshots, protocol payloads, streaming responses, configuration state, and usage/quota summaries are the public contracts consumers need to understand and match safely.
   - Alternative considered: follow file order mechanically. Rejected because it delays the docs that most affect downstream correctness.

5. **Use narrow `#[allow(missing_docs)]` only for intentional exceptions.**
   - Rationale: generated or internal-only modules can be noisy and low-value, but exceptions must be visible and justified. Allowances must use the smallest practical item or module scope, must not sit at the crate root, must include an in-source reason, and must be listed in `openspec/changes/enforce-public-api-documentation/artifacts/exceptions.md`. `langfuse-rs` uses crate-level enforcement with module-level allowances for generated API code, which is the right precedent only when the exception remains narrow and auditable.
   - Alternative considered: broad allow at module trees or crate root while gradually backfilling. Rejected because it undermines the enforcement goal.

6. **Keep docs factual and consumer-facing rather than implementation commentary.**
   - Rationale: project guidance discourages comments that merely restate code. Rustdoc should explain what the public contract represents, what state means, and how callers should interpret outcomes.
   - Alternative considered: add comments near implementation details to satisfy lint output. Rejected because it would pass checks without improving API comprehension.

## Risks / Trade-offs

- **Large warning volume** → Triage by module and prioritize public facade semantics; keep changes reviewable by grouping docs by coherent module sections.
- **Docs accidentally imply behavior not implemented yet** → Anchor wording in existing specs and describe placeholder/boundary types as current contracts or future ownership markers without promising concrete provider SDK, OAuth, or UI behavior.
- **Internal parser details leak into public docs** → Keep raw/internal types crate-private or narrowly allow missing docs where visibility is intentionally scoped and not part of the public facade.
- **Re-export docs duplicate module docs** → Document original public items with meaningful rustdoc and keep facade docs focused on crate/module purpose rather than duplicating every item.
- **CI failures persist after docs are added** → Run the full verification sequence, including `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps`, because rustdoc can reveal broken intra-doc links that cargo check does not.

## Migration Plan

1. Add crate-level `//!` docs and `#![warn(missing_docs)]` to `crates/oxmux/src/oxmux.rs` and `crates/oxidemux/src/main.rs`.
2. Run a workspace missing-docs diagnostic pass to capture the exact remaining public items and write the raw output plus a grouped summary under `openspec/changes/enforce-public-api-documentation/artifacts/`.
3. Backfill `oxmux` module, function, method, associated function, and item docs in priority order: facade/core identity, errors, provider execution, routing, management, protocol, streaming, minimal proxy, local runtime, configuration, usage/quota, mocks, and boundary markers.
4. Backfill `oxidemux` binary docs with app-shell contributor guidance that consumes `oxmux` state without redefining core semantics.
5. Add narrow explicit allowances only for intentionally internal or generated surfaces, with explanatory module docs where appropriate and an `artifacts/exceptions.md` audit entry for each allowance.
6. Add a `doc` mise task that runs `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps`, wire it into `mise run ci`, then run formatting, check, clippy with warnings as errors, tests, and docs generation.
7. Roll back by removing the crate-level missing-docs attributes and newly added docs if the enforcement creates unavoidable release-blocking churn before implementation is complete.

## Resolved Questions

- `mise run ci` is the canonical local and CI verification contract, so this change adds rustdoc generation there instead of relying on a separate manual-only command.
