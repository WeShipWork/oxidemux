## 1. Enforcement setup

- [x] 1.1 Add `//!` crate-level documentation and `#![warn(missing_docs)]` to `crates/oxmux/src/oxmux.rs`, with the crate docs identifying `oxmux` as the headless core facade.
- [x] 1.2 Add `//!` crate-level documentation and `#![warn(missing_docs)]` to `crates/oxidemux/src/main.rs`, with the crate docs identifying `oxidemux` as the app-shell integration consumer of `oxmux` rather than the owner of core proxy semantics.
- [x] 1.3 Run `RUSTFLAGS='-D missing_docs' cargo check --workspace --all-targets --all-features` and write the raw diagnostic output plus a human summary to `openspec/changes/enforce-public-api-documentation/artifacts/missing-docs-report.txt` and `openspec/changes/enforce-public-api-documentation/artifacts/missing-docs-summary.md`. The summary must group remaining affected public modules, source files, functions, methods, associated functions, and items by crate/module before backfill starts.
- [x] 1.4 Add a `doc` mise task that runs `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps`, and include it in `mise run ci` so local and CI verification fail on rustdoc warnings.

## 2. Core facade and semantic documentation

- [x] 2.1 Document the `oxmux` facade modules, re-export strategy, `CoreIdentity`, `CORE_IDENTITY`, and `core_identity` API.
- [x] 2.2 Document `CoreError`, configuration error types, routing failures, streaming failures, provider execution failures, and matchable error variants.
- [x] 2.3 Document provider execution primitives, provider/account summaries, auth state, quota state references, mock provider harnesses, provider boundary marker types, and all public provider functions/methods.
- [x] 2.4 Document routing policy primitives, model aliases, routing targets, availability states, selection results, skipped candidates, fallback/degraded-state semantics, and all public routing functions/methods.
- [x] 2.5 Document protocol request/response primitives, protocol metadata families, opaque payload semantics, translation boundaries, deferred translation outcomes, and all public protocol functions/methods.

## 3. Runtime, management, configuration, streaming, and usage docs

- [x] 3.1 Document management snapshot, lifecycle state, health state, endpoint, uptime, lifecycle control intent types, and all public management functions/methods.
- [x] 3.2 Document minimal proxy request/response, engine configuration, local health runtime configuration/status, local route configuration, runtime constants, and all public proxy/runtime functions/methods.
- [x] 3.3 Document streaming response modes, stream events, terminal states, cancellation reasons, stream content/metadata/failure types, invalid sequence diagnostics, and all public streaming functions/methods.
- [x] 3.4 Document public file-backed and layered configuration types, configuration snapshots, update intents, routing defaults, source metadata, reload outcomes, management-visible configuration state, and all public configuration functions/methods.
- [x] 3.5 Document usage and quota summary types, metered values, quota states, public usage functions/methods, and how unknown/degraded/unavailable states should be interpreted by consumers.

## 4. Exceptions and generated/internal surfaces

- [x] 4.1 Review internal raw configuration/parser modules and keep them outside the public API surface where possible.
- [x] 4.2 Add narrow `#[allow(missing_docs)]` only for intentionally internal or generated surfaces that should not receive public rustdoc, with the smallest practical scope and an in-source explanation for each allowance.
- [x] 4.3 Confirm no broad crate-wide or workspace-wide missing-docs allowance hides undocumented public APIs.
- [x] 4.4 Create `openspec/changes/enforce-public-api-documentation/artifacts/exceptions.md` listing each intentional `#[allow(missing_docs)]` with file path, allowed item or module, scope, and justification.

## 5. App-shell documentation

- [x] 5.1 Document the `oxidemux` binary crate as the app-shell/platform integration consumer of `oxmux`.
- [x] 5.2 Document any public app-shell functions, methods, constants, and other items with contributor-facing guidance that preserves the boundary from core proxy semantics.
- [x] 5.3 Review generated docs for `oxidemux` and confirm they do not imply ownership of proxy, routing, provider, quota, protocol, management, auth/session, or request rewrite semantics that belong in `oxmux`.

## 6. Verification

- [x] 6.1 Run `cargo fmt --all -- --check`.
- [x] 6.2 Run `cargo check --all-targets --all-features`.
- [x] 6.3 Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [x] 6.4 Run `cargo test --all-targets --all-features`.
- [x] 6.5 Run `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps` and fix any missing-doc or rustdoc link regressions.
- [x] 6.6 Run `mise run ci` and confirm it includes the rustdoc generation task.
- [x] 6.7 Review generated rustdoc against `docs/vision.md`, `docs/architecture.md`, `openspec/specs/oxmux-core/spec.md`, and `openspec/specs/oxidemux-app-shell/spec.md` to confirm docs describe current contracts or future ownership markers without promising unimplemented provider SDKs, OAuth flows, UI behavior, or runtime semantics.
- [x] 6.8 Search public rustdoc additions for placeholders such as `TODO`, `TBD`, `Docs TBD`, empty summaries, or docs that only repeat the identifier name, and replace them with factual consumer-facing or contributor-facing semantics.
- [x] 6.9 Run `openspec validate enforce-public-api-documentation --strict`.
