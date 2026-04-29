## 1. Proxy Engine Contracts

- [x] 1.1 Define the minimal proxy engine request, response, explicit configuration seam, and error mapping types in `crates/oxmux` so Rust tests can supply `RoutingPolicy`, `RoutingAvailabilitySnapshot`, and deterministic `ProviderExecutor` inputs without a TCP listener.
- [x] 1.2 Add a narrow OpenAI-compatible chat-completion smoke codec that extracts a non-blank model from bounded JSON input and serializes deterministic non-streaming chat-completion success/error JSON with stable `error.code` values and `Content-Type: application/json` for proxy responses.
- [x] 1.3 Build canonical protocol requests with `ProtocolMetadata::open_ai()` and opaque payload data while keeping full protocol translation deferred.
- [x] 1.4 Coordinate routing selection through `RoutingBoundary::select` using caller-supplied `RoutingPolicy` and `RoutingAvailabilitySnapshot` inputs.
- [x] 1.5 Construct `ProviderExecutionRequest` from the selected provider/account target and execute it through the `ProviderExecutor` trait.
- [x] 1.6 Convert `ResponseMode::Complete` provider results into the minimal OpenAI-shaped response and reject unsupported response modes for this smoke route with structured failures.

## 2. Local Runtime Route Dispatch

- [x] 2.1 Extend the local runtime request parser to read bounded method, path, headers, and body data needed for `/health` and `POST /v1/chat/completions` without unbounded reads or panics.
- [x] 2.2 Preserve existing `GET /health` behavior and tests exactly while adding route dispatch for the minimal proxy engine path.
- [x] 2.3 Return deterministic local HTTP JSON responses for success `200`, invalid or oversized request `400`, routing/provider/unsupported-response-mode failure `502`, and unsupported path `404`.
- [x] 2.4 Keep runtime configuration loopback-only and ensure malformed client I/O does not stop the runtime from serving later valid requests.

## 3. Public Facade and Dependency Boundary

- [x] 3.1 Export any new minimal proxy engine primitives from `crates/oxmux/src/oxmux.rs` for direct Rust consumers.
- [x] 3.2 Extend `CoreError` or related error detail types only as needed to preserve matchable request validation, routing, provider execution, and serialization failures.
- [x] 3.3 Keep `crates/oxmux` free of `oxidemux`, GPUI, tray, updater, packaging, OAuth UI, credential storage, provider SDK, and real upstream provider dependencies.
- [x] 3.4 If adding a JSON dependency, isolate it to minimal codec behavior and update dependency-boundary tests to document the allowed dependency while preserving all app/provider-network exclusions.

## 4. Deterministic Test Coverage

- [x] 4.1 Add engine-level tests proving a valid minimal chat-completion request exercises protocol construction, routing selection, selected provider/account propagation into `ProviderExecutionRequest`, provider execution, and response serialization through public `oxmux` APIs.
- [x] 4.2 Add local runtime tests for successful `POST /v1/chat/completions` using a deterministic mock provider and loopback-only HTTP client requests.
- [x] 4.3 Add invalid request tests for malformed JSON, missing model, blank model, or unsupported minimal request shape that fail before routing/provider execution.
- [x] 4.4 Add provider failure and unsupported response-mode tests using deterministic mock outcomes that return deterministic proxy failure responses and preserve structured provider execution details.
- [x] 4.5 Add unsupported path/method tests proving non-health, non-chat routes return deterministic `404` behavior without success bodies.
- [x] 4.6 Add routing failure or degraded/exhausted fallback tests proving unavailable, missing, exhausted, or disallowed degraded candidates produce deterministic proxy failures without provider execution.
- [x] 4.7 Add or update dependency-boundary tests proving the minimal proxy engine remains headless, avoids external/upstream provider network calls, and stays independent of `oxidemux` desktop concerns.

## 5. Verification

- [x] 5.1 Run `cargo fmt --all --check` and fix formatting issues.
- [x] 5.2 Run `cargo test -p oxmux` and fix core/runtime/proxy test failures.
- [x] 5.3 Run `mise run ci` to verify workspace formatting, checking, clippy, and tests.
- [x] 5.4 Run `openspec validate add-minimal-proxy-request-route --strict` and fix any proposal, design, spec, or task validation issues.
