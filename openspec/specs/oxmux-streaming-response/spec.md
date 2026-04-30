## Purpose

Define headless `oxmux` response primitives that distinguish complete canonical
responses from deterministic streaming response event sequences without binding
the core crate to provider transports, desktop UI, or app-shell dependencies.
## Requirements
### Requirement: Typed response primitives
The `oxmux` core SHALL expose typed response primitives that distinguish complete non-streaming responses from deterministic streaming response event sequences through a `ResponseMode` envelope without depending on GPUI, `oxidemux`, provider SDKs, HTTP clients, OAuth flows, credential storage, async runtime transport adapters, or outbound provider network calls.

#### Scenario: Complete response is represented explicitly
- **WHEN** a Rust consumer receives a non-streaming provider or proxy response through `oxmux`
- **THEN** the response is represented as a typed complete response carrying canonical response data rather than as a single streaming chunk or display string

#### Scenario: Streaming response is represented explicitly
- **WHEN** a Rust consumer receives a streaming response through `oxmux`
- **THEN** the response is represented as an ordered stream event sequence distinct from a complete non-streaming response

#### Scenario: Response mode is matchable
- **WHEN** a Rust consumer receives a response through `oxmux`
- **THEN** the consumer can match whether the response mode is a complete `CanonicalProtocolResponse` or a deterministic `StreamingResponse` without parsing display text or provider-specific metadata

#### Scenario: Streaming primitives remain headless
- **WHEN** maintainers inspect or test streaming response primitives
- **THEN** the `oxmux` crate remains free of GPUI, app-shell, provider SDK, HTTP, OAuth, token refresh, credential storage, async runtime transport, and live upstream streaming dependencies

### Requirement: Deterministic stream event lifecycle
The `oxmux` core SHALL define ordered streaming events for chunk/SSE-style data and SHALL require every stream sequence to contain exactly one in-order terminal event represented as `StreamEvent::Terminal(StreamTerminalState)` where the terminal state is completed, cancelled, or errored.

#### Scenario: Stream chunks preserve order
- **WHEN** a test constructs a streaming response with multiple content or metadata events
- **THEN** consumers can inspect the events in deterministic order without contacting an external provider

#### Scenario: Event order is insertion order
- **WHEN** `oxmux` validates or stores a streaming response sequence
- **THEN** event order is preserved as provided and validation does not sort events, deduplicate events, or synthesize event identities

#### Scenario: Stream completion is explicit
- **WHEN** a streaming response finishes successfully
- **THEN** the stream sequence includes a typed completed terminal event rather than relying on dropped state or missing events

#### Scenario: Empty stream may complete
- **WHEN** a streaming response completes before delivering non-terminal content or metadata events
- **THEN** the sequence with only a completed terminal event is valid and does not require a content chunk

#### Scenario: Metadata-only stream may complete
- **WHEN** a streaming response delivers metadata events without content chunks before termination
- **THEN** validation preserves those metadata events and does not require a content chunk before the terminal event

#### Scenario: Stream cancellation is explicit
- **WHEN** a streaming response is cancelled before successful completion
- **THEN** the stream sequence includes a typed cancelled terminal event with a matchable cancellation reason

#### Scenario: Cancellation reason has minimum typed shape
- **WHEN** a stream terminates as cancelled
- **THEN** the cancellation reason is represented with typed variants for user-requested, client-disconnected, upstream-closed, timeout, and an extensible other/code-message case

#### Scenario: Stream error is explicit
- **WHEN** a streaming response fails after emitting zero or more events
- **THEN** the stream sequence includes a typed errored terminal event with structured failure data that callers can match without parsing display text

#### Scenario: Stream failure has minimum typed shape
- **WHEN** a stream terminates as errored
- **THEN** the terminal failure data includes at least a stable code and human-readable message, with any provider-specific details carried as optional owned metadata rather than provider SDK types

#### Scenario: Invalid stream sequence fails structurally
- **WHEN** a streaming response has no terminal event, more than one terminal event, or events after a terminal event
- **THEN** validation returns a structured `CoreError` instead of panicking or silently accepting the invalid sequence

#### Scenario: Streaming response constructor validates lifecycle
- **WHEN** a Rust consumer constructs a streaming response through the public constructor
- **THEN** construction returns `Result<Self, CoreError>` and applies the same validation rules exposed by any explicit validation method

### Requirement: Streaming tests remain networkless
Default `oxmux` streaming response tests SHALL use deterministic in-memory response and event values, and SHALL NOT require real provider accounts, credentials, provider SDKs, outbound provider network calls, HTTP servers, OAuth flows, token refresh, platform secret storage, GPUI, or `oxidemux` app-shell dependencies.

#### Scenario: Tests cover required stream states
- **WHEN** maintainers run default `oxmux` tests for streaming response primitives
- **THEN** deterministic tests cover complete responses, ordered stream chunks, completed streams, empty-completed streams, metadata-only streams, cancelled streams, errored streams, event order preservation, and invalid stream sequence validation without external services

#### Scenario: Streaming tests preserve core dependency boundary
- **WHEN** maintainers inspect or run streaming response tests
- **THEN** the tests use only `oxmux` core primitives and do not depend on GPUI, `oxidemux`, provider SDKs, HTTP clients, OAuth flows, credential storage, async runtime transport, or live upstream streaming

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
