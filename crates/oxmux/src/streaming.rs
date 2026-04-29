//! Streaming response state and validation contracts.
//!
//! Streaming values distinguish complete responses from ordered stream events,
//! terminal states, cancellation reasons, provider metadata, and invalid sequence
//! diagnostics for provider execution outcomes.

use crate::{CanonicalProtocolResponse, CoreError, ProtocolMetadata, ProtocolPayload};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Marker for streaming response ownership in the headless core.
pub struct StreamingBoundary;

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
        Ok(metadata)
    }

    fn validate(&self) -> Result<(), CoreError> {
        validate_required_text("stream.metadata.name", &self.name)?;
        validate_required_text("stream.metadata.value", &self.value)
    }

    /// Handles name for this public contract.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Handles value for this public contract.
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
}

impl StreamingFailure {
    /// Returns a human-readable message for this diagnostic.
    pub fn message(&self) -> String {
        match self {
            Self::InvalidSequence { reason } => reason.message(),
            Self::PreStreamFailure { failure } => failure.message().to_string(),
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
        }
    }
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

impl StreamFailure {
    fn invalid_field(message: String) -> Self {
        Self {
            code: "invalid_stream_field".to_string(),
            message,
            provider_metadata: None,
        }
    }
}
