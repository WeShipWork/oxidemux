//! Protocol compatibility and translation boundary contracts.
//!
//! Protocol values carry canonical request/response metadata and opaque payloads
//! while current translation functions validate inputs and return deferred
//! outcomes instead of promising provider-specific conversion behavior.

use crate::{CoreError, ProtocolFamily};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Facade for protocol request and response translation boundaries.
pub struct ProtocolBoundary;

impl ProtocolBoundary {
    /// Validates and translates, or defers translation of, a canonical request.
    pub fn translate_request(
        request: CanonicalProtocolRequest,
        target_protocol: ProtocolMetadata,
    ) -> Result<ProtocolTranslationOutcome<CanonicalProtocolRequest>, CoreError> {
        request.validate()?;
        target_protocol.validate()?;

        Ok(ProtocolTranslationOutcome::deferred(
            ProtocolTranslationDirection::Request,
            request.protocol.family(),
            target_protocol.family(),
        ))
    }

    /// Validates and translates, or defers translation of, a canonical response.
    pub fn translate_response(
        response: CanonicalProtocolResponse,
        target_protocol: ProtocolMetadata,
    ) -> Result<ProtocolTranslationOutcome<CanonicalProtocolResponse>, CoreError> {
        response.validate()?;
        target_protocol.validate()?;

        Ok(ProtocolTranslationOutcome::deferred(
            ProtocolTranslationDirection::Response,
            response.protocol.family(),
            target_protocol.family(),
        ))
    }
}

/// Trait for protocol translation implementations.
pub trait ProtocolTranslator {
    /// Validates and translates, or defers translation of, a canonical request.
    fn translate_request(
        &self,
        request: CanonicalProtocolRequest,
        target_protocol: ProtocolMetadata,
    ) -> Result<ProtocolTranslationOutcome<CanonicalProtocolRequest>, CoreError>;

    /// Validates and translates, or defers translation of, a canonical response.
    fn translate_response(
        &self,
        response: CanonicalProtocolResponse,
        target_protocol: ProtocolMetadata,
    ) -> Result<ProtocolTranslationOutcome<CanonicalProtocolResponse>, CoreError>;
}

impl ProtocolTranslator for ProtocolBoundary {
    fn translate_request(
        &self,
        request: CanonicalProtocolRequest,
        target_protocol: ProtocolMetadata,
    ) -> Result<ProtocolTranslationOutcome<CanonicalProtocolRequest>, CoreError> {
        Self::translate_request(request, target_protocol)
    }

