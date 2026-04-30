//! Local client authorization contracts for loopback proxy access.
//!
//! These values describe caller-owned authorization to the local proxy. They do
//! not represent provider credentials, OAuth tokens, desktop credential storage,
//! or upstream provider account state.

use core::fmt;

/// Route scope protected by local client authorization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocalClientRouteScope {
    /// OpenAI-compatible inference route access.
    Inference,
    /// Local management, status, and control route access.
    Management,
}

impl LocalClientRouteScope {
    /// Returns a stable machine-readable scope label.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Inference => "inference",
            Self::Management => "management",
        }
    }
}

/// Secret-bearing local client credential.
#[derive(Clone, Eq, PartialEq)]
pub struct LocalClientCredential {
    secret: String,
}

impl LocalClientCredential {
    /// Creates a local client credential used only for loopback proxy authorization.
    pub fn new(secret: impl Into<String>) -> Result<Self, LocalClientCredentialError> {
        let secret = secret.into();
        if secret.trim().is_empty() {
            return Err(LocalClientCredentialError::EmptySecret);
        }

        Ok(Self { secret })
    }

    /// Returns redacted metadata safe for status and management surfaces.
    pub fn redacted_metadata(&self) -> RedactedLocalClientCredentialMetadata {
        RedactedLocalClientCredentialMetadata {
            configured: true,
            display: "<redacted local client credential>",
        }
    }

    pub(crate) fn matches(&self, presented: &str) -> bool {
        fixed_secret_time_eq(self.secret.as_bytes(), presented.as_bytes())
    }
}

fn fixed_secret_time_eq(expected: &[u8], presented: &[u8]) -> bool {
    let mut difference = expected.len() ^ presented.len();
    for (index, expected_byte) in expected.iter().enumerate() {
        let presented_byte = presented.get(index).copied().unwrap_or(0);
        difference |= usize::from(expected_byte ^ presented_byte);
    }
    difference == 0
}

/// Local client credential construction error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocalClientCredentialError {
    /// Credential secret was empty or whitespace-only.
    EmptySecret,
}

impl fmt::Display for LocalClientCredentialError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySecret => formatter.write_str("local client credential secret is empty"),
        }
    }
}

impl std::error::Error for LocalClientCredentialError {}

impl fmt::Debug for LocalClientCredential {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LocalClientCredential")
            .field("secret", &"<redacted>")
            .finish()
    }
}

/// Redacted credential metadata safe for logs, debug output, and management snapshots.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RedactedLocalClientCredentialMetadata {
    /// Whether a local client credential is configured.
    pub configured: bool,
    /// Stable redacted display label that never contains the raw credential.
    pub display: &'static str,
}

impl RedactedLocalClientCredentialMetadata {
    /// Metadata for a required credential that is not configured.
    pub fn missing() -> Self {
        Self {
            configured: false,
            display: "<missing local client credential>",
        }
    }
}

/// Local client authorization protection policy for a route scope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LocalClientAuthorizationPolicy {
    /// Route scope is classified but does not require local client authorization.
    Disabled,
    /// Route scope requires a matching configured local client credential.
    Required {
        /// Expected local client credential, when configured.
        credential: Option<LocalClientCredential>,
    },
}

impl LocalClientAuthorizationPolicy {
    /// Creates a disabled local client authorization policy.
    pub fn disabled() -> Self {
        Self::Disabled
    }

    /// Creates a required policy with a configured local client credential.
    pub fn required(credential: LocalClientCredential) -> Self {
        Self::Required {
            credential: Some(credential),
        }
    }

    /// Creates a required policy with no configured credential, which fails closed.
    pub fn required_without_credential() -> Self {
        Self::Required { credential: None }
    }

    /// Returns redacted status metadata for this policy.
    pub fn metadata(&self) -> LocalClientAuthorizationPolicyMetadata {
        match self {
            Self::Disabled => LocalClientAuthorizationPolicyMetadata::Disabled,
            Self::Required { credential } => LocalClientAuthorizationPolicyMetadata::Required {
                credential: credential
                    .as_ref()
                    .map(LocalClientCredential::redacted_metadata)
                    .unwrap_or_else(RedactedLocalClientCredentialMetadata::missing),
            },
        }
    }

