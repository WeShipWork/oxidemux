use std::fs;
use std::net::IpAddr;
use std::path::Path;

use super::raw::parse_raw_configuration;
use super::validation::validate_raw_configuration;
use crate::provider::{
    AccountSummary, AuthMethodCategory, AuthState, ProtocolFamily, ProviderCapability,
    ProviderSummary,
};
use crate::routing::{RoutingPolicy, RoutingTarget};
use crate::usage::{QuotaState, QuotaSummary, UsageSummary};
use crate::{
    ConfigurationError, ConfigurationErrorKind, ConfigurationSourceMetadata, CoreError,
    InvalidConfigurationValue,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConfigurationBoundary;

impl ConfigurationBoundary {
    pub fn load_file(path: impl AsRef<Path>) -> Result<ValidatedFileConfiguration, CoreError> {
        ValidatedFileConfiguration::load_file(path)
    }

    pub fn load_contents(contents: &str) -> Result<ValidatedFileConfiguration, CoreError> {
        ValidatedFileConfiguration::load_contents(contents)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigurationSnapshot {
    pub listen_address: IpAddr,
    pub port: u16,
    pub auto_start: bool,
    pub logging_enabled: bool,
    pub usage_collection_enabled: bool,
    pub routing_default: RoutingDefault,
    pub provider_references: Vec<String>,
}

impl ConfigurationSnapshot {
    pub fn local_development() -> Self {
        Self {
            listen_address: IpAddr::from([127, 0, 0, 1]),
            port: 8787,
            auto_start: false,
            logging_enabled: true,
            usage_collection_enabled: false,
            routing_default: RoutingDefault::named("manual"),
            provider_references: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoutingDefault {
    pub name: String,
}

impl RoutingDefault {
    pub fn named(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    fn validate(&self) -> Result<(), CoreError> {
        if self.name.trim().is_empty() {
            return Err(CoreError::ConfigurationValidation {
                field: "routing_default",
                message: "routing default name must not be empty".to_string(),
            });
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConfigurationUpdateIntent {
    pub listen_address: Option<String>,
    pub port: Option<u16>,
    pub auto_start: Option<bool>,
    pub logging_enabled: Option<bool>,
    pub usage_collection_enabled: Option<bool>,
    pub routing_default: Option<RoutingDefault>,
    pub provider_references: Option<Vec<String>>,
}

impl ConfigurationUpdateIntent {
    pub fn validate(&self) -> Result<ValidatedConfigurationUpdate, CoreError> {
        let listen_address = self
            .listen_address
            .as_deref()
            .map(parse_listen_address)
            .transpose()?;

        if matches!(self.port, Some(0)) {
            return Err(CoreError::ConfigurationValidation {
                field: "port",
                message: "port must be greater than 0".to_string(),
            });
        }

        if let Some(routing_default) = &self.routing_default {
            routing_default.validate()?;
        }

        if let Some(provider_references) = &self.provider_references {
            validate_provider_references(provider_references)?;
        }

        Ok(ValidatedConfigurationUpdate {
            listen_address,
            port: self.port,
            auto_start: self.auto_start,
            logging_enabled: self.logging_enabled,
            usage_collection_enabled: self.usage_collection_enabled,
            routing_default: self.routing_default.clone(),
            provider_references: self.provider_references.clone(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedConfigurationUpdate {
    pub listen_address: Option<IpAddr>,
    pub port: Option<u16>,
    pub auto_start: Option<bool>,
    pub logging_enabled: Option<bool>,
    pub usage_collection_enabled: Option<bool>,
    pub routing_default: Option<RoutingDefault>,
    pub provider_references: Option<Vec<String>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedFileConfiguration {
    pub source: ConfigurationSourceMetadata,
    pub proxy: FileProxyConfiguration,
    pub providers: Vec<FileProviderConfiguration>,
    pub routing_defaults: Vec<FileRoutingDefaultConfiguration>,
    pub routing_default_groups: Vec<FileRoutingDefaultGroup>,
    pub routing_policy: RoutingPolicy,
    pub logging: LoggingSetting,
    pub usage_collection_enabled: bool,
    pub auto_start: AutoStartIntent,
    pub warnings: Vec<String>,
}

impl ValidatedFileConfiguration {
    pub fn load_file(path: impl AsRef<Path>) -> Result<Self, CoreError> {
        let path = path.as_ref();
        let source = ConfigurationSourceMetadata::for_path(path);
        if path.extension().and_then(|extension| extension.to_str()) != Some("toml") {
            return Err(configuration_failure(vec![ConfigurationError::new(
                ConfigurationErrorKind::UnsupportedFormat,
                "source.path",
                InvalidConfigurationValue::Unsupported,
                Some(source),
            )]));
        }

        let contents = fs::read_to_string(path).map_err(|error| {
            configuration_failure(vec![ConfigurationError::with_message(
                ConfigurationErrorKind::ReadFailed,
                "source.path",
                InvalidConfigurationValue::Malformed,
                Some(source.clone()),
                error.kind().to_string(),
            )])
        })?;

        Self::load_contents_with_source(&contents, source)
    }

    pub fn load_contents(contents: &str) -> Result<Self, CoreError> {
        Self::load_contents_with_source(contents, ConfigurationSourceMetadata::memory())
    }

    pub fn load_contents_with_source(
        contents: &str,
        source: ConfigurationSourceMetadata,
    ) -> Result<Self, CoreError> {
        let raw = parse_raw_configuration(contents, source.clone())?;
        validate_raw_configuration(raw, source)
    }

    pub fn configuration_snapshot(&self) -> ConfigurationSnapshot {
        ConfigurationSnapshot {
            listen_address: self.proxy.listen_address,
            port: self.proxy.port,
            auto_start: self.auto_start == AutoStartIntent::Enabled,
            logging_enabled: self.logging != LoggingSetting::Off,
            usage_collection_enabled: self.usage_collection_enabled,
            routing_default: self
                .routing_defaults
                .first()
                .map(|routing_default| RoutingDefault::named(routing_default.name.clone()))
                .unwrap_or_else(|| RoutingDefault::named("file")),
            provider_references: self
                .providers
                .iter()
                .map(|provider| provider.id.clone())
                .collect(),
        }
    }

    pub fn provider_summaries(&self) -> Vec<ProviderSummary> {
        self.providers
            .iter()
            .map(|provider| ProviderSummary {
                provider_id: provider.id.clone(),
                display_name: provider.id.clone(),
                capabilities: vec![ProviderCapability {
                    protocol_family: provider.protocol_family,
                    supports_streaming: true,
                    auth_method: AuthMethodCategory::ExternalReference,
                    routing_eligible: provider.routing_eligible,
                }],
                accounts: provider
                    .accounts
                    .iter()
                    .map(|account| AccountSummary {
                        account_id: account.id.clone(),
                        display_name: account.id.clone(),
                        auth_state: AuthState::CredentialReference {
                            reference_name: "configured".to_string(),
                        },
                        quota_state: QuotaState::Unknown,
                        last_checked: None,
                        degraded_reasons: Vec::new(),
                    })
                    .collect(),
                degraded_reasons: Vec::new(),
            })
            .collect()
    }

    pub fn usage_summary(&self) -> UsageSummary {
        if self.usage_collection_enabled {
            UsageSummary::zero()
        } else {
            UsageSummary::unknown()
        }
    }

    pub fn quota_summary(&self) -> QuotaSummary {
        QuotaSummary::unknown()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileProxyConfiguration {
    pub listen_address: IpAddr,
    pub port: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileProviderConfiguration {
    pub id: String,
    pub protocol_family: ProtocolFamily,
    pub routing_eligible: bool,
    pub accounts: Vec<FileAccountConfiguration>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileAccountConfiguration {
    pub id: String,
    pub(super) credential_reference_present: bool,
}

impl FileAccountConfiguration {
    pub fn credential_reference_present(&self) -> bool {
        self.credential_reference_present
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileRoutingDefaultConfiguration {
    pub name: String,
    pub model: String,
    pub target: RoutingTarget,
    pub fallback_enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileRoutingDefaultGroup {
    pub name: String,
    pub model: String,
    pub candidates: Vec<FileRoutingDefaultConfiguration>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoggingSetting {
    Off,
    Standard,
    Verbose,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AutoStartIntent {
    Disabled,
    Enabled,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FileConfigurationState {
    active: Option<ValidatedFileConfiguration>,
    last_failure: Option<ConfigurationLoadFailure>,
}

impl FileConfigurationState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn active(&self) -> Option<&ValidatedFileConfiguration> {
        self.active.as_ref()
    }

    pub fn last_failure(&self) -> Option<&ConfigurationLoadFailure> {
        self.last_failure.as_ref()
    }

    pub fn replace_from_contents(&mut self, contents: &str) -> Result<(), CoreError> {
        self.replace_from_contents_with_source(contents, ConfigurationSourceMetadata::memory())
    }

    pub fn replace_from_contents_with_source(
        &mut self,
        contents: &str,
        source: ConfigurationSourceMetadata,
    ) -> Result<(), CoreError> {
        match ValidatedFileConfiguration::load_contents_with_source(contents, source.clone()) {
            Ok(configuration) => {
                self.active = Some(configuration);
                self.last_failure = None;
                Ok(())
            }
            Err(error) => {
                self.last_failure = Some(ConfigurationLoadFailure::from_core_error(source, &error));
                Err(error)
            }
        }
    }

    pub fn replace_from_file(&mut self, path: impl AsRef<Path>) -> Result<(), CoreError> {
        let path = path.as_ref();
        let source = ConfigurationSourceMetadata::for_path(path);
        match ValidatedFileConfiguration::load_file(path) {
            Ok(configuration) => {
                self.active = Some(configuration);
                self.last_failure = None;
                Ok(())
            }
            Err(error) => {
                self.last_failure = Some(ConfigurationLoadFailure::from_core_error(source, &error));
                Err(error)
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigurationLoadFailure {
    pub source: ConfigurationSourceMetadata,
    pub errors: Vec<ConfigurationError>,
}

impl ConfigurationLoadFailure {
    fn from_core_error(source: ConfigurationSourceMetadata, error: &CoreError) -> Self {
        let errors = match error {
            CoreError::Configuration { errors } => errors.clone(),
            _ => vec![ConfigurationError::with_message(
                ConfigurationErrorKind::ParseFailed,
                "configuration",
                InvalidConfigurationValue::Malformed,
                Some(source.clone()),
                error.to_string(),
            )],
        };
        Self { source, errors }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileBackedManagementConfiguration {
    pub source: ConfigurationSourceMetadata,
    pub proxy: FileProxyConfiguration,
    pub routing_defaults: Vec<FileRoutingDefaultConfiguration>,
    pub routing_default_groups: Vec<FileRoutingDefaultGroup>,
    pub logging: LoggingSetting,
    pub usage_collection_enabled: bool,
    pub auto_start: AutoStartIntent,
    pub warnings: Vec<String>,
}

impl From<&ValidatedFileConfiguration> for FileBackedManagementConfiguration {
    fn from(configuration: &ValidatedFileConfiguration) -> Self {
        Self {
            source: configuration.source.clone(),
            proxy: configuration.proxy.clone(),
            routing_defaults: configuration.routing_defaults.clone(),
            routing_default_groups: configuration.routing_default_groups.clone(),
            logging: configuration.logging,
            usage_collection_enabled: configuration.usage_collection_enabled,
            auto_start: configuration.auto_start,
            warnings: configuration.warnings.clone(),
        }
    }
}

fn configuration_failure(errors: Vec<ConfigurationError>) -> CoreError {
    CoreError::Configuration { errors }
}

fn parse_listen_address(value: &str) -> Result<IpAddr, CoreError> {
    value
        .parse()
        .map_err(|_| CoreError::ConfigurationValidation {
            field: "listen_address",
            message: format!("{value:?} is not a valid IP address"),
        })
}

fn validate_provider_references(provider_references: &[String]) -> Result<(), CoreError> {
    for provider_reference in provider_references {
        if provider_reference.trim().is_empty() {
            return Err(CoreError::ConfigurationValidation {
                field: "provider_references",
                message: "provider references must not contain blank values".to_string(),
            });
        }
    }

    Ok(())
}
