## 1. Configuration schema and dependencies

- [x] 1.1 Add lightweight TOML/Serde support to `crates/oxmux` while preserving the core dependency boundary and avoiding GPUI, app-shell, provider SDK, OAuth, platform secret-store, watcher, remote config, or database dependencies.
- [x] 1.2 Define raw deserializable TOML configuration structures with strict unknown-field rejection, kebab-case field names, explicit defaults, and a version field without accepting a separate format field.
- [x] 1.3 Define validated runtime configuration structures or conversions that keep raw parsed data separate from runtime-ready configuration state.
- [x] 1.4 Add a canonical valid TOML fixture matching the OpenSpec schema and keep it synchronized with parser tests.

## 2. Loader, validation, and structured errors

- [x] 2.1 Implement local TOML loading from a path and from in-memory contents for deterministic tests.
- [x] 2.2 Validate loopback-only listen address and port values before runtime use, returning structured configuration errors for invalid fields and rejecting wildcard/public binds.
- [x] 2.3 Validate provider and account declarations for non-empty identifiers, duplicate identities, accepted protocol families, opaque non-secret credential references, and reference consistency.
- [x] 2.4 Validate routing defaults against declared provider/account references and existing typed routing policy primitives, including `(name, model)` candidate grouping, file-order preservation, fallback scope, provider-only candidates, and duplicate candidate rejection.
- [x] 2.5 Validate logging, usage collection, and auto-start intent as typed settings without reconfiguring runtime logging, analytics persistence, launch-at-login services, or app-shell lifecycle.
- [x] 2.6 Map parse, read, unsupported-format, and semantic validation failures into matchable `CoreError` configuration data with stable reason codes and field paths.
- [x] 2.7 Implement the configuration error taxonomy and dotted field-path grammar from the spec, including parse/read source metadata where available.

## 3. Public facade and management integration

- [x] 3.1 Export file-backed configuration loading, validation, replacement hook, and error types through the `oxmux` public facade.
- [x] 3.2 Update management snapshot construction so a successfully loaded configuration populates source metadata, listen settings, routing defaults, provider/account declaration summaries, logging setting, usage collection setting, auto-start intent, warnings, and errors without implying verified auth, quota, subscription, provider availability, or credential health.
- [x] 3.3 Implement explicit configuration replacement hook points that validate candidate contents before publishing, preserve the last valid active configuration on failure, and leave active file-backed configuration absent on initial failure.
- [x] 3.4 Keep existing in-memory configuration constructors available for tests and consumers that do not load files.
- [x] 3.5 Ensure failed replacement metadata is exposed separately from active configuration and cleared after the next successful replacement.

## 4. Tests and verification

- [x] 4.1 Add valid TOML configuration tests covering proxy listen settings, provider references, routing defaults, logging, usage collection, and auto-start intent.
- [x] 4.2 Add invalid TOML/unsupported-format/missing-file tests covering structured read and parse errors.
- [x] 4.3 Add validation tests for invalid listen address, public/wildcard bind rejection, invalid port, invalid protocol family, invalid credential reference, unknown provider reference, invalid routing default, duplicate routing candidate, duplicate provider/account identity, invalid logging setting, invalid usage collection setting, and invalid auto-start intent.
- [x] 4.4 Add management snapshot tests proving valid file configuration updates app-visible snapshot fields without marking file-loaded provider/account declarations as auth-verified, quota-healthy, subscription-healthy, provider-available, or credential-usable.
- [x] 4.5 Add replacement hook tests proving failed reload attempts preserve the last valid configuration, initial load failure leaves active file-backed configuration absent, and structured validation errors remain exposed separately.
- [x] 4.6 Add fixture tests for unknown fields, unsupported non-`.toml` path, missing required fields, invalid version, parse failure, duplicate provider id, duplicate account id, invalid provider protocol family, invalid credential reference, unknown provider reference, unknown account reference, invalid routing default, duplicate routing candidate, and successful replacement clearing failed-load metadata.
- [x] 4.7 Run `openspec validate add-file-backed-configuration-loading --strict`.
- [x] 4.8 Run `mise run ci`.