    /// Authorizes a local client attempt for a route scope.
    pub fn authorize(
        &self,
        scope: LocalClientRouteScope,
        attempt: &LocalClientAuthorizationAttempt,
    ) -> LocalClientAuthorizationOutcome {
        match self {
            Self::Disabled => LocalClientAuthorizationOutcome::Disabled { scope },
            Self::Required { credential: None } => {
                LocalClientAuthorizationOutcome::Denied(LocalClientAuthorizationFailure::new(
                    scope,
                    LocalClientAuthorizationFailureReason::MissingConfiguredCredential,
                ))
            }
            Self::Required {
                credential: Some(credential),
            } => match attempt {
                LocalClientAuthorizationAttempt::Missing => {
                    LocalClientAuthorizationOutcome::Denied(LocalClientAuthorizationFailure::new(
                        scope,
                        LocalClientAuthorizationFailureReason::MissingCredential,
                    ))
                }
                LocalClientAuthorizationAttempt::Malformed => {
                    LocalClientAuthorizationOutcome::Denied(LocalClientAuthorizationFailure::new(
                        scope,
                        LocalClientAuthorizationFailureReason::MalformedCredential,
                    ))
                }
                LocalClientAuthorizationAttempt::UnsupportedScheme => {
                    LocalClientAuthorizationOutcome::Denied(LocalClientAuthorizationFailure::new(
                        scope,
                        LocalClientAuthorizationFailureReason::UnsupportedScheme,
                    ))
                }
                LocalClientAuthorizationAttempt::Bearer { token } if credential.matches(token) => {
                    LocalClientAuthorizationOutcome::Authorized { scope }
                }
                LocalClientAuthorizationAttempt::Bearer { .. } => {
                    LocalClientAuthorizationOutcome::Denied(LocalClientAuthorizationFailure::new(
                        scope,
                        LocalClientAuthorizationFailureReason::InvalidCredential,
                    ))
                }
            },
        }
    }
}

/// Management-safe metadata for a local client authorization policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocalClientAuthorizationPolicyMetadata {
    /// Protection is disabled for the route scope.
    Disabled,
    /// Protection is required for the route scope.
    Required {
        /// Redacted credential metadata for the required policy.
        credential: RedactedLocalClientCredentialMetadata,
    },
}

/// Independent protection policies for local route categories.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalRouteProtection {
    /// Inference route protection policy.
    pub inference: LocalClientAuthorizationPolicy,
    /// Management/status/control route protection policy.
    pub management: LocalClientAuthorizationPolicy,
}

impl LocalRouteProtection {
    /// Creates route protection with all scopes disabled.
    pub fn disabled() -> Self {
        Self {
            inference: LocalClientAuthorizationPolicy::Disabled,
            management: LocalClientAuthorizationPolicy::Disabled,
        }
    }

    /// Returns management-safe metadata for both route scopes.
    pub fn metadata(&self) -> LocalRouteProtectionMetadata {
        LocalRouteProtectionMetadata {
            inference: self.inference.metadata(),
            management: self.management.metadata(),
        }
    }
}

impl Default for LocalRouteProtection {
    fn default() -> Self {
        Self::disabled()
    }
}

/// Management-safe route protection metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LocalRouteProtectionMetadata {
    /// Inference route authorization metadata.
    pub inference: LocalClientAuthorizationPolicyMetadata,
    /// Management/status/control route authorization metadata.
    pub management: LocalClientAuthorizationPolicyMetadata,
}

impl LocalRouteProtectionMetadata {
    /// Metadata for route protection with all scopes disabled.
    pub fn disabled() -> Self {
        LocalRouteProtection::disabled().metadata()
    }
}

