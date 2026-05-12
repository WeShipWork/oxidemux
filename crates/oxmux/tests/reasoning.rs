use oxmux::*;

fn explicit_effort() -> ReasoningIntent {
    ReasoningIntent::explicit(
        ReasoningMode::Reasoning,
        ReasoningControl::Effort(ReasoningEffort::High),
    )
    .expect("valid explicit effort")
}

fn alias_budget() -> ReasoningIntent {
    ReasoningMetadataInput {
        source: ReasoningSource::Alias {
            requested_model: "smart".to_string(),
            resolved_model: "claude".to_string(),
        },
        mode: ReasoningMode::Thinking,
        effort: None,
        budget: Some(1024),
        handling: ReasoningHandlingPolicy::Permissive,
        diagnostics: ReasoningDiagnostics {
            requested_model: Some("smart".to_string()),
            resolved_model: Some("claude".to_string()),
            ..ReasoningDiagnostics::default()
        },
    }
    .normalize()
    .expect("valid alias budget")
}

#[test]
fn reasoning_validation_covers_absent_effort_budget_and_conflicts() {
    assert_eq!(ReasoningRequest::absent(), ReasoningRequest::Absent);
    assert!(ReasoningRequest::absent().validate().is_ok());

    let valid_effort = explicit_effort();
    assert_eq!(
        valid_effort.control,
        ReasoningControl::Effort(ReasoningEffort::High)
    );

    let valid_budget = ReasoningTokenBudget::new(200_000).expect("max budget is valid");
    assert_eq!(valid_budget.tokens, 200_000);

    assert!(matches!(
        ReasoningTokenBudget::new(0),
        Err(CoreError::ReasoningValidation { failure })
            if failure.code == ReasoningFailureCode::BudgetOutOfRange && failure.field == "budget"
    ));
    assert!(matches!(
        ReasoningTokenBudget::new(200_001),
        Err(CoreError::ReasoningValidation { failure })
            if failure.code == ReasoningFailureCode::BudgetOutOfRange
    ));

    let conflict = ReasoningMetadataInput {
        source: ReasoningSource::Explicit,
        mode: ReasoningMode::Reasoning,
        effort: Some(ReasoningEffort::Low),
        budget: Some(128),
        handling: ReasoningHandlingPolicy::Strict,
        diagnostics: ReasoningDiagnostics::default(),
    };
    assert!(matches!(
        conflict.normalize(),
        Err(CoreError::ReasoningValidation { failure })
            if failure.code == ReasoningFailureCode::MutuallyExclusiveControls
    ));

    let disabled_with_budget = ReasoningIntent::new(
        ReasoningSource::Explicit,
        ReasoningMode::Disabled,
        ReasoningControl::Budget(ReasoningTokenBudget::new(1).expect("valid budget")),
        ReasoningHandlingPolicy::Strict,
        ReasoningDiagnostics::default(),
    );
    assert!(matches!(
        disabled_with_budget,
        Err(CoreError::ReasoningValidation { failure })
            if failure.code == ReasoningFailureCode::InvalidModeControlCombination
    ));

    let explicit_permissive = ReasoningIntent::new(
        ReasoningSource::Explicit,
        ReasoningMode::Reasoning,
        ReasoningControl::None,
        ReasoningHandlingPolicy::Permissive,
        ReasoningDiagnostics::default(),
    );
    assert!(matches!(
        explicit_permissive,
        Err(CoreError::ReasoningValidation { failure })
            if failure.code == ReasoningFailureCode::InvalidHandlingPolicy && failure.field == "handling"
    ));
}

#[test]
fn typed_alias_metadata_preserves_identity_and_explicit_precedence() {
    let alias_intent = alias_budget();
    let alias = ModelAlias::new("smart", "claude").with_reasoning(alias_intent.clone());
    let policy = RoutingPolicy::new(vec![ModelRoute::new(
        "claude",
        vec![RoutingCandidate::new(RoutingTarget::provider("anthropic"))],
    )])
    .with_model_alias(alias);
    let registry = ModelRegistry::from_policy(&policy, &[]);
    let alias_entry = registry
        .all_entries()
        .iter()
        .find(|entry| entry.identity.listed_model_id == "smart")
        .expect("alias entry exists");

    assert_eq!(alias_entry.identity.resolved_model_id, "claude");
    assert_eq!(
        alias_entry
            .alias
            .as_ref()
            .and_then(|metadata| metadata.reasoning.as_ref()),
        Some(&alias_intent)
    );

    let ordinary = ModelAlias::new("claude (thinking 1024)", "claude");
    assert!(ordinary.reasoning.is_none());

    let explicit = explicit_effort();
    let diagnostic = ReasoningDiagnostics {
        ignored_alias: Some(Box::new(alias_intent.clone())),
        ..ReasoningDiagnostics::default()
    };
    let explicit_wins = ReasoningIntent::new(
        ReasoningSource::Explicit,
        explicit.mode,
        explicit.control,
        explicit.handling,
        diagnostic,
    )
    .expect("explicit with ignored alias diagnostic");
    assert_eq!(
        explicit_wins.diagnostics.ignored_alias.as_deref(),
        Some(&alias_intent)
    );
}

