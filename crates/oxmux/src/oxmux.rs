pub mod configuration;
pub mod errors;
pub mod local_proxy_runtime;
pub mod management;
pub mod protocol;
pub mod provider;
pub mod routing;
pub mod streaming;
pub mod usage;

pub use configuration::{
    ConfigurationSnapshot, ConfigurationUpdateIntent, RoutingDefault, ValidatedConfigurationUpdate,
};
pub use errors::CoreError;
pub use local_proxy_runtime::{
    LOCAL_HEALTH_PATH, LOCAL_HEALTH_RESPONSE_BODY, LocalHealthRuntime, LocalHealthRuntimeConfig,
    LocalHealthRuntimeStatus,
};
pub use management::{
    BoundEndpoint, CoreHealthState, LifecycleControlIntent, ManagementSnapshot,
    ProxyLifecycleState, UptimeMetadata,
};
pub use protocol::{
    CanonicalProtocolRequest, CanonicalProtocolResponse, ClaudeProtocolMetadata,
    CodexProtocolMetadata, DeferredProtocolTranslation, GeminiProtocolMetadata,
    OpenAiProtocolMetadata, ProtocolBoundary, ProtocolMetadata, ProtocolPayload,
    ProtocolPayloadBody, ProtocolResponseStatus, ProtocolTranslationDirection,
    ProtocolTranslationOutcome, ProtocolTranslator, ProviderSpecificProtocolMetadata,
};
pub use provider::{
    AccountSummary, AuthMethodCategory, AuthState, DegradedReason, LastCheckedMetadata,
    ProtocolFamily, ProviderCapability, ProviderSummary,
};
pub use usage::{MeteredValue, QuotaState, QuotaSummary, UsageSummary};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CoreIdentity {
    pub crate_name: &'static str,
    pub version: &'static str,
}

pub const CORE_IDENTITY: CoreIdentity = CoreIdentity {
    crate_name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

pub fn core_identity() -> CoreIdentity {
    CORE_IDENTITY
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_core_identity() {
        assert_eq!(core_identity().crate_name, "oxmux");
        assert_eq!(core_identity().version, "0.1.0");
    }
}
