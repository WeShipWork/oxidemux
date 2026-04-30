## ADDED Requirements

### Requirement: Streaming robustness policy
The `oxmux` core SHALL define a headless streaming robustness policy that can represent keepalive interval, bootstrap retry count, timeout metadata behavior, and cancellation behavior without depending on GPUI, `oxidemux`, provider SDKs, HTTP clients, OAuth flows, credential storage, async runtime transport adapters, WebSocket relay support, or outbound provider network calls. Supported duration values SHALL be `1..=300000` milliseconds, supported retry counts SHALL be `0..=10`, and unsupported values SHALL fail validation instead of being silently clamped.

#### Scenario: Streaming policy defaults are disabled and deterministic
- **WHEN** a Rust consumer constructs the default streaming robustness policy
- **THEN** keepalive emission, bootstrap retry, timeout enforcement, and automatic cancellation behavior are disabled or inert unless explicitly configured
- **AND** `bootstrap_retry_count = 0` means the initial streaming attempt is tried once with no additional retry

#### Scenario: Streaming policy validates numeric controls
- **WHEN** a Rust consumer constructs streaming policy with keepalive interval, bootstrap retry count, or timeout values outside supported ranges
- **THEN** `oxmux` returns structured `CoreError` data identifying the invalid streaming policy field without panicking or silently clamping unsupported input

#### Scenario: Streaming policy remains transport agnostic
- **WHEN** maintainers inspect streaming robustness policy types and tests
- **THEN** they use core Rust values and deterministic in-memory events rather than requiring SSE writers, HTTP response bodies, provider SDK streams, async timers, or app-shell lifecycle state

### Requirement: Deterministic stream control events
The `oxmux` core SHALL represent keepalive and timeout metadata as deterministic stream-control data that preserves event order and can be matched by Rust consumers without parsing display strings or provider-specific transport frames. Reserved control metadata keys SHALL use the `oxmux.` namespace and include `oxmux.keepalive`, `oxmux.timeout`, `oxmux.retry_summary`, and `oxmux.retry_exhausted`. Provider-supplied or caller-supplied generic metadata SHALL NOT use the reserved `oxmux.` namespace unless constructed through typed robustness helpers.

#### Scenario: Idle keepalive is represented as metadata
- **WHEN** a deterministic stream emits a keepalive while waiting for provider content
- **THEN** the stream sequence includes an ordered metadata/control event that identifies the keepalive without requiring content payload bytes

#### Scenario: Timeout metadata is represented before terminal cancellation
- **WHEN** a deterministic stream reaches a configured timeout before clean completion
- **THEN** the stream sequence can include timeout metadata followed by a `StreamTerminalState::Cancelled` value with a timeout cancellation reason

#### Scenario: Reserved control metadata cannot be spoofed
- **WHEN** provider-supplied or caller-supplied generic stream metadata uses the `oxmux.` namespace directly
- **THEN** `oxmux` rejects the metadata as invalid unless it was constructed by a typed robustness helper

#### Scenario: Timeout enforcement remains deterministic in core tests
- **WHEN** default `oxmux` tests model timeout behavior
- **THEN** timeout appears as an explicit in-memory mock outcome or constructed stream observation rather than requiring wall-clock timers, async runtime scheduling, HTTP servers, or live transport backpressure

#### Scenario: Control events preserve existing stream lifecycle validation
- **WHEN** keepalive or timeout metadata appears before a terminal event
- **THEN** existing stream sequence validation still requires exactly one terminal event and rejects non-terminal events after termination

### Requirement: Safe bootstrap retry semantics
The `oxmux` core SHALL define bootstrap retry semantics that allow retrying a streaming execution only before the first emitted stream event has been delivered to the consumer. The first emitted event is the stream commit boundary, including keepalive metadata, timeout metadata, retry-summary metadata, content, or any other control metadata.

#### Scenario: Retry is allowed before first emitted event
- **WHEN** a deterministic streaming execution fails before emitting any stream event and the configured bootstrap retry budget is not exhausted
- **THEN** `oxmux` may retry the execution without returning partial stream data or emitted retry events from the failed attempt
- **AND** the committed successful attempt may expose a final retry summary identifying the number of failed pre-event attempts

#### Scenario: Retry is forbidden after stream commit
- **WHEN** a deterministic streaming execution emits content or metadata and later receives a provider error
- **THEN** `oxmux` represents the provider error as a terminal errored stream response rather than retrying and duplicating or corrupting already emitted data

#### Scenario: Retry metadata does not commit failed attempts
- **WHEN** a streaming attempt fails before emitting any stream event and a later retry succeeds
- **THEN** retry-attempt observations from the failed attempt are retained only as provider execution or management metadata until a successful attempt is committed
- **AND** no failed-attempt stream metadata is emitted before the successful attempt begins

#### Scenario: Retry exhaustion is structured
- **WHEN** bootstrap retry attempts are exhausted before any stream event can be emitted
- **THEN** `oxmux` returns a structured streaming failure that identifies retry exhaustion, total attempts, and the underlying stream failure without requiring display-text parsing

#### Scenario: Pre-event timeout and cancellation are structured
- **WHEN** timeout or cancellation is observed before a valid streaming response has emitted any event
- **THEN** `oxmux` returns structured pre-stream failure data
- **AND** timeout or cancellation observed after any emitted event remains terminal streaming response data preserving delivered event history

### Requirement: Streaming robustness tests remain networkless
Default `oxmux` streaming robustness tests SHALL cover keepalive, bootstrap retry, timeout/cancellation, post-partial provider error, and clean completion using deterministic in-memory values without real provider accounts, credentials, provider SDKs, outbound provider network calls, HTTP servers, WebSocket relays, OAuth flows, token refresh, platform secret storage, GPUI, or `oxidemux` app-shell dependencies.

#### Scenario: Tests cover idle keepalive
- **WHEN** maintainers run default `oxmux` streaming robustness tests
- **THEN** tests verify a configured idle keepalive is represented in order without requiring real timers or live network streams

#### Scenario: Tests cover retry before first emitted event
- **WHEN** maintainers run default `oxmux` streaming robustness tests
- **THEN** tests verify a pre-event failure can be retried up to the configured bootstrap retry count and eventually complete deterministically

#### Scenario: Tests cover retry exhaustion and zero retry budget
- **WHEN** maintainers run default `oxmux` streaming robustness tests
- **THEN** tests verify that zero bootstrap retry count performs no additional attempts and retry exhaustion preserves the underlying failure code

#### Scenario: Tests cover post-partial provider error
- **WHEN** maintainers run default `oxmux` streaming robustness tests
- **THEN** tests verify a provider error after emitted stream data becomes terminal response data and is not retried

#### Scenario: Tests cover deferred live transport concerns
- **WHEN** maintainers inspect default streaming robustness tests
- **THEN** they do not require live transport backpressure, real client disconnect propagation, upstream provider streams, or async timer enforcement

#### Scenario: Tests cover deterministic client-disconnect policy
- **WHEN** maintainers run default `oxmux` streaming robustness tests for `client-disconnect` cancellation behavior
- **THEN** tests use deterministic in-memory mock outcomes rather than live request-context disconnect detection
