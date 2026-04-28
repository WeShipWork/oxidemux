use crate::CoreError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RoutingBoundary;

impl RoutingBoundary {
    pub fn select(
        policy: &RoutingPolicy,
        request: &RoutingSelectionRequest,
        availability: &RoutingAvailabilitySnapshot,
    ) -> Result<RoutingSelectionResult, CoreError> {
        policy.select(request, availability)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoutingPolicy {
    pub model_aliases: Vec<ModelAlias>,
    pub routes: Vec<ModelRoute>,
    pub fallback: FallbackBehavior,
    pub default_target: Option<RoutingTarget>,
}

impl RoutingPolicy {
    pub fn new(routes: Vec<ModelRoute>) -> Self {
        Self {
            model_aliases: Vec::new(),
            routes,
            fallback: FallbackBehavior::default(),
            default_target: None,
        }
    }

    pub fn with_model_alias(mut self, alias: ModelAlias) -> Self {
        self.model_aliases.push(alias);
        self
    }

    pub fn with_fallback(mut self, fallback: FallbackBehavior) -> Self {
        self.fallback = fallback;
        self
    }

    pub fn with_default_target(mut self, target: RoutingTarget) -> Self {
        self.default_target = Some(target);
        self
    }

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
pub struct ModelAlias {
    pub requested_model: String,
    pub resolved_model: String,
}

impl ModelAlias {
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
pub struct ModelRoute {
    pub resolved_model: String,
    pub candidates: Vec<RoutingCandidate>,
}

impl ModelRoute {
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
pub struct RoutingCandidate {
    pub target: RoutingTarget,
}

impl RoutingCandidate {
    pub fn new(target: RoutingTarget) -> Self {
        Self { target }
    }

    fn validate(&self) -> Result<(), CoreError> {
        self.target.validate("routes.candidates.target")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoutingTarget {
    pub provider_id: String,
    pub account_id: Option<String>,
}

impl RoutingTarget {
    pub fn provider(provider_id: impl Into<String>) -> Self {
        Self {
            provider_id: provider_id.into(),
            account_id: None,
        }
    }

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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FallbackBehavior {
    pub fallback_enabled: bool,
    pub allow_degraded: bool,
}

impl FallbackBehavior {
    pub const fn new(fallback_enabled: bool, allow_degraded: bool) -> Self {
        Self {
            fallback_enabled,
            allow_degraded,
        }
    }

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
pub struct RoutingAvailabilitySnapshot {
    pub targets: Vec<RoutingTargetAvailability>,
}

impl RoutingAvailabilitySnapshot {
    pub fn new(targets: Vec<RoutingTargetAvailability>) -> Self {
        Self { targets }
    }

    pub fn state_for(&self, target: &RoutingTarget) -> Option<&RoutingAvailabilityState> {
        self.targets
            .iter()
            .find(|availability| availability.target == *target)
            .map(|availability| &availability.state)
    }

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
pub struct RoutingTargetAvailability {
    pub target: RoutingTarget,
    pub state: RoutingAvailabilityState,
}

impl RoutingTargetAvailability {
    pub fn new(target: RoutingTarget, state: RoutingAvailabilityState) -> Self {
        Self { target, state }
    }

    fn validate(&self) -> Result<(), CoreError> {
        self.target.validate("availability.targets.target")?;
        self.state.validate()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RoutingAvailabilityState {
    Available,
    Unavailable { reason: String },
    Exhausted { reason: String },
    Degraded { reason: String },
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
pub struct RoutingSelectionRequest {
    pub requested_model: String,
    pub explicit_target: Option<RoutingTarget>,
    pub fallback_enabled: Option<bool>,
    pub allow_degraded: Option<bool>,
}

impl RoutingSelectionRequest {
    pub fn new(requested_model: impl Into<String>) -> Self {
        Self {
            requested_model: requested_model.into(),
            explicit_target: None,
            fallback_enabled: None,
            allow_degraded: None,
        }
    }

    pub fn with_explicit_target(mut self, target: RoutingTarget) -> Self {
        self.explicit_target = Some(target);
        self
    }

    pub fn with_fallback_enabled(mut self, fallback_enabled: bool) -> Self {
        self.fallback_enabled = Some(fallback_enabled);
        self
    }

    pub fn with_degraded_allowed(mut self, allow_degraded: bool) -> Self {
        self.allow_degraded = Some(allow_degraded);
        self
    }

    fn validate(&self) -> Result<(), CoreError> {
        validate_required_text("requested_model", &self.requested_model)?;

        if let Some(target) = &self.explicit_target {
            target.validate("explicit_target")?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoutingSelectionResult {
    pub requested_model: String,
    pub resolved_model: String,
    pub selected_target: RoutingTarget,
    pub selected_state: RoutingAvailabilityState,
    pub decision_mode: RoutingDecisionMode,
    pub skipped_candidates: Vec<SkippedRoutingCandidate>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoutingDecisionMode {
    ExplicitTarget,
    Priority,
    Fallback,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkippedRoutingCandidate {
    pub target: RoutingTarget,
    pub reason: RoutingSkipReason,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RoutingSkipReason {
    MissingAvailability,
    Unavailable { reason: String },
    Exhausted { reason: String },
    DegradedDisallowed { reason: String },
    DegradedDeferred { reason: String },
}

impl RoutingSkipReason {
    pub fn message(&self) -> String {
        match self {
            Self::MissingAvailability => "availability was not supplied".to_string(),
            Self::Unavailable { reason } => format!("unavailable: {reason}"),
            Self::Exhausted { reason } => format!("exhausted: {reason}"),
            Self::DegradedDisallowed { reason } => {
                format!("degraded routing is not allowed: {reason}")
            }
            Self::DegradedDeferred { reason } => {
                format!("degraded candidate deferred while healthy fallback is available: {reason}")
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RoutingFailure {
    InvalidPolicy {
        field: &'static str,
        message: String,
    },
    InvalidRequest {
        field: &'static str,
        message: String,
    },
    NoRoute {
        requested_model: String,
        resolved_model: String,
    },
    MissingExplicitTarget {
        target: RoutingTarget,
    },
    FallbackDisabled {
        target: RoutingTarget,
        reason: RoutingSkipReason,
    },
    ExhaustedCandidates {
        requested_model: String,
        resolved_model: String,
        skipped: Vec<SkippedRoutingCandidate>,
    },
    DegradedOnlyCandidates {
        requested_model: String,
        resolved_model: String,
        skipped: Vec<SkippedRoutingCandidate>,
    },
    NoAvailableCandidates {
        requested_model: String,
        resolved_model: String,
        skipped: Vec<SkippedRoutingCandidate>,
    },
}

impl RoutingFailure {
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
                let skipped_candidate = SkippedRoutingCandidate {
                    target: target.clone(),
                    reason: RoutingSkipReason::DegradedDeferred {
                        reason: reason.clone(),
                    },
                };
                deferred_degraded.push((target.clone(), reason.clone(), decision_mode));
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

    if skipped.iter().any(|candidate| {
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

fn invalid_policy(field: &'static str, message: String) -> CoreError {
    CoreError::Routing {
        failure: RoutingFailure::InvalidPolicy { field, message },
    }
}
