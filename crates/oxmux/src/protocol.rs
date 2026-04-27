use crate::{CoreError, ProtocolFamily};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProtocolBoundary;

impl ProtocolBoundary {
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

pub trait ProtocolTranslator {
    fn translate_request(
        &self,
        request: CanonicalProtocolRequest,
        target_protocol: ProtocolMetadata,
    ) -> Result<ProtocolTranslationOutcome<CanonicalProtocolRequest>, CoreError>;

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
pub struct CanonicalProtocolRequest {
    pub protocol: ProtocolMetadata,
    pub model: String,
    pub payload: ProtocolPayload,
}

impl CanonicalProtocolRequest {
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

    pub fn validate(&self) -> Result<(), CoreError> {
        self.protocol.validate()?;
        validate_required_text("model", &self.model)?;
        self.payload.validate()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalProtocolResponse {
    pub protocol: ProtocolMetadata,
    pub status: ProtocolResponseStatus,
    pub payload: ProtocolPayload,
}

impl CanonicalProtocolResponse {
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

    pub fn validate(&self) -> Result<(), CoreError> {
        self.protocol.validate()?;
        self.status.validate()?;
        self.payload.validate()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProtocolMetadata {
    OpenAi(OpenAiProtocolMetadata),
    Gemini(GeminiProtocolMetadata),
    Claude(ClaudeProtocolMetadata),
    Codex(CodexProtocolMetadata),
    ProviderSpecific(ProviderSpecificProtocolMetadata),
}

impl ProtocolMetadata {
    pub fn open_ai() -> Self {
        Self::OpenAi(OpenAiProtocolMetadata::default())
    }

    pub fn gemini() -> Self {
        Self::Gemini(GeminiProtocolMetadata::default())
    }

    pub fn claude() -> Self {
        Self::Claude(ClaudeProtocolMetadata::default())
    }

    pub fn codex() -> Self {
        Self::Codex(CodexProtocolMetadata::default())
    }

    pub fn provider_specific(
        provider_id: impl Into<String>,
        format_name: impl Into<String>,
    ) -> Result<Self, CoreError> {
        Ok(Self::ProviderSpecific(
            ProviderSpecificProtocolMetadata::new(provider_id, format_name)?,
        ))
    }

    pub fn family(&self) -> ProtocolFamily {
        match self {
            Self::OpenAi(_) => ProtocolFamily::OpenAi,
            Self::Gemini(_) => ProtocolFamily::Gemini,
            Self::Claude(_) => ProtocolFamily::Claude,
            Self::Codex(_) => ProtocolFamily::Codex,
            Self::ProviderSpecific(_) => ProtocolFamily::ProviderSpecific,
        }
    }

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
pub struct OpenAiProtocolMetadata {
    pub api_version: Option<String>,
}

impl OpenAiProtocolMetadata {
    fn validate(&self) -> Result<(), CoreError> {
        validate_optional_text("openai.api_version", self.api_version.as_deref())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GeminiProtocolMetadata {
    pub api_version: Option<String>,
}

impl GeminiProtocolMetadata {
    fn validate(&self) -> Result<(), CoreError> {
        validate_optional_text("gemini.api_version", self.api_version.as_deref())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ClaudeProtocolMetadata {
    pub api_version: Option<String>,
}

impl ClaudeProtocolMetadata {
    fn validate(&self) -> Result<(), CoreError> {
        validate_optional_text("claude.api_version", self.api_version.as_deref())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CodexProtocolMetadata {
    pub api_version: Option<String>,
}

impl CodexProtocolMetadata {
    fn validate(&self) -> Result<(), CoreError> {
        validate_optional_text("codex.api_version", self.api_version.as_deref())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderSpecificProtocolMetadata {
    pub provider_id: String,
    pub format_name: String,
}

impl ProviderSpecificProtocolMetadata {
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
pub struct ProtocolPayload {
    pub content_type: Option<String>,
    pub body: ProtocolPayloadBody,
}

impl ProtocolPayload {
    pub fn empty() -> Self {
        Self {
            content_type: None,
            body: ProtocolPayloadBody::Empty,
        }
    }

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
pub enum ProtocolPayloadBody {
    Empty,
    Opaque(Vec<u8>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolResponseStatus {
    pub code: u16,
    pub reason: Option<String>,
}

impl ProtocolResponseStatus {
    pub fn new(code: u16, reason: Option<String>) -> Result<Self, CoreError> {
        let status = Self { code, reason };
        status.validate()?;
        Ok(status)
    }

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
pub enum ProtocolTranslationOutcome<T> {
    Translated(T),
    Deferred(DeferredProtocolTranslation),
}

impl<T> ProtocolTranslationOutcome<T> {
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
pub struct DeferredProtocolTranslation {
    pub direction: ProtocolTranslationDirection,
    pub source_family: ProtocolFamily,
    pub target_family: ProtocolFamily,
    pub reason: String,
}

impl DeferredProtocolTranslation {
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
pub enum ProtocolTranslationDirection {
    Request,
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
