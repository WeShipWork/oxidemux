## Context

The repository is currently a minimal Rust 1.95.0, edition 2024 binary crate named `oxidemux`. It has a small `src/main.rs`, a bootstrap metadata test, workspace-wide verification tasks in `mise.toml`, and CI that runs formatting, clippy, check, and tests across supported platforms.

GitHub issue #1 defines a broader product direction: OxideMux should become a GPUI-based native app for AI subscription proxy management while also offering a reusable Rust core. The foundational design decision is to split reusable headless behavior into `oxmux` and keep `oxidemux` as the app and integration shell.

External projects are conceptual references only. CLIProxyAPI provides useful vocabulary for provider execution, protocol translation, model aliases, fallback, streaming, and management surfaces. zero-limit provides useful desktop control and quota-monitoring UX concepts. gpui-component is a candidate for a future GPUI compatibility spike. None of those projects should be copied or introduced as dependencies in this phase.

## Goals / Non-Goals

**Goals:**

- Convert the repository into a two-member Cargo workspace with `oxmux` and `oxidemux`.
- Make `oxmux` a UI-free library crate that can be used directly by Rust agents, CLIs, IDE integrations, and future app code.
- Make `oxidemux` the app shell that depends on `oxmux`, preserving the existing bootstrap binary behavior while moving ownership away from the repository root package.
- Add a direct library-use test or example proving that `oxmux` works without launching the desktop app, IPC, or a local proxy process.
- Keep verification simple and cross-platform by updating existing cargo, mise, and CI commands for the workspace.

**Non-Goals:**

- Implementing provider clients, OAuth flows, token refresh, routing algorithms, protocol translation, streaming transports, quota analytics, management HTTP endpoints, or proxy runtime behavior.
- Adding GPUI, gpui-component, tray, updater, packaging, platform credential storage, or desktop UI dependencies.
- Splitting the project into many small crates before the `oxmux` and `oxidemux` boundary is proven.
- Committing to long-term public API stability beyond the initial crate-level boundary and minimal facade.

## Decisions

### Decision: Use a two-member workspace now

The root `Cargo.toml` will become the workspace manifest, with members for `crates/oxmux` and `crates/oxidemux`.

- **Rationale:** The current repo has almost no implementation, so the boundary is cheapest to establish now. A workspace gives independent package identities while preserving one repository-level verification command.
- **Alternative considered:** Keep one crate and add `src/lib.rs`. This is simpler initially, but it leaves `oxmux` as an internal module rather than a reusable package and makes later extraction more expensive.
- **Alternative considered:** Create additional crates for routing, providers, protocol translation, and UI immediately. This would overfit future architecture before the core/app split is validated.

### Decision: Keep `oxmux` headless and dependency-light

`oxmux` will not depend on GPUI, gpui-component, tray libraries, platform updaters, desktop secret stores, or app lifecycle code. It will expose a minimal facade that establishes ownership of future core domains: proxy lifecycle, provider/auth abstractions, routing, protocol translation, configuration, streaming, management/status, usage/quota, and domain errors.

- **Rationale:** The main product promise for `oxmux` is direct in-process use by Rust consumers without launching `oxidemux`. Desktop dependencies would violate that promise and complicate CLI, IDE, server, and agent integrations.
- **Alternative considered:** Put platform credential storage directly in `oxmux`. The better boundary is for `oxmux` to define traits or domain abstractions later, while `oxidemux` or platform adapters provide OS-specific implementations.

### Decision: Make `oxidemux` a consumer, not the owner, of core behavior

`oxidemux` will own the binary entrypoint, app lifecycle, future GPUI UI, settings UX, tray/background behavior, packaging, update workflows, and integration adapters. Its dependency direction will point inward to `oxmux`.

- **Rationale:** This prevents UI and platform decisions from becoming hidden prerequisites for library consumers.
- **Alternative considered:** Keep proxy behavior in the app and expose it through IPC or a local server. This would force Rust consumers to run an app process and contradict issue #1's first-class embedded library requirement.

### Decision: Defer concrete proxy/provider behavior

This change will only create structural and testable boundary artifacts. Provider auth, token refresh, streaming, routing fallback, protocol translation, proxy startup, and degraded service behavior remain future requirements owned by `oxmux`.

- **Rationale:** CLIProxyAPI shows these domains are substantial. Implementing them during the split would harden premature abstractions and make the first change difficult to review.
- **Alternative considered:** Add no domain names until the engine is implemented. That would keep the first API smaller, but it would not communicate the intended ownership boundary that downstream consumers and app code need.

## Risks / Trade-offs

- **[Risk] The initial `oxmux` facade becomes too abstract to be useful** → Keep it intentionally small and prove it with a direct library-use test or example rather than a large trait hierarchy.
- **[Risk] Future GPUI work leaks into `oxmux` through convenience imports** → Add tests or dependency checks that fail if `oxmux` gains GPUI or app-shell dependencies.
- **[Risk] Moving the package layout breaks CI or bootstrap tests** → Update cargo commands, package metadata expectations, and CI in the same change; run workspace fmt, clippy, check, and tests.
- **[Risk] External references bias the implementation toward copying another architecture** → Treat CLIProxyAPI, zero-limit, and gpui-component as vocabulary and inspiration only; do not copy code or mirror their full structure.
- **[Risk] Workspace paths churn future documentation** → Update README references to describe the new crate locations and clarify what is still unimplemented.

## Migration Plan

1. Convert the root manifest into a workspace manifest.
2. Create `crates/oxmux` as a library crate with an explicit library root path and minimal public facade.
3. Create `crates/oxidemux` as the binary app shell and preserve the existing version-printing bootstrap behavior through direct use of `oxmux` where appropriate.
4. Move or replace bootstrap tests so they validate both workspace members and the direct library-use requirement.
5. Update README, mise tasks, and CI commands only as needed for the workspace layout.
6. Verify with cargo fmt, cargo clippy, cargo check, cargo test, and existing project task wrappers.

Rollback is straightforward while the change remains structural: restore the single-crate manifest, move the binary entrypoint back to `src/main.rs`, and revert workspace-specific tests and documentation.

## Open Questions

- Should `oxmux` eventually be published as its own crate, or remain workspace-only until the core engine stabilizes?
- Which async runtime will `oxmux` use once proxy and streaming behavior are implemented?
- What is the minimum public facade that is useful now without implying long-term stability for provider and routing APIs?
