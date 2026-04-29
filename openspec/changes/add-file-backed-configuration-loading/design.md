## Context

Issue #7 introduces the first durable configuration boundary for `oxmux`. Current specs already reserve configuration state, routing policy primitives, provider/account summaries, management snapshots, usage/quota summaries, and structured errors in the headless core, but no accepted spec defines an on-disk format or the validation path from a local file into those typed runtime surfaces.

This change should establish the smallest useful foundation: one deterministic local file format, typed validation, management snapshot reflection, and reload hook points. It should not absorb later concerns such as layered merge precedence, full filesystem watching, GPUI settings editing, launch-at-login persistence, remote configuration backends, OAuth credential persistence, or platform secret storage.

## Goals / Non-Goals

**Goals:**

- Define TOML as the first deterministic local configuration format for `oxmux`.
- Load a single local file into typed raw configuration, validate it, and produce runtime-ready configuration data.
- Validate listen address, port, provider references, routing defaults, logging, usage collection, and auto-start intent before updating app-visible state.
- Represent validation failures as structured `CoreError` details with stable codes and field paths.
- Update management snapshots from successfully loaded configuration and expose validation failures from failed loads.
- Define explicit reload hook points that accept already-read file contents or a caller-selected path, validate before replacement, preserve the previous valid configuration on failure, and leave active file-backed configuration absent on initial failure.

**Non-Goals:**

- No full hot-reload filesystem watcher.
- No default path discovery, layered configuration merge, bundled defaults merge, generated runtime view layering, or change fingerprinting beyond the replacement hook shape.
- No GPUI settings UI or app-shell editing flow.
- No launch-at-login implementation, service registration, tray lifecycle, or OS persistence.
- No OAuth credential storage, keychain/secret-service integration, raw secret persistence, provider token refresh, or raw secret values in file-backed configuration.
- No remote configuration backend such as Git, PostgreSQL, S3, cloud sync, or network config.

## Decisions

1. **Use TOML for the first file format.**
   - Rationale: TOML is deterministic, human-readable, common for Rust tools, and maps cleanly to `serde`-style typed structures without requiring YAML's broader implicit typing behavior.
   - Alternative considered: YAML aligns with CLIProxyAPI examples, but its implicit typing and richer syntax create more validation ambiguity for a first strict core contract.
   - Alternative considered: JSON is deterministic but less comfortable for hand-edited local configuration with comments absent.

2. **Separate raw file shape from validated runtime configuration.**
   - Rationale: The loader should preserve clear error reporting for missing or invalid fields while runtime code consumes normalized typed values only after validation succeeds.
   - Consequence: implementation will likely define raw deserializable structs, validated config structs, and conversion/validation functions rather than using deserialized values directly.

3. **Define the initial TOML shape explicitly instead of inferring it from Rust types.**
   - Rationale: OpenSpec should be the source of truth for the first user-owned configuration format so the parser, tests, docs, and future settings UI converge on the same table names and field names.
   - The initial format uses `version`, `[proxy]`, `[[providers]]`, nested provider-owned `[[providers.accounts]]`, `[[routing.defaults]]`, `[observability]`, and `[lifecycle]`.
   - Unknown fields are rejected at every level. Optional fields receive documented defaults before semantic validation runs.

4. **Validate references before publishing configuration.**
   - Rationale: Provider references and routing defaults influence user-visible routing outcomes; accepting unknown provider ids or route names would defer failures into unrelated proxy paths.
   - The validator should check that routing defaults reference declared providers/accounts/models and that provider/account identifiers are non-empty, unique where required, and match routing policy primitives.
   - Routing defaults are grouped by `(name, model)`; repeated groups define ordered candidates, and duplicate provider/account candidates in a group are invalid.

5. **Publish management snapshots only from valid configuration.**
   - Rationale: app and future CLI consumers need to distinguish active configuration from rejected configuration. A failed reload must surface structured validation errors while keeping the last valid snapshot stable.
   - Active configuration fields always reflect the last successfully validated configuration. Failed replacement errors are exposed separately as last-load failure metadata and are cleared by the next successful replacement. If the initial load fails, active file-backed configuration remains absent.
   - File-loaded provider/account summaries represent declarations only; auth health, subscription health, quota pressure, provider availability, and credential usability remain unknown until separate core state verifies them.

6. **Represent auto-start as core intent, not OS behavior.**
   - Rationale: users and the app shell need the desired intent visible in core snapshots, but platform-specific launch-at-login persistence belongs to `oxidemux` and later lifecycle issues.

7. **Define reload hook points without watcher ownership.**
   - Rationale: future layered config and watcher work needs an explicit seam, but issue #7 only needs deterministic load/validate/replace semantics. Hook points should be testable with in-memory contents and temporary files.
   - Replacement hooks accept one complete TOML document or path and atomically replace the active validated configuration. They do not merge layers, compute fingerprints, debounce, watch files, or own background tasks.

8. **Use a stable configuration error taxonomy.**
   - Rationale: app-shell display and tests need matchable categories instead of display-string parsing.
   - Errors include a kind, field path, invalid-value category, and source metadata when available. Invalid protocol families, credential references, public bind addresses, duplicate routing candidates, and initial load failures have explicit coverage.

9. **Treat credential references as opaque pointers, not secrets.**
   - Rationale: `oxmux` needs stable account declarations for routing and snapshots without owning platform secret storage or leaking credentials through config files, errors, or management output.
   - Credential references are validated for presence and shape, but raw tokens and secret-store payloads are out of scope for the file format.

## Risks / Trade-offs

- **TOML diverges from CLIProxyAPI YAML examples** → Document TOML as the OxideMux first format and keep the loader extensible enough for future formats if accepted by OpenSpec.
- **Provider reference validation may overfit placeholder provider state** → Validate only identity/reference consistency, routing eligibility, and credential-reference shape, not real auth, network availability, quota state, subscription health, or credential health.
- **Logging and usage settings could become runtime side effects** → In this change, treat them as validated typed settings and snapshot fields only; actual tracing subscriber or analytics persistence rewiring remains separate.
- **Reload hook points may look like full hot reload** → Name and document them as explicit replacement APIs with no filesystem watcher, debounce, or background task ownership.
- **Structured errors can become stringly if underspecified** → Require stable reason codes and field paths in specs and tests.
- **Schema examples can become stale if implementation-only fields are added** → Keep the OpenSpec schema and tests authoritative; implementation-only extensions require a later OpenSpec update.

## Migration Plan

1. Add file-configuration types and validation behind the `oxmux` facade without changing existing runtime behavior.
2. Add management snapshot construction/update paths that can consume validated configuration.
3. Add tests for valid configuration, invalid listen address/port/routing/provider references, logging/usage/auto-start validation, and failed replacement preserving the last valid snapshot.
4. Keep existing in-memory configuration constructors available for tests and callers that do not use file loading yet.
5. Rollback is straightforward because this adds a new capability and does not require existing callers to load configuration files.

## Open Questions

- Exact public type names should be chosen during implementation to align with existing `oxmux` module naming.
