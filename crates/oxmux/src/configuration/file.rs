use std::fs;
use std::net::IpAddr;
use std::path::Path;

use super::raw::{
    RawAccountConfiguration, RawConfiguration, RawLifecycleConfiguration,
    RawObservabilityConfiguration, RawProviderConfiguration, RawProxyConfiguration,
    RawRoutingConfiguration, parse_raw_configuration,
};
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

    pub fn replace_layered(
        state: &mut LayeredConfigurationState,
        inputs: Vec<LayeredConfigurationInput>,
    ) -> LayeredConfigurationReloadOutcome {
        state.replace(inputs)
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
    pub(super) credential_reference: String,
}

impl FileAccountConfiguration {
    pub fn credential_reference_present(&self) -> bool {
        !self.credential_reference.is_empty()
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigurationLayerKind {
    BundledDefaults,
    UserOwned,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigurationLayerSource {
    pub kind: ConfigurationLayerKind,
    pub source: ConfigurationSourceMetadata,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LayeredConfigurationInput {
    pub kind: ConfigurationLayerKind,
    pub source: ConfigurationSourceMetadata,
    pub contents: String,
}

impl LayeredConfigurationInput {
    pub fn bundled_defaults(contents: impl Into<String>) -> Self {
        Self {
            kind: ConfigurationLayerKind::BundledDefaults,
            source: ConfigurationSourceMetadata::memory(),
            contents: contents.into(),
        }
    }

    pub fn user_owned(contents: impl Into<String>, source: ConfigurationSourceMetadata) -> Self {
        Self {
            kind: ConfigurationLayerKind::UserOwned,
            source,
            contents: contents.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigurationFingerprint {
    pub value: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedLayeredConfiguration {
    pub configuration: ValidatedFileConfiguration,
    pub fingerprint: ConfigurationFingerprint,
    pub sources: Vec<ConfigurationLayerSource>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LayeredConfigurationRejectedCandidate {
    pub sources: Vec<ConfigurationLayerSource>,
    pub errors: Vec<ConfigurationError>,
    pub previous_active_fingerprint: Option<ConfigurationFingerprint>,
    pub candidate_fingerprint: Option<ConfigurationFingerprint>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LayeredConfigurationReloadOutcome {
    Unchanged {
        active_fingerprint: ConfigurationFingerprint,
        sources: Vec<ConfigurationLayerSource>,
    },
    Replaced {
        active_fingerprint: ConfigurationFingerprint,
        sources: Vec<ConfigurationLayerSource>,
    },
    Rejected(LayeredConfigurationRejectedCandidate),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LayeredConfigurationState {
    active: Option<ValidatedLayeredConfiguration>,
    latest_reload_outcome: Option<LayeredConfigurationReloadOutcome>,
    failed_candidate: Option<LayeredConfigurationRejectedCandidate>,
}

impl LayeredConfigurationState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn active(&self) -> Option<&ValidatedLayeredConfiguration> {
        self.active.as_ref()
    }

    pub fn latest_reload_outcome(&self) -> Option<&LayeredConfigurationReloadOutcome> {
        self.latest_reload_outcome.as_ref()
    }

    pub fn failed_candidate(&self) -> Option<&LayeredConfigurationRejectedCandidate> {
        self.failed_candidate.as_ref()
    }

    pub fn replace(
        &mut self,
        inputs: Vec<LayeredConfigurationInput>,
    ) -> LayeredConfigurationReloadOutcome {
        match build_layered_configuration(inputs) {
            Ok(candidate) => {
                let sources = candidate.sources.clone();
                let fingerprint = candidate.fingerprint.clone();
                let outcome = if self
                    .active
                    .as_ref()
                    .is_some_and(|active| active.fingerprint == fingerprint)
                {
                    LayeredConfigurationReloadOutcome::Unchanged {
                        active_fingerprint: fingerprint,
                        sources,
                    }
                } else {
                    self.active = Some(candidate);
                    LayeredConfigurationReloadOutcome::Replaced {
                        active_fingerprint: fingerprint,
                        sources,
                    }
                };
                if !matches!(outcome, LayeredConfigurationReloadOutcome::Unchanged { .. }) {
                    self.failed_candidate = None;
                }
                self.latest_reload_outcome = Some(outcome.clone());
                outcome
            }
            Err(rejected) => {
                let rejected = LayeredConfigurationRejectedCandidate {
                    previous_active_fingerprint: self
                        .active
                        .as_ref()
                        .map(|active| active.fingerprint.clone()),
                    ..rejected
                };
                let outcome = LayeredConfigurationReloadOutcome::Rejected(rejected.clone());
                self.failed_candidate = Some(rejected);
                self.latest_reload_outcome = Some(outcome.clone());
                outcome
            }
        }
    }
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

fn build_layered_configuration(
    inputs: Vec<LayeredConfigurationInput>,
) -> Result<ValidatedLayeredConfiguration, LayeredConfigurationRejectedCandidate> {
    let sources: Vec<ConfigurationLayerSource> = inputs
        .iter()
        .map(|input| ConfigurationLayerSource {
            kind: input.kind,
            source: input.source.clone(),
        })
        .collect();

    if inputs.is_empty() {
        return Err(LayeredConfigurationRejectedCandidate {
            sources,
            errors: vec![ConfigurationError::new(
                ConfigurationErrorKind::MissingRequiredField,
                "layers",
                InvalidConfigurationValue::Missing,
                Some(ConfigurationSourceMetadata::memory()),
            )],
            previous_active_fingerprint: None,
            candidate_fingerprint: None,
        });
    }

    let mut parsed_layers = Vec::new();
    let mut errors = Vec::new();
    for input in inputs {
        match parse_raw_configuration(&input.contents, input.source.clone()) {
            Ok(raw) => parsed_layers.push((input.kind, raw)),
            Err(CoreError::Configuration {
                errors: parse_errors,
            }) => errors.extend(parse_errors),
            Err(error) => errors.push(ConfigurationError::with_message(
                ConfigurationErrorKind::ParseFailed,
                "configuration",
                InvalidConfigurationValue::Malformed,
                Some(input.source),
                error.to_string(),
            )),
        }
    }

    if !errors.is_empty() {
        return Err(LayeredConfigurationRejectedCandidate {
            sources,
            errors,
            previous_active_fingerprint: None,
            candidate_fingerprint: None,
        });
    }

    parsed_layers.sort_by_key(|(kind, _)| layer_precedence(*kind));
    let merged = merge_raw_layers(parsed_layers.into_iter().map(|(_, raw)| raw));
    let source = ConfigurationSourceMetadata {
        path: None,
        description: "layered configuration".to_string(),
    };
    match validate_raw_configuration(merged, source) {
        Ok(configuration) => {
            let fingerprint = fingerprint_configuration(&configuration);
            Ok(ValidatedLayeredConfiguration {
                configuration,
                fingerprint,
                sources,
            })
        }
        Err(CoreError::Configuration { errors }) => Err(LayeredConfigurationRejectedCandidate {
            sources,
            errors,
            previous_active_fingerprint: None,
            candidate_fingerprint: None,
        }),
        Err(error) => Err(LayeredConfigurationRejectedCandidate {
            sources,
            errors: vec![ConfigurationError::with_message(
                ConfigurationErrorKind::ParseFailed,
                "configuration",
                InvalidConfigurationValue::Malformed,
                Some(ConfigurationSourceMetadata::memory()),
                error.to_string(),
            )],
            previous_active_fingerprint: None,
            candidate_fingerprint: None,
        }),
    }
}

fn layer_precedence(kind: ConfigurationLayerKind) -> u8 {
    match kind {
        ConfigurationLayerKind::BundledDefaults => 0,
        ConfigurationLayerKind::UserOwned => 1,
    }
}

fn merge_raw_layers(layers: impl IntoIterator<Item = RawConfiguration>) -> RawConfiguration {
    let mut merged = RawConfiguration {
        version: None,
        proxy: None,
        providers: Vec::new(),
        routing: RawRoutingConfiguration::default(),
        observability: RawObservabilityConfiguration::default(),
        lifecycle: RawLifecycleConfiguration::default(),
    };

    for layer in layers {
        if layer.version.is_some() {
            merged.version = layer.version;
        }
        merged.proxy = Some(merge_proxy(merged.proxy, layer.proxy));
        merge_providers(&mut merged.providers, layer.providers);
        if !layer.routing.defaults.is_empty() {
            merged.routing.defaults = layer.routing.defaults;
        }
        if layer.observability.logging.is_some() {
            merged.observability.logging = layer.observability.logging;
        }
        if layer.observability.usage_collection.is_some() {
            merged.observability.usage_collection = layer.observability.usage_collection;
        }
        if layer.lifecycle.auto_start.is_some() {
            merged.lifecycle.auto_start = layer.lifecycle.auto_start;
        }
    }

    sort_provider_identities(&mut merged.providers);
    merged
}

fn merge_proxy(
    current: Option<RawProxyConfiguration>,
    incoming: Option<RawProxyConfiguration>,
) -> RawProxyConfiguration {
    let mut merged = current.unwrap_or(RawProxyConfiguration {
        listen_address: None,
        port: None,
    });
    if let Some(incoming) = incoming {
        if incoming.listen_address.is_some() {
            merged.listen_address = incoming.listen_address;
        }
        if incoming.port.is_some() {
            merged.port = incoming.port;
        }
    }
    merged
}

fn merge_providers(
    current: &mut Vec<RawProviderConfiguration>,
    incoming: Vec<RawProviderConfiguration>,
) {
    for provider in incoming {
        let Some(provider_id) = provider.id.as_deref().filter(|id| !id.trim().is_empty()) else {
            current.push(provider);
            continue;
        };

        match current
            .iter_mut()
            .find(|existing| existing.id.as_deref() == Some(provider_id))
        {
            Some(existing) => merge_provider(existing, provider),
            None => current.push(provider),
        }
    }
}

fn merge_provider(existing: &mut RawProviderConfiguration, incoming: RawProviderConfiguration) {
    if incoming.protocol_family.is_some() {
        existing.protocol_family = incoming.protocol_family;
    }
    if incoming.routing_eligible.is_some() {
        existing.routing_eligible = incoming.routing_eligible;
    }
    merge_accounts(&mut existing.accounts, incoming.accounts);
}

fn merge_accounts(
    current: &mut Vec<RawAccountConfiguration>,
    incoming: Vec<RawAccountConfiguration>,
) {
    for account in incoming {
        let Some(account_id) = account.id.as_deref().filter(|id| !id.trim().is_empty()) else {
            current.push(account);
            continue;
        };

        match current
            .iter_mut()
            .find(|existing| existing.id.as_deref() == Some(account_id))
        {
            Some(existing) => {
                if account.credential_reference.is_some() {
                    existing.credential_reference = account.credential_reference;
                }
            }
            None => current.push(account),
        }
    }
}

fn sort_provider_identities(providers: &mut [RawProviderConfiguration]) {
    providers.sort_by(|left, right| left.id.cmp(&right.id));
    for provider in providers {
        provider
            .accounts
            .sort_by(|left, right| left.id.cmp(&right.id));
    }
}

fn fingerprint_configuration(
    configuration: &ValidatedFileConfiguration,
) -> ConfigurationFingerprint {
    let mut canonical = String::new();
    canonical.push_str("version=1\n");
    canonical.push_str(&format!(
        "proxy.listen-address={}\n",
        configuration.proxy.listen_address
    ));
    canonical.push_str(&format!("proxy.port={}\n", configuration.proxy.port));
    canonical.push_str(&format!(
        "observability.logging={:?}\n",
        configuration.logging
    ));
    canonical.push_str(&format!(
        "observability.usage-collection={}\n",
        configuration.usage_collection_enabled
    ));
    canonical.push_str(&format!(
        "lifecycle.auto-start={:?}\n",
        configuration.auto_start
    ));

    let mut providers = configuration.providers.clone();
    providers.sort_by(|left, right| left.id.cmp(&right.id));
    for mut provider in providers {
        provider
            .accounts
            .sort_by(|left, right| left.id.cmp(&right.id));
        canonical.push_str(&format!(
            "provider:{}:{:?}:{}\n",
            provider.id, provider.protocol_family, provider.routing_eligible
        ));
        for account in provider.accounts {
            let credential_reference_hash = fnv1a64(account.credential_reference.as_bytes());
            canonical.push_str(&format!(
                "account:{}:{credential_reference_hash:016x}\n",
                account.id
            ));
        }
    }

    for routing_default in &configuration.routing_defaults {
        canonical.push_str(&format!(
            "route:{}:{}:{:?}:{}\n",
            routing_default.name,
            routing_default.model,
            routing_default.target,
            routing_default.fallback_enabled
        ));
    }

    ConfigurationFingerprint {
        value: format!("fnv1a64:{:016x}", fnv1a64(canonical.as_bytes())),
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
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