#[test]
fn reasoning_capability_outcomes_cover_supported_ignored_degraded_unknown_and_precedence() {
    let explicit = explicit_effort();
    let supported = ReasoningCapability::supported()
        .evaluate(&explicit)
        .expect("supported outcome");
    assert!(matches!(
        supported,
        ReasoningCompatibilityOutcome::Supported { .. }
    ));

    let unsupported = ReasoningCapability::Unsupported {
        reason: "no reasoning".to_string(),
    }
    .evaluate(&explicit);
    assert!(matches!(
        unsupported,
        Err(CoreError::ReasoningUnsupportedCapability { failure })
            if failure.code == ReasoningFailureCode::UnsupportedCapability
    ));

    let ignored = ReasoningCapability::Unsupported {
        reason: "no reasoning".to_string(),
    }
    .evaluate(&alias_budget())
    .expect("permissive alias can be ignored");
    assert!(matches!(
        ignored,
        ReasoningCompatibilityOutcome::Ignored { .. }
    ));

    let degraded = ReasoningCapability::Degraded {
        support: ReasoningCapabilitySupport::all(),
        reasons: vec!["reduced budget accuracy".to_string()],
    }
    .evaluate(&explicit)
    .expect("degraded outcome");
    assert!(matches!(
        degraded,
        ReasoningCompatibilityOutcome::Degraded { .. }
    ));

    let unknown_explicit = ReasoningCapability::Unknown.evaluate(&explicit);
    assert!(matches!(
        unknown_explicit,
        Err(CoreError::ReasoningUnsupportedCapability { failure })
            if failure.code == ReasoningFailureCode::UnknownCapability
    ));

    let layers = ReasoningCapabilityLayers {
        provider: Some(ReasoningCapability::Unsupported {
            reason: "provider".to_string(),
        }),
        account: Some(ReasoningCapability::Unknown),
        model: Some(ReasoningCapability::supported()),
    };
    let (layer, capability) = layers.effective();
    assert_eq!(layer, ReasoningCapabilityLayer::Model);
    assert!(matches!(capability, ReasoningCapability::Supported(_)));

    let layered_outcome = layers
        .evaluate(&explicit)
        .expect("model layer supports intent");
    assert!(matches!(
        layered_outcome,
        ReasoningCompatibilityOutcome::Supported {
            layer: ReasoningCapabilityLayer::Model,
            ..
        }
    ));

    let unsupported_provider = ReasoningCapabilityLayers {
        provider: Some(ReasoningCapability::Unsupported {
            reason: "provider".to_string(),
        }),
        account: None,
        model: None,
    }
    .evaluate(&explicit);
    assert!(matches!(
        unsupported_provider,
        Err(CoreError::ReasoningUnsupportedCapability { failure })
            if failure.code == ReasoningFailureCode::UnsupportedCapability
                && failure.layer == ReasoningCapabilityLayer::Provider
    ));
}

#[test]
fn protocol_and_provider_boundaries_preserve_reasoning_without_payload_parsing() {
    let intent = explicit_effort();
    let request = CanonicalProtocolRequest::new(
        ProtocolMetadata::open_ai(),
        "gpt",
        ProtocolPayload::opaque(
            "application/json",
            br#"{"reasoning":{"effort":"low"}}"#.to_vec(),
        ),
    )
    .expect("canonical request")
    .with_reasoning(ReasoningRequest::intent(intent.clone()).expect("reasoning request"))
    .expect("attach reasoning");
    assert_eq!(request.reasoning.as_intent(), Some(&intent));

    let translated =
        ProtocolBoundary::translate_request(request.clone(), ProtocolMetadata::claude())
            .expect("deferred translation");
    assert!(matches!(
        translated,
        ProtocolTranslationOutcome::Deferred(DeferredProtocolTranslation {
            preserved_reasoning_metadata: true,
            ..
        })
    ));

    let response = CanonicalProtocolResponse::new(
        ProtocolMetadata::open_ai(),
        ProtocolResponseStatus::success(),
        ProtocolPayload::empty(),
    )
    .expect("response");
    let harness = MockProviderHarness::new(
        "openai",
        "OpenAI",
        ProtocolFamily::OpenAi,
        AuthMethodCategory::ApiKey,
        MockProviderOutcome::Success(response),
    )
    .expect("mock provider");
    let outcome = ReasoningCapability::supported()
        .evaluate(&intent)
        .expect("supported");
    let execution_request = ProviderExecutionRequest::new("openai", None, request)
        .expect("execution request")
        .with_reasoning_outcome(outcome.clone())
        .expect("attach outcome");
    let result = harness.execute(execution_request).expect("mock execute");
    assert_eq!(result.metadata.reasoning_outcome, outcome);
}
