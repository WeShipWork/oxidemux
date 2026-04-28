use oxmux::{
    CoreError, FallbackBehavior, ModelAlias, ModelRoute, RoutingAvailabilitySnapshot,
    RoutingAvailabilityState, RoutingBoundary, RoutingCandidate, RoutingDecisionMode,
    RoutingFailure, RoutingPolicy, RoutingSelectionRequest, RoutingSkipReason, RoutingTarget,
    RoutingTargetAvailability,
};

#[test]
fn model_alias_resolution_preserves_requested_and_resolved_models() -> Result<(), CoreError> {
    let openai = RoutingTarget::provider_account("openai", "primary");
    let policy = RoutingPolicy::new(vec![ModelRoute::new(
        "gpt-4o",
        vec![RoutingCandidate::new(openai.clone())],
    )])
    .with_model_alias(ModelAlias::new("smart", "gpt-4o"));
    let availability = available_snapshot(vec![openai.clone()]);

    let selection = RoutingBoundary::select(
        &policy,
        &RoutingSelectionRequest::new("smart"),
        &availability,
    )?;

    assert_eq!(selection.requested_model, "smart");
    assert_eq!(selection.resolved_model, "gpt-4o");
    assert_eq!(selection.selected_target, openai);
    assert_eq!(selection.decision_mode, RoutingDecisionMode::Priority);
    Ok(())
}

#[test]
fn priority_order_selects_first_available_candidate() -> Result<(), CoreError> {
    let primary = RoutingTarget::provider_account("openai", "primary");
    let secondary = RoutingTarget::provider_account("anthropic", "fallback");
    let policy = policy_with_candidates(vec![primary.clone(), secondary]);
    let availability = available_snapshot(vec![primary.clone()]);

    let selection = policy.select(&RoutingSelectionRequest::new("gpt-4o"), &availability)?;

    assert_eq!(selection.selected_target, primary);
    assert_eq!(selection.decision_mode, RoutingDecisionMode::Priority);
    assert!(selection.skipped_candidates.is_empty());
    Ok(())
}

#[test]
fn fallback_enabled_skips_unavailable_candidate() -> Result<(), CoreError> {
    let primary = RoutingTarget::provider_account("openai", "primary");
    let secondary = RoutingTarget::provider_account("anthropic", "fallback");
    let policy = policy_with_candidates(vec![primary.clone(), secondary.clone()]);
    let availability = RoutingAvailabilitySnapshot::new(vec![
        RoutingTargetAvailability::new(
            primary.clone(),
            RoutingAvailabilityState::Unavailable {
                reason: "maintenance".to_string(),
            },
        ),
        RoutingTargetAvailability::new(secondary.clone(), RoutingAvailabilityState::Available),
    ]);

    let selection = policy.select(&RoutingSelectionRequest::new("gpt-4o"), &availability)?;

    assert_eq!(selection.selected_target, secondary);
    assert_eq!(selection.decision_mode, RoutingDecisionMode::Fallback);
    assert_eq!(selection.skipped_candidates.len(), 1);
    assert!(matches!(
        selection.skipped_candidates[0].reason,
        RoutingSkipReason::Unavailable { .. }
    ));
    Ok(())
}

#[test]
fn fallback_disabled_returns_structured_failure() {
    let primary = RoutingTarget::provider_account("openai", "primary");
    let secondary = RoutingTarget::provider_account("anthropic", "fallback");
    let policy = policy_with_candidates(vec![primary.clone(), secondary.clone()]);
    let availability = RoutingAvailabilitySnapshot::new(vec![
        RoutingTargetAvailability::new(
            primary.clone(),
            RoutingAvailabilityState::Exhausted {
                reason: "quota exhausted".to_string(),
            },
        ),
        RoutingTargetAvailability::new(secondary, RoutingAvailabilityState::Available),
    ]);
    let request = RoutingSelectionRequest::new("gpt-4o").with_fallback_enabled(false);

    let error = policy.select(&request, &availability);

    assert!(matches!(
        error,
        Err(CoreError::Routing {
            failure: RoutingFailure::FallbackDisabled { target, reason }
        }) if target == primary && matches!(reason, RoutingSkipReason::Exhausted { .. })
    ));
}

#[test]
fn explicit_account_target_wins_over_priority_candidates() -> Result<(), CoreError> {
    let primary = RoutingTarget::provider_account("openai", "primary");
    let explicit = RoutingTarget::provider_account("anthropic", "team");
    let policy = policy_with_candidates(vec![primary, explicit.clone()]);
    let availability = available_snapshot(vec![explicit.clone()]);
    let request = RoutingSelectionRequest::new("gpt-4o").with_explicit_target(explicit.clone());

    let selection = policy.select(&request, &availability)?;

    assert_eq!(selection.selected_target, explicit);
    assert_eq!(selection.decision_mode, RoutingDecisionMode::ExplicitTarget);
    Ok(())
}

#[test]
fn missing_explicit_target_fails_without_fallback() {
    let primary = RoutingTarget::provider_account("openai", "primary");
    let explicit = RoutingTarget::provider_account("anthropic", "team");
    let policy = policy_with_candidates(vec![primary.clone()]);
    let availability = available_snapshot(vec![primary]);
    let request = RoutingSelectionRequest::new("gpt-4o").with_explicit_target(explicit.clone());

    let error = policy.select(&request, &availability);

    assert!(matches!(
        error,
        Err(CoreError::Routing {
            failure: RoutingFailure::MissingExplicitTarget { target }
        }) if target == explicit
    ));
}

