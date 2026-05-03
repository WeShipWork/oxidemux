//! Static model registry and listing contracts for the headless core.
//!
//! The registry describes configured model entries, aliases, provider-native
//! targets, provider/account applicability, and listing state. It does not query
//! providers, validate credentials, inspect quotas, perform routing selection, or
//! execute provider requests.

use crate::configuration::ValidatedFileConfiguration;
use crate::provider::{AuthMethodCategory, DegradedReason, ProtocolFamily, ProviderSummary};
use crate::routing::{
    RoutingAvailabilitySnapshot, RoutingAvailabilityState, RoutingPolicy, RoutingTarget,
};

#[derive(Clone, Debug, Eq, PartialEq)]
/// Deterministic model registry built from static core configuration inputs.
pub struct ModelRegistry {
    /// Ordered configured model entries.
    pub entries: Vec<ModelRegistryEntry>,
}

impl ModelRegistry {
    /// Builds a registry from routing policy and provider summaries without availability metadata.
    pub fn from_policy(policy: &RoutingPolicy, providers: &[ProviderSummary]) -> Self {
        Self::from_policy_with_availability(policy, providers, None)
    }

    /// Builds a registry from routing policy, provider summaries, and optional availability metadata.
    pub fn from_policy_with_availability(
        policy: &RoutingPolicy,
        providers: &[ProviderSummary],
        availability: Option<&RoutingAvailabilitySnapshot>,
    ) -> Self {
        let entries = policy
            .routes
            .iter()
            .flat_map(|route| {
                let aliases: Vec<&crate::routing::ModelAlias> = policy
                    .model_aliases
                    .iter()
                    .filter(|alias| alias.resolved_model == route.resolved_model)
                    .collect();
                let candidates = route
                    .candidates
                    .iter()
                    .enumerate()
                    .map(|(candidate_order, candidate)| {
                        ModelRegistryCandidate::from_target(
                            &route.resolved_model,
                            &candidate.target,
                            candidate_order,
                            None,
                            Some(policy.fallback.fallback_enabled),
                            providers,
                            availability,
                        )
                    })
                    .collect::<Vec<_>>();

                let canonical = ModelRegistryEntry::new(
                    ListedModelIdentity::new(&route.resolved_model, &route.resolved_model),
                    None,
                    ModelForkMetadata::from_candidate_count(candidates.len()),
                    candidates.clone(),
                );

                let alias_entries = aliases.into_iter().map(move |alias| {
                    ModelRegistryEntry::new(
                        ListedModelIdentity::new(&alias.requested_model, &route.resolved_model),
                        Some(ModelAliasMetadata::new(
                            &alias.requested_model,
                            &alias.resolved_model,
                        )),
                        ModelForkMetadata::from_candidate_count(candidates.len()),
                        candidates.clone(),
                    )
                });

                std::iter::once(canonical).chain(alias_entries)
            })
            .collect();

        Self { entries }
    }

    /// Builds a registry from validated file-backed configuration.
    pub fn from_file_configuration(configuration: &ValidatedFileConfiguration) -> Self {
        let providers = configuration.provider_summaries();
        Self::from_file_configuration_with_availability(configuration, &providers, None)
    }

    /// Builds a registry from validated file-backed configuration, provider summaries, and optional availability metadata.
    pub fn from_file_configuration_with_availability(
        configuration: &ValidatedFileConfiguration,
        providers: &[ProviderSummary],
        availability: Option<&RoutingAvailabilitySnapshot>,
    ) -> Self {
        let entries = configuration
            .routing_default_groups
            .iter()
            .map(|group| {
                let candidates = group
                    .candidates
                    .iter()
                    .enumerate()
                    .map(|(candidate_order, candidate)| {
                        ModelRegistryCandidate::from_target(
                            &candidate.model,
                            &candidate.target,
                            candidate_order,
                            Some(&candidate.name),
                            Some(candidate.fallback_enabled),
                            providers,
                            availability,
                        )
                    })
                    .collect::<Vec<_>>();

                ModelRegistryEntry::new(
                    ListedModelIdentity::new(&group.model, &group.model),
                    None,
                    ModelForkMetadata::from_candidate_count(candidates.len()),
                    candidates,
                )
            })
            .collect();

        Self { entries }
    }

