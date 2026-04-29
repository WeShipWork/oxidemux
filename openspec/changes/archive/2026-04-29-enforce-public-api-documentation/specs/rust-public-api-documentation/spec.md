## ADDED Requirements

### Requirement: Workspace enforces public Rust API documentation
The workspace SHALL enable Rust missing-documentation diagnostics for public Rust API surfaces exposed by all workspace crates, including public items declared in module files below each crate root. The workspace's canonical `mise run ci` verification SHALL include rustdoc generation with warnings treated as errors so missing documentation and broken documentation links are visible before merge.

#### Scenario: Missing docs are enabled for the core facade
- **WHEN** maintainers inspect the `oxmux` crate root
- **THEN** it declares missing-documentation enforcement and crate-level documentation for the headless core facade

#### Scenario: Missing docs are enabled for the app shell
- **WHEN** maintainers inspect the `oxidemux` crate root
- **THEN** it declares missing-documentation enforcement and crate-level documentation for the app-shell binary responsibilities

#### Scenario: CI catches undocumented public APIs
- **WHEN** a public API item is added without documentation and the documented CI verification is run
- **THEN** warnings-as-errors checks fail instead of allowing the undocumented public surface to merge silently

#### Scenario: CI catches rustdoc warnings
- **WHEN** maintainers run `mise run ci`
- **THEN** it includes `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps` or an equivalent rustdoc warnings-as-errors task

### Requirement: Public core semantics are documented for Rust consumers
Public documentation for workspace APIs SHALL explain the consumer-facing and contributor-facing semantics of each crate, including `oxmux` provider/account state, routing, protocol compatibility, streaming response state, management snapshots, usage/quota summaries, configuration state, structured errors, and `oxidemux` app-shell responsibilities.

#### Scenario: Core state docs describe product semantics
- **WHEN** a Rust consumer reads generated documentation for `oxmux` provider, routing, protocol, management, streaming, configuration, usage, or error types
- **THEN** the documentation explains what the public state represents without requiring the consumer to infer behavior from implementation details

#### Scenario: App shell docs describe contributor responsibilities
- **WHEN** a contributor reads generated documentation for `oxidemux`
- **THEN** the documentation explains that the crate adapts `oxmux` state into platform shell behavior without owning core proxy, routing, provider, quota, protocol, or management semantics

#### Scenario: Crate boundary remains visible in docs
- **WHEN** a Rust consumer reads generated documentation for `oxmux` and `oxidemux` boundaries
- **THEN** the documentation identifies `oxmux` as the headless core owner of reusable proxy semantics and `oxidemux` as the app-shell owner of platform presentation and OS integration concerns

#### Scenario: Documentation does not promise unimplemented behavior
- **WHEN** generated rustdoc describes placeholder, boundary, or future ownership types
- **THEN** it describes current contracts or ownership markers without promising unimplemented provider SDKs, OAuth flows, UI behavior, telemetry behavior, platform secret-store behavior, or runtime proxy semantics

### Requirement: Documentation backfill covers the exposed public surface
The documentation backfill SHALL cover public modules, re-exports, structs, enum variants, traits, functions, methods, associated functions, constants, fields, and public helper types exposed by the workspace, regardless of which Rust source file defines them.

#### Scenario: Public functions in module files are documented
- **WHEN** a public function, method, or associated function is declared in any workspace Rust source file
- **THEN** it has meaningful rustdoc describing its consumer-facing or contributor-facing purpose

#### Scenario: Facade re-exports remain documented through original items
- **WHEN** a public item is re-exported from the `oxmux` facade
- **THEN** the underlying public item has meaningful rustdoc so generated facade documentation remains useful to consumers

#### Scenario: Public mocks and boundary markers are documented
- **WHEN** public test harnesses, mock provider types, or boundary marker types remain exposed for deterministic consumers
- **THEN** they have concise documentation explaining their intended public role rather than being left undocumented because they are simple

#### Scenario: Placeholder documentation is rejected
- **WHEN** public rustdoc is added to satisfy the missing-docs lint
- **THEN** it is non-empty, factual, and not a placeholder such as `TODO`, `TBD`, `Docs TBD`, or a summary that only repeats the documented identifier name

### Requirement: Internal and generated exceptions are explicit
Generated code or intentionally internal-only Rust surfaces SHALL be excluded from missing-docs enforcement only through narrow, explicit allowances that make the exception visible to maintainers. Allowances SHALL use the smallest practical item or module scope, SHALL NOT be applied at the crate root, SHALL include an in-source reason, and SHALL be listed in `openspec/changes/enforce-public-api-documentation/artifacts/exceptions.md` with file path, allowed item or module, scope, and justification.

#### Scenario: Internal parser surfaces are not silently ignored
- **WHEN** an internal raw parser or generated module is not useful as public API documentation
- **THEN** the implementation either keeps it outside the public API surface or applies a narrowly scoped allowance with an explanation of why missing docs are acceptable there

#### Scenario: Exceptions are auditable
- **WHEN** a `#[allow(missing_docs)]` exception remains after implementation
- **THEN** the exception appears in `openspec/changes/enforce-public-api-documentation/artifacts/exceptions.md` with an in-source justification and the smallest practical scope

#### Scenario: Broad suppression is rejected
- **WHEN** documentation enforcement is added
- **THEN** the workspace does not use a broad crate-wide or workspace-wide missing-docs allowance to hide undocumented public APIs

### Requirement: Documentation verification includes generated docs
The change SHALL be verified with formatting, checking, linting, tests, and warnings-as-errors rustdoc generation so missing docs and broken documentation links are caught before implementation is considered complete.

#### Scenario: Full verification passes
- **WHEN** implementation of this change is complete
- **THEN** `cargo fmt --all -- --check`, `cargo check --all-targets --all-features`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets --all-features`, and `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps` all pass

#### Scenario: Mise CI remains the main quality gate
- **WHEN** maintainers run `mise run ci` after missing-docs enforcement is enabled
- **THEN** the existing workspace quality gate passes without undocumented public APIs, rustdoc link regressions, or other warning regressions

#### Scenario: Documentation review checks product boundaries
- **WHEN** implementation of this change is reviewed
- **THEN** generated rustdoc is checked against `docs/vision.md`, `docs/architecture.md`, `openspec/specs/oxmux-core/spec.md`, and `openspec/specs/oxidemux-app-shell/spec.md` for boundary accuracy and factual current behavior
