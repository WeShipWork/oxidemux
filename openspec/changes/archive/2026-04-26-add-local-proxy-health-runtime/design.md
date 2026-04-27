## Context

Issue #2 asks for the first user-testable OxideMux runtime slice: a minimal local proxy health server that binds a configurable localhost endpoint, reports lifecycle state through `oxmux`, and can be exercised without GPUI, providers, OAuth, routing, platform credential storage, or desktop lifecycle code. The current `oxmux` facade already defines management snapshots, configuration snapshots, lifecycle states, endpoint metadata, health state, and structured errors, but it is intentionally runtime-inert.

This change turns that inert facade into a small deterministic runtime boundary. It should provide enough real behavior for later protocol translation, GPUI compatibility, and status UI work to depend on, while avoiding early commitments to provider clients, request routing, streaming transports, or app-owned desktop concerns.

## Goals / Non-Goals

**Goals:**

- Add a headless `oxmux` runtime type that can bind a local loopback HTTP listener from deterministic configuration.
- Serve a stable health endpoint, such as `GET /health`, with predictable status and response content suitable for smoke tests.
- Report startup, successful bind, bind failure, shutdown, and stopped state through typed lifecycle/status facade values.
- Provide explicit shutdown behavior that releases the listener and lets tests verify cleanup without leaked background tasks.
- Keep `oxmux` free of GPUI, tray, updater, packaging, platform credential storage, OAuth, provider client, and `oxidemux` dependencies.

**Non-Goals:**

- Implement OpenAI, Gemini, Claude, Codex, or provider-specific protocol translation.
- Add provider authentication, token refresh, quota fetching, routing, fallback, model aliases, streaming, or request forwarding.
- Add GPUI windows, status widgets, tray/menu-bar lifecycle, update logic, native packaging, platform secret stores, or long-lived daemon installation.
- Expose a public management HTTP API beyond the minimal health response needed for smoke testing.

## Decisions

### Decision: Implement the runtime inside `oxmux`, not `oxidemux`

The local health listener belongs to `oxmux` because it is reusable proxy runtime behavior, while `oxidemux` remains the app/integration shell.

- **Rationale:** Downstream issues depend on a headless runtime substrate. Keeping startup, bind failure, health response, lifecycle state, and shutdown in `oxmux` lets Rust consumers test the runtime without launching the desktop app.
- **Alternative considered:** Start the listener from `oxidemux` only. That would satisfy a binary smoke test but would make the app shell own core proxy lifecycle behavior and duplicate the facade boundary.

### Decision: Use deterministic local configuration with loopback-only binding

The runtime should accept an explicit listen address and port, validate that it targets a local endpoint, and allow tests to request an OS-assigned port where needed.

- **Rationale:** Issue #2 requires a configurable localhost endpoint, not public network exposure. Loopback-only binding minimizes security risk and avoids premature external proxy configuration.
- **Alternative considered:** Bind all interfaces for future remote clients. That is unsafe for a first runtime slice and would need authentication and access-control decisions that are out of scope.

### Decision: Keep the health protocol stable and minimal

`GET /health` should return a successful HTTP status and stable body that identifies the runtime as healthy without exposing provider, account, quota, routing, or credential information.

- **Rationale:** Smoke tests need a deterministic response. A small response reduces compatibility surface while still proving listener accept, route handling, and lifecycle wiring.
- **Alternative considered:** Add a richer `/status` management endpoint now. That would be useful for the app, but it risks designing the broader management API before provider/account/routing data is real.

### Decision: Make lifecycle transitions observable through existing facade concepts

Startup should move through starting to running on successful bind, failed on bind error, and stopped after shutdown. The running state should include the bound endpoint, and failed state should preserve structured error data.

- **Rationale:** Reusing the existing lifecycle facade proves that the earlier management work can describe real runtime behavior without app-specific status models.
- **Alternative considered:** Return ad hoc runtime booleans. That would be simpler to implement but would bypass the typed facade and make future UI/status work harder.

### Decision: Keep runtime dependencies minimal and Rust-native

The implementation may use a small Rust HTTP/runtime dependency only if it remains headless and compatible with library consumers. Any added dependency must avoid desktop UI, provider SDK, credential storage, updater, packaging, or app-shell concerns.

- **Rationale:** A health listener needs asynchronous or threaded IO, but dependency growth should stay proportional to the minimal runtime slice.
- **Alternative considered:** Hand-roll HTTP parsing on `std::net`. That minimizes dependencies but adds fragile protocol code and distracts from lifecycle correctness.

## Risks / Trade-offs

- **[Risk] The minimal listener becomes a public proxy API too early** → Limit the route surface to health behavior and keep protocol translation/routing explicitly out of scope.
- **[Risk] Bind behavior is flaky in tests** → Use deterministic loopback configuration, OS-assigned ports for success tests, and an already-bound port for bind-failure tests.
- **[Risk] Background runtime tasks leak after tests** → Require explicit shutdown and tests that prove stopped state and listener release.
- **[Risk] `oxmux` gains app or desktop dependencies** → Add verification tasks that inspect manifests and run workspace checks.
- **[Risk] Health response shape changes unintentionally** → Specify a stable response contract and test exact response semantics needed for smoke testing.

## Migration Plan

1. Add runtime configuration and a local health runtime abstraction to `oxmux` using existing configuration/lifecycle/error concepts where possible.
2. Implement startup so bind success records the actual local endpoint and bind failure records a structured failed lifecycle state.
3. Implement `GET /health` with stable response semantics and no provider, routing, credential, quota, or UI dependencies.
4. Implement explicit shutdown and stopped-state reporting.
5. Add `oxmux` tests for successful bind, health request, bind failure, lifecycle status, and shutdown.
6. Optionally add a minimal `oxidemux` smoke path if needed to prove app-shell consumption without moving runtime ownership out of `oxmux`.
7. Run formatting, clippy, check, tests, and `mise run ci`.

Rollback is straightforward while this remains isolated to the first runtime slice: remove the runtime facade, health listener code, and tests, restoring the existing inert management/lifecycle facade.

## Open Questions

- Should the health response body be plain text for maximum simplicity or JSON to prepare for future management endpoints?
- Should the public runtime API expose async methods directly, or should it hide runtime mechanics behind synchronous facade methods plus handles?
- Should `oxidemux` exercise the runtime in its binary path now, or should app-shell consumption remain limited to tests until a status UI exists?
