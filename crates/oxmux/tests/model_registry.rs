//! Integration tests for static oxmux model registry listing.

use oxmux::{
    AccountSummary, AuthMethodCategory, AuthState, ConfigurationBoundary, DegradedReason,
    FallbackBehavior, ModelAlias, ModelListingFilter, ModelListingState, ModelRegistry, ModelRoute,
    ProtocolFamily, ProviderCapability, ProviderSummary, QuotaState, RoutingAvailabilitySnapshot,
    RoutingAvailabilityState, RoutingCandidate, RoutingPolicy, RoutingTarget,
    RoutingTargetAvailability,
};

const VALID_TOML: &str = include_str!("fixtures/file_configuration/valid.toml");

#[test]
fn in_memory_registry_preserves_aliases_forks_and_candidates_without_selection() {
    let openai_target = RoutingTarget::provider_account("openai", "primary");
    let claude_target = RoutingTarget::provider_account("claude", "fallback");
    let policy = RoutingPolicy::new(vec![ModelRoute::new(
        "gpt-4o",
        vec![
            RoutingCandidate::new(openai_target.clone()),
            RoutingCandidate::new(claude_target.clone()),
        ],
    )])
    .with_model_alias(ModelAlias::new("fast-chat", "gpt-4o"))
    .with_fallback(FallbackBehavior::new(true, false));
    let original_policy = policy.clone();
    let providers = vec![
        provider_summary("openai", ProtocolFamily::OpenAi, true, true, "primary"),
        provider_summary("claude", ProtocolFamily::Claude, true, false, "fallback"),
    ];

    let registry = ModelRegistry::from_policy(&policy, &providers);

    assert_eq!(policy, original_policy);
    assert_eq!(registry.all_entries().len(), 2);
    assert_eq!(registry.all_entries()[0].identity.listed_model_id, "gpt-4o");
    assert_eq!(
        registry.all_entries()[0].identity.resolved_model_id,
        "gpt-4o"
    );
    assert!(registry.all_entries()[0].alias.is_none());
    assert!(registry.all_entries()[0].fork.forked);
    assert_eq!(registry.all_entries()[0].fork.candidate_count, 2);
    assert_eq!(
        registry.all_entries()[0].candidates[0]
            .native_target
            .provider_native_model_id,
        "gpt-4o"
    );
    assert_eq!(
        registry.all_entries()[0].candidates[1].native_target.target,
        claude_target
    );
    assert_eq!(
        registry.all_entries()[0].candidates[0]
            .candidate
            .fallback_enabled,
        Some(true)
    );
    assert_eq!(
        registry.all_entries()[0].candidates[1]
            .candidate
            .fallback_enabled,
        Some(true)
    );
    assert_eq!(
        registry.all_entries()[1].identity.listed_model_id,
        "fast-chat"
    );
    assert_eq!(
        registry.all_entries()[1]
            .alias
            .as_ref()
            .map(|alias| (&alias.requested_model_id, &alias.resolved_model_id)),
        Some((&"fast-chat".to_string(), &"gpt-4o".to_string()))
    );
}

#[test]
fn policy_registry_preserves_disabled_fallback_metadata() {
    let policy = RoutingPolicy::new(vec![ModelRoute::new(
        "single-shot-model",
        vec![RoutingCandidate::new(RoutingTarget::provider("openai"))],
    )])
    .with_fallback(FallbackBehavior::disabled());
    let providers = vec![provider_summary_without_account(
        "openai",
        ProtocolFamily::OpenAi,
        true,
        true,
    )];

    let registry = ModelRegistry::from_policy(&policy, &providers);

    assert_eq!(
        registry.all_entries()[0].candidates[0]
            .candidate
            .fallback_enabled,
        Some(false)
    );
}

