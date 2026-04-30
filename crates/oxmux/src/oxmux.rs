//! Headless core facade for OxideMux subscription-aware proxy semantics.
//!
//! `oxmux` owns reusable behavior that must work without the desktop shell:
//! provider/account state, routing policy, protocol compatibility, request and
//! response translation boundaries, local proxy contracts, management snapshots,
//! configuration state, usage/quota summaries, streaming state, and structured
//! errors. The `oxidemux` application crate consumes this facade to adapt those
//! core contracts into platform-specific presentation and lifecycle behavior.

#![warn(missing_docs)]

/// Configuration loading, validation, layering, and management snapshot state.
pub mod configuration;
/// Structured error types shared by headless core boundaries.
pub mod errors;
/// Local client authorization contracts for loopback proxy access.
pub mod local_client_auth;
/// Local loopback runtime contracts for health and minimal proxy serving.
pub mod local_proxy_runtime;
/// Management snapshot and lifecycle state contracts.
pub mod management;
/// Minimal local proxy request, response, and smoke-route engine contracts.
pub mod minimal_proxy;
/// Protocol metadata, payload, and deferred translation contracts.
pub mod protocol;
/// Provider execution, account summary, auth state, and mock harness contracts.
pub mod provider;
/// Model routing, target availability, fallback, and selection contracts.
pub mod routing;
/// Complete and streaming response state contracts.
pub mod streaming;
/// Usage meter and quota summary contracts.
pub mod usage;

pub use configuration::{
    AutoStartIntent, ConfigurationBoundary, ConfigurationFingerprint, ConfigurationLayerKind,
    ConfigurationLayerSource, ConfigurationLoadFailure, ConfigurationSnapshot,
    ConfigurationUpdateIntent, FileAccountConfiguration, FileBackedManagementConfiguration,
    FileConfigurationState, FileProviderConfiguration, FileProxyConfiguration,
    FileRoutingDefaultConfiguration, FileRoutingDefaultGroup, LayeredConfigurationInput,
    LayeredConfigurationRejectedCandidate, LayeredConfigurationReloadOutcome,
    LayeredConfigurationState, LoggingSetting, RoutingDefault, ValidatedConfigurationUpdate,
    ValidatedFileConfiguration, ValidatedLayeredConfiguration,
};
pub use errors::{
    ConfigurationError, ConfigurationErrorKind, ConfigurationSourceMetadata, CoreError,
    InvalidConfigurationValue,
};
pub use local_client_auth::{
    LocalClientAuthorizationAttempt, LocalClientAuthorizationFailure,
    LocalClientAuthorizationFailureReason, LocalClientAuthorizationOutcome,
    LocalClientAuthorizationPolicy, LocalClientAuthorizationPolicyMetadata, LocalClientCredential,
    LocalClientCredentialError, LocalClientRouteScope, LocalRouteProtection,
    LocalRouteProtectionMetadata, RedactedLocalClientCredentialMetadata,
};
pub use local_proxy_runtime::{
    LOCAL_HEALTH_PATH, LOCAL_HEALTH_RESPONSE_BODY, LocalHealthRuntime, LocalHealthRuntimeConfig,
    LocalHealthRuntimeStatus, LocalProxyRouteConfig,
};
pub use management::{
    BoundEndpoint, CoreHealthState, LayeredManagementConfiguration, LifecycleControlIntent,
    ManagementSnapshot, ProxyLifecycleState, UptimeMetadata,
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
/// Static identity metadata for the reusable core crate.
pub struct CoreIdentity {
    /// Cargo package name for the core facade crate.
    pub crate_name: &'static str,
    /// Cargo package version compiled into the crate.
    pub version: &'static str,
}

/// Compile-time identity for the `oxmux` core facade.
pub const CORE_IDENTITY: CoreIdentity = CoreIdentity {
    crate_name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

/// Returns the compile-time identity for the reusable core facade.
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
