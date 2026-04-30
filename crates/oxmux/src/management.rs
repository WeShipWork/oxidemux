//! Management snapshot contracts for observing headless core state.
//!
//! Management values aggregate identity, lifecycle, health, configuration,
//! provider, usage, quota, warning, and error state for app-shell and headless
//! consumers without moving core semantics into the desktop crate.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Marker for management snapshot ownership in the headless core.
pub struct ManagementBoundary;

use std::net::SocketAddr;
use std::time::Duration;

use crate::configuration::{
    ConfigurationLoadFailure, ConfigurationSnapshot, FileBackedManagementConfiguration,
    FileConfigurationState, LayeredConfigurationRejectedCandidate,
    LayeredConfigurationReloadOutcome, LayeredConfigurationState,
};
use crate::provider::{DegradedReason, ProviderSummary};
use crate::streaming::{StreamingRobustnessOutcome, StreamingRobustnessOutcomeKind};
use crate::usage::{QuotaSummary, UsageSummary};
use crate::{CoreError, CoreIdentity, LocalRouteProtectionMetadata, core_identity};

#[derive(Clone, Debug, Eq, PartialEq)]
/// Aggregate management view of identity, lifecycle, health, configuration, provider, usage, quota, and error state.
pub struct ManagementSnapshot {
    /// Compiled core identity included in management state.
    pub identity: CoreIdentity,
    /// Current local proxy lifecycle state.
    pub lifecycle: ProxyLifecycleState,
    /// Current core health state.
    pub health: CoreHealthState,
    /// Configuration snapshot visible to management consumers.
    pub configuration: ConfigurationSnapshot,
    /// File-backed configuration details, when active.
    pub file_configuration: Option<FileBackedManagementConfiguration>,
    /// Layered configuration details, when active.
    pub layered_configuration: Option<LayeredManagementConfiguration>,
    /// Most recent file configuration load failure, when any.
    pub last_configuration_load_failure: Option<ConfigurationLoadFailure>,
    /// Most recent rejected layered configuration candidate, when any.
    pub last_layered_configuration_failure: Option<LayeredConfigurationRejectedCandidate>,
    /// Provider summaries visible in management state.
    pub providers: Vec<ProviderSummary>,
    /// Usage summary visible in management state.
    pub usage: UsageSummary,
    /// Quota summary visible in management state.
    pub quota: QuotaSummary,
    /// Local route protection metadata visible in management state.
    pub local_route_protection: LocalRouteProtectionMetadata,
    /// Latest stream robustness outcome supplied by provider or proxy execution.
    pub latest_streaming_outcome: Option<StreamingRobustnessOutcome>,
    /// Non-fatal warnings visible to management consumers.
    pub warnings: Vec<String>,
    /// Structured errors associated with this state.
    pub errors: Vec<CoreError>,
}