#[test]
fn file_configuration_builds_deterministic_registry_entries()
-> Result<(), Box<dyn std::error::Error>> {
    let configuration = ConfigurationBoundary::load_contents(VALID_TOML)?;

    let first = ModelRegistry::from_file_configuration(&configuration);
    let second = ModelRegistry::from_file_configuration(&configuration);

    assert_eq!(first, second);
    assert_eq!(first.all_entries().len(), 2);
    assert_eq!(
        first.all_entries()[0].identity.listed_model_id,
        "gpt-4o-mini"
    );
    assert_eq!(first.all_entries()[0].fork.candidate_count, 2);
    assert_eq!(
        first.all_entries()[0].candidates[0]
            .applicability
            .provider_id,
        "mock-openai"
    );
    assert_eq!(
        first.all_entries()[0].candidates[0]
            .applicability
            .account_id,
        Some("default".to_string())
    );
    assert_eq!(
        first.all_entries()[0].candidates[0].candidate.route_name,
        Some("chat".to_string())
    );
    assert_eq!(
        first.all_entries()[0].candidates[0]
            .candidate
            .fallback_enabled,
        Some(true)
    );
    assert_eq!(
        first.all_entries()[0].candidates[1]
            .capabilities
            .protocol_family,
        Some(ProtocolFamily::Claude)
    );
    assert!(
        first.all_entries()[1].candidates[0]
            .capabilities
            .supports_streaming
    );

    Ok(())
}

#[test]
fn file_configuration_can_surface_disabled_and_degraded_listing_metadata()
-> Result<(), Box<dyn std::error::Error>> {
    let configuration = ConfigurationBoundary::load_contents(VALID_TOML)?;
    let disabled_target = RoutingTarget::provider_account("mock-openai", "default");
    let degraded_target = RoutingTarget::provider_account("mock-claude", "fallback");
    let availability = RoutingAvailabilitySnapshot::new(vec![
        RoutingTargetAvailability::new(
            disabled_target,
            RoutingAvailabilityState::Unavailable {
                reason: "disabled by operator".to_string(),
            },
        ),
        RoutingTargetAvailability::new(
            degraded_target,
            RoutingAvailabilityState::Degraded {
                reason: "provider quota pressure".to_string(),
            },
        ),
    ]);
    let providers = configuration.provider_summaries();

    let registry = ModelRegistry::from_file_configuration_with_availability(
        &configuration,
        &providers,
        Some(&availability),
    );

    assert_eq!(
        model_ids_refs(registry.entries_matching(ModelListingFilter::Disabled)),
        vec!["gpt-4o-mini"]
    );
    assert_eq!(
        model_ids_refs(registry.entries_matching(ModelListingFilter::Degraded)),
        vec!["gpt-4o-mini"]
    );
    assert!(matches!(
        registry.all_entries()[0].candidates[0].listing_state,
        ModelListingState::Disabled { .. }
    ));
    assert!(matches!(
        registry.all_entries()[0].candidates[1].listing_state,
        ModelListingState::Degraded { .. }
    ));

    Ok(())
}

