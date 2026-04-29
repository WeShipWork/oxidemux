## Context

Issue #7 established deterministic single-file TOML loading in `oxmux`: callers choose a path or pass already-read contents, the core parses and validates a whole document, successful replacements publish a new active configuration, and failed replacements preserve the last valid state. That archived design intentionally deferred bundled defaults, user-owned overrides, generated runtime views, fingerprinting, and watcher behavior.

Issue #17 takes the next step. The core needs deterministic layered semantics so bundled product defaults and user-owned configuration can be combined without making `oxidemux` or future CLI consumers duplicate merge, validation, and reload decisions. The desktop app can later watch files, debounce platform events, and display notifications, but it should call a headless `oxmux` reload hook with already-read layer contents and receive a stable outcome.

## Goals / Non-Goals

**Goals:**

- Define layered configuration as an `oxmux` core capability with bundled defaults below user-owned overrides.
- Produce one validated merged runtime configuration before publishing state or updating management snapshots.
- Preserve existing single-file loading and replacement APIs for current tests and consumers.
- Preserve user-owned provider/account declarations when bundled defaults change.
- Add deterministic fingerprints and reload outcomes so callers can distinguish unchanged, replaced, and rejected candidates.
- Keep reload behavior testable with in-memory contents and without filesystem watcher timing.
- Expose layered configuration metadata through the public core facade and management snapshots.

**Non-Goals:**

- No full filesystem watcher, debounce task, background reload worker, tray notification, or GPUI settings editor in `oxmux`.
- No cloud, database, Git, S3, network, or remote model registry configuration backend.
- No OAuth credential persistence, platform secret-store integration, raw token storage, or credential refresh behavior.
- No write-back of merged configuration into the user-owned file.
- No arbitrary unknown TOML pass-through or new custom settings section in this change; custom settings require a later typed schema proposal.
- No changes to provider execution, routing selection, protocol translation, or local proxy runtime semantics beyond consuming the merged runtime configuration they already understand.

## Decisions

1. **Add layered state beside single-file state rather than weakening `FileConfigurationState`.**
   - Rationale: existing tests and consumers rely on whole-document replacement semantics. A new layered state or layered replacement API can reuse parse/validate/publish behavior without changing the meaning of `replace_from_contents`.
   - Alternative considered: mutate `FileConfigurationState` into a layered-only type. Rejected because it creates avoidable migration risk for current single-file callers.

2. **Treat bundled defaults as lowest precedence and user-owned configuration as highest precedence.**
   - Rationale: bundled defaults should make the product usable, while user-owned files remain authoritative for local proxy choices.
   - Alternative considered: allow arbitrary layer priority. Rejected for this foundation because two named layer kinds cover the issue and keep merge tests deterministic.

3. **Merge provider/account collections by stable identity and treat user-owned routes as an ordered list override.**
   - Rationale: issue #17 requires preserving user-owned provider settings. Whole-array replacement would let bundled defaults erase user providers or accounts whenever defaults change, while route ordering is user-visible and easier to reason about as one ordered list.
   - Proposed rule: scalar fields use the highest-precedence explicitly present value; explicit `false`, `off`, and `disabled` values count as present values, while empty strings remain invalid where the schema forbids them. Provider entries merge by provider `id`. Account entries merge by `(provider id, account id)`. If the user-owned layer declares any `routing.defaults`, that full ordered route list replaces bundled default routes; otherwise bundled default routes remain in effect. Arbitrary custom TOML settings are not accepted in this change.
   - Alternative considered: replace arrays wholesale. Rejected because it violates preservation requirements.

4. **Validate only the merged runtime candidate before publish.**
   - Rationale: user layers may intentionally omit fields supplied by bundled defaults. The final runtime view must satisfy the existing strict configuration contract before it becomes active.
   - Alternative considered: require each layer to be independently valid. Rejected because partial override files would be impossible.

5. **Use a normalized effective-runtime fingerprint to decide unchanged reloads.**
   - Rationale: file mtimes, watcher events, TOML whitespace, comments, and table ordering are noisy and platform-specific. The reload decision should answer whether the effective validated runtime configuration changed, not whether bytes were rewritten.
   - Proposed rule: compute the active fingerprint from the normalized merged runtime configuration after successful validation, with deterministic ordering for collections. `Unchanged` is returned when the candidate validates and its effective-runtime fingerprint matches the active fingerprint. Rejected candidates MAY include a candidate fingerprint only when parsing/merging progressed far enough to compute one, but rejection diagnostics remain authoritative.
   - Alternative considered: fingerprint raw ordered layer bytes. Rejected because semantically equivalent formatting changes would trigger spurious reloads.

6. **Return explicit reload outcomes instead of sending notifications from `oxmux`.**
   - Rationale: `oxmux` has no app event loop or UI. A result enum such as `Unchanged`, `Replaced`, and `Rejected` gives `oxidemux`, CLI, and embedded callers enough information to notify, poll, or ignore.
   - Proposed rule: `Rejected` includes candidate source summaries, structured parse/merge/validation errors, the current active fingerprint when one exists, and a candidate fingerprint only when available.
   - Alternative considered: callback subscriptions in the core. Deferred until a concrete app-shell consumer proves it needs push-based behavior.

7. **Expose management metadata without implying provider health.**
   - Rationale: loaded configuration declares providers/accounts and routing intent; it does not verify auth, quota, subscription, provider availability, or credential usability.
   - Alternative considered: mark configured providers as available after merge. Rejected because it would conflict with subscription-aware UX truthfulness.

## Risks / Trade-offs

- **Merge semantics become surprising** → Keep rules small and covered by scenario tests for scalars, provider/account identity merges, and user-owned routing list replacement.
- **Partial layer parsing conflicts with strict unknown-field validation** → Use a raw layer representation that allows missing fields but still rejects unknown supported-level fields before merge; validate required fields after merge.
- **Fingerprinting either reloads too often or hides important changes** → Use normalized effective-runtime fingerprints for reload decisions and test unchanged, changed, syntactic-only, and rejected cases.
- **Management snapshots become noisy** → Expose compact source metadata and active fingerprint, not full raw layer contents or credential references.
- **Core file grows too large** → Keep layering helpers in a focused configuration module if `file.rs` approaches the project’s 800-line guardrail.
- **User-owned custom settings are ambiguous** → Defer arbitrary custom settings to a later typed schema rather than weakening strict schema behavior.

## Migration Plan

1. Add layered configuration types and merge/fingerprint helpers while leaving existing single-file APIs intact.
2. Add layered replacement hooks that parse layers, merge a candidate, validate the merged runtime view, and publish only on success.
3. Extend management snapshot construction with layered metadata and reload outcome data.
4. Add deterministic in-memory tests for precedence, preservation, validation failure, unchanged fingerprint behavior, and management output.
5. Roll back by keeping single-file configuration code paths unchanged and removing the new layered APIs before adoption by `oxidemux`.

## Open Questions

- Should a later app-shell settings editor expose route-list replacement directly, or provide a higher-level route editing model that writes the ordered user-owned list?
- Should a future typed custom settings capability be introduced for provider-specific metadata after provider execution requirements are more mature?
