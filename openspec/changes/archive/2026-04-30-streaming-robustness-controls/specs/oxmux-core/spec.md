## ADDED Requirements

### Requirement: Core exposes streaming robustness controls
The `oxmux` public facade SHALL expose streaming robustness policy, control metadata, retry outcome, timeout/cancellation, and structured stream error primitives needed by Rust consumers and tests without importing `oxidemux` or desktop-specific code.

#### Scenario: Rust consumers can configure streaming robustness through oxmux
- **WHEN** a Rust consumer imports the `oxmux` facade
- **THEN** it can construct and inspect streaming robustness policy values for keepalive interval, bootstrap retry count, timeout metadata, and cancellation behavior without depending on app-shell or provider SDK types

#### Scenario: Stream robustness failures are matchable
- **WHEN** stream robustness validation, retry exhaustion, timeout handling, or cancellation propagation fails
- **THEN** the returned `CoreError` includes structured streaming failure data that Rust consumers can match without parsing display strings

#### Scenario: Streaming configuration failures remain configuration errors
- **WHEN** a file-backed streaming robustness policy contains invalid TOML fields, unsupported enum values, or out-of-range duration/count values
- **THEN** the returned `CoreError` uses structured configuration error data with streaming field paths rather than reclassifying the failure as a stream execution error

#### Scenario: Delivered stream terminal errors remain response data
- **WHEN** a streaming execution has already emitted any stream event and then terminates as cancelled or errored
- **THEN** the cancellation or stream error remains typed terminal response data rather than being converted into `Err(CoreError)` and losing delivered event history

#### Scenario: Pre-stream failures preserve root cause
- **WHEN** retry exhaustion, timeout, cancellation, or provider failure occurs before any valid stream event has been emitted
- **THEN** `oxmux` returns structured pre-stream failure data that preserves retry counts and the underlying failure code when available

#### Scenario: First emitted event commits stream
- **WHEN** retry exhaustion, timeout, cancellation, or provider failure occurs after any keepalive metadata, timeout metadata, retry-summary metadata, content, or other stream event has been emitted
- **THEN** `oxmux` treats the stream as committed and preserves the terminal condition as response data rather than retrying or returning a pre-stream error

### Requirement: Core preserves headless dependency boundary for streaming robustness
Streaming robustness controls in `oxmux` SHALL NOT require GPUI, `oxidemux`, provider SDKs, HTTP clients, OAuth/token refresh flows, credential storage libraries, WebSocket relay support, async runtime transport adapters, or live upstream streaming dependencies.

#### Scenario: Dependency boundary remains enforceable
- **WHEN** maintainers inspect `crates/oxmux/Cargo.toml` and run core dependency-boundary tests after adding streaming robustness controls
- **THEN** `oxmux` remains free of desktop, provider SDK, live transport, and app-shell dependencies

#### Scenario: App shell consumes but does not redefine stream semantics
- **WHEN** future `oxidemux` app-shell code displays streaming timeout, cancellation, retry, or error state
- **THEN** it consumes `oxmux` state rather than creating separate app-owned streaming robustness semantics
