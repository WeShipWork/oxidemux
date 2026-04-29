use std::collections::{BTreeMap, BTreeSet};
use std::net::IpAddr;

use super::file::{
    AutoStartIntent, FileAccountConfiguration, FileProviderConfiguration, FileProxyConfiguration,
    FileRoutingDefaultConfiguration, FileRoutingDefaultGroup, LoggingSetting,
    ValidatedFileConfiguration,
};
use super::raw::{
    RawAccountConfiguration, RawConfiguration, RawProviderConfiguration, RawProxyConfiguration,
    RawRoutingDefaultConfiguration,
};
use crate::provider::ProtocolFamily;
use crate::routing::{ModelRoute, RoutingCandidate, RoutingPolicy, RoutingTarget};
use crate::{
    ConfigurationError, ConfigurationErrorKind, ConfigurationSourceMetadata, CoreError,
    InvalidConfigurationValue,
};

pub(super) fn validate_raw_configuration(
    raw: RawConfiguration,
    source: ConfigurationSourceMetadata,
) -> Result<ValidatedFileConfiguration, CoreError> {
    let mut errors = Vec::new();

    match raw.version {
        Some(1) => {}
        Some(_) => errors.push(error(
            ConfigurationErrorKind::InvalidVersion,
            "version",
            InvalidConfigurationValue::Unsupported,
            &source,
        )),
        None => errors.push(error(
            ConfigurationErrorKind::MissingRequiredField,
            "version",
            InvalidConfigurationValue::Missing,
            &source,
        )),
    }

    let proxy = validate_proxy(raw.proxy, &source, &mut errors);
    let providers = validate_providers(raw.providers, &source, &mut errors);
    let provider_index = provider_account_index(&providers);
    let routing_defaults =
        validate_routing_defaults(raw.routing.defaults, &provider_index, &source, &mut errors);
    let routing_default_groups = group_routing_defaults(&routing_defaults);
    let logging = validate_logging(raw.observability.logging, &source, &mut errors);
    let usage_collection_enabled =
        validate_usage_collection(raw.observability.usage_collection, &source, &mut errors);
    let auto_start = validate_auto_start(raw.lifecycle.auto_start, &source, &mut errors);

    if !errors.is_empty() {
        return Err(configuration_failure(errors));
    }

    let Some(proxy) = proxy else {
        return Err(configuration_failure(vec![error(
            ConfigurationErrorKind::MissingRequiredField,
            "proxy",
            InvalidConfigurationValue::Missing,
            &source,
        )]));
    };

    let logging = logging.unwrap_or(LoggingSetting::Standard);
    let usage_collection_enabled = usage_collection_enabled.unwrap_or(true);
    let auto_start = auto_start.unwrap_or(AutoStartIntent::Disabled);
    let routing_policy = build_routing_policy(&routing_default_groups);

    Ok(ValidatedFileConfiguration {
        source,
        proxy,
        providers,
        routing_defaults,
        routing_default_groups,
        routing_policy,
        logging,
        usage_collection_enabled,
        auto_start,
        warnings: Vec::new(),
    })
}

