//! Provider and account execution contracts for the headless core.
//!
//! The types here describe provider capabilities, account health, auth state,
//! quota references, deterministic mock execution, and execution failures without
//! implementing provider SDKs or OAuth flows.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Marker for provider authentication ownership in the headless core boundary.
pub struct ProviderAuthBoundary;

use crate::CoreError;
use crate::protocol::{CanonicalProtocolRequest, CanonicalProtocolResponse};
use crate::streaming::{ResponseMode, StreamingResponse};
use crate::usage::QuotaState;

#[derive(Clone, Debug, Eq, PartialEq)]
/// Canonical request addressed to a selected provider and optional account.
pub struct ProviderExecutionRequest {
    /// Provider identifier used by routing, execution, and management state.
    pub provider_id: String,
    /// Optional account identifier scoped to the provider.
    pub account_id: Option<String>,
    /// Canonical protocol request to execute.
    pub request: CanonicalProtocolRequest,
}

impl ProviderExecutionRequest {
    /// Creates a provider execution request for a provider, optional account, and canonical request.
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

    /// Validates this value and returns a structured core error on failure.
    pub fn validate(&self) -> Result<(), CoreError> {
        validate_required_text("provider_id", &self.provider_id)?;
        validate_optional_text("account_id", self.account_id.as_deref())?;
        self.request.validate()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Provider execution outcome together with provider/account metadata.
pub struct ProviderExecutionResult {
    /// Execution outcome returned by the provider boundary.
    pub outcome: ProviderExecutionOutcome,
    /// Provider and account metadata observed during execution.
    pub metadata: ProviderExecutionMetadata,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Successful, degraded, or quota-limited provider execution state.
pub enum ProviderExecutionOutcome {
    /// Provider execution completed successfully.
    Success(ResponseMode),
    /// Provider execution completed with reduced quality or capability.
    Degraded {
        /// Complete or streaming response mode for this outcome.
        response_mode: ResponseMode,
        /// Reasons explaining degraded state.
        reasons: Vec<DegradedReason>,
    },
    /// Provider execution is limited by quota state.
    QuotaLimited {
        /// Complete or streaming response mode for this outcome.
        response_mode: ResponseMode,
        /// Quota state associated with this account or outcome.
        quota_state: QuotaState,
    },
}

impl ProviderExecutionOutcome {
    /// Returns the response mode carried by this provider execution outcome.
    pub fn response_mode(&self) -> &ResponseMode {
        match self {
            Self::Success(response_mode)
            | Self::Degraded { response_mode, .. }
            | Self::QuotaLimited { response_mode, .. } => response_mode,
        }
    }

    /// Returns the complete response when this mode is complete.
    pub fn complete_response(&self) -> Option<&CanonicalProtocolResponse> {
        self.response_mode().complete_response()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Provider and account summary attached to an execution result.
pub struct ProviderExecutionMetadata {
    /// Provider summary associated with this value.
    pub provider: ProviderSummary,
    /// Optional account summary associated with this value.
    pub account: Option<AccountSummary>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Matchable failure returned by provider selection or execution.
pub enum ProviderExecutionFailure {
    /// Requested provider or account did not match execution state.
    InvalidSelection {
        /// Human-readable diagnostic message.
        message: String,
    },
    /// Provider execution failed with a stable code and message.
    FailedOutcome {
        /// Stable code for this failure or response.
        code: String,
        /// Human-readable diagnostic message.
        message: String,
    },
}

/// Trait implemented by provider execution backends used by routing and proxy code.
pub trait ProviderExecutor {
    /// Executes this boundary operation and returns structured core errors on failure.
    fn execute(
        &self,
        request: ProviderExecutionRequest,
    ) -> Result<ProviderExecutionResult, CoreError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Deterministic provider executor for tests and headless examples.
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
    /// Creates a mock provider harness for deterministic provider execution tests.
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

    /// Attaches an account to the mock provider harness.
    pub fn with_account(mut self, account: MockProviderAccount) -> Self {
        self.account = Some(account);
        self
    }

    /// Sets whether the mock provider is eligible for routing.
    pub fn with_routing_eligible(mut self, routing_eligible: bool) -> Self {
        self.routing_eligible = routing_eligible;
        self
    }

    /// Returns provider summary state for this mock provider.
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

    /// Returns account summary state for this mock provider, when configured.
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
/// Configurable mock account state used by the mock provider harness.
pub struct MockProviderAccount {
    /// Optional account identifier scoped to the provider.
    pub account_id: String,
    /// Human-readable name for provider or account display.
    pub display_name: String,
    /// Current authentication state for this account.
    pub auth_state: AuthState,
    /// Quota state associated with this account or outcome.
    pub quota_state: QuotaState,
    /// Optional freshness metadata for account health.
    pub last_checked: Option<LastCheckedMetadata>,
    /// Reasons this provider or account is degraded.
    pub degraded_reasons: Vec<DegradedReason>,
}

impl MockProviderAccount {
    /// Creates a mock provider account with authenticated, unknown-quota defaults.
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

    /// Sets mock account quota state.
    pub fn with_quota_state(mut self, quota_state: QuotaState) -> Self {
        self.quota_state = quota_state;
        self
    }

    /// Sets mock account authentication state.
    pub fn with_auth_state(mut self, auth_state: AuthState) -> Self {
        self.auth_state = auth_state;
        self
    }

    /// Sets mock account health-check freshness metadata.
    pub fn with_last_checked(mut self, last_checked: LastCheckedMetadata) -> Self {
        self.last_checked = Some(last_checked);
        self
    }

    /// Adds a degraded reason to the mock account.
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
/// Deterministic outcome emitted by a mock provider harness.
pub enum MockProviderOutcome {
    /// Provider execution completed successfully.
    Success(CanonicalProtocolResponse),
    /// Mock outcome succeeds with an explicit response mode.
    SuccessWithMode {
        /// Complete or streaming response mode for this outcome.
        response_mode: ResponseMode,
        /// Whether this capability or outcome supports streaming responses.
        supports_streaming: bool,
    },
    /// Mock provider returns a degraded response with explicit reasons.
    Degraded {
        /// Canonical response associated with this outcome.
        response: CanonicalProtocolResponse,
        /// Reasons explaining degraded state.
        reasons: Vec<DegradedReason>,
    },
    /// Mock outcome is degraded with an explicit response mode.
    DegradedWithMode {
        /// Complete or streaming response mode for this outcome.
        response_mode: ResponseMode,
        /// Whether this outcome advertises streaming support.
        supports_streaming: bool,
        /// Reasons explaining degraded state.
        reasons: Vec<DegradedReason>,
    },
    /// Provider execution is limited by quota state.
    QuotaLimited {
        /// Canonical response associated with this outcome.
        response: CanonicalProtocolResponse,
        /// Quota state associated with this account or outcome.
        quota_state: QuotaState,
    },
    /// Mock outcome is quota-limited with an explicit response mode.
    QuotaLimitedWithMode {
        /// Complete or streaming response mode for this outcome.
        response_mode: ResponseMode,
        /// Whether this outcome advertises streaming support.
        supports_streaming: bool,
        /// Quota state associated with this account or outcome.
        quota_state: QuotaState,
    },
    /// Provider execution returns a streaming response.
    Streaming(StreamingResponse),
    /// Operation or account state failed with a reason.
    Failed(ProviderExecutionFailure),
}

impl MockProviderOutcome {
    /// Creates a complete mock outcome that advertises streaming capability.
    pub fn complete_streaming_capable(response: CanonicalProtocolResponse) -> Self {
        Self::SuccessWithMode {
            response_mode: ResponseMode::complete(response),
            supports_streaming: true,
        }
    }

    /// Creates a validated streaming response mode.
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
    /// Creates a failed provider execution outcome.
    pub fn failed_outcome(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::FailedOutcome {
            code: code.into(),
            message: message.into(),
        }
    }

    /// Returns a human-readable message for this diagnostic.
    pub fn message(&self) -> &str {
        match self {
            Self::InvalidSelection { message } | Self::FailedOutcome { message, .. } => message,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Management-visible provider capability and account summary.
pub struct ProviderSummary {
    /// Provider identifier used by routing, execution, and management state.
    pub provider_id: String,
    /// Human-readable name for provider or account display.
    pub display_name: String,
    /// Capabilities advertised by this provider.
    pub capabilities: Vec<ProviderCapability>,
    /// Account summaries or declarations for this provider.
    pub accounts: Vec<AccountSummary>,
    /// Reasons this provider or account is degraded.
    pub degraded_reasons: Vec<DegradedReason>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// One protocol/auth/routing capability advertised by a provider.
pub struct ProviderCapability {
    /// Protocol family supported by this capability.
    pub protocol_family: ProtocolFamily,
    /// Whether this capability or outcome supports streaming responses.
    pub supports_streaming: bool,
    /// Authentication method category for this capability.
    pub auth_method: AuthMethodCategory,
    /// Whether routing may select this provider.
    pub routing_eligible: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Protocol family understood by core protocol and provider metadata.
pub enum ProtocolFamily {
    /// OpenAI-compatible protocol family or metadata.
    OpenAi,
    /// Gemini-compatible protocol family or metadata.
    Gemini,
    /// Claude-compatible protocol family or metadata.
    Claude,
    /// Codex-compatible protocol family or metadata.
    Codex,
    /// Provider-specific protocol family or metadata.
    ProviderSpecific,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// High-level credential method category for provider/account summaries.
pub enum AuthMethodCategory {
    /// API-key based authentication category.
    ApiKey,
    /// OAuth-based authentication category marker; no OAuth flow is implemented here.
    OAuth,
    /// Credential reference managed outside the core.
    ExternalReference,
    /// No authentication category is required or known.
    None,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Management-visible provider account health and quota summary.
pub struct AccountSummary {
    /// Optional account identifier scoped to the provider.
    pub account_id: String,
    /// Human-readable name for provider or account display.
    pub display_name: String,
    /// Current authentication state for this account.
    pub auth_state: AuthState,
    /// Quota state associated with this account or outcome.
    pub quota_state: QuotaState,
    /// Optional freshness metadata for account health.
    pub last_checked: Option<LastCheckedMetadata>,
    /// Reasons this provider or account is degraded.
    pub degraded_reasons: Vec<DegradedReason>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Current credential or session state known for an account.
pub enum AuthState {
    /// State is not known to the core.
    Unknown,
    /// No credential or account configuration is present.
    Unconfigured,
    /// Credentials are represented by an external reference name.
    CredentialReference {
        /// External credential reference name.
        reference_name: String,
    },
    /// Account is considered authenticated by supplied state.
    Authenticated,
    /// Account credentials or session are expired.
    Expired,
    /// Operation or account state failed with a reason.
    Failed {
        /// Human-readable reason for this state.
        reason: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Timestamp and age metadata for a provider/account health check.
pub struct LastCheckedMetadata {
    /// Unix timestamp for the last check.
    pub unix_timestamp_seconds: u64,
    /// Age in seconds since the last check.
    pub age_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Human-readable reason that a provider, account, route, or stream is degraded.
pub struct DegradedReason {
    /// Component reporting the degraded reason.
    pub component: String,
    /// Human-readable diagnostic message.
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
