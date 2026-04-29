use std::ops::Range;

use serde::Deserialize;

use crate::{
    ConfigurationError, ConfigurationErrorKind, ConfigurationSourceMetadata, CoreError,
    InvalidConfigurationValue,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub(super) struct RawConfiguration {
    pub(super) version: Option<i64>,
    pub(super) proxy: Option<RawProxyConfiguration>,
    #[serde(default)]
    pub(super) providers: Vec<RawProviderConfiguration>,
    #[serde(default)]
    pub(super) routing: RawRoutingConfiguration,
    #[serde(default)]
    pub(super) observability: RawObservabilityConfiguration,
    #[serde(default)]
    pub(super) lifecycle: RawLifecycleConfiguration,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub(super) struct RawProxyConfiguration {
    pub(super) listen_address: Option<String>,
    pub(super) port: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub(super) struct RawProviderConfiguration {
    pub(super) id: Option<String>,
    pub(super) protocol_family: Option<String>,
    pub(super) routing_eligible: Option<bool>,
    #[serde(default)]
    pub(super) accounts: Vec<RawAccountConfiguration>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub(super) struct RawAccountConfiguration {
    pub(super) id: Option<String>,
    pub(super) credential_reference: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub(super) struct RawRoutingConfiguration {
    #[serde(default)]
    pub(super) defaults: Vec<RawRoutingDefaultConfiguration>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub(super) struct RawRoutingDefaultConfiguration {
    pub(super) name: Option<String>,
    pub(super) model: Option<String>,
    pub(super) provider_id: Option<String>,
    pub(super) account_id: Option<String>,
    pub(super) fallback_enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub(super) struct RawObservabilityConfiguration {
    pub(super) logging: Option<String>,
    pub(super) usage_collection: Option<toml::Value>,
}

impl Default for RawObservabilityConfiguration {
    fn default() -> Self {
        Self {
            logging: Some("standard".to_string()),
            usage_collection: Some(toml::Value::Boolean(true)),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub(super) struct RawLifecycleConfiguration {
    pub(super) auto_start: Option<String>,
}

impl Default for RawLifecycleConfiguration {
    fn default() -> Self {
        Self {
            auto_start: Some("disabled".to_string()),
        }
    }
}

pub(super) fn parse_raw_configuration(
    contents: &str,
    source: ConfigurationSourceMetadata,
) -> Result<RawConfiguration, CoreError> {
    toml::from_str(contents).map_err(|error| {
        let message = error.message().to_string();
        let (kind, field, invalid_value) = classify_parse_error(contents, error.span(), &message);
        configuration_failure(vec![ConfigurationError::with_message(
            kind,
            field,
            invalid_value,
            Some(source),
            message,
        )])
    })
}

fn configuration_failure(errors: Vec<ConfigurationError>) -> CoreError {
    CoreError::Configuration { errors }
}

fn classify_parse_error(
    contents: &str,
    span: Option<Range<usize>>,
    message: &str,
) -> (ConfigurationErrorKind, String, InvalidConfigurationValue) {
    if message.contains("unknown field") {
        (
            ConfigurationErrorKind::UnknownField,
            infer_unknown_field_path(contents, span, message),
            InvalidConfigurationValue::Unsupported,
        )
    } else if message.contains("missing field") {
        (
            ConfigurationErrorKind::MissingRequiredField,
            "configuration".to_string(),
            InvalidConfigurationValue::Missing,
        )
    } else {
        (
            ConfigurationErrorKind::ParseFailed,
            "configuration".to_string(),
            InvalidConfigurationValue::Malformed,
        )
    }
}

fn infer_unknown_field_path(contents: &str, span: Option<Range<usize>>, message: &str) -> String {
    let field = message
        .split('`')
        .nth(1)
        .filter(|field| !field.trim().is_empty())
        .unwrap_or("configuration");
    let Some(span) = span else {
        return field.to_string();
    };

    let prefix = &contents[..span.start.min(contents.len())];
    let table = prefix.lines().rev().find_map(|line| {
        let line = line.trim();
        if line.starts_with("[[") && line.ends_with("]]") {
            Some(
                line.trim_start_matches("[[")
                    .trim_end_matches("]]")
                    .to_string(),
            )
        } else if line.starts_with('[') && line.ends_with(']') {
            Some(
                line.trim_start_matches('[')
                    .trim_end_matches(']')
                    .to_string(),
            )
        } else {
            None
        }
    });

    match table {
        Some(table) if !table.is_empty() => format!("{table}.{field}"),
        _ => field.to_string(),
    }
}
