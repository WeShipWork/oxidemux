## 1. Layered configuration model

- [x] 1.1 Define public `oxmux` layer types for bundled defaults, user-owned inputs, layer source metadata, active fingerprints, and reload outcomes.
- [x] 1.2 Add deterministic effective-runtime fingerprint computation for normalized merged configuration without using filesystem metadata, watcher events, formatting, comments, or platform-specific state.
- [x] 1.3 Implement layered raw configuration parsing that permits partial layers while preserving strict unknown-field rejection and structured parse/source errors.
- [x] 1.4 Implement merge helpers for explicitly present scalar settings, provider declarations, account declarations, and user-owned routing-list replacement according to the design precedence rules.

## 2. Validation and replacement semantics

- [x] 2.1 Convert merged layered candidates into the existing validated runtime configuration shape and reuse current semantic validation/error taxonomy.
- [x] 2.2 Add layered replacement hooks that return unchanged, replaced, or rejected outcomes and preserve the last valid active configuration on failure.
- [x] 2.3 Ensure initial invalid layered loads leave active configuration absent while preserving failed-load diagnostics.
- [x] 2.4 Ensure rejected outcomes include candidate diagnostics and previous active fingerprint when present.
- [x] 2.5 Keep existing single-file `load_file`, `load_contents`, and replacement APIs behavior-compatible with current tests.

## 3. Facade and management integration

- [x] 3.1 Export layered configuration input, source metadata, fingerprint, state, and reload outcome types through the public `oxmux` facade.
- [x] 3.2 Extend management snapshot construction with active layered fingerprint, layer source summaries, latest reload outcome, and failed candidate diagnostics.
- [x] 3.3 Ensure management-visible provider/account summaries from layered configuration remain auth-unverified, quota-unknown, subscription-unverified, and free of raw credential-reference values.

## 4. Tests and verification

- [x] 4.1 Add tests proving bundled defaults fill missing user fields and user-owned scalar values override bundled defaults.
- [x] 4.2 Add tests proving user-owned provider/account declarations survive bundled default updates and matching identities merge deterministically.
- [x] 4.3 Add tests proving invalid merged candidates return structured errors and preserve prior active configuration.
- [x] 4.4 Add tests proving unchanged effective-runtime fingerprints, including syntactic-only TOML changes, return an unchanged outcome and do not publish spurious replacements.
- [x] 4.5 Add tests proving management snapshots expose active layered metadata, preserve failed reload diagnostics separately, and clear diagnostics after successful replacement.
- [x] 4.6 Add a boundary verification proving `oxmux` remains free of GPUI, app-shell, watcher, platform secret-store, provider SDK, remote config, and database dependencies.
- [x] 4.7 Run `openspec validate add-layered-configuration-hot-reload --strict`.
- [x] 4.8 Run `mise run ci`.
