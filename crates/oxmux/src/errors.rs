//! Structured error contracts shared across the headless core.
//!
//! Errors in this module are matchable public values for configuration, routing,
//! provider execution, protocol translation, streaming, local runtime, minimal
//! proxy, usage, and management boundaries.

use core::fmt;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
/// Structured configuration diagnostic tied to a field, invalid value class, and optional source.
pub struct ConfigurationError {
    /// Configuration layer or error kind for this value.
    pub kind: ConfigurationErrorKind,
    /// Configuration field path associated with the diagnostic.
    pub field_path: String,
    /// Class of invalid value detected for the field.
    pub invalid_value: InvalidConfigurationValue,
    /// Configuration source metadata for the diagnostic.
    pub source: Option<ConfigurationSourceMetadata>,
    /// Human-readable diagnostic message.
    pub message: Option<String>,
}

impl ConfigurationError {
    /// Creates a configuration diagnostic without an additional human-readable message.
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

    /// Creates a configuration diagnostic with an additional human-readable message.
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
/// Machine-matchable category for configuration validation or loading failures.
pub enum ConfigurationErrorKind {
    /// Configuration source could not be read.
    ReadFailed,
    /// Configuration source could not be parsed.
    ParseFailed,
    /// Configuration source format is unsupported.
    UnsupportedFormat,
    /// Required configuration field is missing.
    MissingRequiredField,
    /// Configuration field is unknown for the schema.
    UnknownField,
    /// Configuration version is unsupported or invalid.
    InvalidVersion,
    /// Listen address is invalid for local runtime use.
    InvalidListenAddress,
    /// Port is outside the supported range.
    InvalidPort,
    /// Provider identifier is duplicated.
    DuplicateProviderId,
    /// Account identifier is duplicated within a provider.
    DuplicateAccountId,
    /// Provider protocol-family value is invalid.
    InvalidProviderProtocolFamily,
    /// Credential reference is malformed or secret-like.
    InvalidCredentialReference,
    /// Routing references an unknown provider.
    UnknownProviderReference,
    /// Routing references an unknown account.
    UnknownAccountReference,
    /// Routing default configuration is invalid.
    InvalidRoutingDefault,
    /// Logging setting is invalid.
    InvalidLoggingSetting,
    /// Usage collection setting is invalid.
    InvalidUsageCollectionSetting,
    /// Auto-start setting is invalid.
    InvalidAutoStartIntent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Classifies why a configuration value was rejected.
pub enum InvalidConfigurationValue {
    /// Value was missing.
    Missing,
    /// Value had malformed syntax or shape.
    Malformed,
    /// Value is syntactically valid but unsupported.
    Unsupported,
    /// Value duplicates another configuration entry.
    Duplicate,
    /// Value references an unknown configured item.
    UnknownReference,
    /// Value appears to contain a secret instead of a reference.
    SecretLike,
    /// Value is outside the accepted numeric range.
    OutOfRange,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Human-readable source information for configuration diagnostics.
pub struct ConfigurationSourceMetadata {
    /// Filesystem path associated with this source, when one exists.
    pub path: Option<PathBuf>,
    /// Human-readable description of this source.
    pub description: String,
}

impl ConfigurationSourceMetadata {
    /// Creates source metadata for in-memory configuration contents.
    pub fn memory() -> Self {
        Self {
            path: None,
            description: "in-memory TOML contents".to_string(),
        }
    }

    /// Creates source metadata for a filesystem path.
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
/// Top-level structured error for public headless core operations.
pub enum CoreError {
    /// Not implemented state for this public enum.
    NotImplemented {
        /// Named core boundary whose runtime behavior is not implemented here.
        boundary: &'static str,
    },
    /// Management snapshot construction failed.
    ManagementSnapshot {
        /// Human-readable diagnostic message.
        message: String,
    },
    /// Lifecycle control intent could not be applied.
    LifecycleIntent {
        /// Human-readable diagnostic message.
        message: String,
    },
    /// Configuration update validation failed.
    ConfigurationValidation {
        /// Field path associated with this validation diagnostic.
        field: &'static str,
        /// Human-readable diagnostic message.
        message: String,
    },
    /// Configuration loading or validation returned structured errors.
    Configuration {
        /// Structured errors associated with this state.
        errors: Vec<ConfigurationError>,
    },
    /// Local runtime configuration was invalid.
    LocalRuntimeConfiguration {
        /// Field path associated with this validation diagnostic.
        field: &'static str,
        /// Human-readable diagnostic message.
        message: String,
    },
    /// Local runtime could not bind the requested endpoint.
    LocalRuntimeBind {
        /// Bound endpoint associated with this lifecycle state.
        endpoint: String,
        /// Human-readable diagnostic message.
        message: String,
    },
    /// Local runtime health serving failed.
    LocalRuntimeHealthServing {
        /// Human-readable diagnostic message.
        message: String,
    },
    /// Local runtime shutdown failed.
    LocalRuntimeShutdown {
        /// Human-readable diagnostic message.
        message: String,
    },
    /// Provider account summary construction failed.
    ProviderAccountSummary {
        /// Human-readable diagnostic message.
        message: String,
    },
    /// Provider execution failed for a selected provider/account.
    ProviderExecution {
        /// Provider identifier associated with this failure.
        provider_id: String,
        /// Optional account identifier associated with this failure.
        account_id: Option<String>,
        /// Structured failure associated with this state.
        failure: ProviderExecutionFailure,
    },
    /// Usage or quota summary construction failed.
    UsageQuotaSummary {
        /// Human-readable diagnostic message.
        message: String,
    },
    /// Protocol request or response validation failed.
    ProtocolValidation {
        /// Field path associated with this validation diagnostic.
        field: &'static str,
        /// Human-readable diagnostic message.
        message: String,
    },
    /// Protocol translation is valid but intentionally deferred.
    ProtocolTranslationDeferred {
        /// Request or response translation direction.
        direction: ProtocolTranslationDirection,
        /// Original protocol family for translation.
        source_family: ProtocolFamily,
        /// Desired protocol family for translation.
        target_family: ProtocolFamily,
        /// Human-readable diagnostic message.
        message: String,
    },
    /// Routing failed with a matchable routing failure.
    Routing {
        /// Structured failure associated with this state.
        failure: RoutingFailure,
    },
    /// Provider execution returns a streaming response.
    Streaming {
        /// Structured failure associated with this state.
        failure: StreamingFailure,
    },
    /// Minimal proxy request validation failed.
    MinimalProxyRequestValidation {
        /// Field path associated with this validation diagnostic.
        field: &'static str,
        /// Stable code for this failure or response.
        code: MinimalProxyErrorCode,
        /// Human-readable diagnostic message.
        message: String,
    },
    /// Minimal proxy cannot serialize the provider response mode.
    MinimalProxyUnsupportedResponseMode {
        /// Human-readable diagnostic message.
        message: String,
    },
    /// Minimal proxy response serialization failed.
    MinimalProxyResponseSerialization {
        /// Human-readable diagnostic message.
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