fn validate_proxy(
    raw: Option<RawProxyConfiguration>,
    source: &ConfigurationSourceMetadata,
    errors: &mut Vec<ConfigurationError>,
) -> Option<FileProxyConfiguration> {
    let Some(raw) = raw else {
        errors.push(error(
            ConfigurationErrorKind::MissingRequiredField,
            "proxy",
            InvalidConfigurationValue::Missing,
            source,
        ));
        return None;
    };

    let listen_address = match raw.listen_address {
        Some(value) if value.trim().is_empty() => {
            errors.push(error(
                ConfigurationErrorKind::InvalidListenAddress,
                "proxy.listen-address",
                InvalidConfigurationValue::Malformed,
                source,
            ));
            None
        }
        Some(value) => match value.parse::<IpAddr>() {
            Ok(address) if address.is_loopback() => Some(address),
            Ok(_) => {
                errors.push(error(
                    ConfigurationErrorKind::InvalidListenAddress,
                    "proxy.listen-address",
                    InvalidConfigurationValue::Unsupported,
                    source,
                ));
                None
            }
            Err(_) => {
                errors.push(error(
                    ConfigurationErrorKind::InvalidListenAddress,
                    "proxy.listen-address",
                    InvalidConfigurationValue::Malformed,
                    source,
                ));
                None
            }
        },
        None => {
            errors.push(error(
                ConfigurationErrorKind::MissingRequiredField,
                "proxy.listen-address",
                InvalidConfigurationValue::Missing,
                source,
            ));
            None
        }
    };

    let port = match raw.port {
        Some(value) if (1..=u16::MAX as i64).contains(&value) => Some(value as u16),
        Some(_) => {
            errors.push(error(
                ConfigurationErrorKind::InvalidPort,
                "proxy.port",
                InvalidConfigurationValue::OutOfRange,
                source,
            ));
            None
        }
        None => {
            errors.push(error(
                ConfigurationErrorKind::MissingRequiredField,
                "proxy.port",
                InvalidConfigurationValue::Missing,
                source,
            ));
            None
        }
    };

    match (listen_address, port) {
        (Some(listen_address), Some(port)) => Some(FileProxyConfiguration {
            listen_address,
            port,
        }),
        _ => None,
    }
}

fn validate_providers(
    raw_providers: Vec<RawProviderConfiguration>,
    source: &ConfigurationSourceMetadata,
    errors: &mut Vec<ConfigurationError>,
) -> Vec<FileProviderConfiguration> {
    let mut providers = Vec::new();
    let mut provider_ids = BTreeSet::new();

    for (provider_index, raw_provider) in raw_providers.into_iter().enumerate() {
        let id_field = indexed("providers", provider_index, "id");
        let protocol_field = indexed("providers", provider_index, "protocol-family");
        let routing_field = indexed("providers", provider_index, "routing-eligible");
        let Some(id) = validate_required_string(
            raw_provider.id,
            &id_field,
            ConfigurationErrorKind::MissingRequiredField,
            source,
            errors,
        ) else {
            continue;
        };

        if !provider_ids.insert(id.clone()) {
            errors.push(owned_error(
                ConfigurationErrorKind::DuplicateProviderId,
                id_field,
                InvalidConfigurationValue::Duplicate,
                source,
            ));
        }

        let protocol_family = match raw_provider.protocol_family {
            Some(value) => match protocol_family_from_str(&value) {
                Some(protocol_family) => Some(protocol_family),
                None => {
                    errors.push(owned_error(
                        ConfigurationErrorKind::InvalidProviderProtocolFamily,
                        protocol_field,
                        InvalidConfigurationValue::Unsupported,
                        source,
                    ));
                    None
                }
            },
            None => {
                errors.push(owned_error(
                    ConfigurationErrorKind::MissingRequiredField,
                    protocol_field,
                    InvalidConfigurationValue::Missing,
                    source,
                ));
                None
            }
        };

        let routing_eligible = match raw_provider.routing_eligible {
            Some(value) => Some(value),
            None => {
                errors.push(owned_error(
                    ConfigurationErrorKind::MissingRequiredField,
                    routing_field,
                    InvalidConfigurationValue::Missing,
                    source,
                ));
                None
            }
        };

        let accounts = validate_accounts(provider_index, raw_provider.accounts, source, errors);

        if let (Some(protocol_family), Some(routing_eligible)) = (protocol_family, routing_eligible)
        {
            providers.push(FileProviderConfiguration {
                id,
                protocol_family,
                routing_eligible,
                accounts,
            });
        }
    }

    providers
}

