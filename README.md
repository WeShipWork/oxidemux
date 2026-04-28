# OxideMux

OxideMux is a cross-platform Rust workspace that will become a GPUI-based
local AI subscription proxy and multiplexer. Its proxy and gateway direction is
inspired by CLIProxyAPI, and its desktop control and monitoring direction is
inspired by zero-limit.

OxideMux exists because VibeProxy and zero-limit prove the subscription-aware
local proxy experience is valuable, but this project needs that experience with
first-class Linux, macOS, and Windows support. Subscription UX is the product
center: account health, auth state, quota pressure, provider availability,
routing choices, and recovery paths should be visible and controllable from one
native app while remaining testable through a shared headless core.

## Status

### Completed

- Repository bootstrap and minimal Rust 1.95.0 foundation.
- Two-member Cargo workspace split between the `oxmux` core crate and the
  `oxidemux` app shell crate.
- Dual licensing with MIT and Apache 2.0.
- Cargo manifest and toolchain configuration.
- Minimal cargo-verifiable source and test stubs.
- Reproducible development baseline with mise and hk configured for local checks.
- CI baseline for Rust verification across Linux, macOS, and Windows.
- README roadmap and project vision documentation.

### Upcoming

- Local proxy server implementation.
- Provider authentication and account management.
- Multi-account routing and provider failover.
- GPUI-based desktop shell and user interface.
- Configuration management and hot reloading.
- Tooling integrations and developer experience.
- Automated packaging, updates, and distribution.

## Planned Features

### Local Proxy

- High-performance local proxy server for AI API requests.
- Protocol compatibility with major AI provider interfaces.

### Provider Authentication

- Secure handling of multiple provider API keys.
- OAuth integration for supported providers.

### Multi-Account Routing

- Dynamic routing of requests across multiple accounts.
- Intelligent account rotation to manage rate limits.

### Provider Priority and Failover

- Priority-based provider selection logic.
- Automatic failover to secondary providers on failure.

### GPUI Desktop Shell

- Native desktop interface built with GPUI.
- System tray and menu bar integration for quick access.

### Configuration and Hot Reload

- Flexible configuration system for providers and routing rules.
- Hot reload support for configuration changes without restart.

### Tool Integrations

- Integration with popular developer tools and IDEs.
- CLI for headless management and automation.

### Packaging and Updates

- Native installers for Linux, macOS, and Windows.
- Automated update mechanism for seamless improvements.

## Workspace Layout

- `Cargo.toml`: Workspace manifest for all repository crates.
- `crates/oxmux`: Headless reusable Rust core boundary. It currently exposes a
  small identity facade plus domain boundaries for provider execution, routing,
  protocol, configuration, management, usage, streaming, and error ownership.
  It is the home for subscription-aware routing, request rewrite primitives,
  model aliases, reasoning/thinking compatibility behavior, and local proxy
  contracts that must work without a desktop shell.
- `crates/oxidemux`: App and integration shell. It owns the binary entrypoint
  and will own future GPUI, tray/background lifecycle, updater, packaging,
  platform credential storage, subscription onboarding/repair UI, and IDE
  adapter work.
- `docs/vision.md`: Canonical product intent for humans and agents.
- `docs/architecture.md`: Crate boundary and architecture guardrails.
- `rust-toolchain.toml`: Pinned Rust 1.95.0 toolchain.
- `LICENSE-MIT` and `LICENSE-APACHE`: Dual-license files.
- `.github/workflows/ci.yml`: Multi-platform verification workflow.
- `CONTRIBUTING.md`: Contributor setup, OpenSpec workflow, and PR hygiene
  guidance.
- `CHANGELOG.md`: Notable project changes following Keep a Changelog format.

## Current Bootstrap Status

- The `oxmux` crate can be used directly by Rust code without launching the app,
  opening a window, starting IPC, or binding a local proxy server.
- The `oxidemux` binary remains a minimal metadata stub and consumes shared core
  identity data from `oxmux`.
- Workspace tests verify package metadata, direct core use, binary output, and
  the initial core dependency boundary.

## What Is Not Implemented Yet

- Proxy server logic and request handling.
- Provider authentication and credential storage.
- Multi-account routing and rotation logic.
- GPUI windows, views, or rendering.
- System tray or menu bar components.
- Setup guides or end-user documentation.
- Update mechanism or packaging scripts.

## Development

### Prerequisites

- Rust 1.95.0
- [mise](https://mise.jdx.dev/) for tool management
- [hk](https://github.com/vibe-sh/hk) for git hook management

### Setup

```bash
# Install and trust tools
mise install
mise trust

# Install git hooks
mise run hk-install

# Run quality checks
mise run ci
mise run hk-check
```

The `mise run ci` task runs workspace-wide formatting, checking, clippy, and
tests across both `oxmux` and `oxidemux`. The `mise run hk-check` task runs the
repository hook checks without requiring contributors to remember the underlying
hk command.

See `CONTRIBUTING.md` for the OpenSpec-first development workflow, PR template
expectations, and the reason CI uses the same mise task graph as local
verification.

## Project Direction

OxideMux aims to provide a high-performance, native desktop experience for
managing AI subscriptions. By using [GPUI](https://www.gpui.rs/) as our
foundation, we intend to deliver a responsive and resource-efficient interface
across Linux, macOS, and Windows. Platform support will be validated and
refined during the UI implementation phase.

The shared `oxmux` core owns behavior that must be consistent across platforms:
protocol compatibility, request rewriting, model aliasing, reasoning/thinking
options, subscription-aware routing, provider/account state, usage/quota
summaries, and structured failures. The `oxidemux` app shell adapts that core to
GPUI, tray/menu integration, notifications, platform credential storage,
packaging, updates, and OS-specific lifecycle UX.

## Inspiration and Attribution

OxideMux is an independent, from-scratch implementation. Its proxy/gateway
concepts are informed by
[CLIProxyAPI](https://github.com/router-for-me/CLIProxyAPI), especially provider
auth boundaries, API compatibility, routing, model aliases, fallback, streaming,
and management endpoints. Its desktop concepts are informed by
[zero-limit](https://github.com/0xtbug/zero-limit), especially quota monitoring,
proxy lifecycle controls, tray/background operation, themes, and updates.

The original product idea also drew inspiration from
[VibeProxy](https://github.com/automazeio/vibeproxy), especially its
subscription-first local proxy UX, auth/session handling, model aliases,
provider-specific routing, and thinking/reasoning request compatibility layer.
No code or wording has been copied from any inspiration project.

## License

MIT OR Apache-2.0

This project is dual-licensed under the [MIT License](LICENSE-MIT) OR
[Apache License, Version 2.0](LICENSE-APACHE).