    /// Returns all configured entries in deterministic registry order.
    pub fn all_entries(&self) -> &[ModelRegistryEntry] {
        &self.entries
    }

    /// Returns entries with at least one visible and routable candidate.
    pub fn visible_entries(&self) -> Vec<&ModelRegistryEntry> {
        self.entries
            .iter()
            .filter(|entry| {
                entry
                    .candidates
                    .iter()
                    .any(ModelRegistryCandidate::is_visible)
            })
            .collect()
    }

    /// Returns entries with at least one disabled or routing-ineligible candidate.
    pub fn disabled_entries(&self) -> Vec<&ModelRegistryEntry> {
        self.entries
            .iter()
            .filter(|entry| {
                entry
                    .candidates
                    .iter()
                    .any(ModelRegistryCandidate::is_disabled)
            })
            .collect()
    }

    /// Returns entries with at least one degraded candidate.
    pub fn degraded_entries(&self) -> Vec<&ModelRegistryEntry> {
        self.entries
            .iter()
            .filter(|entry| {
                entry
                    .candidates
                    .iter()
                    .any(ModelRegistryCandidate::is_degraded)
            })
            .collect()
    }

    /// Returns entries matching one deterministic listing filter.
    pub fn entries_matching(&self, filter: ModelListingFilter) -> Vec<&ModelRegistryEntry> {
        match filter {
            ModelListingFilter::AllConfigured => self.entries.iter().collect(),
            ModelListingFilter::Visible => self.visible_entries(),
            ModelListingFilter::Disabled => self.disabled_entries(),
            ModelListingFilter::Degraded => self.degraded_entries(),
        }
    }