/// Local client authorization presented by a request adapter.
#[derive(Clone, Eq, PartialEq)]
pub enum LocalClientAuthorizationAttempt {
    /// No local client authorization credential was presented.
    Missing,
    /// A bearer token was presented.
    Bearer {
        /// Presented bearer token. Debug output redacts this value.
        token: String,
    },
    /// Authorization shape was malformed.
    Malformed,
    /// Authorization scheme was unsupported.
    UnsupportedScheme,
}

impl LocalClientAuthorizationAttempt {
    /// Creates a bearer-token authorization attempt.
    pub fn bearer(token: impl Into<String>) -> Self {
        Self::Bearer {
            token: token.into(),
        }
    }
}

impl fmt::Debug for LocalClientAuthorizationAttempt {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => formatter.write_str("Missing"),
            Self::Bearer { .. } => formatter
                .debug_struct("Bearer")
                .field("token", &"<redacted>")
                .finish(),
            Self::Malformed => formatter.write_str("Malformed"),
            Self::UnsupportedScheme => formatter.write_str("UnsupportedScheme"),
        }
    }
}

/// Structured local client authorization outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LocalClientAuthorizationOutcome {
    /// Route scope authorized the presented local client credential.
    Authorized {
        /// Authorized route scope.
        scope: LocalClientRouteScope,
    },
    /// Route scope did not require authorization.
    Disabled {
        /// Unprotected route scope.
        scope: LocalClientRouteScope,
    },
    /// Route scope denied the request.
    Denied(LocalClientAuthorizationFailure),
}

impl LocalClientAuthorizationOutcome {
    /// Converts the outcome into a result suitable for route dispatch.
    pub fn into_result(self) -> Result<(), LocalClientAuthorizationFailure> {
        match self {
            Self::Authorized { .. } | Self::Disabled { .. } => Ok(()),
            Self::Denied(failure) => Err(failure),
        }
    }
}

/// Structured local client authorization failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LocalClientAuthorizationFailure {
    /// Route scope that rejected the request.
    pub scope: LocalClientRouteScope,
    /// Stable failure reason.
    pub reason: LocalClientAuthorizationFailureReason,
}

impl LocalClientAuthorizationFailure {
    /// Creates a local client authorization failure.
    pub fn new(
        scope: LocalClientRouteScope,
        reason: LocalClientAuthorizationFailureReason,
    ) -> Self {
        Self { scope, reason }
    }
}

impl fmt::Display for LocalClientAuthorizationFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "local client authorization failed for {}: {}",
            self.scope.as_str(),
            self.reason.as_str()
        )
    }
}

impl std::error::Error for LocalClientAuthorizationFailure {}

/// Stable local client authorization failure reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocalClientAuthorizationFailureReason {
    /// Request omitted required local client authorization.
    MissingCredential,
    /// Request authorization header was malformed.
    MalformedCredential,
    /// Request used an unsupported authorization scheme.
    UnsupportedScheme,
    /// Request credential did not match the configured credential.
    InvalidCredential,
    /// Route required authorization but no credential was configured.
    MissingConfiguredCredential,
}

impl LocalClientAuthorizationFailureReason {
    /// Returns a stable serialized failure code.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MissingCredential => "missing_credential",
            Self::MalformedCredential => "malformed_credential",
            Self::UnsupportedScheme => "unsupported_scheme",
            Self::InvalidCredential => "invalid_credential",
            Self::MissingConfiguredCredential => "missing_configured_credential",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LocalClientCredential, fixed_secret_time_eq};

    #[test]
    fn local_client_credential_matches_exact_secret_only() {
        let credential = LocalClientCredential::new("expected-token").expect("valid credential");

        assert!(credential.matches("expected-token"));
        assert!(!credential.matches("wrong-token"));
        assert!(!credential.matches("expected-token-extra"));
        assert!(!credential.matches("expected"));
    }

    #[test]
    fn fixed_secret_time_comparison_rejects_length_mismatches() {
        assert!(fixed_secret_time_eq(b"secret", b"secret"));
        assert!(!fixed_secret_time_eq(b"secret", b"secret-extra"));
        assert!(!fixed_secret_time_eq(b"secret", b"sec"));
        assert!(!fixed_secret_time_eq(b"secret", b"secreu"));
    }
}
