use core::fmt;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigurationError {
    pub kind: ConfigurationErrorKind,
    pub field_path: String,
    pub invalid_value: InvalidConfigurationValue,
    pub source: Option<ConfigurationSourceMetadata>,
    pub message: Option<String>,
}

impl ConfigurationError {
    pub fn new(
        kind: ConfigurationErrorKind,
        field_path: impl Into<String>,
        invalid_value: InvalidConfigurationValue,
        source: Option<ConfigurationSourceMetadata>,
    ) -> Self {
        Self {
            kind,
            field_path: field_path.into(),
            invalid_value,
            source,
            message: None,
        }
    }

    pub fn with_message(
        kind: ConfigurationErrorKind,
        field_path: impl Into<String>,
        invalid_value: InvalidConfigurationValue,
        source: Option<ConfigurationSourceMetadata>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            field_path: field_path.into(),
            invalid_value,
            source,
            message: Some(message.into()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigurationErrorKind {
    ReadFailed,
    ParseFailed,
    UnsupportedFormat,
    MissingRequiredField,
    UnknownField,
    InvalidVersion,
    InvalidListenAddress,
    InvalidPort,
    DuplicateProviderId,
    DuplicateAccountId,
    InvalidProviderProtocolFamily,
    InvalidCredentialReference,
    UnknownProviderReference,
    UnknownAccountReference,
    InvalidRoutingDefault,
    InvalidLoggingSetting,
    InvalidUsageCollectionSetting,
    InvalidAutoStartIntent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvalidConfigurationValue {
    Missing,
    Malformed,
    Unsupported,
    Duplicate,
    UnknownReference,
    SecretLike,
    OutOfRange,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigurationSourceMetadata {
    pub path: Option<PathBuf>,
    pub description: String,
}

impl ConfigurationSourceMetadata {
    pub fn memory() -> Self {
        Self {
            path: None,
            description: "in-memory TOML contents".to_string(),
        }
    }

    pub fn for_path(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref().to_path_buf();
        Self {
            description: path.display().to_string(),
            path: Some(path),
        }
    }
}

use crate::{
    MinimalProxyErrorCode, ProtocolFamily, ProtocolTranslationDirection, ProviderExecutionFailure,
    RoutingFailure, StreamingFailure,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CoreError {
    NotImplemented {
        boundary: &'static str,
    },
    ManagementSnapshot {
        message: String,
    },
    LifecycleIntent {
        message: String,
    },
    ConfigurationValidation {
        field: &'static str,
        message: String,
    },
    Configuration {
        errors: Vec<ConfigurationError>,
    },
    LocalRuntimeConfiguration {
        field: &'static str,
        message: String,
    },
    LocalRuntimeBind {
        endpoint: String,
        message: String,
    },
    LocalRuntimeHealthServing {
        message: String,
    },
    LocalRuntimeShutdown {
        message: String,
    },
    ProviderAccountSummary {
        message: String,
    },
    ProviderExecution {
        provider_id: String,
        account_id: Option<String>,
        failure: ProviderExecutionFailure,
    },
    UsageQuotaSummary {
        message: String,
    },
    ProtocolValidation {
        field: &'static str,
        message: String,
    },
    ProtocolTranslationDeferred {
        direction: ProtocolTranslationDirection,
        source_family: ProtocolFamily,
        target_family: ProtocolFamily,
        message: String,
    },
    Routing {
        failure: RoutingFailure,
    },
    Streaming {
        failure: StreamingFailure,
    },
    MinimalProxyRequestValidation {
        field: &'static str,
        code: MinimalProxyErrorCode,
        message: String,
    },
    MinimalProxyUnsupportedResponseMode {
        message: String,
    },
    MinimalProxyResponseSerialization {
        message: String,
    },
}

impl fmt::Display for CoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotImplemented { boundary } => {
                write!(formatter, "{boundary} core behavior is not implemented yet")
            }
            Self::ManagementSnapshot { message } => {
                write!(formatter, "management snapshot failed: {message}")
            }
            Self::LifecycleIntent { message } => {
                write!(formatter, "lifecycle intent failed: {message}")
            }
            Self::ConfigurationValidation { field, message } => {
                write!(
                    formatter,
                    "configuration field {field} is invalid: {message}"
                )
            }
            Self::Configuration { errors } => {
                write!(
                    formatter,
                    "configuration load failed with {} error(s)",
                    errors.len()
                )
            }
            Self::LocalRuntimeConfiguration { field, message } => {
                write!(
                    formatter,
                    "local runtime configuration field {field} is invalid: {message}"
                )
            }
            Self::LocalRuntimeBind { endpoint, message } => {
                write!(
                    formatter,
                    "local runtime failed to bind {endpoint}: {message}"
                )
            }
            Self::LocalRuntimeHealthServing { message } => {
                write!(formatter, "local runtime health serving failed: {message}")
            }
            Self::LocalRuntimeShutdown { message } => {
                write!(formatter, "local runtime shutdown failed: {message}")
            }
            Self::ProviderAccountSummary { message } => {
                write!(formatter, "provider account summary failed: {message}")
            }
            Self::ProviderExecution {
                provider_id,
                account_id,
                failure,
            } => {
                write!(formatter, "provider execution failed for {provider_id}")?;
                if let Some(account_id) = account_id {
                    write!(formatter, "/{account_id}")?;
                }
                write!(formatter, ": {}", failure.message())
            }
            Self::UsageQuotaSummary { message } => {
                write!(formatter, "usage quota summary failed: {message}")
            }
            Self::ProtocolValidation { field, message } => {
                write!(formatter, "protocol field {field} is invalid: {message}")
            }
            Self::ProtocolTranslationDeferred {
                direction,
                source_family,
                target_family,
                message,
            } => write!(
                formatter,
                "protocol {direction:?} translation from {source_family:?} to {target_family:?} is deferred: {message}"
            ),
            Self::Routing { failure } => {
                write!(formatter, "routing failed: {}", failure.message())
            }
            Self::Streaming { failure } => {
                write!(formatter, "streaming failed: {}", failure.message())
            }
            Self::MinimalProxyRequestValidation { field, message, .. } => {
                write!(
                    formatter,
                    "minimal proxy request field {field} is invalid: {message}"
                )
            }
            Self::MinimalProxyUnsupportedResponseMode { message } => {
                write!(
                    formatter,
                    "minimal proxy response mode is unsupported: {message}"
                )
            }
            Self::MinimalProxyResponseSerialization { message } => {
                write!(
                    formatter,
                    "minimal proxy response serialization failed: {message}"
                )
            }
        }
    }
}

impl std::error::Error for CoreError {}
