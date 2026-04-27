use core::fmt;

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
    UsageQuotaSummary {
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
            Self::UsageQuotaSummary { message } => {
                write!(formatter, "usage quota summary failed: {message}")
            }
        }
    }
}

impl std::error::Error for CoreError {}
