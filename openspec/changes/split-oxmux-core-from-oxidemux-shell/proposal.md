## Why

OxideMux needs a reusable Rust core before proxy, provider, routing, dashboard, and desktop work grows around the current bootstrap binary. Splitting `oxmux` from the `oxidemux` app shell now keeps headless library consumers from depending on GPUI, desktop lifecycle, or a running local app process.

## What Changes

- Convert the project from a single binary crate into a Cargo workspace with separate `oxmux` and `oxidemux` members.
- Introduce `oxmux` as a headless, embeddable Rust library crate with an intentionally small public facade for future proxy lifecycle, provider, routing, protocol, configuration, management, usage, and error primitives.
- Move the existing app entrypoint into an `oxidemux` workspace member that consumes `oxmux` directly instead of owning core behavior.
- Add a direct library-use example or test demonstrating that Rust code can use `oxmux` without launching the desktop app, IPC server, or local proxy process.
- Update bootstrap tests and workspace verification so formatting, clippy, check, and tests run across both crates.
- Keep GPUI, tray, updater, packaging, platform credential storage, and desktop UI dependencies out of `oxmux`; those remain future `oxidemux` responsibilities.
- Defer provider implementations, routing algorithms, protocol translation engines, OAuth flows, streaming transports, quota dashboards, and GPUI component adoption to later changes.

## Capabilities

### New Capabilities

- `oxmux-core`: Defines the reusable headless core crate boundary, public API expectations, dependency limits, and direct embeddability requirements.
- `oxidemux-app-shell`: Defines the desktop app and integration shell boundary, including its dependency direction on `oxmux` and ownership of future UI/platform concerns.

### Modified Capabilities

- None.

## Impact

- Affects Cargo workspace structure, package manifests, source layout, bootstrap tests, CI commands, and development task assumptions.
- Establishes a public crate boundary for downstream Rust consumers and future app code, but does not commit to long-term stability of detailed provider/routing APIs yet.
- Keeps the initial dependency impact low by avoiding GPUI, provider SDKs, network runtimes, platform secret stores, and external reference repo code in this phase.
- Aligns OpenSpec planning with the product PRD in GitHub issue #1 and the existing `openspec/config.yaml` guidance to distinguish `oxmux` library requirements from `oxidemux` app requirements.
