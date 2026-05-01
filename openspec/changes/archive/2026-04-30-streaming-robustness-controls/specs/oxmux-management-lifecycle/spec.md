## ADDED Requirements

### Requirement: Management state exposes streaming robustness outcomes
The `oxmux` management snapshot SHALL expose stream robustness warnings and structured errors for timeout, cancellation, bootstrap retry exhaustion, and provider stream failures so headless consumers and the future `oxidemux` app shell can surface state without parsing display strings or duplicating core semantics.

Management state SHALL expose the active streaming robustness policy and the latest streaming robustness outcome when such outcome data is supplied by provider or proxy execution. Outcome data SHALL include provider, account, and routing target context when known, and SHALL avoid raw provider secrets, raw transport frames, and app-shell presentation copies. Latest outcome replacement is last-writer-wins. Aggregate outcome history is deferred from this change.

#### Scenario: Management snapshot exposes active streaming policy
- **WHEN** core configuration includes streaming robustness policy values
- **THEN** management state can expose the active keepalive, retry, timeout, and cancellation policy values to headless consumers without requiring `oxidemux` code

#### Scenario: Management snapshot can include stream timeout state
- **WHEN** the latest core state includes a streaming timeout or timeout-driven cancellation
- **THEN** management state can expose a warning, degraded health reason, or structured error containing matchable timeout/cancellation data and provider/account context when known

#### Scenario: Management snapshot can include retry exhaustion state
- **WHEN** bootstrap retries are exhausted before any stream event is emitted
- **THEN** management state can expose a structured error or degraded reason identifying retry exhaustion and the affected provider or account when known

#### Scenario: Management snapshot can include retry metadata without stream event leakage
- **WHEN** provider execution records failed pre-event retry attempts that are not emitted as delivered stream events
- **THEN** management state can expose the retry attempt count, exhaustion state, and affected provider/account when known without representing failed-attempt metadata as delivered stream output

#### Scenario: Management snapshot can include post-partial stream error state
- **WHEN** a provider stream emits partial data and then terminates with a structured stream error
- **THEN** management state can expose that terminal failure without converting the delivered stream history into an unrelated generic proxy error

#### Scenario: Management snapshot replacement is deterministic
- **WHEN** a later streaming robustness outcome is supplied after an earlier timeout, cancellation, retry exhaustion, or stream failure outcome
- **THEN** management state replaces the latest outcome deterministically using last-writer-wins semantics without retaining stale latest state as current health

#### Scenario: Management snapshot omits transport secrets and frames
- **WHEN** management state exposes streaming robustness outcomes
- **THEN** it does not expose raw provider credentials, OAuth tokens, raw HTTP/SSE frames, WebSocket frames, or provider SDK error objects

#### Scenario: App status surfaces consume oxmux state
- **WHEN** `oxidemux` or another management consumer presents stream timeout, cancellation, retry, or error state
- **THEN** it reads state represented by `oxmux` management data rather than creating app-shell-specific copies of stream robustness semantics
