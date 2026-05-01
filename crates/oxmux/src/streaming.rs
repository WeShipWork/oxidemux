//! Streaming response state and validation contracts.
//!
//! Streaming values distinguish complete responses from ordered stream events,
//! terminal states, cancellation reasons, provider metadata, and invalid sequence
//! diagnostics for provider execution outcomes.

use crate::{CanonicalProtocolResponse, CoreError, ProtocolMetadata, ProtocolPayload};

/// Minimum supported stream-control duration in milliseconds.
pub const MIN_STREAMING_CONTROL_DURATION_MS: u64 = 1;
/// Maximum supported stream-control duration in milliseconds.
pub const MAX_STREAMING_CONTROL_DURATION_MS: u64 = 300_000;
/// Maximum supported additional bootstrap retry attempts after the initial attempt.
pub const MAX_STREAMING_BOOTSTRAP_RETRY_COUNT: u8 = 10;
/// Reserved metadata key emitted by typed keepalive helpers.
pub const OXMUX_KEEPALIVE_METADATA_KEY: &str = "oxmux.keepalive";
/// Reserved metadata key emitted by typed timeout helpers.
pub const OXMUX_TIMEOUT_METADATA_KEY: &str = "oxmux.timeout";
/// Reserved metadata key emitted by typed committed retry-summary helpers.
pub const OXMUX_RETRY_SUMMARY_METADATA_KEY: &str = "oxmux.retry_summary";
/// Reserved metadata key emitted by typed retry-exhaustion helpers.
pub const OXMUX_RETRY_EXHAUSTED_METADATA_KEY: &str = "oxmux.retry_exhausted";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Marker for streaming response ownership in the headless core.
pub struct StreamingBoundary;

#[derive(Clone, Debug, Eq, PartialEq)]
/// Deterministic headless policy for stream keepalive, bootstrap retry, timeout, and cancellation behavior.
pub struct StreamingRobustnessPolicy {
    /// Optional keepalive interval represented in milliseconds; omission disables keepalive metadata.
    pub keepalive_interval_ms: Option<u64>,
    /// Additional bootstrap attempts after the initial streaming attempt.
    pub bootstrap_retry_count: u8,
    /// Optional timeout duration represented in milliseconds; omission disables timeout metadata.
    pub timeout_ms: Option<u64>,
    /// Cancellation behavior enabled for deterministic stream-control handling.
    pub cancellation: StreamingCancellationPolicy,
}

impl Default for StreamingRobustnessPolicy {
    fn default() -> Self {
        Self {
            keepalive_interval_ms: None,
            bootstrap_retry_count: 0,
            timeout_ms: None,
            cancellation: StreamingCancellationPolicy::Disabled,
        }
    }
}

impl StreamingRobustnessPolicy {
    /// Creates a policy and validates all supported ranges and cross-field rules.
    pub fn new(
        keepalive_interval_ms: Option<u64>,
        bootstrap_retry_count: u8,
        timeout_ms: Option<u64>,
        cancellation: StreamingCancellationPolicy,
    ) -> Result<Self, CoreError> {
        let policy = Self {
            keepalive_interval_ms,
            bootstrap_retry_count,
            timeout_ms,
            cancellation,
        };
        policy.validate()?;
        Ok(policy)
    }

    /// Validates this policy and returns structured streaming errors for invalid code-owned values.
    pub fn validate(&self) -> Result<(), CoreError> {
        validate_optional_duration(
            "streaming.keepalive-interval-ms",
            self.keepalive_interval_ms,
        )?;
        validate_retry_count(self.bootstrap_retry_count)?;
        validate_optional_duration("streaming.timeout-ms", self.timeout_ms)?;
        if self.cancellation == StreamingCancellationPolicy::Timeout && self.timeout_ms.is_none() {
            return Err(invalid_policy(
                "streaming.cancellation",
                "timeout cancellation requires streaming.timeout-ms",
            ));
        }
        Ok(())
    }

    /// Returns true when no stream robustness behavior is enabled.
    pub fn is_disabled(&self) -> bool {
        self == &Self::default()
    }

