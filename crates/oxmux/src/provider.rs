#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProviderAuthBoundary;

use crate::CoreError;
use crate::protocol::{CanonicalProtocolRequest, CanonicalProtocolResponse};
use crate::streaming::{ResponseMode, StreamingResponse};
use crate::usage::QuotaState;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderExecutionRequest {
    pub provider_id: String,
    pub account_id: Option<String>,
    pub request: CanonicalProtocolRequest,
}

impl ProviderExecutionRequest {
    pub fn new(
        provider_id: impl Into<String>,
        account_id: Option<String>,
        request: CanonicalProtocolRequest,
    ) -> Result<Self, CoreError> {
        let request = Self {
            provider_id: provider_id.into(),
            account_id,
            request,
        };
        request.validate()?;
        Ok(request)
    }

    pub fn validate(&self) -> Result<(), CoreError> {
        validate_required_text("provider_id", &self.provider_id)?;
        validate_optional_text("account_id", self.account_id.as_deref())?;
        self.request.validate()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderExecutionResult {
    pub outcome: ProviderExecutionOutcome,
    pub metadata: ProviderExecutionMetadata,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProviderExecutionOutcome {
    Success(ResponseMode),
    Degraded {
        response_mode: ResponseMode,
        reasons: Vec<DegradedReason>,
    },
    QuotaLimited {
        response_mode: ResponseMode,
        quota_state: QuotaState,
    },
}

impl ProviderExecutionOutcome {
    pub fn response_mode(&self) -> &ResponseMode {
        match self {
            Self::Success(response_mode)
            | Self::Degraded { response_mode, .. }
            | Self::QuotaLimited { response_mode, .. } => response_mode,
        }
    }

    pub fn complete_response(&self) -> Option<&CanonicalProtocolResponse> {
        self.response_mode().complete_response()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderExecutionMetadata {
    pub provider: ProviderSummary,
    pub account: Option<AccountSummary>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProviderExecutionFailure {
    InvalidSelection { message: String },
    FailedOutcome { code: String, message: String },
}

pub trait ProviderExecutor {
    fn execute(
        &self,
        request: ProviderExecutionRequest,
    ) -> Result<ProviderExecutionResult, CoreError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MockProviderHarness {
    provider_id: String,
    display_name: String,
    protocol_family: ProtocolFamily,
    auth_method: AuthMethodCategory,
    routing_eligible: bool,
    account: Option<MockProviderAccount>,
    outcome: MockProviderOutcome,
}

impl MockProviderHarness {
    pub fn new(
        provider_id: impl Into<String>,
        display_name: impl Into<String>,
        protocol_family: ProtocolFamily,
        auth_method: AuthMethodCategory,
        outcome: MockProviderOutcome,
    ) -> Result<Self, CoreError> {
        let harness = Self {
            provider_id: provider_id.into(),
            display_name: display_name.into(),
            protocol_family,
            auth_method,
            routing_eligible: true,
            account: None,
            outcome,
        };
        harness.validate()?;
        Ok(harness)
    }

    pub fn with_account(mut self, account: MockProviderAccount) -> Self {
        self.account = Some(account);
        self
    }

    pub fn with_routing_eligible(mut self, routing_eligible: bool) -> Self {
        self.routing_eligible = routing_eligible;
        self
    }

    pub fn provider_summary(&self) -> ProviderSummary {
        let account = self
            .account
            .as_ref()
            .map(|account| account.summary(&self.outcome));
        ProviderSummary {
            provider_id: self.provider_id.clone(),
            display_name: self.display_name.clone(),
            capabilities: vec![ProviderCapability {
                protocol_family: self.protocol_family,
                supports_streaming: self.outcome.supports_streaming(),
                auth_method: self.auth_method,
                routing_eligible: self.routing_eligible,
            }],
            accounts: account.iter().cloned().collect(),
            degraded_reasons: self.outcome.degraded_reasons(),
        }
    }

    pub fn account_summary(&self) -> Option<AccountSummary> {
        self.account
            .as_ref()
            .map(|account| account.summary(&self.outcome))
    }

    fn metadata(&self) -> ProviderExecutionMetadata {
        ProviderExecutionMetadata {
            provider: self.provider_summary(),
            account: self.account_summary(),
        }
    }

    fn validate(&self) -> Result<(), CoreError> {
        validate_required_text("provider_id", &self.provider_id)?;
        validate_required_text("display_name", &self.display_name)?;
        if let Some(account) = &self.account {
            account.validate()?;
        }
        self.outcome.validate()?;
        Ok(())
    }
}

impl ProviderExecutor for MockProviderHarness {
    fn execute(
        &self,
        request: ProviderExecutionRequest,
    ) -> Result<ProviderExecutionResult, CoreError> {
        request.validate()?;
        if request.provider_id != self.provider_id {
            return Err(CoreError::ProviderExecution {
                provider_id: request.provider_id,
                account_id: request.account_id,
                failure: ProviderExecutionFailure::InvalidSelection {
                    message: "request provider does not match mock provider".to_string(),
                },
            });
        }

        if request.account_id
            != self
                .account
                .as_ref()
                .map(|account| account.account_id.clone())
        {
            return Err(CoreError::ProviderExecution {
                provider_id: request.provider_id,
                account_id: request.account_id,
                failure: ProviderExecutionFailure::InvalidSelection {
                    message: "request account does not match mock provider account".to_string(),
                },
            });
        }

        match &self.outcome {
            MockProviderOutcome::Success(response) => Ok(ProviderExecutionResult {
                outcome: ProviderExecutionOutcome::Success(ResponseMode::complete(
                    response.clone(),
                )),
                metadata: self.metadata(),
            }),
            MockProviderOutcome::SuccessWithMode { response_mode, .. } => {
                Ok(ProviderExecutionResult {
                    outcome: ProviderExecutionOutcome::Success(response_mode.clone()),
                    metadata: self.metadata(),
                })
            }
            MockProviderOutcome::Degraded { response, reasons } => Ok(ProviderExecutionResult {
                outcome: ProviderExecutionOutcome::Degraded {
                    response_mode: ResponseMode::complete(response.clone()),
                    reasons: reasons.clone(),
                },
                metadata: self.metadata(),
            }),
            MockProviderOutcome::DegradedWithMode {
                response_mode,
                supports_streaming: _,
                reasons,
            } => Ok(ProviderExecutionResult {
                outcome: ProviderExecutionOutcome::Degraded {
                    response_mode: response_mode.clone(),
                    reasons: reasons.clone(),
                },
                metadata: self.metadata(),
            }),
            MockProviderOutcome::QuotaLimited {
                response,
                quota_state,
            } => Ok(ProviderExecutionResult {
                outcome: ProviderExecutionOutcome::QuotaLimited {
                    response_mode: ResponseMode::complete(response.clone()),
                    quota_state: quota_state.clone(),
                },
                metadata: self.metadata(),
            }),
            MockProviderOutcome::QuotaLimitedWithMode {
                response_mode,
                supports_streaming: _,
                quota_state,
            } => Ok(ProviderExecutionResult {
                outcome: ProviderExecutionOutcome::QuotaLimited {
                    response_mode: response_mode.clone(),
                    quota_state: quota_state.clone(),
                },
                metadata: self.metadata(),
            }),
            MockProviderOutcome::Streaming(response) => Ok(ProviderExecutionResult {
                outcome: ProviderExecutionOutcome::Success(ResponseMode::Streaming(
                    response.clone(),
                )),
                metadata: self.metadata(),
            }),
            MockProviderOutcome::Failed(failure) => Err(CoreError::ProviderExecution {
                provider_id: request.provider_id,
                account_id: request.account_id,
                failure: failure.clone(),
            }),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MockProviderAccount {
    pub account_id: String,
    pub display_name: String,
    pub auth_state: AuthState,
    pub quota_state: QuotaState,
    pub last_checked: Option<LastCheckedMetadata>,
    pub degraded_reasons: Vec<DegradedReason>,
}

impl MockProviderAccount {
    pub fn new(account_id: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            account_id: account_id.into(),
            display_name: display_name.into(),
            auth_state: AuthState::Authenticated,
            quota_state: QuotaState::Unknown,
            last_checked: None,
            degraded_reasons: Vec::new(),
        }
    }

    pub fn with_quota_state(mut self, quota_state: QuotaState) -> Self {
        self.quota_state = quota_state;
        self
    }

    pub fn with_auth_state(mut self, auth_state: AuthState) -> Self {
        self.auth_state = auth_state;
        self
    }

    pub fn with_last_checked(mut self, last_checked: LastCheckedMetadata) -> Self {
        self.last_checked = Some(last_checked);
        self
    }

    pub fn with_degraded_reason(mut self, degraded_reason: DegradedReason) -> Self {
        self.degraded_reasons.push(degraded_reason);
        self
    }

    fn summary(&self, outcome: &MockProviderOutcome) -> AccountSummary {
        let mut degraded_reasons = self.degraded_reasons.clone();
        degraded_reasons.extend(outcome.degraded_reasons());
        AccountSummary {
            account_id: self.account_id.clone(),
            display_name: self.display_name.clone(),
            auth_state: self.auth_state.clone(),
            quota_state: outcome
                .quota_state()
                .unwrap_or_else(|| self.quota_state.clone()),
            last_checked: self.last_checked,
            degraded_reasons,
        }
    }

    fn validate(&self) -> Result<(), CoreError> {
        validate_required_text("account_id", &self.account_id)?;
        validate_required_text("account_display_name", &self.display_name)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MockProviderOutcome {
    Success(CanonicalProtocolResponse),
    SuccessWithMode {
        response_mode: ResponseMode,
        supports_streaming: bool,
    },
    Degraded {
        response: CanonicalProtocolResponse,
        reasons: Vec<DegradedReason>,
    },
    DegradedWithMode {
        response_mode: ResponseMode,
        supports_streaming: bool,
        reasons: Vec<DegradedReason>,
    },
    QuotaLimited {
        response: CanonicalProtocolResponse,
        quota_state: QuotaState,
    },
    QuotaLimitedWithMode {
        response_mode: ResponseMode,
        supports_streaming: bool,
        quota_state: QuotaState,
    },
    Streaming(StreamingResponse),
    Failed(ProviderExecutionFailure),
}

impl MockProviderOutcome {
    pub fn complete_streaming_capable(response: CanonicalProtocolResponse) -> Self {
        Self::SuccessWithMode {
            response_mode: ResponseMode::complete(response),
            supports_streaming: true,
        }
    }

    pub fn streaming(response: StreamingResponse) -> Self {
        Self::Streaming(response)
    }

    fn supports_streaming(&self) -> bool {
        match self {
            Self::SuccessWithMode {
                supports_streaming,
                response_mode,
            } => *supports_streaming || matches!(response_mode, ResponseMode::Streaming(_)),
            Self::DegradedWithMode {
                response_mode,
                supports_streaming,
                ..
            }
            | Self::QuotaLimitedWithMode {
                response_mode,
                supports_streaming,
                ..
            } => *supports_streaming || matches!(response_mode, ResponseMode::Streaming(_)),
            Self::Streaming(_) => true,
            Self::Success(_)
            | Self::Degraded { .. }
            | Self::QuotaLimited { .. }
            | Self::Failed(_) => false,
        }
    }

    fn degraded_reasons(&self) -> Vec<DegradedReason> {
        match self {
            Self::Degraded { reasons, .. } | Self::DegradedWithMode { reasons, .. } => {
                reasons.clone()
            }
            Self::Failed(failure) => vec![DegradedReason {
                component: "provider_execution".to_string(),
                message: failure.message().to_string(),
            }],
            _ => Vec::new(),
        }
    }

    fn quota_state(&self) -> Option<QuotaState> {
        match self {
            Self::QuotaLimited { quota_state, .. }
            | Self::QuotaLimitedWithMode { quota_state, .. } => Some(quota_state.clone()),
            _ => None,
        }
    }

    fn validate(&self) -> Result<(), CoreError> {
        match self {
            Self::SuccessWithMode { response_mode, .. }
            | Self::DegradedWithMode { response_mode, .. }
            | Self::QuotaLimitedWithMode { response_mode, .. } => {
                if let ResponseMode::Streaming(response) = response_mode {
                    response.validate()?;
                }
            }
            Self::Streaming(response) => response.validate()?,
            Self::Success(_)
            | Self::Degraded { .. }
            | Self::QuotaLimited { .. }
            | Self::Failed(_) => {}
        }

        Ok(())
    }
}

impl ProviderExecutionFailure {
    pub fn failed_outcome(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::FailedOutcome {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        match self {
            Self::InvalidSelection { message } | Self::FailedOutcome { message, .. } => message,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderSummary {
    pub provider_id: String,
    pub display_name: String,
    pub capabilities: Vec<ProviderCapability>,
    pub accounts: Vec<AccountSummary>,
    pub degraded_reasons: Vec<DegradedReason>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderCapability {
    pub protocol_family: ProtocolFamily,
    pub supports_streaming: bool,
    pub auth_method: AuthMethodCategory,
    pub routing_eligible: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProtocolFamily {
    OpenAi,
    Gemini,
    Claude,
    Codex,
    ProviderSpecific,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthMethodCategory {
    ApiKey,
    OAuth,
    ExternalReference,
    None,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountSummary {
    pub account_id: String,
    pub display_name: String,
    pub auth_state: AuthState,
    pub quota_state: QuotaState,
    pub last_checked: Option<LastCheckedMetadata>,
    pub degraded_reasons: Vec<DegradedReason>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthState {
    Unknown,
    Unconfigured,
    CredentialReference { reference_name: String },
    Authenticated,
    Expired,
    Failed { reason: String },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LastCheckedMetadata {
    pub unix_timestamp_seconds: u64,
    pub age_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DegradedReason {
    pub component: String,
    pub message: String,
}

fn validate_required_text(field: &'static str, value: &str) -> Result<(), CoreError> {
    if value.trim().is_empty() {
        return Err(CoreError::ProviderExecution {
            provider_id: String::new(),
            account_id: None,
            failure: ProviderExecutionFailure::InvalidSelection {
                message: format!("{field} must not be blank"),
            },
        });
    }

    Ok(())
}

fn validate_optional_text(field: &'static str, value: Option<&str>) -> Result<(), CoreError> {
    if matches!(value, Some(value) if value.trim().is_empty()) {
        return Err(CoreError::ProviderExecution {
            provider_id: String::new(),
            account_id: None,
            failure: ProviderExecutionFailure::InvalidSelection {
                message: format!("{field} must not be blank when present"),
            },
        });
    }

    Ok(())
}
