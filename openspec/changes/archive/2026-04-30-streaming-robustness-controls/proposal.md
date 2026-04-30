## Why

Issue #20 needs the existing `oxmux` streaming response primitives to grow from deterministic event values into deterministic stream-control semantics before real provider adapters and proxy routes depend on them. CLIProxyAPI/VibeProxy demonstrate the product risk: long-lived local AI streams need explicit keepalives, safe retry before any payload is emitted, timeout/cancellation metadata, and structured terminal errors so clients and app status surfaces never infer state from a dropped connection.

## What Changes

- Add headless streaming robustness policy to `oxmux` for keepalive interval, bootstrap retry count, stream timeout metadata, and cancellation behavior.
- Add deterministic keepalive/control metadata events that can be represented without binding `oxmux` to SSE, HTTP transports, async runtimes, provider SDKs, or desktop UI, with reserved `oxmux.` metadata keys enforced against provider/custom metadata spoofing.
- Define safe bootstrap retry semantics: retry is allowed only before the first emitted stream event; after any emitted event, including keepalive or timeout metadata/control events, failures remain terminal stream data.
- Define strict `[streaming]` TOML configuration fields, disabled defaults, concrete numeric validation ranges, validation field paths, and configuration-vs-streaming error taxonomy.
- Extend mock provider/test harness behavior so networkless tests can cover idle keepalive, retry-before-first-emitted-event, retry exhaustion, deterministic timeout, cancellation propagation, post-partial provider errors, latest management outcome replacement, and clean completion.
- Surface active streaming policy, timeout, cancellation, retry exhaustion, and structured stream errors through matchable `oxmux` response/error/management data for proxy consumers and future `oxidemux` status presentation.
- Keep real upstream streaming endpoints, WebSocket relay, live backpressure/client-disconnect propagation, provider-specific thinking/reasoning behavior, and UI rendering outside this change; `client-disconnect` remains a deterministic policy/mock outcome until a later transport change wires live request contexts.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `oxmux-streaming-response`: Add deterministic stream-control policy, keepalive metadata, timeout metadata, bootstrap retry semantics, cancellation propagation, and post-partial structured error behavior to existing streaming response contracts.
- `oxmux-core`: Require the public facade to expose the new streaming robustness primitives and preserve matchable error/response semantics without app-shell or provider SDK dependencies.
- `oxmux-provider-execution`: Extend mock provider execution semantics so tests can deterministically model retry-before-first-event, cancellation, post-partial provider errors, and clean stream completion.
- `oxmux-management-lifecycle`: Require app-visible/headless management state to expose streaming cancellation, timeout, retry exhaustion, and structured stream error state without UI-specific copies.
- `oxmux-file-configuration`: Extend file-backed configuration semantics so streaming robustness policy can be represented and validated with structured errors.

## Impact

- Affects `crates/oxmux/src/streaming.rs`, `provider.rs`, `errors.rs`, `management.rs`, configuration parsing/validation, and public facade exports in `oxmux.rs`.
- Affects deterministic tests in `crates/oxmux/tests/streaming_response.rs`, `provider_execution.rs`, `file_configuration.rs`, and management/local proxy related tests where stream errors or unsupported streaming modes surface to consumers.
- Does not add provider SDKs, live network streaming, WebSocket support, GPUI, `oxidemux` UI rendering, or new runtime transport dependencies.
- May add small focused `oxmux` modules if implementation pressure would otherwise make `streaming.rs` or `provider.rs` grow beyond maintainable size.
