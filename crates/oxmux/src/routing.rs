//! Subscription-aware routing policy and selection contracts.
//!
//! Routing state resolves requested models, evaluates provider/account targets,
//! records skipped candidates, and reports fallback or degraded-selection reasons
//! for consumers of the headless core.

use crate::CoreError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Facade for selecting routing targets from policy and availability state.
pub struct RoutingBoundary;

impl RoutingBoundary {
    /// Selects a routing target from policy, request, and availability state.
    pub fn select(
        policy: &RoutingPolicy,
        request: &RoutingSelectionRequest,
        availability: &RoutingAvailabilitySnapshot,
    ) -> Result<RoutingSelectionResult, CoreError> {
        policy.select(request, availability)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Model alias, route, fallback, and default-target policy for core selection.
pub struct RoutingPolicy {
    /// Aliases used to resolve requested model names.
    pub model_aliases: Vec<ModelAlias>,
    /// Configured model routes in this policy.
    pub routes: Vec<ModelRoute>,
    /// Default fallback behavior for the policy.
    pub fallback: FallbackBehavior,
    /// Optional explicit target used when requests omit one.
    pub default_target: Option<RoutingTarget>,
}

impl RoutingPolicy {
    /// Creates a routing policy with the supplied model routes and default fallback behavior.
    pub fn new(routes: Vec<ModelRoute>) -> Self {
        Self {
            model_aliases: Vec::new(),
            routes,
            fallback: FallbackBehavior::default(),
            default_target: None,
        }
    }

    /// Adds one model alias to this routing policy.
    pub fn with_model_alias(mut self, alias: ModelAlias) -> Self {
        self.model_aliases.push(alias);
        self
    }

    /// Replaces the default fallback behavior for this routing policy.
    pub fn with_fallback(mut self, fallback: FallbackBehavior) -> Self {
        self.fallback = fallback;
        self
    }

    /// Sets the target used when no model route matches the request.
    pub fn with_default_target(mut self, target: RoutingTarget) -> Self {
        self.default_target = Some(target);
        self
    }

    /// Validates this value and returns a structured core error on failure.
    pub fn validate(&self) -> Result<(), CoreError> {
        if let Some(target) = &self.default_target {
            target.validate("default_target")?;
        }

        for alias in &self.model_aliases {
            alias.validate()?;
        }

        for (index, alias) in self.model_aliases.iter().enumerate() {
            if self
                .model_aliases
                .iter()
                .skip(index + 1)
                .any(|candidate| candidate.requested_model == alias.requested_model)
            {
                return Err(invalid_policy(
                    "model_aliases",
                    format!(
                        "model alias {} is defined more than once",
                        alias.requested_model
                    ),
                ));
            }
        }

        for route in &self.routes {
            route.validate()?;
        }

        for (index, route) in self.routes.iter().enumerate() {
            if self
                .routes
                .iter()
                .skip(index + 1)
                .any(|candidate| candidate.resolved_model == route.resolved_model)
            {
                return Err(invalid_policy(
                    "routes",
                    format!(
                        "route for model {} is defined more than once",
                        route.resolved_model
                    ),
                ));
            }
        }

        Ok(())
    }

    /// Selects a routing target from policy, request, and availability state.
    pub fn select(
        &self,
        request: &RoutingSelectionRequest,
        availability: &RoutingAvailabilitySnapshot,
    ) -> Result<RoutingSelectionResult, CoreError> {
        self.validate()?;
        request.validate()?;
        availability.validate()?;

        let requested_model = request.requested_model.clone();
        let resolved_model = self.resolve_model(&requested_model);

        if let Some(target) = request
            .explicit_target
            .as_ref()
            .or(self.default_target.as_ref())
        {
            return self.select_explicit_target(
                target,
                &requested_model,
                &resolved_model,
                request
                    .allow_degraded
                    .unwrap_or(self.fallback.allow_degraded),
                availability,
            );
        }

        let Some(route) = self
            .routes
            .iter()
            .find(|route| route.resolved_model == resolved_model)
        else {
            return Err(CoreError::Routing {
                failure: RoutingFailure::NoRoute {
                    requested_model,
                    resolved_model,
                },
            });
        };

        if route.candidates.is_empty() {
            return Err(CoreError::Routing {
                failure: RoutingFailure::NoRoute {
                    requested_model,
                    resolved_model,
                },
            });
        }

        let fallback_enabled = request
            .fallback_enabled
            .unwrap_or(self.fallback.fallback_enabled);
        let allow_degraded = request
            .allow_degraded
            .unwrap_or(self.fallback.allow_degraded);

        evaluate_candidates(
            &requested_model,
            &resolved_model,
            &route.candidates,
            fallback_enabled,
            allow_degraded,
            availability,
        )
    }

    fn resolve_model(&self, requested_model: &str) -> String {
        self.model_aliases
            .iter()
            .find(|alias| alias.requested_model == requested_model)
            .map(|alias| alias.resolved_model.clone())
            .unwrap_or_else(|| requested_model.to_string())
    }

    fn select_explicit_target(
        &self,
        target: &RoutingTarget,
        requested_model: &str,
        resolved_model: &str,
        allow_degraded: bool,
        availability: &RoutingAvailabilitySnapshot,
    ) -> Result<RoutingSelectionResult, CoreError> {
        let Some(state) = availability.state_for(target) else {
            return Err(CoreError::Routing {
                failure: RoutingFailure::MissingExplicitTarget {
                    target: target.clone(),
                },
            });
        };

        match state {
            RoutingAvailabilityState::Available => Ok(selection(
                requested_model,
                resolved_model,
                target,
                state,
                RoutingDecisionMode::ExplicitTarget,
                Vec::new(),
            )),
            RoutingAvailabilityState::Degraded { .. } if allow_degraded => Ok(selection(
                requested_model,
                resolved_model,
                target,
                state,
                RoutingDecisionMode::ExplicitTarget,
                Vec::new(),
            )),
            RoutingAvailabilityState::Degraded { reason } => Err(CoreError::Routing {
                failure: RoutingFailure::DegradedOnlyCandidates {
                    requested_model: requested_model.to_string(),
                    resolved_model: resolved_model.to_string(),
                    skipped: vec![SkippedRoutingCandidate {
                        target: target.clone(),
                        reason: RoutingSkipReason::DegradedDisallowed {
                            reason: reason.clone(),
                        },
                    }],
                },
            }),
            RoutingAvailabilityState::Exhausted { reason } => Err(CoreError::Routing {
                failure: RoutingFailure::ExhaustedCandidates {
                    requested_model: requested_model.to_string(),
                    resolved_model: resolved_model.to_string(),
                    skipped: vec![SkippedRoutingCandidate {
                        target: target.clone(),
                        reason: RoutingSkipReason::Exhausted {
                            reason: reason.clone(),
                        },
                    }],
                },
            }),
            RoutingAvailabilityState::Unavailable { reason } => Err(CoreError::Routing {
                failure: RoutingFailure::NoAvailableCandidates {
                    requested_model: requested_model.to_string(),
                    resolved_model: resolved_model.to_string(),
                    skipped: vec![SkippedRoutingCandidate {
                        target: target.clone(),
                        reason: RoutingSkipReason::Unavailable {
                            reason: reason.clone(),
                        },
                    }],
                },
            }),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Mapping from requested model name to the configured resolved model.
pub struct ModelAlias {
    /// Model name requested by the caller before alias resolution.
    pub requested_model: String,
    /// Model name after applying configured aliases.
    pub resolved_model: String,
}

impl ModelAlias {
    /// Creates a model alias from a requested model name to the resolved model name.
    pub fn new(requested_model: impl Into<String>, resolved_model: impl Into<String>) -> Self {
        Self {
            requested_model: requested_model.into(),
            resolved_model: resolved_model.into(),
        }
    }

    fn validate(&self) -> Result<(), CoreError> {
        validate_required_text("model_aliases.requested_model", &self.requested_model)?;
        validate_required_text("model_aliases.resolved_model", &self.resolved_model)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Ordered candidate list for one resolved model.
pub struct ModelRoute {
    /// Model name after applying configured aliases.
    pub resolved_model: String,
    /// Ordered routing candidates for this model or group.
    pub candidates: Vec<RoutingCandidate>,
}

impl ModelRoute {
    /// Creates a model route for a resolved model and its ordered target candidates.
    pub fn new(resolved_model: impl Into<String>, candidates: Vec<RoutingCandidate>) -> Self {
        Self {
            resolved_model: resolved_model.into(),
            candidates,
        }
    }

    fn validate(&self) -> Result<(), CoreError> {
        validate_required_text("routes.resolved_model", &self.resolved_model)?;

        for candidate in &self.candidates {
            candidate.validate()?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Candidate provider/account target in a route.
pub struct RoutingCandidate {
    /// Provider/account target associated with routing.
    pub target: RoutingTarget,
}

impl RoutingCandidate {
    /// Creates a routing candidate for a provider or provider account target.
    pub fn new(target: RoutingTarget) -> Self {
        Self { target }
    }

    fn validate(&self) -> Result<(), CoreError> {
        self.target.validate("routes.candidates.target")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Provider and optional account selected by routing policy.
pub struct RoutingTarget {
    /// Provider identifier used by routing, execution, and management state.
    pub provider_id: String,
    /// Optional account identifier scoped to the provider.
    pub account_id: Option<String>,
}

impl RoutingTarget {
    /// Creates a routing target for an entire provider.
    pub fn provider(provider_id: impl Into<String>) -> Self {
        Self {
            provider_id: provider_id.into(),
            account_id: None,
        }
    }

    /// Creates a routing target for a specific provider account.
    pub fn provider_account(provider_id: impl Into<String>, account_id: impl Into<String>) -> Self {
        Self {
            provider_id: provider_id.into(),
            account_id: Some(account_id.into()),
        }
    }

    fn validate(&self, field: &'static str) -> Result<(), CoreError> {
        validate_required_text(field, &self.provider_id)?;

        if let Some(account_id) = &self.account_id {
            validate_required_text(field, account_id)?;
        }

        Ok(())
    }

    fn validate_request(&self, field: &'static str) -> Result<(), CoreError> {
        validate_required_request_text(field, &self.provider_id)?;

        if let Some(account_id) = &self.account_id {
            validate_required_request_text(field, account_id)?;
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Default fallback and degraded-candidate behavior for routing decisions.
pub struct FallbackBehavior {
    /// Whether routing may continue after a skipped candidate.
    pub fallback_enabled: bool,
    /// Whether degraded targets may be selected.
    pub allow_degraded: bool,
}

impl FallbackBehavior {
    /// Creates explicit fallback behavior for routing candidate evaluation.
    pub const fn new(fallback_enabled: bool, allow_degraded: bool) -> Self {
        Self {
            fallback_enabled,
            allow_degraded,
        }
    }

    /// Returns fallback behavior that stops after the first skipped candidate.
    pub const fn disabled() -> Self {
        Self::new(false, false)
    }
}

impl Default for FallbackBehavior {
    fn default() -> Self {
        Self::new(true, false)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Point-in-time availability states for routing targets.
pub struct RoutingAvailabilitySnapshot {
    /// Availability entries keyed by routing target.
    pub targets: Vec<RoutingTargetAvailability>,
}

impl RoutingAvailabilitySnapshot {
    /// Creates an availability snapshot for the targets known to routing.
    pub fn new(targets: Vec<RoutingTargetAvailability>) -> Self {
        Self { targets }
    }

    /// Returns the availability state for a routing target, when present.
    pub fn state_for(&self, target: &RoutingTarget) -> Option<&RoutingAvailabilityState> {
        self.targets
            .iter()
            .find(|availability| availability.target == *target)
            .map(|availability| &availability.state)
    }

    /// Validates this value and returns a structured core error on failure.
    pub fn validate(&self) -> Result<(), CoreError> {
        for availability in &self.targets {
            availability.validate()?;
        }

        for (index, availability) in self.targets.iter().enumerate() {
            if self
                .targets
                .iter()
                .skip(index + 1)
                .any(|candidate| candidate.target == availability.target)
            {
                return Err(invalid_policy(
                    "availability.targets",
                    format!(
                        "availability for provider {} is defined more than once",
                        availability.target.provider_id
                    ),
                ));
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Availability state for one provider/account target.
pub struct RoutingTargetAvailability {
    /// Provider/account target associated with routing.
    pub target: RoutingTarget,
    /// Availability state for this target.
    pub state: RoutingAvailabilityState,
}

impl RoutingTargetAvailability {
    /// Creates a routing target availability entry for one provider or account target.
    pub fn new(target: RoutingTarget, state: RoutingAvailabilityState) -> Self {
        Self { target, state }
    }

    fn validate(&self) -> Result<(), CoreError> {
        self.target.validate("availability.targets.target")?;
        self.state.validate()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Operational state used when selecting a routing target.
pub enum RoutingAvailabilityState {
    /// Target is available for selection.
    Available,
    /// Target cannot currently be used.
    Unavailable {
        /// Human-readable reason for this state.
        reason: String,
    },
    /// Target quota or capacity is exhausted.
    Exhausted {
        /// Human-readable reason for this state.
        reason: String,
    },
    /// Target is usable only as a degraded routing candidate.
    Degraded {
        /// Human-readable reason for this state.
        reason: String,
    },
}

impl RoutingAvailabilityState {
    fn validate(&self) -> Result<(), CoreError> {
        match self {
            Self::Available => Ok(()),
            Self::Unavailable { reason } => validate_required_text("availability.reason", reason),
            Self::Exhausted { reason } => validate_required_text("availability.reason", reason),
            Self::Degraded { reason } => validate_required_text("availability.reason", reason),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Model routing request with optional explicit target and fallback overrides.
pub struct RoutingSelectionRequest {
    /// Model name requested by the caller before alias resolution.
    pub requested_model: String,
    /// Provider/account target requested by the caller instead of policy selection.
    pub explicit_target: Option<RoutingTarget>,
    /// Whether routing may continue after a skipped candidate.
    pub fallback_enabled: Option<bool>,
    /// Whether degraded targets may be selected.
    pub allow_degraded: Option<bool>,
}

impl RoutingSelectionRequest {
    /// Creates a selection request for a model before aliases and policy defaults are applied.
    pub fn new(requested_model: impl Into<String>) -> Self {
        Self {
            requested_model: requested_model.into(),
            explicit_target: None,
            fallback_enabled: None,
            allow_degraded: None,
        }
    }

    /// Sets the provider/account target that should bypass policy selection.
    pub fn with_explicit_target(mut self, target: RoutingTarget) -> Self {
        self.explicit_target = Some(target);
        self
    }

    /// Overrides whether routing may continue after a skipped candidate.
    pub fn with_fallback_enabled(mut self, fallback_enabled: bool) -> Self {
        self.fallback_enabled = Some(fallback_enabled);
        self
    }

    /// Overrides whether degraded routing candidates may be selected.
    pub fn with_degraded_allowed(mut self, allow_degraded: bool) -> Self {
        self.allow_degraded = Some(allow_degraded);
        self
    }

    fn validate(&self) -> Result<(), CoreError> {
        validate_required_request_text("requested_model", &self.requested_model)?;

        if let Some(target) = &self.explicit_target {
            target.validate_request("explicit_target")?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Selected routing target plus resolved model and skipped candidates.
pub struct RoutingSelectionResult {
    /// Model name requested by the caller before alias resolution.
    pub requested_model: String,
    /// Model name after applying configured aliases.
    pub resolved_model: String,
    /// Provider/account target selected by routing.
    pub selected_target: RoutingTarget,
    /// Availability state observed for the selected target.
    pub selected_state: RoutingAvailabilityState,
    /// Selection mode that explains why this route was chosen.
    pub decision_mode: RoutingDecisionMode,
    /// Candidates skipped while choosing the selected target.
    pub skipped_candidates: Vec<SkippedRoutingCandidate>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Mode that explains why a target was selected.
pub enum RoutingDecisionMode {
    /// Selection used an explicit or default target.
    ExplicitTarget,
    /// Selection used the first available route candidate.
    Priority,
    /// Selection used a later fallback candidate.
    Fallback,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Candidate skipped during route evaluation and its reason.
pub struct SkippedRoutingCandidate {
    /// Provider/account target associated with routing.
    pub target: RoutingTarget,
    /// Human-readable reason for this state.
    pub reason: RoutingSkipReason,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Reason a routing candidate was not selected.
pub enum RoutingSkipReason {
    /// No availability entry existed for the candidate.
    MissingAvailability,
    /// Target cannot currently be used.
    Unavailable {
        /// Human-readable reason for this state.
        reason: String,
    },
    /// Target quota or capacity is exhausted.
    Exhausted {
        /// Human-readable reason for this state.
        reason: String,
    },
    /// Degraded candidate was skipped because degraded routing is disabled.
    DegradedDisallowed {
        /// Human-readable reason for this state.
        reason: String,
    },
    /// Degraded candidate was held while healthier fallbacks were evaluated.
    DegradedDeferred {
        /// Human-readable reason for this state.
        reason: String,
    },
}

impl RoutingSkipReason {
    /// Returns a human-readable message for this diagnostic.
    pub fn message(&self) -> String {
        match self {
            Self::MissingAvailability => "availability was not supplied".to_string(),
            Self::Unavailable { reason } => format!("unavailable: {reason}"),
            Self::Exhausted { reason } => format!("exhausted: {reason}"),
            Self::DegradedDisallowed { reason } => {
                format!("degraded routing is not allowed: {reason}")
            }
            Self::DegradedDeferred { reason } => {
                format!(
                    "degraded candidate deferred while checking for a healthier route: {reason}"
                )
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Matchable failure for invalid policy, invalid request, or unavailable routes.
pub enum RoutingFailure {
    /// Routing policy validation failed.
    InvalidPolicy {
        /// Field path associated with this validation diagnostic.
        field: &'static str,
        /// Human-readable diagnostic message.
        message: String,
    },
    /// Routing request validation failed.
    InvalidRequest {
        /// Field path associated with this validation diagnostic.
        field: &'static str,
        /// Human-readable diagnostic message.
        message: String,
    },
    /// No route exists for the resolved model.
    NoRoute {
        /// Model name requested by the caller before alias resolution.
        requested_model: String,
        /// Model name after applying configured aliases.
        resolved_model: String,
    },
    /// Explicit target was absent from availability state.
    MissingExplicitTarget {
        /// Provider/account target associated with routing.
        target: RoutingTarget,
    },
    /// Fallback was disabled after a candidate was skipped.
    FallbackDisabled {
        /// Provider/account target associated with routing.
        target: RoutingTarget,
        /// Human-readable reason for this state.
        reason: RoutingSkipReason,
    },
    /// All candidates were exhausted.
    ExhaustedCandidates {
        /// Model name requested by the caller before alias resolution.
        requested_model: String,
        /// Model name after applying configured aliases.
        resolved_model: String,
        /// Candidates skipped while evaluating this failure.
        skipped: Vec<SkippedRoutingCandidate>,
    },
    /// Only degraded candidates remained and could not be selected.
    DegradedOnlyCandidates {
        /// Model name requested by the caller before alias resolution.
        requested_model: String,
        /// Model name after applying configured aliases.
        resolved_model: String,
        /// Candidates skipped while evaluating this failure.
        skipped: Vec<SkippedRoutingCandidate>,
    },
    /// No available candidate remained.
    NoAvailableCandidates {
        /// Model name requested by the caller before alias resolution.
        requested_model: String,
        /// Model name after applying configured aliases.
        resolved_model: String,
        /// Candidates skipped while evaluating this failure.
        skipped: Vec<SkippedRoutingCandidate>,
    },
}

impl RoutingFailure {
    /// Returns a human-readable message for this diagnostic.
    pub fn message(&self) -> String {
        match self {
            Self::InvalidPolicy { field, message } => {
                format!("routing policy field {field} is invalid: {message}")
            }
            Self::InvalidRequest { field, message } => {
                format!("routing request field {field} is invalid: {message}")
            }
            Self::NoRoute {
                requested_model,
                resolved_model,
            } => format!(
                "no route is configured for requested model {requested_model} resolved as {resolved_model}"
            ),
            Self::MissingExplicitTarget { target } => {
                format!(
                    "explicit target {} is missing from availability",
                    target.label()
                )
            }
            Self::FallbackDisabled { target, reason } => format!(
                "fallback is disabled after target {} failed selection: {}",
                target.label(),
                reason.message()
            ),
            Self::ExhaustedCandidates {
                resolved_model,
                skipped,
                ..
            } => format!(
                "all routing candidates for model {resolved_model} are exhausted ({} skipped)",
                skipped.len()
            ),
            Self::DegradedOnlyCandidates {
                resolved_model,
                skipped,
                ..
            } => format!(
                "only degraded routing candidates remain for model {resolved_model} ({} skipped)",
                skipped.len()
            ),
            Self::NoAvailableCandidates {
                resolved_model,
                skipped,
                ..
            } => format!(
                "no available routing candidates remain for model {resolved_model} ({} skipped)",
                skipped.len()
            ),
        }
    }
}

impl RoutingTarget {
    fn label(&self) -> String {
        match &self.account_id {
            Some(account_id) => format!("{}/{}", self.provider_id, account_id),
            None => self.provider_id.clone(),
        }
    }
}

fn evaluate_candidates(
    requested_model: &str,
    resolved_model: &str,
    candidates: &[RoutingCandidate],
    fallback_enabled: bool,
    allow_degraded: bool,
    availability: &RoutingAvailabilitySnapshot,
) -> Result<RoutingSelectionResult, CoreError> {
    let mut skipped = Vec::new();
    let mut deferred_degraded = Vec::new();

    for (index, candidate) in candidates.iter().enumerate() {
        let target = &candidate.target;
        let decision_mode = if index == 0 {
            RoutingDecisionMode::Priority
        } else {
            RoutingDecisionMode::Fallback
        };
        let skip_reason = match availability.state_for(target) {
            Some(RoutingAvailabilityState::Available) => {
                return Ok(selection(
                    requested_model,
                    resolved_model,
                    target,
                    &RoutingAvailabilityState::Available,
                    decision_mode,
                    skipped,
                ));
            }
            Some(RoutingAvailabilityState::Degraded { reason }) => {
                if allow_degraded && !fallback_enabled {
                    return Ok(selection(
                        requested_model,
                        resolved_model,
                        target,
                        &RoutingAvailabilityState::Degraded {
                            reason: reason.clone(),
                        },
                        decision_mode,
                        skipped,
                    ));
                }

                let skipped_candidate = SkippedRoutingCandidate {
                    target: target.clone(),
                    reason: if allow_degraded {
                        RoutingSkipReason::DegradedDeferred {
                            reason: reason.clone(),
                        }
                    } else {
                        RoutingSkipReason::DegradedDisallowed {
                            reason: reason.clone(),
                        }
                    },
                };

                if allow_degraded {
                    deferred_degraded.push((target.clone(), reason.clone(), decision_mode));
                }

                skipped_candidate
            }
            Some(RoutingAvailabilityState::Unavailable { reason }) => SkippedRoutingCandidate {
                target: target.clone(),
                reason: RoutingSkipReason::Unavailable {
                    reason: reason.clone(),
                },
            },
            Some(RoutingAvailabilityState::Exhausted { reason }) => SkippedRoutingCandidate {
                target: target.clone(),
                reason: RoutingSkipReason::Exhausted {
                    reason: reason.clone(),
                },
            },
            None => SkippedRoutingCandidate {
                target: target.clone(),
                reason: RoutingSkipReason::MissingAvailability,
            },
        };

        if !fallback_enabled {
            return Err(CoreError::Routing {
                failure: RoutingFailure::FallbackDisabled {
                    target: target.clone(),
                    reason: skip_reason.reason,
                },
            });
        }

        skipped.push(skip_reason);
    }

    if allow_degraded && let Some((target, reason, decision_mode)) = deferred_degraded.first() {
        let skipped = skipped
            .into_iter()
            .filter(|candidate| candidate.target != *target)
            .collect();

        return Ok(selection(
            requested_model,
            resolved_model,
            target,
            &RoutingAvailabilityState::Degraded {
                reason: reason.clone(),
            },
            *decision_mode,
            skipped,
        ));
    }

    if skipped.iter().all(|candidate| {
        matches!(
            candidate.reason,
            RoutingSkipReason::DegradedDeferred { .. }
                | RoutingSkipReason::DegradedDisallowed { .. }
        )
    }) {
        return Err(CoreError::Routing {
            failure: RoutingFailure::DegradedOnlyCandidates {
                requested_model: requested_model.to_string(),
                resolved_model: resolved_model.to_string(),
                skipped,
            },
        });
    }

    if skipped
        .iter()
        .all(|candidate| matches!(candidate.reason, RoutingSkipReason::Exhausted { .. }))
    {
        return Err(CoreError::Routing {
            failure: RoutingFailure::ExhaustedCandidates {
                requested_model: requested_model.to_string(),
                resolved_model: resolved_model.to_string(),
                skipped,
            },
        });
    }

    Err(CoreError::Routing {
        failure: RoutingFailure::NoAvailableCandidates {
            requested_model: requested_model.to_string(),
            resolved_model: resolved_model.to_string(),
            skipped,
        },
    })
}

fn selection(
    requested_model: &str,
    resolved_model: &str,
    target: &RoutingTarget,
    state: &RoutingAvailabilityState,
    decision_mode: RoutingDecisionMode,
    skipped_candidates: Vec<SkippedRoutingCandidate>,
) -> RoutingSelectionResult {
    RoutingSelectionResult {
        requested_model: requested_model.to_string(),
        resolved_model: resolved_model.to_string(),
        selected_target: target.clone(),
        selected_state: state.clone(),
        decision_mode,
        skipped_candidates,
    }
}

fn validate_required_text(field: &'static str, value: &str) -> Result<(), CoreError> {
    if value.trim().is_empty() {
        return Err(invalid_policy(field, format!("{field} must not be empty")));
    }

    Ok(())
}

fn validate_required_request_text(field: &'static str, value: &str) -> Result<(), CoreError> {
    if value.trim().is_empty() {
        return Err(invalid_request(field, format!("{field} must not be empty")));
    }

    Ok(())
}

fn invalid_request(field: &'static str, message: String) -> CoreError {
    CoreError::Routing {
        failure: RoutingFailure::InvalidRequest { field, message },
    }
}

fn invalid_policy(field: &'static str, message: String) -> CoreError {
    CoreError::Routing {
        failure: RoutingFailure::InvalidPolicy { field, message },
    }
}
