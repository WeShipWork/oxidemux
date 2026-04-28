use crate::{CanonicalProtocolResponse, CoreError, ProtocolMetadata, ProtocolPayload};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StreamingBoundary;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResponseMode {
    Complete(CanonicalProtocolResponse),
    Streaming(StreamingResponse),
}

impl ResponseMode {
    pub fn complete(response: CanonicalProtocolResponse) -> Self {
        Self::Complete(response)
    }

    pub fn streaming(events: Vec<StreamEvent>) -> Result<Self, CoreError> {
        Ok(Self::Streaming(StreamingResponse::new(events)?))
    }

    pub fn complete_response(&self) -> Option<&CanonicalProtocolResponse> {
        match self {
            Self::Complete(response) => Some(response),
            Self::Streaming(_) => None,
        }
    }

    pub fn streaming_response(&self) -> Option<&StreamingResponse> {
        match self {
            Self::Complete(_) => None,
            Self::Streaming(response) => Some(response),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamingResponse {
    events: Vec<StreamEvent>,
}

impl StreamingResponse {
    pub fn new(events: Vec<StreamEvent>) -> Result<Self, CoreError> {
        Self::validate_events(&events)?;
        Ok(Self { events })
    }

    pub fn events(&self) -> &[StreamEvent] {
        &self.events
    }

    pub fn terminal(&self) -> Option<&StreamTerminalState> {
        match self.events.last() {
            Some(StreamEvent::Terminal(terminal)) => Some(terminal),
            Some(_) | None => None,
        }
    }

    pub fn into_events(self) -> Vec<StreamEvent> {
        self.events
    }

    pub fn validate(&self) -> Result<(), CoreError> {
        Self::validate_events(&self.events)
    }

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
pub enum StreamEvent {
    Content(StreamContent),
    Metadata(StreamMetadata),
    Terminal(StreamTerminalState),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamContent {
    pub protocol: ProtocolMetadata,
    pub payload: ProtocolPayload,
}

impl StreamContent {
    pub fn new(protocol: ProtocolMetadata, payload: ProtocolPayload) -> Result<Self, CoreError> {
        protocol.validate()?;
        Ok(Self { protocol, payload })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamMetadata {
    name: String,
    value: String,
}

impl StreamMetadata {
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StreamTerminalState {
    Completed,
    Cancelled { reason: CancellationReason },
    Errored { failure: StreamFailure },
}

impl StreamTerminalState {
    pub fn completed() -> Self {
        Self::Completed
    }

    pub fn cancelled(reason: CancellationReason) -> Self {
        Self::Cancelled { reason }
    }

    pub fn errored(failure: StreamFailure) -> Self {
        Self::Errored { failure }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CancellationReason {
    UserRequested,
    ClientDisconnected,
    UpstreamClosed,
    Timeout,
    Other { code: String, message: String },
}

impl CancellationReason {
    pub fn other(code: impl Into<String>, message: impl Into<String>) -> Result<Self, CoreError> {
        let code = code.into();
        let message = message.into();
        validate_required_text("stream.cancellation.code", &code)?;
        validate_required_text("stream.cancellation.message", &message)?;
        Ok(Self::Other { code, message })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamFailure {
    code: String,
    message: String,
    provider_metadata: Option<String>,
}

impl StreamFailure {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Result<Self, CoreError> {
        Self::with_provider_metadata(code, message, None)
    }

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

    pub fn validate(&self) -> Result<(), CoreError> {
        validate_required_text("stream.failure.code", &self.code)?;
        validate_required_text("stream.failure.message", &self.message)?;
        validate_optional_text(
            "stream.failure.provider_metadata",
            self.provider_metadata.as_deref(),
        )
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn provider_metadata(&self) -> Option<&str> {
        self.provider_metadata.as_deref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StreamingFailure {
    InvalidSequence { reason: InvalidStreamSequence },
    PreStreamFailure { failure: StreamFailure },
}

impl StreamingFailure {
    pub fn message(&self) -> String {
        match self {
            Self::InvalidSequence { reason } => reason.message(),
            Self::PreStreamFailure { failure } => failure.message().to_string(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InvalidStreamSequence {
    MissingTerminal,
    MultipleTerminals {
        first_terminal_index: usize,
        terminal_index: usize,
    },
    EventAfterTerminal {
        terminal_index: usize,
        event_index: usize,
    },
}

impl InvalidStreamSequence {
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
