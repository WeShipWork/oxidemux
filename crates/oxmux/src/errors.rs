use core::fmt;

use crate::{ProtocolFamily, ProtocolTranslationDirection, ProviderExecutionFailure};

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
        }
    }
}

impl std::error::Error for CoreError {}