#[test]
fn exhausted_candidates_fail_structurally() {
    let primary = RoutingTarget::provider_account("openai", "primary");
    let fallback = RoutingTarget::provider_account("anthropic", "fallback");
    let policy = policy_with_candidates(vec![primary.clone(), fallback.clone()]);
    let availability = RoutingAvailabilitySnapshot::new(vec![
        RoutingTargetAvailability::new(
            primary,
            RoutingAvailabilityState::Exhausted {
                reason: "daily quota".to_string(),
            },
        ),
        RoutingTargetAvailability::new(
            fallback,
            RoutingAvailabilityState::Exhausted {
                reason: "monthly quota".to_string(),
            },
        ),
    ]);

    let error = policy.select(&RoutingSelectionRequest::new("gpt-4o"), &availability);

    assert!(matches!(
        error,
        Err(CoreError::Routing {
            failure: RoutingFailure::ExhaustedCandidates { skipped, .. }
        }) if skipped.len() == 2
    ));
}

#[test]
fn degraded_candidate_is_selected_only_when_allowed() -> Result<(), CoreError> {
    let degraded = RoutingTarget::provider_account("openai", "primary");
    let policy = policy_with_candidates(vec![degraded.clone()]);
    let availability = RoutingAvailabilitySnapshot::new(vec![RoutingTargetAvailability::new(
        degraded.clone(),
        RoutingAvailabilityState::Degraded {
            reason: "stale quota".to_string(),
        },
    )]);

    let disallowed = policy.select(&RoutingSelectionRequest::new("gpt-4o"), &availability);
    assert!(matches!(
        disallowed,
        Err(CoreError::Routing {
            failure: RoutingFailure::DegradedOnlyCandidates { .. }
        })
    ));

    let allowed = policy.select(
        &RoutingSelectionRequest::new("gpt-4o").with_degraded_allowed(true),
        &availability,
    )?;
    assert_eq!(allowed.selected_target, degraded);
    assert!(matches!(
        allowed.selected_state,
        RoutingAvailabilityState::Degraded { .. }
    ));
    Ok(())
}

#[test]
fn healthy_fallback_wins_over_degraded_higher_priority_candidate() -> Result<(), CoreError> {
    let degraded = RoutingTarget::provider_account("openai", "primary");
    let healthy = RoutingTarget::provider_account("anthropic", "fallback");
    let policy = policy_with_candidates(vec![degraded.clone(), healthy.clone()]);
    let availability = RoutingAvailabilitySnapshot::new(vec![
        RoutingTargetAvailability::new(
            degraded,
            RoutingAvailabilityState::Degraded {
                reason: "latency high".to_string(),
            },
        ),
        RoutingTargetAvailability::new(healthy.clone(), RoutingAvailabilityState::Available),
    ]);

    let selection = policy.select(
        &RoutingSelectionRequest::new("gpt-4o").with_degraded_allowed(true),
        &availability,
    )?;

    assert_eq!(selection.selected_target, healthy);
    assert_eq!(selection.decision_mode, RoutingDecisionMode::Fallback);
    assert!(matches!(
        selection.skipped_candidates[0].reason,
        RoutingSkipReason::DegradedDeferred { .. }
    ));
    Ok(())
}

#[test]
fn no_route_for_model_fails_structurally() {
    let policy = RoutingPolicy::new(Vec::new());
    let availability = RoutingAvailabilitySnapshot::new(Vec::new());

    let error = policy.select(&RoutingSelectionRequest::new("unknown"), &availability);

    assert!(matches!(
        error,
        Err(CoreError::Routing {
            failure: RoutingFailure::NoRoute {
                requested_model,
                resolved_model,
            }
        }) if requested_model == "unknown" && resolved_model == "unknown"
    ));
}

#[test]
fn invalid_policy_returns_structured_core_error() {
    let target = RoutingTarget::provider_account("openai", "primary");
    let policy = RoutingPolicy::new(vec![ModelRoute::new(
        " ",
        vec![RoutingCandidate::new(target.clone())],
    )]);
    let availability = available_snapshot(vec![target]);

    let error = policy.select(&RoutingSelectionRequest::new("gpt-4o"), &availability);

    assert!(matches!(
        error,
        Err(CoreError::Routing {
            failure: RoutingFailure::InvalidPolicy {
                field: "routes.resolved_model",
                ..
            }
        })
    ));
}

fn policy_with_candidates(targets: Vec<RoutingTarget>) -> RoutingPolicy {
    RoutingPolicy::new(vec![ModelRoute::new(
        "gpt-4o",
        targets.into_iter().map(RoutingCandidate::new).collect(),
    )])
    .with_fallback(FallbackBehavior::default())
}

fn available_snapshot(targets: Vec<RoutingTarget>) -> RoutingAvailabilitySnapshot {
    RoutingAvailabilitySnapshot::new(
        targets
            .into_iter()
            .map(|target| {
                RoutingTargetAvailability::new(target, RoutingAvailabilityState::Available)
            })
            .collect(),
    )
}