    /// Returns the maximum number of total attempts allowed by this policy.
    pub fn max_attempts(&self) -> u8 {
        self.bootstrap_retry_count.saturating_add(1)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Deterministic cancellation behavior configured for stream-control handling.
pub enum StreamingCancellationPolicy {
    /// Automatic cancellation is disabled.
    Disabled,
    /// Client disconnect may be represented by deterministic mock outcomes.
    ClientDisconnect,
    /// Timeout policy may convert timeout observations into cancellation.
    Timeout,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Provider response mode, either complete or streaming.
pub enum ResponseMode {
    /// Response is available as a complete canonical payload.
    Complete(CanonicalProtocolResponse),
    /// Provider execution returns a streaming response.
    Streaming(StreamingResponse),
}

impl ResponseMode {
    /// Wraps a complete canonical response mode.
    pub fn complete(response: CanonicalProtocolResponse) -> Self {
        Self::Complete(response)
    }

    /// Creates a validated streaming response mode.
    pub fn streaming(events: Vec<StreamEvent>) -> Result<Self, CoreError> {
        Ok(Self::Streaming(StreamingResponse::new(events)?))
    }

    /// Returns the complete response when this mode is complete.
    pub fn complete_response(&self) -> Option<&CanonicalProtocolResponse> {
        match self {
            Self::Complete(response) => Some(response),
            Self::Streaming(_) => None,
        }
    }

    /// Returns the streaming response when this mode is streaming.
    pub fn streaming_response(&self) -> Option<&StreamingResponse> {
        match self {
            Self::Complete(_) => None,
            Self::Streaming(response) => Some(response),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Validated ordered sequence of stream events ending in one terminal state.
pub struct StreamingResponse {
    events: Vec<StreamEvent>,
}

impl StreamingResponse {
    /// Creates a streaming response after validating terminal event ordering.
    pub fn new(events: Vec<StreamEvent>) -> Result<Self, CoreError> {
        Self::validate_events(&events)?;
        Ok(Self { events })
    }

    /// Returns the ordered stream events.
    pub fn events(&self) -> &[StreamEvent] {
        &self.events
    }

    /// Returns the terminal stream state, when present.
    pub fn terminal(&self) -> Option<&StreamTerminalState> {
        match self.events.last() {
            Some(StreamEvent::Terminal(terminal)) => Some(terminal),
            Some(_) | None => None,
        }
    }

    /// Consumes the response and returns its ordered stream events.
    pub fn into_events(self) -> Vec<StreamEvent> {
        self.events
    }

    /// Validates this value and returns a structured core error on failure.
    pub fn validate(&self) -> Result<(), CoreError> {
        Self::validate_events(&self.events)
    }

    /// Validates stream event ordering and terminal state.
    pub fn validate_events(events: &[StreamEvent]) -> Result<(), CoreError> {
        let mut terminal_index = None;

        for (event_index, event) in events.iter().enumerate() {
            if matches!(event, StreamEvent::Terminal(_)) {
                if let Some(first_terminal_index) = terminal_index {
                    return Err(CoreError::Streaming {
                        failure: StreamingFailure::InvalidSequence {
                            reason: InvalidStreamSequence::MultipleTerminals {
                                first_terminal_index,
                                terminal_index: event_index,
                            },
                        },
                    });
                }

                terminal_index = Some(event_index);
            } else if let Some(terminal_index) = terminal_index {
                return Err(CoreError::Streaming {
                    failure: StreamingFailure::InvalidSequence {
                        reason: InvalidStreamSequence::EventAfterTerminal {
                            terminal_index,
                            event_index,
                        },
                    },
                });
            }
        }

        if terminal_index.is_none() {
            return Err(CoreError::Streaming {
                failure: StreamingFailure::InvalidSequence {
                    reason: InvalidStreamSequence::MissingTerminal,
                },
            });
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// One event in a streaming provider response.
pub enum StreamEvent {
    /// Stream event carrying protocol content.
    Content(StreamContent),
    /// Stream event carrying name/value metadata.
    Metadata(StreamMetadata),
    /// Stream event carrying the terminal state.
    Terminal(StreamTerminalState),
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Protocol-tagged content payload emitted by a stream.
pub struct StreamContent {
    /// Protocol metadata attached to this payload.
    pub protocol: ProtocolMetadata,
    /// Opaque protocol payload for this request, response, or stream event.
    pub payload: ProtocolPayload,
}

impl StreamContent {
    /// Creates a content event payload for a specific protocol family.
    pub fn new(protocol: ProtocolMetadata, payload: ProtocolPayload) -> Result<Self, CoreError> {
        protocol.validate()?;
        Ok(Self { protocol, payload })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Name/value metadata event emitted by a stream.
pub struct StreamMetadata {
    name: String,
    value: String,
}

impl StreamMetadata {
    /// Creates a stream metadata entry with non-empty name and value fields.
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Result<Self, CoreError> {
        let metadata = Self {
            name: name.into(),
            value: value.into(),
        };
        metadata.validate()?;
        if is_reserved_oxmux_metadata_key(&metadata.name) {
            return Err(CoreError::Streaming {
                failure: StreamingFailure::InvalidSequence {
                    reason: InvalidStreamSequence::ReservedMetadataKey {
                        name: metadata.name,
                    },
                },
            });
        }
        Ok(metadata)
    }

    /// Creates deterministic keepalive metadata using the reserved `oxmux.keepalive` key.
    pub fn keepalive() -> Self {
        Self::reserved(OXMUX_KEEPALIVE_METADATA_KEY, "true")
    }

    /// Creates deterministic timeout metadata using the reserved `oxmux.timeout` key.
    pub fn timeout(timeout_ms: u64) -> Result<Self, CoreError> {
        validate_duration("streaming.timeout-ms", timeout_ms)?;
        Ok(Self::reserved(
            OXMUX_TIMEOUT_METADATA_KEY,
            timeout_ms.to_string(),
        ))
    }

    /// Creates committed-attempt retry summary metadata using the reserved `oxmux.retry_summary` key.
    pub fn retry_summary(failed_attempts: u8, total_attempts: u8) -> Result<Self, CoreError> {
        if total_attempts == 0 || failed_attempts >= total_attempts {
            return Err(invalid_policy(
                "streaming.bootstrap-retry-count",
                "retry summary requires failed attempts to be less than total attempts",
            ));
        }
        Ok(Self::reserved(
            OXMUX_RETRY_SUMMARY_METADATA_KEY,
            format!("failed_attempts={failed_attempts};total_attempts={total_attempts}"),
        ))
    }

    /// Creates retry exhaustion metadata using the reserved `oxmux.retry_exhausted` key.
    pub fn retry_exhausted(total_attempts: u8, failure: &StreamFailure) -> Result<Self, CoreError> {
        if total_attempts == 0 {
            return Err(invalid_policy(
                "streaming.bootstrap-retry-count",
                "retry exhaustion requires at least one attempted stream execution",
            ));
        }
        Ok(Self::reserved(
            OXMUX_RETRY_EXHAUSTED_METADATA_KEY,
            format!(
                "total_attempts={total_attempts};failure_code={}",
                failure.code()
            ),
        ))
    }

    fn reserved(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }

    fn validate(&self) -> Result<(), CoreError> {
        validate_required_text("stream.metadata.name", &self.name)?;
        validate_required_text("stream.metadata.value", &self.value)
    }

    /// Returns the metadata entry name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the metadata entry value.
    pub fn value(&self) -> &str {
        &self.value
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Terminal outcome of a stream.
pub enum StreamTerminalState {
    /// Stream completed normally.
    Completed,
    /// Stream ended because it was cancelled.
    Cancelled {
        /// Human-readable reason for this state.
        reason: CancellationReason,
    },
    /// Stream ended with a failure.
    Errored {
        /// Structured failure associated with this state.
        failure: StreamFailure,
    },
}

impl StreamTerminalState {
    /// Creates a completed stream terminal state.
    pub fn completed() -> Self {
        Self::Completed
    }

    /// Creates a cancelled stream terminal state.
    pub fn cancelled(reason: CancellationReason) -> Self {
        Self::Cancelled { reason }
    }

    /// Creates an errored stream terminal state.
    pub fn errored(failure: StreamFailure) -> Self {
        Self::Errored { failure }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Reason a stream ended by cancellation rather than completion or error.
pub enum CancellationReason {
    /// Cancellation was requested by the user or caller.
    UserRequested,
    /// Cancellation followed a client disconnect.
    ClientDisconnected,
    /// Cancellation followed upstream closure.
    UpstreamClosed,
    /// Cancellation followed a timeout.
    Timeout,
    /// Cancellation reason is provider- or caller-specific.
    Other {
        /// Stable code for this failure or response.
        code: String,
        /// Human-readable diagnostic message.
        message: String,
    },
}

impl CancellationReason {
    /// Creates a provider- or caller-specific cancellation reason.
    pub fn other(code: impl Into<String>, message: impl Into<String>) -> Result<Self, CoreError> {
        let code = code.into();
        let message = message.into();
        validate_required_text("stream.cancellation.code", &code)?;
        validate_required_text("stream.cancellation.message", &message)?;
        Ok(Self::Other { code, message })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Provider or validation failure attached to stream errors.
pub struct StreamFailure {
    code: String,
    message: String,
    provider_metadata: Option<String>,
}

impl StreamFailure {
    /// Creates a stream failure without provider-specific metadata.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Result<Self, CoreError> {
        Self::with_provider_metadata(code, message, None)
    }

    /// Creates a stream failure with optional provider metadata.
    pub fn with_provider_metadata(
        code: impl Into<String>,
        message: impl Into<String>,
        provider_metadata: Option<String>,
    ) -> Result<Self, CoreError> {
        let failure = Self {
            code: code.into(),
            message: message.into(),
            provider_metadata,
        };
        failure.validate()?;
        Ok(failure)
    }

    /// Validates this value and returns a structured core error on failure.
    pub fn validate(&self) -> Result<(), CoreError> {
        validate_required_text("stream.failure.code", &self.code)?;
        validate_required_text("stream.failure.message", &self.message)?;
        validate_optional_text(
            "stream.failure.provider_metadata",
            self.provider_metadata.as_deref(),
        )
    }

    /// Returns the stream failure code.
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Returns a human-readable message for this diagnostic.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns provider metadata attached to the stream failure, when any.
    pub fn provider_metadata(&self) -> Option<&str> {
        self.provider_metadata.as_deref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Failure that prevents or invalidates a streaming response.
pub enum StreamingFailure {
    /// Stream event ordering failed validation.
    InvalidSequence {
        /// Human-readable reason for this state.
        reason: InvalidStreamSequence,
    },
    /// Failure occurred before a valid stream sequence existed.
    PreStreamFailure {
        /// Structured failure associated with this state.
        failure: StreamFailure,
    },
    /// Provider stream failed after a stream response existed or stream output committed.
    ProviderStreamFailure {
        /// Structured provider-neutral stream failure.
        failure: StreamFailure,
    },
    /// Bootstrap retries were exhausted before any stream event was emitted.
    RetryExhausted {
        /// Total attempts that were executed, including the initial attempt.
        total_attempts: u8,
        /// Final underlying stream failure that exhausted retry budget.
        failure: StreamFailure,
    },
    /// Timeout occurred before any stream event was emitted.
    PreStreamTimeout {
        /// Configured timeout duration in milliseconds.
        timeout_ms: u64,
        /// Structured failure associated with the timeout.
        failure: StreamFailure,
    },
    /// Cancellation occurred before any stream event was emitted.
    PreStreamCancellation {
        /// Cancellation reason observed before stream commit.
        reason: CancellationReason,
        /// Structured failure associated with cancellation.
        failure: StreamFailure,
    },
    /// Streaming robustness policy failed validation.
    InvalidPolicy {
        /// Field path associated with this policy diagnostic.
        field: &'static str,
        /// Human-readable diagnostic message.
        message: String,
    },
}

impl StreamingFailure {
    /// Returns a human-readable message for this diagnostic.
    pub fn message(&self) -> String {
        match self {
            Self::InvalidSequence { reason } => reason.message(),
            Self::PreStreamFailure { failure } => failure.message().to_string(),
            Self::ProviderStreamFailure { failure } => {
                format!("provider stream failed after commit: {}", failure.message())
            }
            Self::RetryExhausted {
                total_attempts,
                failure,
            } => format!(
                "stream retry exhausted after {total_attempts} attempt(s): {}",
                failure.message()
            ),
            Self::PreStreamTimeout {
                timeout_ms,
                failure,
            } => format!(
                "stream timed out before first event after {timeout_ms} ms: {}",
                failure.message()
            ),
            Self::PreStreamCancellation { failure, .. } => {
                format!("stream cancelled before first event: {}", failure.message())
            }
            Self::InvalidPolicy { field, message } => {
                format!("streaming policy field {field} is invalid: {message}")
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Validation failure for stream event ordering.
pub enum InvalidStreamSequence {
    /// Stream sequence did not include a terminal event.
    MissingTerminal,
    /// Stream sequence included more than one terminal event.
    MultipleTerminals {
        /// First terminal index value for the surrounding public enum variant.
        first_terminal_index: usize,
        /// Terminal index value for the surrounding public enum variant.
        terminal_index: usize,
    },
    /// Stream sequence included events after terminal state.
    EventAfterTerminal {
        /// Index of a terminal event in the stream sequence.
        terminal_index: usize,
        /// Index of a non-terminal event in the stream sequence.
        event_index: usize,
    },
    /// Caller attempted to create generic metadata using the reserved `oxmux.` namespace.
    ReservedMetadataKey {
        /// Rejected metadata key.
        name: String,
    },
}

impl InvalidStreamSequence {
    /// Returns a human-readable message for this diagnostic.
    pub fn message(&self) -> String {
        match self {
            Self::MissingTerminal => {
                "stream sequence must include exactly one terminal event".to_string()
            }
            Self::MultipleTerminals {
                first_terminal_index,
                terminal_index,
            } => format!(
                "stream sequence has multiple terminal events at indexes {first_terminal_index} and {terminal_index}"
            ),
            Self::EventAfterTerminal {
                terminal_index,
                event_index,
            } => format!(
                "stream sequence has non-terminal event at index {event_index} after terminal event at index {terminal_index}"
            ),
            Self::ReservedMetadataKey { name } => {
                format!("stream metadata key {name} is reserved for typed oxmux helpers")
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Latest stream robustness outcome exposed through management state.
pub struct StreamingRobustnessOutcome {
    /// Kind of stream robustness outcome observed by core execution.
    pub kind: StreamingRobustnessOutcomeKind,
    /// Provider identifier associated with the outcome, when known.
    pub provider_id: Option<String>,
    /// Account identifier associated with the outcome, when known.
    pub account_id: Option<String>,
}

impl StreamingRobustnessOutcome {
    /// Creates a stream robustness outcome without provider or account context.
    pub fn new(kind: StreamingRobustnessOutcomeKind) -> Self {
        Self {
            kind,
            provider_id: None,
            account_id: None,
        }
    }

    /// Attaches provider/account context to this outcome.
    pub fn with_provider_context(
        mut self,
        provider_id: impl Into<String>,
        account_id: Option<String>,
    ) -> Self {
        self.provider_id = Some(provider_id.into());
        self.account_id = account_id;
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Matchable category for the latest stream robustness outcome.
pub enum StreamingRobustnessOutcomeKind {
    /// Timeout was observed for a streaming execution.
    Timeout {
        /// Timeout duration in milliseconds.
        timeout_ms: u64,
    },
    /// Cancellation was observed for a streaming execution.
    Cancellation {
        /// Cancellation reason associated with the stream.
        reason: CancellationReason,
    },
    /// Bootstrap retry budget was exhausted before stream commit.
    RetryExhausted {
        /// Total attempts that were executed, including the initial attempt.
        total_attempts: u8,
        /// Underlying stream failure that exhausted retry budget.
        failure: StreamFailure,
    },
    /// Provider stream failed after a valid response existed or stream committed.
    ProviderStreamFailure {
        /// Structured provider-neutral stream failure.
        failure: StreamFailure,
    },
}

fn validate_required_text(field: &'static str, value: &str) -> Result<(), CoreError> {
    if value.trim().is_empty() {
        return Err(CoreError::Streaming {
            failure: StreamingFailure::PreStreamFailure {
                failure: StreamFailure::invalid_field(format!("{field} must not be blank")),
            },
        });
    }

    Ok(())
}

fn validate_optional_text(field: &'static str, value: Option<&str>) -> Result<(), CoreError> {
    if matches!(value, Some(value) if value.trim().is_empty()) {
        return Err(CoreError::Streaming {
            failure: StreamingFailure::PreStreamFailure {
                failure: StreamFailure::invalid_field(format!(
                    "{field} must not be blank when present"
                )),
            },
        });
    }

    Ok(())
}

fn validate_optional_duration(field: &'static str, value: Option<u64>) -> Result<(), CoreError> {
    match value {
        Some(value) => validate_duration(field, value),
        None => Ok(()),
    }
}

fn validate_duration(field: &'static str, value: u64) -> Result<(), CoreError> {
    if (MIN_STREAMING_CONTROL_DURATION_MS..=MAX_STREAMING_CONTROL_DURATION_MS).contains(&value) {
        Ok(())
    } else {
        Err(invalid_policy(
            field,
            "duration must be between 1 and 300000 milliseconds",
        ))
    }
}

fn validate_retry_count(value: u8) -> Result<(), CoreError> {
    if value <= MAX_STREAMING_BOOTSTRAP_RETRY_COUNT {
        Ok(())
    } else {
        Err(invalid_policy(
            "streaming.bootstrap-retry-count",
            "retry count must be between 0 and 10",
        ))
    }
}

fn invalid_policy(field: &'static str, message: impl Into<String>) -> CoreError {
    CoreError::Streaming {
        failure: StreamingFailure::InvalidPolicy {
            field,
            message: message.into(),
        },
    }
}

fn is_reserved_oxmux_metadata_key(name: &str) -> bool {
    name.starts_with("oxmux.")
}

impl StreamFailure {
    fn invalid_field(message: String) -> Self {
        Self {
            code: "invalid_stream_field".to_string(),
            message,
            provider_metadata: None,
        }
    }
}
