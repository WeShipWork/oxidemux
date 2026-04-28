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
