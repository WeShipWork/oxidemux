pub mod configuration;
pub mod errors;
pub mod local_proxy_runtime;
pub mod management;
pub mod minimal_proxy;
pub mod protocol;
pub mod provider;
pub mod routing;
pub mod streaming;
pub mod usage;

pub use configuration::{
    AutoStartIntent, ConfigurationBoundary, ConfigurationLoadFailure, ConfigurationSnapshot,
    ConfigurationUpdateIntent, FileAccountConfiguration, FileBackedManagementConfiguration,
    FileConfigurationState, FileProviderConfiguration, FileProxyConfiguration,
    FileRoutingDefaultConfiguration, FileRoutingDefaultGroup, LoggingSetting, RoutingDefault,
    ValidatedConfigurationUpdate, ValidatedFileConfiguration,
};
pub use errors::{
    ConfigurationError, ConfigurationErrorKind, ConfigurationSourceMetadata, CoreError,
    InvalidConfigurationValue,
};
pub use local_proxy_runtime::{
    LOCAL_HEALTH_PATH, LOCAL_HEALTH_RESPONSE_BODY, LocalHealthRuntime, LocalHealthRuntimeConfig,
    LocalHealthRuntimeStatus, LocalProxyRouteConfig,
};
pub use management::{
    BoundEndpoint, CoreHealthState, LifecycleControlIntent, ManagementSnapshot,
    ProxyLifecycleState, UptimeMetadata,
};
pub use minimal_proxy::{
    MAX_MINIMAL_PROXY_BODY_BYTES, MINIMAL_CHAT_COMPLETIONS_PATH, MINIMAL_PROXY_JSON_CONTENT_TYPE,
    MinimalProxyEngine, MinimalProxyEngineConfig, MinimalProxyErrorCode, MinimalProxyRequest,
    MinimalProxyResponse,
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
    MockProviderAccount, MockProviderHarness, MockProviderOutcome, ProtocolFamily,
    ProviderCapability, ProviderExecutionFailure, ProviderExecutionMetadata,
    ProviderExecutionOutcome, ProviderExecutionRequest, ProviderExecutionResult, ProviderExecutor,
    ProviderSummary,
};
pub use routing::{
    FallbackBehavior, ModelAlias, ModelRoute, RoutingAvailabilitySnapshot,
    RoutingAvailabilityState, RoutingBoundary, RoutingCandidate, RoutingDecisionMode,
    RoutingFailure, RoutingPolicy, RoutingSelectionRequest, RoutingSelectionResult,
    RoutingSkipReason, RoutingTarget, RoutingTargetAvailability, SkippedRoutingCandidate,
};
pub use streaming::{
    CancellationReason, InvalidStreamSequence, ResponseMode, StreamContent, StreamEvent,
    StreamFailure, StreamMetadata, StreamTerminalState, StreamingBoundary, StreamingFailure,
    StreamingResponse,
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
