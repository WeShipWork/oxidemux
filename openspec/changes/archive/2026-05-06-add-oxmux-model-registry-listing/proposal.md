## Why

Issue #26 needs `oxmux` to expose a typed model registry and listing contract before provider adapters, reasoning controls, and app-shell model pickers depend on model metadata. Existing routing, provider, protocol, and file-configuration primitives can identify configured providers, provider accounts, model routes, aliases, protocol families, routing eligibility, streaming capability, and degraded state, but there is not yet a single headless catalog that answers "what models can this core configuration offer?"

The registry must support provider access UX without becoming live provider discovery. Clients and the future `oxidemux` shell should consume deterministic `oxmux` registry data instead of inventing endpoint-only `/v1/models` semantics, duplicating app-owned model lists, scraping providers, or making network calls during model listing.

## What Changes

- Add a headless `oxmux` model registry/listing contract for configured model entries, provider-native model targets, aliases, forks, capability metadata, routing eligibility, disabled/degraded status, and provider/account applicability.
- Derive the first implementation from deterministic file-backed configuration, routing policy, provider declarations, provider/account summaries, and streaming capability metadata already owned by `oxmux`.
- Define how a future OpenAI-compatible `/v1/models` route serializes from the typed registry without making endpoint-specific model semantics the source of truth.
- Preserve routing-policy ownership: the registry describes and validates listing/eligibility metadata while routing selection remains in the routing policy primitives.
- Preserve scope boundaries: no provider scraping, remote model updater, real provider network calls, GPUI picker, app-shell model catalog, OAuth, platform credential storage, or provider SDK integration in this change.

## Capabilities

### New Capabilities

- `oxmux-model-registry`: Defines typed model registry entries, alias/fork metadata, provider/account applicability, listing filters, static config-backed registry construction, and future `/v1/models` serialization semantics for the reusable core.

### Modified Capabilities

- `oxmux-core`: Public core API surface SHALL expose model registry/listing primitives without introducing app-shell, provider SDK, outbound network, OAuth, credential storage, or UI dependencies.
- `oxmux-file-configuration`: Validated file-backed configuration SHALL provide deterministic inputs for static model registry entries and alias/fork listing tests.
- `oxmux-routing-policy`: Routing policy aliases, model routes, fallback eligibility, and provider/account targets SHALL remain the source of route-selection truth while supplying model registry metadata.
- `oxmux-management-lifecycle`: Management snapshots MAY surface model registry summaries in future changes, but this change only defines the reusable core model registry semantics unless explicitly scoped otherwise.

## Impact

- Affected crate: `crates/oxmux`.
- Affected specs: `oxmux-core`, new `oxmux-model-registry`, and focused deltas to `oxmux-file-configuration` and `oxmux-routing-policy`.
- Affected tests: new model registry tests plus targeted file-configuration/routing tests for static config-backed entries, alias preservation, fork behavior, disabled/degraded status, and streaming capability metadata.
- Deferred work: concrete HTTP `/v1/models` route wiring, live provider model discovery, model updater jobs, app-shell model picker UX, provider-specific scraping, and real provider network calls.