fn validate_accounts(
    provider_index: usize,
    raw_accounts: Vec<RawAccountConfiguration>,
    source: &ConfigurationSourceMetadata,
    errors: &mut Vec<ConfigurationError>,
) -> Vec<FileAccountConfiguration> {
    let mut accounts = Vec::new();
    let mut account_ids = BTreeSet::new();

    for (account_index, raw_account) in raw_accounts.into_iter().enumerate() {
        let id_field = indexed_nested("providers", provider_index, "accounts", account_index, "id");
        let credential_field = indexed_nested(
            "providers",
            provider_index,
            "accounts",
            account_index,
            "credential-reference",
        );
        let Some(id) = validate_required_string(
            raw_account.id,
            &id_field,
            ConfigurationErrorKind::MissingRequiredField,
            source,
            errors,
        ) else {
            continue;
        };

        if !account_ids.insert(id.clone()) {
            errors.push(owned_error(
                ConfigurationErrorKind::DuplicateAccountId,
                id_field,
                InvalidConfigurationValue::Duplicate,
                source,
            ));
        }

        let credential_reference_present = match raw_account.credential_reference {
            Some(value) => match validate_credential_reference(&value) {
                CredentialReferenceValidation::Valid => true,
                CredentialReferenceValidation::Malformed => {
                    errors.push(owned_error(
                        ConfigurationErrorKind::InvalidCredentialReference,
                        credential_field,
                        InvalidConfigurationValue::Malformed,
                        source,
                    ));
                    false
                }
                CredentialReferenceValidation::SecretLike => {
                    errors.push(owned_error(
                        ConfigurationErrorKind::InvalidCredentialReference,
                        credential_field,
                        InvalidConfigurationValue::SecretLike,
                        source,
                    ));
                    false
                }
            },
            None => {
                errors.push(owned_error(
                    ConfigurationErrorKind::MissingRequiredField,
                    credential_field,
                    InvalidConfigurationValue::Missing,
                    source,
                ));
                false
            }
        };

        accounts.push(FileAccountConfiguration {
            id,
            credential_reference_present,
        });
    }

    accounts
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ProviderRoutingReference {
    routing_eligible: bool,
    accounts: BTreeSet<String>,
}

fn provider_account_index(
    providers: &[FileProviderConfiguration],
) -> BTreeMap<String, ProviderRoutingReference> {
    providers
        .iter()
        .map(|provider| {
            (
                provider.id.clone(),
                ProviderRoutingReference {
                    routing_eligible: provider.routing_eligible,
                    accounts: provider
                        .accounts
                        .iter()
                        .map(|account| account.id.clone())
                        .collect(),
                },
            )
        })
        .collect()
}

fn validate_routing_defaults(
    raw_defaults: Vec<RawRoutingDefaultConfiguration>,
    provider_index: &BTreeMap<String, ProviderRoutingReference>,
    source: &ConfigurationSourceMetadata,
    errors: &mut Vec<ConfigurationError>,
) -> Vec<FileRoutingDefaultConfiguration> {
    let mut defaults = Vec::new();
    let mut candidates = BTreeSet::new();

    for (index, raw_default) in raw_defaults.iter().enumerate() {
        let name_field = indexed("routing.defaults", index, "name");
        let model_field = indexed("routing.defaults", index, "model");
        let provider_field = indexed("routing.defaults", index, "provider-id");
        let account_field = indexed("routing.defaults", index, "account-id");
        let fallback_field = indexed("routing.defaults", index, "fallback-enabled");

        let name = validate_required_string(
            raw_default.name.clone(),
            &name_field,
            ConfigurationErrorKind::InvalidRoutingDefault,
            source,
            errors,
        );
        let model = validate_required_string(
            raw_default.model.clone(),
            &model_field,
            ConfigurationErrorKind::InvalidRoutingDefault,
            source,
            errors,
        );
        let provider_id = validate_required_string(
            raw_default.provider_id.clone(),
            &provider_field,
            ConfigurationErrorKind::InvalidRoutingDefault,
            source,
            errors,
        );
        let fallback_enabled = match raw_default.fallback_enabled {
            Some(value) => Some(value),
            None => {
                errors.push(owned_error(
                    ConfigurationErrorKind::InvalidRoutingDefault,
                    fallback_field.clone(),
                    InvalidConfigurationValue::Missing,
                    source,
                ));
                None
            }
        };

        let account_id = match raw_default.account_id.clone() {
            Some(value) if value.trim().is_empty() => {
                errors.push(owned_error(
                    ConfigurationErrorKind::InvalidRoutingDefault,
                    account_field.clone(),
                    InvalidConfigurationValue::Malformed,
                    source,
                ));
                None
            }
            Some(value) => Some(value),
            None => None,
        };

        if let Some(provider_id) = &provider_id {
            match provider_index.get(provider_id) {
                Some(reference) => {
                    if !reference.routing_eligible {
                        errors.push(owned_error(
                            ConfigurationErrorKind::InvalidRoutingDefault,
                            provider_field.clone(),
                            InvalidConfigurationValue::Unsupported,
                            source,
                        ));
                    }
                    if let Some(account_id) = &account_id
                        && !reference.accounts.contains(account_id)
                    {
                        errors.push(owned_error(
                            ConfigurationErrorKind::UnknownAccountReference,
                            account_field.clone(),
                            InvalidConfigurationValue::UnknownReference,
                            source,
                        ));
                    }
                }
                None => errors.push(owned_error(
                    ConfigurationErrorKind::UnknownProviderReference,
                    provider_field,
                    InvalidConfigurationValue::UnknownReference,
                    source,
                )),
            }
        }

        if let (Some(name), Some(model), Some(provider_id), Some(fallback_enabled)) =
            (name, model, provider_id, fallback_enabled)
        {
            let candidate_key = (
                name.clone(),
                model.clone(),
                provider_id.clone(),
                account_id.clone(),
            );
            if !candidates.insert(candidate_key) {
                errors.push(owned_error(
                    ConfigurationErrorKind::InvalidRoutingDefault,
                    indexed("routing.defaults", index, "provider-id"),
                    InvalidConfigurationValue::Duplicate,
                    source,
                ));
            }

            if !fallback_enabled
                && raw_defaults
                    .iter()
                    .enumerate()
                    .any(|(candidate_index, candidate)| {
                        candidate_index > index
                            && candidate.name.as_deref() == Some(name.as_str())
                            && candidate.model.as_deref() == Some(model.as_str())
                    })
            {
                errors.push(owned_error(
                    ConfigurationErrorKind::InvalidRoutingDefault,
                    fallback_field,
                    InvalidConfigurationValue::Unsupported,
                    source,
                ));
            }

            defaults.push(FileRoutingDefaultConfiguration {
                name,
                model,
                target: match account_id {
                    Some(account_id) => RoutingTarget::provider_account(provider_id, account_id),
                    None => RoutingTarget::provider(provider_id),
                },
                fallback_enabled,
            });
        }
    }

    defaults
}

fn group_routing_defaults(
    defaults: &[FileRoutingDefaultConfiguration],
) -> Vec<FileRoutingDefaultGroup> {
    let mut groups: Vec<FileRoutingDefaultGroup> = Vec::new();

    for default in defaults {
        if let Some(group) = groups
            .iter_mut()
            .find(|group| group.name == default.name && group.model == default.model)
        {
            group.candidates.push(default.clone());
        } else {
            groups.push(FileRoutingDefaultGroup {
                name: default.name.clone(),
                model: default.model.clone(),
                candidates: vec![default.clone()],
            });
        }
    }

    groups
}

fn validate_logging(
    raw_logging: Option<String>,
    source: &ConfigurationSourceMetadata,
    errors: &mut Vec<ConfigurationError>,
) -> Option<LoggingSetting> {
    match raw_logging.as_deref() {
        Some("off") => Some(LoggingSetting::Off),
        Some("standard") => Some(LoggingSetting::Standard),
        Some("verbose") => Some(LoggingSetting::Verbose),
        Some(_) => {
            errors.push(error(
                ConfigurationErrorKind::InvalidLoggingSetting,
                "observability.logging",
                InvalidConfigurationValue::Unsupported,
                source,
            ));
            None
        }
        None => Some(LoggingSetting::Standard),
    }
}

fn validate_usage_collection(
    usage_collection: Option<toml::Value>,
    source: &ConfigurationSourceMetadata,
    errors: &mut Vec<ConfigurationError>,
) -> Option<bool> {
    match usage_collection {
        Some(toml::Value::Boolean(value)) => Some(value),
        Some(_) => {
            errors.push(error(
                ConfigurationErrorKind::InvalidUsageCollectionSetting,
                "observability.usage-collection",
                InvalidConfigurationValue::Malformed,
                source,
            ));
            None
        }
        None => Some(true),
    }
}

fn validate_auto_start(
    raw_auto_start: Option<String>,
    source: &ConfigurationSourceMetadata,
    errors: &mut Vec<ConfigurationError>,
) -> Option<AutoStartIntent> {
    match raw_auto_start.as_deref() {
        Some("disabled") => Some(AutoStartIntent::Disabled),
        Some("enabled") => Some(AutoStartIntent::Enabled),
        Some(_) => {
            errors.push(error(
                ConfigurationErrorKind::InvalidAutoStartIntent,
                "lifecycle.auto-start",
                InvalidConfigurationValue::Unsupported,
                source,
            ));
            None
        }
        None => Some(AutoStartIntent::Disabled),
    }
}

fn build_routing_policy(groups: &[FileRoutingDefaultGroup]) -> RoutingPolicy {
    let mut grouped: BTreeMap<String, Vec<RoutingCandidate>> = BTreeMap::new();
    for group in groups {
        for routing_default in &group.candidates {
            grouped
                .entry(group.model.clone())
                .or_default()
                .push(RoutingCandidate::new(routing_default.target.clone()));
        }
    }

    RoutingPolicy::new(
        grouped
            .into_iter()
            .map(|(model, candidates)| ModelRoute::new(model, candidates))
            .collect(),
    )
}

fn protocol_family_from_str(value: &str) -> Option<ProtocolFamily> {
    match value {
        "openai" => Some(ProtocolFamily::OpenAi),
        "gemini" => Some(ProtocolFamily::Gemini),
        "claude" => Some(ProtocolFamily::Claude),
        "codex" => Some(ProtocolFamily::Codex),
        "provider-specific" => Some(ProtocolFamily::ProviderSpecific),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CredentialReferenceValidation {
    Valid,
    Malformed,
    SecretLike,
}

fn validate_credential_reference(value: &str) -> CredentialReferenceValidation {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed != value {
        return CredentialReferenceValidation::Malformed;
    }

    let lower = trimmed.to_ascii_lowercase();
    if lower.contains("secret")
        || lower.contains("token")
        || lower.contains("api-key")
        || lower.contains("api_key")
        || lower.starts_with("sk-")
    {
        return CredentialReferenceValidation::SecretLike;
    }

    if trimmed.chars().all(|character| {
        character.is_ascii_alphanumeric() || matches!(character, '/' | '-' | '_' | '.')
    }) {
        CredentialReferenceValidation::Valid
    } else {
        CredentialReferenceValidation::Malformed
    }
}

fn validate_required_string(
    value: Option<String>,
    field: &str,
    missing_kind: ConfigurationErrorKind,
    source: &ConfigurationSourceMetadata,
    errors: &mut Vec<ConfigurationError>,
) -> Option<String> {
    match value {
        Some(value) if !value.trim().is_empty() => Some(value),
        Some(_) => {
            errors.push(owned_error(
                missing_kind,
                field.to_string(),
                InvalidConfigurationValue::Malformed,
                source,
            ));
            None
        }
        None => {
            errors.push(owned_error(
                missing_kind,
                field.to_string(),
                InvalidConfigurationValue::Missing,
                source,
            ));
            None
        }
    }
}

fn error(
    kind: ConfigurationErrorKind,
    field_path: &'static str,
    invalid_value: InvalidConfigurationValue,
    source: &ConfigurationSourceMetadata,
) -> ConfigurationError {
    ConfigurationError::new(kind, field_path, invalid_value, Some(source.clone()))
}

fn owned_error(
    kind: ConfigurationErrorKind,
    field_path: String,
    invalid_value: InvalidConfigurationValue,
    source: &ConfigurationSourceMetadata,
) -> ConfigurationError {
    ConfigurationError::new(kind, field_path, invalid_value, Some(source.clone()))
}

fn configuration_failure(errors: Vec<ConfigurationError>) -> CoreError {
    CoreError::Configuration { errors }
}

fn indexed(prefix: &str, index: usize, field: &str) -> String {
    format!("{prefix}[{index}].{field}")
}

fn indexed_nested(
    prefix: &str,
    outer_index: usize,
    nested: &str,
    nested_index: usize,
    field: &str,
) -> String {
    format!("{prefix}[{outer_index}].{nested}[{nested_index}].{field}")
}
