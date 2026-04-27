#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProviderAuthBoundary;

use crate::usage::QuotaState;

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