#[test]
fn filters_distinguish_visible_disabled_and_degraded_entries_without_execution() {
    let policy = RoutingPolicy::new(vec![
        ModelRoute::new(
            "healthy-model",
            vec![RoutingCandidate::new(RoutingTarget::provider_account(
                "healthy", "default",
            ))],
        ),
        ModelRoute::new(
            "disabled-model",
            vec![RoutingCandidate::new(RoutingTarget::provider_account(
                "disabled", "default",
            ))],
        ),
        ModelRoute::new(
            "degraded-model",
            vec![RoutingCandidate::new(RoutingTarget::provider_account(
                "degraded", "default",
            ))],
        ),
        ModelRoute::new(
            "unknown-model",
            vec![RoutingCandidate::new(RoutingTarget::provider_account(
                "missing", "default",
            ))],
        ),
    ]);
    let providers = vec![
        provider_summary("healthy", ProtocolFamily::OpenAi, true, true, "default"),
        provider_summary("disabled", ProtocolFamily::OpenAi, false, true, "default"),
        provider_summary("degraded", ProtocolFamily::Claude, true, true, "default")
            .with_degraded("provider", "quota pressure"),
    ];

    let registry = ModelRegistry::from_policy(&policy, &providers);

    assert_eq!(
        model_ids(registry.all_entries()),
        vec![
            "healthy-model",
            "disabled-model",
            "degraded-model",
            "unknown-model",
        ]
    );
    assert_eq!(
        model_ids_refs(registry.visible_entries()),
        vec!["healthy-model", "degraded-model"]
    );
    assert_eq!(
        model_ids_refs(registry.disabled_entries()),
        vec!["disabled-model", "unknown-model"]
    );
    assert_eq!(
        model_ids_refs(registry.degraded_entries()),
        vec!["degraded-model"]
    );
    assert_eq!(
        model_ids_refs(registry.entries_matching(ModelListingFilter::AllConfigured)),
        vec![
            "healthy-model",
            "disabled-model",
            "degraded-model",
            "unknown-model",
        ]
    );
    assert_eq!(
        model_ids_refs(registry.entries_matching(ModelListingFilter::Visible)),
        vec!["healthy-model", "degraded-model"]
    );
    assert!(matches!(
        registry.all_entries()[1].candidates[0].listing_state,
        ModelListingState::RoutingIneligible { .. }
    ));
    assert!(matches!(
        registry.all_entries()[3].candidates[0].listing_state,
        ModelListingState::UnknownProvider { .. }
    ));
    assert!(
        !registry.all_entries()[3].candidates[0]
            .applicability
            .provider_known
    );
    assert!(
        !registry.all_entries()[3].candidates[0]
            .applicability
            .account_known
    );
}

#[test]
fn availability_metadata_marks_disabled_and_degraded_without_selecting_route() {
    let primary = RoutingTarget::provider_account("openai", "primary");
    let fallback = RoutingTarget::provider_account("openai", "fallback");
    let policy = RoutingPolicy::new(vec![ModelRoute::new(
        "gpt-4o",
        vec![
            RoutingCandidate::new(primary.clone()),
            RoutingCandidate::new(fallback.clone()),
        ],
    )]);
    let providers = vec![ProviderSummary {
        accounts: vec![
            account_summary("primary"),
            account_summary("fallback").with_degraded("account", "reduced quota"),
        ],
        ..provider_summary("openai", ProtocolFamily::OpenAi, true, true, "primary")
    }];
    let availability = RoutingAvailabilitySnapshot::new(vec![
        RoutingTargetAvailability::new(
            primary,
            RoutingAvailabilityState::Unavailable {
                reason: "manually disabled".to_string(),
            },
        ),
        RoutingTargetAvailability::new(
            fallback,
            RoutingAvailabilityState::Degraded {
                reason: "limited capacity".to_string(),
            },
        ),
    ]);

    let registry =
        ModelRegistry::from_policy_with_availability(&policy, &providers, Some(&availability));

    assert!(matches!(
        registry.all_entries()[0].candidates[0].listing_state,
        ModelListingState::Disabled { .. }
    ));
    assert!(matches!(
        registry.all_entries()[0].candidates[1].listing_state,
        ModelListingState::Degraded { .. }
    ));
    assert_eq!(registry.visible_entries().len(), 1);
    assert_eq!(registry.disabled_entries().len(), 1);
    assert_eq!(registry.degraded_entries().len(), 1);
}

#[test]
fn unknown_account_metadata_is_preserved_for_known_provider() {
    let policy = RoutingPolicy::new(vec![ModelRoute::new(
        "gpt-4o",
        vec![RoutingCandidate::new(RoutingTarget::provider_account(
            "openai",
            "missing-account",
        ))],
    )]);
    let providers = vec![provider_summary(
        "openai",
        ProtocolFamily::OpenAi,
        true,
        true,
        "primary",
    )];

    let registry = ModelRegistry::from_policy(&policy, &providers);

    assert!(
        registry.all_entries()[0].candidates[0]
            .applicability
            .provider_known
    );
    assert!(
        !registry.all_entries()[0].candidates[0]
            .applicability
            .account_known
    );
    assert!(matches!(
        registry.all_entries()[0].candidates[0].listing_state,
        ModelListingState::UnknownAccount { .. }
    ));
    assert_eq!(
        model_ids_refs(registry.entries_matching(ModelListingFilter::Disabled)),
        vec!["gpt-4o"]
    );
}

