use crate::{
    CanonicalProtocolRequest, CoreError, ProtocolMetadata, ProtocolPayload, ProtocolPayloadBody,
    ProviderExecutionRequest, ProviderExecutor, ResponseMode, RoutingAvailabilitySnapshot,
    RoutingBoundary, RoutingPolicy, RoutingSelectionRequest,
};

pub const MINIMAL_CHAT_COMPLETIONS_PATH: &str = "/v1/chat/completions";
pub const MINIMAL_PROXY_JSON_CONTENT_TYPE: &str = "application/json";
pub const MAX_MINIMAL_PROXY_BODY_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MinimalProxyRequest {
    pub body: Vec<u8>,
}

impl MinimalProxyRequest {
    pub fn open_ai_chat_completions(body: impl Into<Vec<u8>>) -> Result<Self, CoreError> {
        let request = Self { body: body.into() };
        request.validate()?;
        Ok(request)
    }

    pub fn validate(&self) -> Result<(), CoreError> {
        if self.body.len() > MAX_MINIMAL_PROXY_BODY_BYTES {
            return Err(invalid_request(
                "body",
                MinimalProxyErrorCode::RequestTooLarge,
                "request body exceeds minimal proxy body limit",
            ));
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MinimalProxyResponse {
    pub status_code: u16,
    pub content_type: &'static str,
    pub body: String,
}

impl MinimalProxyResponse {
    pub fn success(body: String) -> Self {
        Self {
            status_code: 200,
            content_type: MINIMAL_PROXY_JSON_CONTENT_TYPE,
            body,
        }
    }

    pub fn invalid_request(error: &CoreError) -> Self {
        Self::error(400, MinimalProxyErrorCode::from_core_error(error), error)
    }

    pub fn proxy_failure(error: &CoreError) -> Self {
        Self::error(502, MinimalProxyErrorCode::from_core_error(error), error)
    }

    pub fn unsupported_path() -> Self {
        Self::error_body(
            404,
            MinimalProxyErrorCode::UnsupportedPath,
            "unsupported local proxy path",
        )
    }

    pub fn from_core_error(error: &CoreError) -> Self {
        match error {
            CoreError::MinimalProxyRequestValidation { .. } => Self::invalid_request(error),
            _ => Self::proxy_failure(error),
        }
    }

    fn error(status_code: u16, code: MinimalProxyErrorCode, error: &CoreError) -> Self {
        Self::error_body(status_code, code, &error.to_string())
    }

    fn error_body(status_code: u16, code: MinimalProxyErrorCode, message: &str) -> Self {
        let body = serde_json::json!({
            "error": {
                "code": code.as_str(),
                "message": message,
                "type": "oxmux_proxy_error"
            }
        })
        .to_string();

        Self {
            status_code,
            content_type: MINIMAL_PROXY_JSON_CONTENT_TYPE,
            body,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MinimalProxyErrorCode {
    InvalidJson,
    MissingModel,
    BlankModel,
    UnsupportedRequestShape,
    RequestTooLarge,
    RoutingFailed,
    ProviderExecutionFailed,
    UnsupportedResponseMode,
    ResponseSerializationFailed,
    UnsupportedPath,
}

impl MinimalProxyErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidJson => "invalid_json",
            Self::MissingModel => "missing_model",
            Self::BlankModel => "blank_model",
            Self::UnsupportedRequestShape => "unsupported_request_shape",
            Self::RequestTooLarge => "request_too_large",
            Self::RoutingFailed => "routing_failed",
            Self::ProviderExecutionFailed => "provider_execution_failed",
            Self::UnsupportedResponseMode => "unsupported_response_mode",
            Self::ResponseSerializationFailed => "response_serialization_failed",
            Self::UnsupportedPath => "unsupported_path",
        }
    }

    fn from_core_error(error: &CoreError) -> Self {
        match error {
            CoreError::MinimalProxyRequestValidation { code, .. } => *code,
            CoreError::Routing { .. } => Self::RoutingFailed,
            CoreError::ProviderExecution { .. } => Self::ProviderExecutionFailed,
            CoreError::MinimalProxyUnsupportedResponseMode { .. } => Self::UnsupportedResponseMode,
            CoreError::MinimalProxyResponseSerialization { .. } => {
                Self::ResponseSerializationFailed
            }
            _ => Self::ProviderExecutionFailed,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MinimalProxyEngine;

impl MinimalProxyEngine {
    pub fn execute(
        request: MinimalProxyRequest,
        config: MinimalProxyEngineConfig<'_>,
    ) -> Result<MinimalProxyResponse, CoreError> {
        request.validate()?;
        let decoded_request = decode_chat_completion_request(&request.body)?;
        let requested_model = decoded_request.model;
        let canonical_request = CanonicalProtocolRequest::new(
            ProtocolMetadata::open_ai(),
            requested_model.clone(),
            ProtocolPayload::opaque(MINIMAL_PROXY_JSON_CONTENT_TYPE, request.body),
        )?;
        let selection = RoutingBoundary::select(
            config.routing_policy,
            &RoutingSelectionRequest::new(requested_model.clone()),
            config.availability,
        )?;
        let provider_request = ProviderExecutionRequest::new(
            selection.selected_target.provider_id,
            selection.selected_target.account_id,
            canonical_request,
        )?;
        let provider_result = config.provider_executor.execute(provider_request)?;
        let response = match provider_result.outcome.response_mode() {
            ResponseMode::Complete(response) => response,
            ResponseMode::Streaming(_) => {
                return Err(CoreError::MinimalProxyUnsupportedResponseMode {
                    message: "minimal chat-completion route only supports complete responses"
                        .to_string(),
                });
            }
        };
        let body = encode_chat_completion_response(&requested_model, response)?;

        Ok(MinimalProxyResponse::success(body))
    }

    pub fn execute_to_response(
        request: MinimalProxyRequest,
        config: MinimalProxyEngineConfig<'_>,
    ) -> MinimalProxyResponse {
        match Self::execute(request, config) {
            Ok(response) => response,
            Err(error) => MinimalProxyResponse::from_core_error(&error),
        }
    }
}

#[derive(Clone, Copy)]
pub struct MinimalProxyEngineConfig<'a> {
    pub routing_policy: &'a RoutingPolicy,
    pub availability: &'a RoutingAvailabilitySnapshot,
    pub provider_executor: &'a dyn ProviderExecutor,
}

impl<'a> MinimalProxyEngineConfig<'a> {
    pub fn new(
        routing_policy: &'a RoutingPolicy,
        availability: &'a RoutingAvailabilitySnapshot,
        provider_executor: &'a dyn ProviderExecutor,
    ) -> Self {
        Self {
            routing_policy,
            availability,
            provider_executor,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DecodedChatCompletionRequest {
    model: String,
}

fn decode_chat_completion_request(body: &[u8]) -> Result<DecodedChatCompletionRequest, CoreError> {
    let value: serde_json::Value = serde_json::from_slice(body).map_err(|_| {
        invalid_request(
            "body",
            MinimalProxyErrorCode::InvalidJson,
            "request body must be valid JSON",
        )
    })?;
    let Some(object) = value.as_object() else {
        return Err(invalid_request(
            "body",
            MinimalProxyErrorCode::UnsupportedRequestShape,
            "request body must be a JSON object",
        ));
    };
    let Some(model_value) = object.get("model") else {
        return Err(invalid_request(
            "model",
            MinimalProxyErrorCode::MissingModel,
            "model is required",
        ));
    };
    let Some(model) = model_value.as_str() else {
        return Err(invalid_request(
            "model",
            MinimalProxyErrorCode::UnsupportedRequestShape,
            "model must be a string",
        ));
    };
    let model = model.trim();
    if model.is_empty() {
        return Err(invalid_request(
            "model",
            MinimalProxyErrorCode::BlankModel,
            "model must not be blank",
        ));
    }
    let Some(messages) = object.get("messages").and_then(|value| value.as_array()) else {
        return Err(invalid_request(
            "messages",
            MinimalProxyErrorCode::UnsupportedRequestShape,
            "messages must be an array for the minimal smoke route",
        ));
    };
    if messages.is_empty() {
        return Err(invalid_request(
            "messages",
            MinimalProxyErrorCode::UnsupportedRequestShape,
            "messages must not be empty for the minimal smoke route",
        ));
    }

    Ok(DecodedChatCompletionRequest {
        model: model.to_string(),
    })
}

fn encode_chat_completion_response(
    requested_model: &str,
    response: &crate::CanonicalProtocolResponse,
) -> Result<String, CoreError> {
    // This smoke route intentionally treats the provider payload as assistant text;
    // full provider response translation is deferred to the protocol compatibility layer.
    let content = match &response.payload.body {
        ProtocolPayloadBody::Empty => String::new(),
        ProtocolPayloadBody::Opaque(bytes) => {
            String::from_utf8(bytes.clone()).map_err(|error| {
                CoreError::MinimalProxyResponseSerialization {
                    message: format!("provider response payload is not valid UTF-8: {error}"),
                }
            })?
        }
    };

    Ok(serde_json::json!({
        "id": "chatcmpl-oxmux-smoke",
        "object": "chat.completion",
        "created": 0,
        "model": requested_model,
        "choices": [
            {
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": content
                },
                "finish_reason": "stop"
            }
        ]
    })
    .to_string())
}

fn invalid_request(
    field: &'static str,
    code: MinimalProxyErrorCode,
    message: impl Into<String>,
) -> CoreError {
    CoreError::MinimalProxyRequestValidation {
        field,
        code,
        message: message.into(),
    }
}
