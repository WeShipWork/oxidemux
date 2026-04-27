## Context

OxideMux now has a two-member Rust workspace: `oxmux` is the headless reusable core crate and `oxidemux` is the app and integration shell that depends on it. The archived baseline specs reserve ownership for proxy lifecycle, configuration, provider/auth, management/status, usage/quota, and error boundaries, but the current implementation only exposes identity metadata and placeholder boundary structs.

The next product decision is shaped by the first consumer. Since `oxidemux` will consume `oxmux` before third-party Rust agents do, the first real core facade should answer app-shell questions: what is the proxy lifecycle state, what configuration is visible, which providers/accounts exist, what status or degraded conditions should be shown, and which control intents can the app invoke. CLIProxyAPI is useful as proxy/gateway vocabulary, especially management/status, auth manager, model aliases, fallback, and provider executors. zero-limit is useful as app-shell vocabulary, especially dashboard, provider/account status, quota cards, settings, and one-click proxy control. This change uses those concepts without copying code or introducing their full feature set.

## Goals / Non-Goals

**Goals:**

- Define a small typed `oxmux` management/lifecycle facade that `oxidemux` can consume directly.
- Represent proxy lifecycle state, health, bound endpoint metadata, uptime, warnings, errors, and degraded conditions without starting a real proxy server.
- Represent app-visible configuration snapshots and update intents for listen address, auto-start intent, logging, usage collection, and routing defaults.
- Represent provider/account summaries, auth state, provider capabilities, quota/status placeholders, and degraded/error reasons without implementing OAuth or provider network calls.
- Keep errors structured and visible so app and library consumers can display failures instead of silently discarding them.
- Add tests for direct Rust use and app-shell consumption of the facade.

**Non-Goals:**

- Implementing HTTP proxy binding, `/v1/*` compatibility endpoints, streaming transports, protocol translators, model execution, routing algorithms, fallback schedulers, OAuth, token refresh, keychain/secret-service integrations, quota analytics, hot reload, GPUI views, tray lifecycle, updater, packaging, or IDE adapters.
- Adding GPUI, gpui-component, Tauri, web UI, platform credential storage, or desktop lifecycle dependencies to `oxmux`.
- Copying CLIProxyAPI, zero-limit, VibeProxy, or any upstream implementation details.
- Splitting `oxmux` into additional crates before the app-facing facade proves useful.

## Decisions

### Decision: Make management/status the first real core facade

`oxmux` will expose a typed management snapshot and lifecycle control surface before implementing concrete provider execution.

- **Rationale:** `oxidemux` is the first consumer, and desktop value depends on reliable status/control data. zero-limit shows that users need running/healthy state, provider/account status, settings, quota cards, logs, and start/stop control before dashboard polish matters.
- **Alternative considered:** Start with mock provider execution traits. This helps library tests, but it does not give the app shell enough product-shaped state to consume.
- **Alternative considered:** Start with a local HTTP proxy server. This proves runtime behavior, but it hardens endpoint and protocol decisions before configuration, lifecycle, and status semantics are clear.

### Decision: Use inert snapshots and control intents in this change

Lifecycle APIs will describe and validate state/control semantics, but they will not bind ports, spawn background servers, or call external providers.

- **Rationale:** The facade can be tested deterministically and can support `oxidemux` integration without introducing async runtime, networking, process lifecycle, or OS-specific behavior too early.
- **Alternative considered:** Start a no-op local server. That would exercise more runtime code, but it would blur the line between status facade and proxy implementation.

### Decision: Keep provider/account status separate from credential storage

Provider/account types will describe provider ids, display names, capabilities, auth state, quota/status placeholders, and degraded reasons. Platform credential storage and OAuth flows remain app/platform work or later core abstractions.

- **Rationale:** CLIProxyAPI and zero-limit both show provider/account state is central to UX, but OxideMux must keep `oxmux` independent from desktop secret stores and OAuth UI.
- **Alternative considered:** Store credentials directly in `oxmux`. This would make the app prototype faster but violate the baseline credential boundary and make non-desktop consumers inherit platform assumptions.

### Decision: Make protocol/routing compatibility visible but not executable

Configuration and provider capability types will mention future protocol families and routing defaults, but this change will not translate OpenAI, Gemini, Claude, Codex, or provider-specific requests.

- **Rationale:** The OpenSpec project rules require protocol and routing semantics to be explicit. Representing them as configuration/status vocabulary now prevents the app shell from inventing its own names.
- **Alternative considered:** Implement protocol translators immediately. This would skip necessary provider/auth and lifecycle groundwork.

## Risks / Trade-offs

- **[Risk] The facade becomes a second placeholder layer** → Require tests that construct realistic snapshots and require `oxidemux` to consume them directly.
- **[Risk] API names imply long-term stability too early** → Keep the facade small, focused on current app-shell needs, and avoid broad trait hierarchies or provider SDK abstractions.
- **[Risk] Runtime implementation later conflicts with inert lifecycle types** → Model lifecycle states and control intents around observable states rather than implementation mechanics.
- **[Risk] App-shell concerns leak into `oxmux`** → Maintain dependency-boundary tests and keep GPUI, tray, updater, packaging, keychain, and platform lifecycle code out of the core crate.
- **[Risk] Protocol/routing vocabulary is too vague for future endpoints** → Include protocol family, provider capability, routing default, degraded, and quota placeholder scenarios in specs so later proxy work has stable language.

## Migration Plan

1. Add typed management, lifecycle, configuration, provider/account, usage/quota, and error structures inside existing `oxmux` modules.
2. Re-export the minimal public facade through `oxmux.rs`.
3. Update `oxidemux` bootstrap behavior to read and display or otherwise verify core management/status data through `oxmux`.
4. Add direct core tests and app-shell tests for snapshots, lifecycle states, provider/account summaries, and structured errors.
5. Run workspace formatting, clippy, check, tests, and `mise run ci`.

Rollback is straightforward while this remains type-level and deterministic: remove the new facade types/tests and restore the previous identity-only app shell behavior.

## Open Questions

- Should lifecycle control methods be synchronous for now, or should the public shape reserve async behavior before a runtime is chosen?
- Should configuration snapshots use plain Rust structs only, or introduce serde in this change to prepare for future file-backed configuration?
- How much provider capability vocabulary is enough now without prematurely committing to the full CLIProxyAPI compatibility matrix?