    /// Projects visible entries into the minimal OpenAI-compatible model list shape.
    pub fn open_ai_model_list(&self) -> OpenAiModelListProjection {
        OpenAiModelListProjection::from_entries(self.visible_entries())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// One listed model and its configured provider/account candidates.
pub struct ModelRegistryEntry {
    /// Listed and resolved model identity metadata.
    pub identity: ListedModelIdentity,
    /// Alias metadata when this entry was created from a requested alias.
    pub alias: Option<ModelAliasMetadata>,
    /// Fork metadata describing whether this listed model fans out to multiple targets.
    pub fork: ModelForkMetadata,
    /// Ordered provider/account candidates for this listed model.
    pub candidates: Vec<ModelRegistryCandidate>,
}

impl ModelRegistryEntry {
    /// Creates one registry entry from typed identity, alias, fork, and candidate metadata.
    pub fn new(
        identity: ListedModelIdentity,
        alias: Option<ModelAliasMetadata>,
        fork: ModelForkMetadata,
        candidates: Vec<ModelRegistryCandidate>,
    ) -> Self {
        Self {
            identity,
            alias,
            fork,
            candidates,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Listed model identity with user-facing and resolved model identifiers kept separate.
pub struct ListedModelIdentity {
    /// Model identifier exposed to callers and listing consumers.
    pub listed_model_id: String,
    /// Model identifier after applying core alias metadata.
    pub resolved_model_id: String,
}

impl ListedModelIdentity {
    /// Creates listed model identity metadata.
    pub fn new(listed_model_id: impl Into<String>, resolved_model_id: impl Into<String>) -> Self {
        Self {
            listed_model_id: listed_model_id.into(),
            resolved_model_id: resolved_model_id.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Provider-native model target for one routing candidate.
pub struct ProviderNativeModelTarget {
    /// Provider-native model identifier sent to the selected provider in future execution paths.
    pub provider_native_model_id: String,
    /// Provider/account target associated with this native model.
    pub target: RoutingTarget,
}

impl ProviderNativeModelTarget {
    /// Creates provider-native model target metadata.
    pub fn new(provider_native_model_id: impl Into<String>, target: RoutingTarget) -> Self {
        Self {
            provider_native_model_id: provider_native_model_id.into(),
            target,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Alias metadata preserving requested and resolved model identifiers.
pub struct ModelAliasMetadata {
    /// User-facing requested model identifier.
    pub requested_model_id: String,
    /// Resolved model identifier used by routing policy metadata.
    pub resolved_model_id: String,
}

impl ModelAliasMetadata {
    /// Creates alias metadata for a requested-to-resolved model mapping.
    pub fn new(
        requested_model_id: impl Into<String>,
        resolved_model_id: impl Into<String>,
    ) -> Self {
        Self {
            requested_model_id: requested_model_id.into(),
            resolved_model_id: resolved_model_id.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Metadata describing whether a listed model has multiple candidate targets.
pub struct ModelForkMetadata {
    /// Whether this listed model has more than one configured candidate target.
    pub forked: bool,
    /// Number of configured candidate targets under this listed model.
    pub candidate_count: usize,
}

impl ModelForkMetadata {
    /// Creates fork metadata from a candidate count.
    pub const fn from_candidate_count(candidate_count: usize) -> Self {
        Self {
            forked: candidate_count > 1,
            candidate_count,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// One ordered provider/account candidate for a registry entry.
pub struct ModelRegistryCandidate {
    /// Provider-native model target metadata.
    pub native_target: ProviderNativeModelTarget,
    /// Candidate ordering and file-routing default metadata.
    pub candidate: RoutingCandidateMetadata,
    /// Provider/account applicability metadata.
    pub applicability: ProviderAccountApplicability,
    /// Capability metadata derived from provider declarations or summaries.
    pub capabilities: ModelCapabilityMetadata,
    /// Listing state for disabled, degraded, unknown, or visible candidates.
    pub listing_state: ModelListingState,
}

impl ModelRegistryCandidate {
    /// Creates one model registry candidate from explicit typed metadata.
    pub fn new(
        native_target: ProviderNativeModelTarget,
        candidate: RoutingCandidateMetadata,
        applicability: ProviderAccountApplicability,
        capabilities: ModelCapabilityMetadata,
        listing_state: ModelListingState,
    ) -> Self {
        Self {
            native_target,
            candidate,
            applicability,
            capabilities,
            listing_state,
        }
    }

    /// Returns true when this candidate should be visible as routable configured catalog data.
    pub fn is_visible(&self) -> bool {
        matches!(
            self.listing_state,
            ModelListingState::Available | ModelListingState::Degraded { .. }
        )
    }

    /// Returns true when this candidate is disabled, ineligible, unavailable, exhausted, or unknown.
    pub fn is_disabled(&self) -> bool {
        matches!(
            self.listing_state,
            ModelListingState::Disabled { .. }
                | ModelListingState::RoutingIneligible { .. }
                | ModelListingState::UnknownProvider { .. }
                | ModelListingState::UnknownAccount { .. }
        )
    }

    /// Returns true when this candidate has degraded metadata.
    pub fn is_degraded(&self) -> bool {
        matches!(self.listing_state, ModelListingState::Degraded { .. })
    }

    fn from_target(
        provider_native_model_id: &str,
        target: &RoutingTarget,
        candidate_order: usize,
        route_name: Option<&str>,
        fallback_enabled: Option<bool>,
        providers: &[ProviderSummary],
        availability: Option<&RoutingAvailabilitySnapshot>,
    ) -> Self {
        let provider = providers
            .iter()
            .find(|provider| provider.provider_id == target.provider_id);
        let account = provider.and_then(|provider| {
            target.account_id.as_ref().map(|account_id| {
                provider
                    .accounts
                    .iter()
                    .any(|account| account.account_id == *account_id)
            })
        });
        let capabilities = provider
            .and_then(|provider| provider.capabilities.first())
            .map(ModelCapabilityMetadata::from_capability)
            .unwrap_or_else(ModelCapabilityMetadata::unknown);
        let applicability = ProviderAccountApplicability::new(
            target.provider_id.clone(),
            target.account_id.clone(),
            provider.is_some(),
            account.unwrap_or(true),
        );
        let listing_state =
            listing_state_for(provider, account, &capabilities, target, availability);

        Self::new(
            ProviderNativeModelTarget::new(provider_native_model_id, target.clone()),
            RoutingCandidateMetadata::new(
                candidate_order,
                route_name.map(str::to_string),
                fallback_enabled,
            ),
            applicability,
            capabilities,
            listing_state,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Candidate order, route label, and fallback metadata supplied by routing configuration.
pub struct RoutingCandidateMetadata {
    /// Zero-based deterministic candidate order within the listed model.
    pub candidate_order: usize,
    /// Optional routing default or group name associated with this candidate.
    pub route_name: Option<String>,
    /// Optional fallback flag from routing policy or file-backed routing defaults.
    pub fallback_enabled: Option<bool>,
}

impl RoutingCandidateMetadata {
    /// Creates routing candidate metadata.
    pub fn new(
        candidate_order: usize,
        route_name: Option<String>,
        fallback_enabled: Option<bool>,
    ) -> Self {
        Self {
            candidate_order,
            route_name,
            fallback_enabled,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Provider/account applicability metadata for one model candidate.
pub struct ProviderAccountApplicability {
    /// Provider identifier from routing target metadata.
    pub provider_id: String,
    /// Optional account identifier from routing target metadata.
    pub account_id: Option<String>,
    /// Whether provider metadata was supplied for this target.
    pub provider_known: bool,
    /// Whether account metadata was supplied or no account was requested.
    pub account_known: bool,
}

impl ProviderAccountApplicability {
    /// Creates provider/account applicability metadata.
    pub fn new(
        provider_id: impl Into<String>,
        account_id: Option<String>,
        provider_known: bool,
        account_known: bool,
    ) -> Self {
        Self {
            provider_id: provider_id.into(),
            account_id,
            provider_known,
            account_known,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Protocol, auth, streaming, and routing eligibility metadata for one candidate.
pub struct ModelCapabilityMetadata {
    /// Protocol family associated with the provider capability, when known.
    pub protocol_family: Option<ProtocolFamily>,
    /// Whether the provider advertises streaming support.
    pub supports_streaming: bool,
    /// Authentication method category advertised by the provider, when known.
    pub auth_method: Option<AuthMethodCategory>,
    /// Whether routing may select this provider according to static metadata.
    pub routing_eligible: bool,
}

impl ModelCapabilityMetadata {
    /// Creates capability metadata from explicit values.
    pub const fn new(
        protocol_family: Option<ProtocolFamily>,
        supports_streaming: bool,
        auth_method: Option<AuthMethodCategory>,
        routing_eligible: bool,
    ) -> Self {
        Self {
            protocol_family,
            supports_streaming,
            auth_method,
            routing_eligible,
        }
    }

    /// Creates unknown capability metadata for missing provider state.
    pub const fn unknown() -> Self {
        Self::new(None, false, None, false)
    }

    fn from_capability(capability: &crate::provider::ProviderCapability) -> Self {
        Self::new(
            Some(capability.protocol_family),
            capability.supports_streaming,
            Some(capability.auth_method),
            capability.routing_eligible,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Listing state for one model candidate.
pub enum ModelListingState {
    /// Candidate is statically configured and routable.
    Available,
    /// Candidate is statically configured but disabled or unavailable.
    Disabled {
        /// Human-readable reason for disabled state.
        reason: String,
    },
    /// Candidate provider is known but marked ineligible for routing.
    RoutingIneligible {
        /// Human-readable reason for routing-ineligible state.
        reason: String,
    },
    /// Candidate is statically configured but degraded.
    Degraded {
        /// Reasons explaining degraded state.
        reasons: Vec<DegradedReason>,
    },
    /// Candidate references an unknown provider.
    UnknownProvider {
        /// Human-readable reason for unknown provider state.
        reason: String,
    },
    /// Candidate references an unknown account for a known provider.
    UnknownAccount {
        /// Human-readable reason for unknown account state.
        reason: String,
    },
}

impl ModelListingState {
    /// Returns a human-readable reason for non-available listing states.
    pub fn reason(&self) -> Option<String> {
        match self {
            Self::Available => None,
            Self::Disabled { reason }
            | Self::RoutingIneligible { reason }
            | Self::UnknownProvider { reason }
            | Self::UnknownAccount { reason } => Some(reason.clone()),
            Self::Degraded { reasons } => Some(
                reasons
                    .iter()
                    .map(|reason| format!("{}: {}", reason.component, reason.message))
                    .collect::<Vec<_>>()
                    .join("; "),
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Filter modes supported by deterministic model registry listing APIs.
pub enum ModelListingFilter {
    /// Include all configured registry entries.
    AllConfigured,
    /// Include entries with at least one visible/routable candidate.
    Visible,
    /// Include entries with at least one disabled candidate.
    Disabled,
    /// Include entries with at least one degraded candidate.
    Degraded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Minimal OpenAI-compatible model list projection derived from registry entries.
pub struct OpenAiModelListProjection {
    /// OpenAI-compatible object kind for list responses.
    pub object: String,
    /// Projected model entries.
    pub data: Vec<OpenAiModelProjection>,
}

impl OpenAiModelListProjection {
    /// Creates an OpenAI-compatible projection from visible registry entries.
    pub fn from_entries(entries: Vec<&ModelRegistryEntry>) -> Self {
        Self {
            object: "list".to_string(),
            data: entries
                .into_iter()
                .map(OpenAiModelProjection::from_registry_entry)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Minimal OpenAI-compatible model projection for one registry entry.
pub struct OpenAiModelProjection {
    /// OpenAI-compatible model identifier.
    pub id: String,
    /// OpenAI-compatible object kind for model entries.
    pub object: String,
    /// Deterministic static timestamp placeholder for configured catalog data.
    pub created: u64,
    /// Provider ownership label derived from first visible candidate metadata.
    pub owned_by: String,
}

impl OpenAiModelProjection {
    /// Creates an OpenAI-compatible projection from one registry entry.
    pub fn from_registry_entry(entry: &ModelRegistryEntry) -> Self {
        let owned_by = entry
            .candidates
            .iter()
            .find(|candidate| candidate.is_visible())
            .map(|candidate| candidate.applicability.provider_id.clone())
            .unwrap_or_else(|| "oxmux".to_string());

        Self {
            id: entry.identity.listed_model_id.clone(),
            object: "model".to_string(),
            created: 0,
            owned_by,
        }
    }
}

fn listing_state_for(
    provider: Option<&ProviderSummary>,
    account_known: Option<bool>,
    capabilities: &ModelCapabilityMetadata,
    target: &RoutingTarget,
    availability: Option<&RoutingAvailabilitySnapshot>,
) -> ModelListingState {
    let Some(provider) = provider else {
        return ModelListingState::UnknownProvider {
            reason: format!("provider {} is not declared", target.provider_id),
        };
    };

    if account_known == Some(false) {
        let account_id = target.account_id.as_deref().unwrap_or("<none>");
        return ModelListingState::UnknownAccount {
            reason: format!(
                "account {account_id} is not declared for provider {}",
                target.provider_id
            ),
        };
    }

    if !capabilities.routing_eligible {
        return ModelListingState::RoutingIneligible {
            reason: format!("provider {} is not routing eligible", target.provider_id),
        };
    }

    if let Some(state) = availability.and_then(|availability| availability.state_for(target)) {
        return match state {
            RoutingAvailabilityState::Available => degraded_or_available(provider, target),
            RoutingAvailabilityState::Degraded { reason } => ModelListingState::Degraded {
                reasons: vec![DegradedReason {
                    component: target_label(target),
                    message: reason.clone(),
                }],
            },
            RoutingAvailabilityState::Unavailable { reason }
            | RoutingAvailabilityState::Exhausted { reason } => ModelListingState::Disabled {
                reason: reason.clone(),
            },
        };
    }

    degraded_or_available(provider, target)
}

fn degraded_or_available(provider: &ProviderSummary, target: &RoutingTarget) -> ModelListingState {
    let mut reasons = provider.degraded_reasons.clone();
    if let Some(account_id) = &target.account_id
        && let Some(account) = provider
            .accounts
            .iter()
            .find(|account| account.account_id == *account_id)
    {
        reasons.extend(account.degraded_reasons.clone());
    }

    if reasons.is_empty() {
        ModelListingState::Available
    } else {
        ModelListingState::Degraded { reasons }
    }
}

fn target_label(target: &RoutingTarget) -> String {
    match &target.account_id {
        Some(account_id) => format!("{}/{}", target.provider_id, account_id),
        None => target.provider_id.clone(),
    }
}
