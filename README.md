# OxideMux

OxideMux is a cross-platform Rust workspace that will become a GPUI-based
local AI subscription proxy and multiplexer inspired by VibeProxy.

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
  small identity facade plus placeholder domain boundaries for future provider,
  routing, protocol, configuration, management, usage, streaming, and error
  ownership.
- `crates/oxidemux`: App and integration shell. It owns the binary entrypoint
  and will own future GPUI, tray/background lifecycle, updater, packaging,
  platform credential storage, and IDE adapter work.
- `rust-toolchain.toml`: Pinned Rust 1.95.0 toolchain.
- `LICENSE-MIT` and `LICENSE-APACHE`: Dual-license files.
- `.github/workflows/ci.yml`: Multi-platform verification workflow.

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

# Configure git hooks
git config --local core.hooksPath .hk-hooks

# Run quality checks
mise run ci
hk validate
hk check --all
```

The `mise run ci` task runs workspace-wide formatting, checking, clippy, and
tests across both `oxmux` and `oxidemux`.

## Project Direction

OxideMux aims to provide a high-performance, native desktop experience for
managing AI subscriptions. By using [GPUI](https://www.gpui.rs/) as our
foundation, we intend to deliver a responsive and resource-efficient interface
across Linux, macOS, and Windows. Platform support will be validated and
refined during the UI implementation phase.

## Inspiration and Attribution

The concept for OxideMux is inspired by
[VibeProxy](https://github.com/automazeio/vibeproxy). We appreciate the vision
behind that project. OxideMux is an independent, from-scratch implementation;
no code or wording has been copied from the original repository.

## License

MIT OR Apache-2.0

This project is dual-licensed under the [MIT License](LICENSE-MIT) OR
[Apache License, Version 2.0](LICENSE-APACHE).
