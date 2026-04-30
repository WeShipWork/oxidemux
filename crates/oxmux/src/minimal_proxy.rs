//! Minimal local proxy smoke-route contracts.
//!
//! This module validates a small OpenAI-compatible chat-completions request shape,
//! routes it through core policy and provider execution, and serializes basic JSON
//! responses for local runtime tests and bootstrap behavior.

use crate::{
    CanonicalProtocolRequest, CoreError, LocalClientAuthorizationFailure, ProtocolMetadata,
    ProtocolPayload, ProtocolPayloadBody, ProviderExecutionRequest, ProviderExecutor, ResponseMode,
    RoutingAvailabilitySnapshot, RoutingBoundary, RoutingPolicy, RoutingSelectionRequest,
};

/// OpenAI-compatible chat completions path served by the minimal proxy engine.
pub const MINIMAL_CHAT_COMPLETIONS_PATH: &str = "/v1/chat/completions";
/// JSON content type returned by deterministic minimal proxy responses.
pub const MINIMAL_PROXY_JSON_CONTENT_TYPE: &str = "application/json";
/// Maximum request body size accepted by the minimal proxy parser.
pub const MAX_MINIMAL_PROXY_BODY_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
/// Validated request body for the minimal local chat-completions route.
pub struct MinimalProxyRequest {
    /// Opaque body bytes or serialized response body.
    pub body: Vec<u8>,
}

impl MinimalProxyRequest {
    /// Creates a minimal OpenAI chat-completions request body.
    pub fn open_ai_chat_completions(body: impl Into<Vec<u8>>) -> Result<Self, CoreError> {
        let request = Self { body: body.into() };
        request.validate()?;
        Ok(request)
    }

    /// Validates this value and returns a structured core error on failure.
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
/// HTTP-style response emitted by the minimal proxy engine.
pub struct MinimalProxyResponse {
    /// HTTP status code returned to the local client.
    pub status_code: u16,
    /// Optional MIME-like content type for the payload.
    pub content_type: &'static str,
    /// Opaque body bytes or serialized response body.
    pub body: String,
}

impl MinimalProxyResponse {
    /// Creates a successful response value.
    pub fn success(body: String) -> Self {
        Self {
            status_code: 200,
            content_type: MINIMAL_PROXY_JSON_CONTENT_TYPE,
            body,
        }
    }

    /// Creates a minimal proxy response for a client request error.
    pub fn invalid_request(error: &CoreError) -> Self {
        Self::error(400, MinimalProxyErrorCode::from_core_error(error), error)
    }

    /// Creates a minimal proxy response for an upstream core failure.
    pub fn proxy_failure(error: &CoreError) -> Self {
        Self::error(502, MinimalProxyErrorCode::from_core_error(error), error)
    }

    /// Creates a minimal proxy response for unsupported local paths.
    pub fn unsupported_path() -> Self {
        Self::error_body(
            404,
            MinimalProxyErrorCode::UnsupportedPath,
            "unsupported local proxy path",
        )
    }

    /// Creates a minimal proxy response for local client authorization failures.
    pub fn local_client_unauthorized(failure: &LocalClientAuthorizationFailure) -> Self {
        let body = serde_json::json!({
            "error": {
                "code": MinimalProxyErrorCode::LocalClientUnauthorized.as_str(),
                "message": failure.to_string(),
                "reason": failure.reason.as_str(),
                "scope": failure.scope.as_str(),
                "type": "oxmux_proxy_error"
            }
        })
        .to_string();

        Self {
            status_code: 401,
            content_type: MINIMAL_PROXY_JSON_CONTENT_TYPE,
            body,
        }
    }

    /// Creates a deterministic protected management boundary response.
    pub fn management_boundary() -> Self {
        let body = serde_json::json!({
            "object": "oxmux.management.boundary",
            "status": "authorized",
            "message": "local management boundary reserved"
        })
        .to_string();

        Self::success(body)
    }

    /// Maps a core error into a minimal proxy response.
    pub fn from_core_error(error: &CoreError) -> Self {
        match error {
            CoreError::MinimalProxyRequestValidation { .. } => Self::invalid_request(error),
            CoreError::LocalClientAuthorization { failure } => {
                Self::local_client_unauthorized(failure)
            }
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
/// Stable error code serialized by minimal proxy responses.
pub enum MinimalProxyErrorCode {
    /// Request body was not valid JSON.
    InvalidJson,
    /// Request body omitted the model field.
    MissingModel,
    /// Request model was blank.
    BlankModel,
    /// Request shape is outside the minimal route contract.
    UnsupportedRequestShape,
    /// Request body exceeded the minimal proxy size limit.
    RequestTooLarge,
    /// Routing selection failed.
    RoutingFailed,
    /// Provider execution failed.
    ProviderExecutionFailed,
    /// Provider response mode cannot be serialized by this route.
    UnsupportedResponseMode,
    /// Minimal proxy response serialization failed.
    ResponseSerializationFailed,
    /// Local request path is not supported by the minimal runtime.
    UnsupportedPath,
    /// Local client authorization failed for a protected route.
    LocalClientUnauthorized,
}

impl MinimalProxyErrorCode {
    /// Returns the stable serialized error code string.
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
            Self::LocalClientUnauthorized => "local_client_unauthorized",
        }
    }

    fn from_core_error(error: &CoreError) -> Self {
        match error {
            CoreError::MinimalProxyRequestValidation { code, .. } => *code,
            CoreError::LocalClientAuthorization { .. } => Self::LocalClientUnauthorized,
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
/// Synchronous engine for the minimal local chat-completions smoke route.
pub struct MinimalProxyEngine;

impl MinimalProxyEngine {
    /// Executes this boundary operation and returns structured core errors on failure.
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

    /// Executes a minimal proxy request and maps failures into proxy responses.
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
/// Borrowed routing, availability, and provider executor inputs for minimal proxy execution.
pub struct MinimalProxyEngineConfig<'a> {
    /// Routing policy used by the minimal proxy route.
    pub routing_policy: &'a RoutingPolicy,
    /// Target availability used by the minimal proxy route.
    pub availability: &'a RoutingAvailabilitySnapshot,
    /// Provider executor used by the minimal proxy route.
    pub provider_executor: &'a dyn ProviderExecutor,
}

impl<'a> MinimalProxyEngineConfig<'a> {
    /// Creates engine configuration from routing, availability, and provider execution inputs.
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
