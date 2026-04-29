#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ManagementBoundary;

use std::net::SocketAddr;
use std::time::Duration;

use crate::configuration::{
    ConfigurationLoadFailure, ConfigurationSnapshot, FileBackedManagementConfiguration,
    FileConfigurationState,
};
use crate::provider::{DegradedReason, ProviderSummary};
use crate::usage::{QuotaSummary, UsageSummary};
use crate::{CoreError, CoreIdentity, core_identity};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManagementSnapshot {
    pub identity: CoreIdentity,
    pub lifecycle: ProxyLifecycleState,
    pub health: CoreHealthState,
    pub configuration: ConfigurationSnapshot,
    pub file_configuration: Option<FileBackedManagementConfiguration>,
    pub last_configuration_load_failure: Option<ConfigurationLoadFailure>,
    pub providers: Vec<ProviderSummary>,
    pub usage: UsageSummary,
    pub quota: QuotaSummary,
    pub warnings: Vec<String>,
    pub errors: Vec<CoreError>,
}

impl ManagementSnapshot {
    pub fn inert_bootstrap() -> Self {
        Self {
            identity: core_identity(),
            lifecycle: ProxyLifecycleState::Stopped,
            health: CoreHealthState::Healthy,
            configuration: ConfigurationSnapshot::local_development(),
            file_configuration: None,
            last_configuration_load_failure: None,
            providers: Vec::new(),
            usage: UsageSummary::zero(),
            quota: QuotaSummary::unknown(),
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CoreHealthState {
    Healthy,
    Degraded { reasons: Vec<DegradedReason> },
    Failed { error: CoreError },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProxyLifecycleState {
    Stopped,
    Starting,
    Running {
        endpoint: BoundEndpoint,
        uptime: UptimeMetadata,
    },
    Degraded {
        endpoint: Option<BoundEndpoint>,
        uptime: Option<UptimeMetadata>,
        reasons: Vec<DegradedReason>,
    },
    Failed {
        last_error: CoreError,
    },
    Stopping,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BoundEndpoint {
    pub socket_addr: SocketAddr,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UptimeMetadata {
    pub started_at_unix_seconds: u64,
    pub elapsed: Duration,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleControlIntent {
    Start,
    Stop,
    Restart,
    RefreshStatus,
}

impl LifecycleControlIntent {
    pub fn validate(self) -> Result<Self, CoreError> {
        Ok(self)
    }
}
