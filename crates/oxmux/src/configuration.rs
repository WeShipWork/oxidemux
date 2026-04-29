//! Configuration contracts for the headless core.
//!
//! This module exposes validated file-backed and layered configuration state used
//! by `oxmux` consumers and management snapshots. It describes core proxy
//! configuration data without owning desktop presentation or platform secret
//! storage behavior.

mod file;
mod raw;
mod validation;

/// File value for the surrounding public contract.
pub use file::{
    AutoStartIntent, ConfigurationBoundary, ConfigurationFingerprint, ConfigurationLayerKind,
    ConfigurationLayerSource, ConfigurationLoadFailure, ConfigurationSnapshot,
    ConfigurationUpdateIntent, FileAccountConfiguration, FileBackedManagementConfiguration,
    FileConfigurationState, FileProviderConfiguration, FileProxyConfiguration,
    FileRoutingDefaultConfiguration, FileRoutingDefaultGroup, LayeredConfigurationInput,
    LayeredConfigurationRejectedCandidate, LayeredConfigurationReloadOutcome,
    LayeredConfigurationState, LoggingSetting, RoutingDefault, ValidatedConfigurationUpdate,
    ValidatedFileConfiguration, ValidatedLayeredConfiguration,
};