#[test]
fn open_ai_projection_uses_visible_registry_entries_only() {
    let policy = RoutingPolicy::new(vec![
        ModelRoute::new(
            "visible-model",
            vec![RoutingCandidate::new(RoutingTarget::provider("openai"))],
        ),
        ModelRoute::new(
            "hidden-model",
            vec![RoutingCandidate::new(RoutingTarget::provider("disabled"))],
        ),
    ]);
    let providers = vec![
        provider_summary_without_account("openai", ProtocolFamily::OpenAi, true, true),
        provider_summary_without_account("disabled", ProtocolFamily::OpenAi, false, true),
    ];

    let projection = ModelRegistry::from_policy(&policy, &providers).open_ai_model_list();

    assert_eq!(projection.object, "list");
    assert_eq!(projection.data.len(), 1);
    assert_eq!(projection.data[0].id, "visible-model");
    assert_eq!(projection.data[0].object, "model");
    assert_eq!(projection.data[0].created, 0);
    assert_eq!(projection.data[0].owned_by, "openai");
}

fn provider_summary(
    provider_id: &str,
    protocol_family: ProtocolFamily,
    routing_eligible: bool,
    supports_streaming: bool,
    account_id: &str,
) -> ProviderSummary {
    ProviderSummary {
        provider_id: provider_id.to_string(),
        display_name: provider_id.to_string(),
        capabilities: vec![ProviderCapability {
            protocol_family,
            supports_streaming,
            auth_method: AuthMethodCategory::ApiKey,
            routing_eligible,
        }],
        accounts: vec![account_summary(account_id)],
        degraded_reasons: Vec::new(),
    }
}

fn provider_summary_without_account(
    provider_id: &str,
    protocol_family: ProtocolFamily,
    routing_eligible: bool,
    supports_streaming: bool,
) -> ProviderSummary {
    ProviderSummary {
        provider_id: provider_id.to_string(),
        display_name: provider_id.to_string(),
        capabilities: vec![ProviderCapability {
            protocol_family,
            supports_streaming,
            auth_method: AuthMethodCategory::ApiKey,
            routing_eligible,
        }],
        accounts: Vec::new(),
        degraded_reasons: Vec::new(),
    }
}

fn account_summary(account_id: &str) -> AccountSummary {
    AccountSummary {
        account_id: account_id.to_string(),
        display_name: account_id.to_string(),
        auth_state: AuthState::Authenticated,
        quota_state: QuotaState::Unknown,
        last_checked: None,
        degraded_reasons: Vec::new(),
    }
}

trait WithDegradedReason {
    fn with_degraded(self, component: &str, message: &str) -> Self;
}

impl WithDegradedReason for ProviderSummary {
    fn with_degraded(mut self, component: &str, message: &str) -> Self {
        self.degraded_reasons.push(DegradedReason {
            component: component.to_string(),
            message: message.to_string(),
        });
        self
    }
}

impl WithDegradedReason for AccountSummary {
    fn with_degraded(mut self, component: &str, message: &str) -> Self {
        self.degraded_reasons.push(DegradedReason {
            component: component.to_string(),
            message: message.to_string(),
        });
        self
    }
}

fn model_ids(entries: &[oxmux::ModelRegistryEntry]) -> Vec<&str> {
    entries
        .iter()
        .map(|entry| entry.identity.listed_model_id.as_str())
        .collect()
}

fn model_ids_refs(entries: Vec<&oxmux::ModelRegistryEntry>) -> Vec<&str> {
    entries
        .into_iter()
        .map(|entry| entry.identity.listed_model_id.as_str())
        .collect()
}