    fn translate_response(
        &self,
        response: CanonicalProtocolResponse,
        target_protocol: ProtocolMetadata,
    ) -> Result<ProtocolTranslationOutcome<CanonicalProtocolResponse>, CoreError> {
        Self::translate_response(response, target_protocol)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Protocol-family-tagged request payload used by core routing and providers.
pub struct CanonicalProtocolRequest {
    /// Protocol metadata attached to this payload.
    pub protocol: ProtocolMetadata,
    /// Model associated with this protocol request or route.
    pub model: String,
    /// Opaque protocol payload for this request, response, or stream event.
    pub payload: ProtocolPayload,
}

impl CanonicalProtocolRequest {
    /// Creates a validated value for this public contract.
    pub fn new(
        protocol: ProtocolMetadata,
        model: impl Into<String>,
        payload: ProtocolPayload,
    ) -> Result<Self, CoreError> {
        let request = Self {
            protocol,
            model: model.into(),
            payload,
        };
        request.validate()?;
        Ok(request)
    }

    /// Validates this value and returns a structured core error on failure.
    pub fn validate(&self) -> Result<(), CoreError> {
        self.protocol.validate()?;
        validate_required_text("model", &self.model)?;
        self.payload.validate()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Protocol-family-tagged response payload returned by providers.
pub struct CanonicalProtocolResponse {
    /// Protocol metadata attached to this payload.
    pub protocol: ProtocolMetadata,
    /// Response status associated with the canonical response.
    pub status: ProtocolResponseStatus,
    /// Opaque protocol payload for this request, response, or stream event.
    pub payload: ProtocolPayload,
}

impl CanonicalProtocolResponse {
    /// Creates a validated value for this public contract.
    pub fn new(
        protocol: ProtocolMetadata,
        status: ProtocolResponseStatus,
        payload: ProtocolPayload,
    ) -> Result<Self, CoreError> {
        let response = Self {
            protocol,
            status,
            payload,
        };
        response.validate()?;
        Ok(response)
    }

    /// Validates this value and returns a structured core error on failure.
    pub fn validate(&self) -> Result<(), CoreError> {
        self.protocol.validate()?;
        self.status.validate()?;
        self.payload.validate()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Protocol family metadata attached to canonical requests and responses.
pub enum ProtocolMetadata {
    /// OpenAI-compatible protocol family or metadata.
    OpenAi(OpenAiProtocolMetadata),
    /// Gemini-compatible protocol family or metadata.
    Gemini(GeminiProtocolMetadata),
    /// Claude-compatible protocol family or metadata.
    Claude(ClaudeProtocolMetadata),
    /// Codex-compatible protocol family or metadata.
    Codex(CodexProtocolMetadata),
    /// Provider-specific protocol family or metadata.
    ProviderSpecific(ProviderSpecificProtocolMetadata),
}

impl ProtocolMetadata {
    /// Creates default OpenAI-compatible protocol metadata.
    pub fn open_ai() -> Self {
        Self::OpenAi(OpenAiProtocolMetadata::default())
    }

    /// Creates default Gemini-compatible protocol metadata.
    pub fn gemini() -> Self {
        Self::Gemini(GeminiProtocolMetadata::default())
    }

    /// Creates default Claude-compatible protocol metadata.
    pub fn claude() -> Self {
        Self::Claude(ClaudeProtocolMetadata::default())
    }

    /// Creates default Codex-compatible protocol metadata.
    pub fn codex() -> Self {
        Self::Codex(CodexProtocolMetadata::default())
    }

    /// Creates provider-specific protocol metadata.
    pub fn provider_specific(
        provider_id: impl Into<String>,
        format_name: impl Into<String>,
    ) -> Result<Self, CoreError> {
        Ok(Self::ProviderSpecific(
            ProviderSpecificProtocolMetadata::new(provider_id, format_name)?,
        ))
    }

    /// Returns the protocol family represented by this metadata.
    pub fn family(&self) -> ProtocolFamily {
        match self {
            Self::OpenAi(_) => ProtocolFamily::OpenAi,
            Self::Gemini(_) => ProtocolFamily::Gemini,
            Self::Claude(_) => ProtocolFamily::Claude,
            Self::Codex(_) => ProtocolFamily::Codex,
            Self::ProviderSpecific(_) => ProtocolFamily::ProviderSpecific,
        }
    }

    /// Validates this value and returns a structured core error on failure.
    pub fn validate(&self) -> Result<(), CoreError> {
        match self {
            Self::OpenAi(metadata) => metadata.validate(),
            Self::Gemini(metadata) => metadata.validate(),
            Self::Claude(metadata) => metadata.validate(),
            Self::Codex(metadata) => metadata.validate(),
            Self::ProviderSpecific(metadata) => metadata.validate(),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
/// OpenAI-compatible protocol metadata currently limited to optional version data.
pub struct OpenAiProtocolMetadata {
    /// Optional API version label for this protocol family.
    pub api_version: Option<String>,
}

impl OpenAiProtocolMetadata {
    fn validate(&self) -> Result<(), CoreError> {
        validate_optional_text("openai.api_version", self.api_version.as_deref())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
/// Gemini-compatible protocol metadata currently limited to optional version data.
pub struct GeminiProtocolMetadata {
    /// Optional API version label for this protocol family.
    pub api_version: Option<String>,
}

impl GeminiProtocolMetadata {
    fn validate(&self) -> Result<(), CoreError> {
        validate_optional_text("gemini.api_version", self.api_version.as_deref())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
/// Claude-compatible protocol metadata currently limited to optional version data.
pub struct ClaudeProtocolMetadata {
    /// Optional API version label for this protocol family.
    pub api_version: Option<String>,
}

impl ClaudeProtocolMetadata {
    fn validate(&self) -> Result<(), CoreError> {
        validate_optional_text("claude.api_version", self.api_version.as_deref())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
/// Codex-compatible protocol metadata currently limited to optional version data.
pub struct CodexProtocolMetadata {
    /// Optional API version label for this protocol family.
    pub api_version: Option<String>,
}

impl CodexProtocolMetadata {
    fn validate(&self) -> Result<(), CoreError> {
        validate_optional_text("codex.api_version", self.api_version.as_deref())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Provider-specific protocol metadata for formats outside built-in families.
pub struct ProviderSpecificProtocolMetadata {
    /// Provider identifier used by routing, execution, and management state.
    pub provider_id: String,
    /// Provider-specific protocol format name.
    pub format_name: String,
}

impl ProviderSpecificProtocolMetadata {
    /// Creates a validated value for this public contract.
    pub fn new(
        provider_id: impl Into<String>,
        format_name: impl Into<String>,
    ) -> Result<Self, CoreError> {
        let metadata = Self {
            provider_id: provider_id.into(),
            format_name: format_name.into(),
        };
        metadata.validate()?;
        Ok(metadata)
    }

    fn validate(&self) -> Result<(), CoreError> {
        validate_required_text("provider_specific.provider_id", &self.provider_id)?;
        validate_required_text("provider_specific.format_name", &self.format_name)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Opaque protocol payload plus optional content type.
pub struct ProtocolPayload {
    /// Optional MIME-like content type for the payload.
    pub content_type: Option<String>,
    /// Opaque body bytes or serialized response body.
    pub body: ProtocolPayloadBody,
}

impl ProtocolPayload {
    /// Creates an empty protocol payload.
    pub fn empty() -> Self {
        Self {
            content_type: None,
            body: ProtocolPayloadBody::Empty,
        }
    }

    /// Creates an opaque protocol payload with a content type.
    pub fn opaque(content_type: impl Into<String>, bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            content_type: Some(content_type.into()),
            body: ProtocolPayloadBody::Opaque(bytes.into()),
        }
    }

    fn validate(&self) -> Result<(), CoreError> {
        validate_optional_text("payload.content_type", self.content_type.as_deref())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Body storage for canonical protocol payloads.
pub enum ProtocolPayloadBody {
    /// Payload body is empty.
    Empty,
    /// Payload body is stored as opaque bytes.
    Opaque(Vec<u8>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// HTTP-like status attached to a canonical protocol response.
pub struct ProtocolResponseStatus {
    /// Stable code or numeric status for this value.
    pub code: u16,
    /// Human-readable reason for this state.
    pub reason: Option<String>,
}

impl ProtocolResponseStatus {
    /// Creates a validated value for this public contract.
    pub fn new(code: u16, reason: Option<String>) -> Result<Self, CoreError> {
        let status = Self { code, reason };
        status.validate()?;
        Ok(status)
    }

    /// Creates a successful response value.
    pub fn success() -> Self {
        Self {
            code: 200,
            reason: Some("OK".to_string()),
        }
    }

    fn validate(&self) -> Result<(), CoreError> {
        if !(100..=599).contains(&self.code) {
            return Err(CoreError::ProtocolValidation {
                field: "status.code",
                message: "status code must be in the 100..=599 range".to_string(),
            });
        }

        validate_optional_text("status.reason", self.reason.as_deref())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Result of a translation attempt or deferred translation marker.
pub enum ProtocolTranslationOutcome<T> {
    /// Translation produced a converted value.
    Translated(T),
    /// Translation was validated but deliberately deferred.
    Deferred(DeferredProtocolTranslation),
}

impl<T> ProtocolTranslationOutcome<T> {
    /// Creates a deferred protocol translation outcome.
    pub fn deferred(
        direction: ProtocolTranslationDirection,
        source_family: ProtocolFamily,
        target_family: ProtocolFamily,
    ) -> Self {
        Self::Deferred(DeferredProtocolTranslation {
            direction,
            source_family,
            target_family,
            reason: "protocol translation behavior is intentionally deferred".to_string(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Deferred protocol translation details that can be converted into a core error.
pub struct DeferredProtocolTranslation {
    /// Request or response translation direction.
    pub direction: ProtocolTranslationDirection,
    /// Original protocol family for translation.
    pub source_family: ProtocolFamily,
    /// Desired protocol family for translation.
    pub target_family: ProtocolFamily,
    /// Human-readable reason for this state.
    pub reason: String,
}

impl DeferredProtocolTranslation {
    /// Converts deferred translation details into a structured core error.
    pub fn into_error(self) -> CoreError {
        CoreError::ProtocolTranslationDeferred {
            direction: self.direction,
            source_family: self.source_family,
            target_family: self.target_family,
            message: self.reason,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Direction of a request or response protocol translation.
pub enum ProtocolTranslationDirection {
    /// Translation applies to a request.
    Request,
    /// Translation applies to a response.
    Response,
}

fn validate_required_text(field: &'static str, value: &str) -> Result<(), CoreError> {
    if value.trim().is_empty() {
        return Err(CoreError::ProtocolValidation {
            field,
            message: "value must not be blank".to_string(),
        });
    }

    Ok(())
}

fn validate_optional_text(field: &'static str, value: Option<&str>) -> Result<(), CoreError> {
    if matches!(value, Some(value) if value.trim().is_empty()) {
        return Err(CoreError::ProtocolValidation {
            field,
            message: "value must not be blank when present".to_string(),
        });
    }

    Ok(())
}
