## ADDED Requirements

### Requirement: Mock providers support deterministic streaming robustness outcomes
The `oxmux` core SHALL extend in-repo mock provider behavior so tests can deterministically model pre-event stream failure, bootstrap retry success, retry exhaustion, idle keepalive, cancellation, post-partial provider error, and clean completion without a real upstream provider streaming endpoint.

#### Scenario: Mock provider can fail before first event then succeed
- **WHEN** a test configures a mock streaming provider to fail before emitting any event and then succeed within the configured bootstrap retry budget
- **THEN** provider execution returns a deterministic streaming response from the successful attempt and exposes final retry summary data without delivering stream events from the failed attempt

#### Scenario: Mock provider can exhaust bootstrap retries
- **WHEN** a test configures a mock streaming provider to fail before emitting any event for more attempts than the configured bootstrap retry budget
- **THEN** provider execution returns a structured pre-stream failure identifying retry exhaustion, total attempts, and the underlying provider stream failure

#### Scenario: Zero retry budget performs no retry
- **WHEN** a test configures `bootstrap_retry_count = 0` and the initial mock streaming attempt fails before emitting an event
- **THEN** provider execution returns the pre-stream failure after the initial attempt without running an additional attempt

#### Scenario: Mock provider can error after partial stream
- **WHEN** a test configures a mock streaming provider to emit content or metadata and then fail
- **THEN** provider execution returns a `ResponseMode::Streaming` value whose ordered events include the delivered data and an errored terminal state rather than retrying the request

#### Scenario: Mock provider can propagate cancellation
- **WHEN** a test configures a mock streaming provider to observe cancellation behavior
- **THEN** provider execution returns a deterministic streaming response with a cancelled terminal state and matchable cancellation reason

#### Scenario: Mock provider can model client disconnect deterministically
- **WHEN** a test configures a mock streaming provider to observe `client-disconnect` cancellation behavior
- **THEN** provider execution uses deterministic in-memory cancellation outcomes without requiring a live HTTP request context, socket close, or transport adapter

#### Scenario: Mock provider can model pre-event cancellation
- **WHEN** a test configures a mock streaming provider to observe cancellation before any stream event is emitted
- **THEN** provider execution returns structured pre-stream cancellation data rather than a delivered streaming response

#### Scenario: Mock provider can model deterministic timeout
- **WHEN** a test configures a mock streaming provider to time out deterministically before any event is emitted
- **THEN** provider execution returns structured pre-stream timeout failure data using in-memory mock outcomes rather than requiring wall-clock timers, HTTP streams, or async transport backpressure

#### Scenario: Mock provider can model committed timeout metadata
- **WHEN** a test configures a mock streaming provider to emit any event and then time out deterministically
- **THEN** provider execution returns terminal streaming response data preserving delivered event history and timeout metadata

### Requirement: Provider streaming robustness remains metadata-driven
Provider execution SHALL keep streaming capability and robustness observations as typed metadata rather than provider-specific SDK values or transport-specific frames.

#### Scenario: Retry and timeout metadata are provider-neutral
- **WHEN** provider execution records bootstrap retry attempts, timeout metadata, or cancellation state
- **THEN** the data is represented through `oxmux` owned types and provider/account summaries without importing provider SDK error types

#### Scenario: Failed-attempt retry observations are not stream events
- **WHEN** provider execution records failed pre-event retry attempts
- **THEN** those observations remain provider execution or management metadata until a successful stream attempt is committed
- **AND** failed-attempt metadata does not appear in the ordered delivered stream sequence

#### Scenario: Streaming capability remains independent from response mode
- **WHEN** a provider advertises streaming capability or a robustness test configures a streaming outcome
- **THEN** provider capability metadata remains inspectable independently from whether the current execution returned a complete or streaming response