impl ManagementSnapshot {
    /// Creates a management snapshot for an inactive bootstrap core.
    pub fn inert_bootstrap() -> Self {
        Self {
            identity: core_identity(),
            lifecycle: ProxyLifecycleState::Stopped,
            health: CoreHealthState::Healthy,
            configuration: ConfigurationSnapshot::local_development(),
            file_configuration: None,
            layered_configuration: None,
            last_configuration_load_failure: None,
            last_layered_configuration_failure: None,
            providers: Vec::new(),
            usage: UsageSummary::zero(),
            quota: QuotaSummary::unknown(),
            local_route_protection: LocalRouteProtectionMetadata::disabled(),
            latest_streaming_outcome: None,
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Builds a management snapshot from file configuration state.
    pub fn from_file_configuration_state(state: &FileConfigurationState) -> Self {
        let mut snapshot = Self::inert_bootstrap();
        if let Some(configuration) = state.active() {
            snapshot.configuration = configuration.configuration_snapshot();
            snapshot.file_configuration =
                Some(FileBackedManagementConfiguration::from(configuration));
            snapshot.providers = configuration.provider_summaries();
            snapshot.usage = configuration.usage_summary();
            snapshot.quota = configuration.quota_summary();
            snapshot.warnings = configuration.warnings.clone();
        }
        snapshot.last_configuration_load_failure = state.last_failure().cloned();
        if let Some(failure) = state.last_failure() {
            snapshot.errors = vec![CoreError::Configuration {
                errors: failure.errors.clone(),
            }];
        }
        snapshot
    }

    /// Builds a management snapshot from layered configuration state.
    pub fn from_layered_configuration_state(state: &LayeredConfigurationState) -> Self {
        let mut snapshot = Self::inert_bootstrap();
        if let Some(active) = state.active() {
            snapshot.configuration = active.configuration.configuration_snapshot();
            snapshot.file_configuration = Some(FileBackedManagementConfiguration::from(
                &active.configuration,
            ));
            snapshot.layered_configuration = Some(LayeredManagementConfiguration {
                active_fingerprint: active.fingerprint.clone(),
                sources: active.sources.clone(),
                latest_reload_outcome: state.latest_reload_outcome().cloned(),
            });
            snapshot.providers = active.configuration.provider_summaries();
            snapshot.usage = active.configuration.usage_summary();
            snapshot.quota = active.configuration.quota_summary();
            snapshot.warnings = active.configuration.warnings.clone();
        }
        snapshot.last_layered_configuration_failure = state.failed_candidate().cloned();
        if let Some(failure) = state.failed_candidate() {
            snapshot.errors = vec![CoreError::Configuration {
                errors: failure.errors.clone(),
            }];
        }
        snapshot
    }

    /// Replaces the latest stream robustness outcome using last-writer-wins semantics.
    pub fn with_streaming_outcome(mut self, outcome: StreamingRobustnessOutcome) -> Self {
        self.record_streaming_outcome(outcome);
        self
    }

    /// Records the latest stream robustness outcome and updates visible warning/error state.
    pub fn record_streaming_outcome(&mut self, outcome: StreamingRobustnessOutcome) {
        self.warnings
            .retain(|warning| !warning.starts_with("streaming: "));
        self.errors
            .retain(|error| !matches!(error, CoreError::Streaming { .. }));
        self.warnings
            .push(format!("streaming: {}", outcome.message()));
        if let Some(error) = outcome.core_error() {
            self.errors.push(error);
        }
        self.latest_streaming_outcome = Some(outcome);
    }
}

impl StreamingRobustnessOutcome {
    fn message(&self) -> String {
        match &self.kind {
            StreamingRobustnessOutcomeKind::Timeout { timeout_ms } => {
                format!("timeout after {timeout_ms} ms")
            }
            StreamingRobustnessOutcomeKind::Cancellation { reason } => {
                format!("cancelled with reason {reason:?}")
            }
            StreamingRobustnessOutcomeKind::RetryExhausted {
                total_attempts,
                failure,
            } => format!(
                "retry exhausted after {total_attempts} attempt(s): {}",
                failure.message()
            ),
            StreamingRobustnessOutcomeKind::ProviderStreamFailure { failure } => {
                format!("provider stream failure: {}", failure.message())
            }
        }
    }

    fn core_error(&self) -> Option<CoreError> {
        match &self.kind {
            StreamingRobustnessOutcomeKind::RetryExhausted {
                total_attempts,
                failure,
            } => Some(CoreError::Streaming {
                failure: crate::StreamingFailure::RetryExhausted {
                    total_attempts: *total_attempts,
                    failure: failure.clone(),
                },
            }),
            StreamingRobustnessOutcomeKind::ProviderStreamFailure { failure } => {
                Some(CoreError::Streaming {
                    failure: crate::StreamingFailure::ProviderStreamFailure {
                        failure: failure.clone(),
                    },
                })
            }
            StreamingRobustnessOutcomeKind::Timeout { .. }
            | StreamingRobustnessOutcomeKind::Cancellation { .. } => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Management-visible details about the active layered configuration.
pub struct LayeredManagementConfiguration {
    /// Fingerprint for the active layered configuration.
    pub active_fingerprint: crate::configuration::ConfigurationFingerprint,
    /// Configuration layer sources that contributed to this state.
    pub sources: Vec<crate::configuration::ConfigurationLayerSource>,
    /// Most recent layered reload outcome, when any.
    pub latest_reload_outcome: Option<LayeredConfigurationReloadOutcome>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Overall health state reported by the headless core.
pub enum CoreHealthState {
    /// Core state is healthy.
    Healthy,
    /// Core is usable with degraded components or stale state.
    Degraded {
        /// Reasons explaining degraded state.
        reasons: Vec<DegradedReason>,
    },
    /// Operation or account state failed with a reason.
    Failed {
        /// Structured core error for this failed state.
        error: CoreError,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Lifecycle state for a local proxy/runtime instance.
pub enum ProxyLifecycleState {
    /// Runtime is stopped.
    Stopped,
    /// Runtime is starting.
    Starting,
    /// Runtime is running with endpoint and uptime metadata.
    Running {
        /// Bound endpoint associated with this lifecycle state.
        endpoint: BoundEndpoint,
        /// Uptime metadata associated with a running or degraded runtime.
        uptime: UptimeMetadata,
    },
    /// Runtime is partially available or running with degraded behavior.
    Degraded {
        /// Bound endpoint associated with this lifecycle state.
        endpoint: Option<BoundEndpoint>,
        /// Uptime metadata associated with the runtime state.
        uptime: Option<UptimeMetadata>,
        /// Reasons explaining degraded state.
        reasons: Vec<DegradedReason>,
    },
    /// Operation or account state failed with a reason.
    Failed {
        /// Last error that moved lifecycle into a failed state.
        last_error: CoreError,
    },
    /// Runtime is stopping.
    Stopping,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Socket endpoint bound by a local runtime.
pub struct BoundEndpoint {
    /// Socket address bound by the runtime.
    pub socket_addr: SocketAddr,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Runtime start timestamp and elapsed uptime.
pub struct UptimeMetadata {
    /// Unix timestamp when the runtime started.
    pub started_at_unix_seconds: u64,
    /// Elapsed time since the runtime started.
    pub elapsed: Duration,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Requested lifecycle action for app-shell or management controls.
pub enum LifecycleControlIntent {
    /// Request to start the local lifecycle.
    Start,
    /// Request to stop the local lifecycle.
    Stop,
    /// Request to restart the local lifecycle.
    Restart,
    /// Request to refresh lifecycle status.
    RefreshStatus,
}

impl LifecycleControlIntent {
    /// Validates this value and returns a structured core error on failure.
    pub fn validate(self) -> Result<Self, CoreError> {
        Ok(self)
    }
}
