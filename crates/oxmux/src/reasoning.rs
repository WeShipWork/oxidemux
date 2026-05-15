//! Provider-neutral reasoning and thinking control contracts.
//!
//! This module owns typed reasoning intent, validation, capability metadata, and
//! compatibility outcomes without provider-specific payload rewrites or live
//! discovery.

use crate::CoreError;

/// Lowest accepted provider-neutral reasoning token budget.
pub const MIN_REASONING_TOKEN_BUDGET: u32 = 1;
/// Highest accepted provider-neutral reasoning token budget.
pub const MAX_REASONING_TOKEN_BUDGET: u32 = 200_000;

#[derive(Clone, Debug, Eq, PartialEq)]
/// Optional reasoning intent carried through core request boundaries.
#[derive(Default)]
pub enum ReasoningRequest {
    /// No reasoning or thinking behavior was requested.
    #[default]
    Absent,
    /// A normalized reasoning or thinking behavior was requested.
    Intent(ReasoningIntent),
}

impl ReasoningRequest {
    /// Creates an absent reasoning request marker.
    pub const fn absent() -> Self {
        Self::Absent
    }

    /// Creates a reasoning request with normalized intent.
    pub fn intent(intent: ReasoningIntent) -> Result<Self, CoreError> {
        intent.validate()?;
        Ok(Self::Intent(intent))
    }

    /// Returns normalized intent when one exists.
    pub fn as_intent(&self) -> Option<&ReasoningIntent> {
        match self {
            Self::Absent => None,
            Self::Intent(intent) => Some(intent),
        }
    }

    /// Validates this value and returns structured core errors.
    pub fn validate(&self) -> Result<(), CoreError> {
        match self {
            Self::Absent => Ok(()),
            Self::Intent(intent) => intent.validate(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Provider-neutral reasoning or thinking intent.
pub struct ReasoningIntent {
    /// Source of the normalized reasoning controls.
    pub source: ReasoningSource,
    /// Requested reasoning mode.
    pub mode: ReasoningMode,
    /// Requested effort level or token budget.
    pub control: ReasoningControl,
    /// Handling policy for unsupported or unknown capabilities.
    pub handling: ReasoningHandlingPolicy,
    /// Diagnostics preserving model identity and source precedence.
    pub diagnostics: ReasoningDiagnostics,
}

impl ReasoningIntent {
    /// Creates normalized intent and validates it.
    pub fn new(
        source: ReasoningSource,
        mode: ReasoningMode,
        control: ReasoningControl,
        handling: ReasoningHandlingPolicy,
        diagnostics: ReasoningDiagnostics,
    ) -> Result<Self, CoreError> {
        let intent = Self {
            source,
            mode,
            control,
            handling,
            diagnostics,
        };
        intent.validate()?;
        Ok(intent)
    }

    /// Creates explicit request metadata.
    pub fn explicit(mode: ReasoningMode, control: ReasoningControl) -> Result<Self, CoreError> {
        Self::new(
            ReasoningSource::Explicit,
            mode,
            control,
            ReasoningHandlingPolicy::Strict,
            ReasoningDiagnostics::default(),
        )
    }

    /// Validates this intent and returns structured reasoning errors.
    pub fn validate(&self) -> Result<(), CoreError> {
        self.source.validate()?;
        if self.source.is_explicit() && matches!(self.handling, ReasoningHandlingPolicy::Permissive)
        {
            return Err(reasoning_validation_error(
                ReasoningFailureCode::InvalidHandlingPolicy,
                "handling",
                "explicit reasoning intent must use strict handling",
            ));
        }
        self.mode.validate(&self.control)?;
        self.diagnostics.validate()?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Raw typed reasoning metadata that may contain mutually exclusive fields before validation.
pub struct ReasoningMetadataInput {
    /// Source of the metadata.
    pub source: ReasoningSource,
    /// Requested reasoning mode.
    pub mode: ReasoningMode,
    /// Optional effort level.
    pub effort: Option<ReasoningEffort>,
    /// Optional provider-neutral token budget.
    pub budget: Option<u32>,
    /// Handling policy.
    pub handling: ReasoningHandlingPolicy,
    /// Diagnostics preserving model identity.
    pub diagnostics: ReasoningDiagnostics,
}

impl ReasoningMetadataInput {
    /// Normalizes typed metadata into a single provider-neutral intent.
    pub fn normalize(self) -> Result<ReasoningIntent, CoreError> {
        let control = match (self.effort, self.budget) {
            (Some(_), Some(_)) => {
                return Err(reasoning_validation_error(
                    ReasoningFailureCode::MutuallyExclusiveControls,
                    "control",
                    "reasoning effort and token budget are mutually exclusive",
                ));
            }
            (Some(effort), None) => ReasoningControl::Effort(effort),
            (None, Some(budget)) => ReasoningControl::Budget(ReasoningTokenBudget::new(budget)?),
            (None, None) => ReasoningControl::None,
        };

        ReasoningIntent::new(
            self.source,
            self.mode,
            control,
            self.handling,
            self.diagnostics,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Source of normalized reasoning controls.
pub enum ReasoningSource {
    /// Explicit typed Rust request metadata.
    Explicit,
    /// Typed in-memory alias metadata.
    Alias {
        /// Requested model alias.
        requested_model: String,
        /// Resolved model identifier.
        resolved_model: String,
    },
    /// Core default metadata supplied by a caller.
    Default,
}

impl ReasoningSource {
    fn validate(&self) -> Result<(), CoreError> {
        match self {
            Self::Explicit | Self::Default => Ok(()),
            Self::Alias {
                requested_model,
                resolved_model,
            } => {
                validate_required_reasoning_text("source.requested_model", requested_model)?;
                validate_required_reasoning_text("source.resolved_model", resolved_model)
            }
        }
    }

    /// Returns true when this source should be treated as explicit caller intent.
    pub fn is_explicit(&self) -> bool {
        matches!(self, Self::Explicit)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Provider-neutral reasoning mode.
pub enum ReasoningMode {
    /// General reasoning behavior.
    Reasoning,
    /// Extended thinking behavior.
    Thinking,
    /// Reasoning behavior is intentionally disabled.
    Disabled,
}

impl ReasoningMode {
    fn validate(&self, control: &ReasoningControl) -> Result<(), CoreError> {
        if matches!(self, Self::Disabled) && !matches!(control, ReasoningControl::None) {
            return Err(reasoning_validation_error(
                ReasoningFailureCode::InvalidModeControlCombination,
                "mode",
                "disabled reasoning mode cannot carry effort or budget controls",
            ));
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Provider-neutral effort levels.
pub enum ReasoningEffort {
    /// Low reasoning effort.
    Low,
    /// Medium reasoning effort.
    Medium,
    /// High reasoning effort.
    High,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Provider-neutral token budget validated to the core range.
pub struct ReasoningTokenBudget {
    /// Token budget value in the inclusive core range.
    pub tokens: u32,
}

impl ReasoningTokenBudget {
    /// Creates a validated provider-neutral token budget.
    pub fn new(tokens: u32) -> Result<Self, CoreError> {
        if !(MIN_REASONING_TOKEN_BUDGET..=MAX_REASONING_TOKEN_BUDGET).contains(&tokens) {
            return Err(reasoning_validation_error(
                ReasoningFailureCode::BudgetOutOfRange,
                "budget",
                format!(
                    "reasoning token budget must be in the {}..={} range",
                    MIN_REASONING_TOKEN_BUDGET, MAX_REASONING_TOKEN_BUDGET
                ),
            ));
        }
        Ok(Self { tokens })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Mutually exclusive provider-neutral reasoning control.
pub enum ReasoningControl {
    /// No effort or token budget was supplied.
    None,
    /// Provider-neutral effort level.
    Effort(ReasoningEffort),
    /// Provider-neutral token budget.
    Budget(ReasoningTokenBudget),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Handling policy for unsupported or unknown reasoning capabilities.
pub enum ReasoningHandlingPolicy {
    /// Unsupported or unknown capability is a structured error.
    Strict,
    /// Unsupported or unknown capability can become ignored outcome metadata.
    Permissive,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
/// Diagnostics preserving model identity and source precedence.
pub struct ReasoningDiagnostics {
    /// Requested model alias or model identifier when known.
    pub requested_model: Option<String>,
    /// Resolved model identifier when known.
    pub resolved_model: Option<String>,
    /// Provider-native model identifier when known.
    pub provider_native_model: Option<String>,
    /// Selected provider identifier when known.
    pub selected_provider_id: Option<String>,
    /// Selected account identifier when known.
    pub selected_account_id: Option<String>,
    /// Ignored alias metadata when explicit intent wins precedence.
    pub ignored_alias: Option<Box<ReasoningIntent>>,
}

impl ReasoningDiagnostics {
    fn validate(&self) -> Result<(), CoreError> {
        validate_optional_reasoning_text(
            "diagnostics.requested_model",
            self.requested_model.as_deref(),
        )?;
        validate_optional_reasoning_text(
            "diagnostics.resolved_model",
            self.resolved_model.as_deref(),
        )?;
        validate_optional_reasoning_text(
            "diagnostics.provider_native_model",
            self.provider_native_model.as_deref(),
        )?;
        validate_optional_reasoning_text(
            "diagnostics.selected_provider_id",
            self.selected_provider_id.as_deref(),
        )?;
        validate_optional_reasoning_text(
            "diagnostics.selected_account_id",
            self.selected_account_id.as_deref(),
        )?;
        if let Some(alias) = &self.ignored_alias {
            alias.validate()?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Reasoning capability metadata declared by a provider, account, model, or registry candidate.
#[derive(Default)]
pub enum ReasoningCapability {
    /// Capability support is not known.
    #[default]
    Unknown,
    /// Target does not support reasoning controls.
    Unsupported {
        /// Typed reason for unsupported state.
        reason: String,
    },
    /// Target supports reasoning controls with optional limits.
    Supported(ReasoningCapabilitySupport),
    /// Target can only partially honor reasoning controls.
    Degraded {
        /// Supported subset and limits.
        support: ReasoningCapabilitySupport,
        /// Typed degradation reasons.
        reasons: Vec<String>,
    },
}

impl ReasoningCapability {
    /// Returns supported reasoning capability with no extra limits.
    pub fn supported() -> Self {
        Self::Supported(ReasoningCapabilitySupport::all())
    }

    /// Evaluates one normalized intent against this capability.
    pub fn evaluate(
        &self,
        intent: &ReasoningIntent,
    ) -> Result<ReasoningCompatibilityOutcome, CoreError> {
        intent.validate()?;
        match self {
            Self::Supported(support) if support.supports_intent(intent) => {
                Ok(ReasoningCompatibilityOutcome::Supported {
                    intent: intent.clone(),
                    capability: self.clone(),
                    layer: ReasoningCapabilityLayer::Unknown,
                })
            }
            Self::Supported(_) => unsupported_or_ignored(
                intent,
                self.clone(),
                "capability limits do not support requested reasoning controls",
            ),
            Self::Degraded { support, reasons } if support.supports_intent(intent) => {
                Ok(ReasoningCompatibilityOutcome::Degraded {
                    intent: intent.clone(),
                    capability: self.clone(),
                    layer: ReasoningCapabilityLayer::Unknown,
                    reasons: reasons.clone(),
                })
            }
            Self::Degraded { .. } => unsupported_or_ignored(
                intent,
                self.clone(),
                "degraded capability cannot support requested reasoning controls",
            ),
            Self::Unsupported { .. } => unsupported_or_ignored(
                intent,
                self.clone(),
                "target does not support reasoning controls",
            ),
            Self::Unknown => unknown_or_ignored(intent, self.clone()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Supported reasoning modes, effort levels, and token-budget range.
pub struct ReasoningCapabilitySupport {
    /// Supported modes.
    pub modes: Vec<ReasoningMode>,
    /// Supported effort levels.
    pub efforts: Vec<ReasoningEffort>,
    /// Optional supported token-budget range.
    pub budget_range: Option<ReasoningBudgetRange>,
}

impl ReasoningCapabilitySupport {
    /// Creates capability support accepting all provider-neutral controls.
    pub fn all() -> Self {
        Self {
            modes: vec![
                ReasoningMode::Reasoning,
                ReasoningMode::Thinking,
                ReasoningMode::Disabled,
            ],
            efforts: vec![
                ReasoningEffort::Low,
                ReasoningEffort::Medium,
                ReasoningEffort::High,
            ],
            budget_range: Some(ReasoningBudgetRange {
                min: MIN_REASONING_TOKEN_BUDGET,
                max: MAX_REASONING_TOKEN_BUDGET,
            }),
        }
    }

    fn supports_intent(&self, intent: &ReasoningIntent) -> bool {
        if !self.modes.contains(&intent.mode) {
            return false;
        }
        match intent.control {
            ReasoningControl::None => true,
            ReasoningControl::Effort(effort) => self.efforts.contains(&effort),
            ReasoningControl::Budget(budget) => self
                .budget_range
                .as_ref()
                .is_some_and(|range| (range.min..=range.max).contains(&budget.tokens)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Inclusive token-budget support range.
pub struct ReasoningBudgetRange {
    /// Minimum supported token budget.
    pub min: u32,
    /// Maximum supported token budget.
    pub max: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Capability metadata resolved from provider/account/model layers.
pub struct ReasoningCapabilityLayers {
    /// Provider-level capability metadata.
    pub provider: Option<ReasoningCapability>,
    /// Account-level capability metadata.
    pub account: Option<ReasoningCapability>,
    /// Model-candidate capability metadata.
    pub model: Option<ReasoningCapability>,
}

impl ReasoningCapabilityLayers {
    /// Returns the most specific capability and the layer it came from.
    pub fn effective(&self) -> (ReasoningCapabilityLayer, ReasoningCapability) {
        if let Some(capability) = &self.model {
            (ReasoningCapabilityLayer::Model, capability.clone())
        } else if let Some(capability) = &self.account {
            (ReasoningCapabilityLayer::Account, capability.clone())
        } else if let Some(capability) = &self.provider {
            (ReasoningCapabilityLayer::Provider, capability.clone())
        } else {
            (
                ReasoningCapabilityLayer::Unknown,
                ReasoningCapability::Unknown,
            )
        }
    }

    /// Evaluates the effective capability against an intent.
    pub fn evaluate(
        &self,
        intent: &ReasoningIntent,
    ) -> Result<ReasoningCompatibilityOutcome, CoreError> {
        let (layer, capability) = self.effective();
        capability
            .evaluate(intent)
            .map(|outcome| outcome.with_layer(layer))
            .map_err(|error| error.with_reasoning_layer(layer))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Layer that supplied effective reasoning capability metadata.
pub enum ReasoningCapabilityLayer {
    /// No layer supplied metadata.
    Unknown,
    /// Provider-level metadata was used.
    Provider,
    /// Account-level metadata was used.
    Account,
    /// Model-level metadata was used.
    Model,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Compatibility result for one selected route and normalized intent.
pub enum ReasoningCompatibilityOutcome {
    /// No reasoning intent was present, so no capability lookup was needed.
    Absent,
    /// Target supports the normalized intent.
    Supported {
        /// Normalized intent.
        intent: ReasoningIntent,
        /// Capability metadata used for evaluation.
        capability: ReasoningCapability,
        /// Layer that supplied the capability metadata.
        layer: ReasoningCapabilityLayer,
    },
    /// Intent was ignored under permissive handling.
    Ignored {
        /// Normalized intent.
        intent: ReasoningIntent,
        /// Capability metadata used for evaluation.
        capability: ReasoningCapability,
        /// Layer that supplied the capability metadata.
        layer: ReasoningCapabilityLayer,
        /// Reason the intent was ignored.
        reason: String,
    },
    /// Target can only partially honor the intent.
    Degraded {
        /// Normalized intent.
        intent: ReasoningIntent,
        /// Capability metadata used for evaluation.
        capability: ReasoningCapability,
        /// Layer that supplied the capability metadata.
        layer: ReasoningCapabilityLayer,
        /// Degradation reasons.
        reasons: Vec<String>,
    },
}

impl ReasoningCompatibilityOutcome {
    fn with_layer(self, layer: ReasoningCapabilityLayer) -> Self {
        match self {
            Self::Absent => Self::Absent,
            Self::Supported {
                intent,
                capability,
                layer: _,
            } => Self::Supported {
                intent,
                capability,
                layer,
            },
            Self::Ignored {
                intent,
                capability,
                layer: _,
                reason,
            } => Self::Ignored {
                intent,
                capability,
                layer,
                reason,
            },
            Self::Degraded {
                intent,
                capability,
                layer: _,
                reasons,
            } => Self::Degraded {
                intent,
                capability,
                layer,
                reasons,
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Structured reasoning validation failure.
pub struct ReasoningValidationFailure {
    /// Stable failure code.
    pub code: ReasoningFailureCode,
    /// Field path associated with the failure.
    pub field: &'static str,
    /// Human-readable diagnostic message.
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Structured unsupported or unknown capability failure.
pub struct ReasoningCapabilityFailure {
    /// Stable failure code.
    pub code: ReasoningFailureCode,
    /// Requested reasoning intent.
    pub intent: ReasoningIntent,
    /// Capability metadata used for evaluation.
    pub capability: ReasoningCapability,
    /// Layer that supplied the capability metadata.
    pub layer: ReasoningCapabilityLayer,
    /// Human-readable diagnostic message.
    pub message: String,
}

impl ReasoningCapabilityFailure {
    pub(crate) fn with_layer(mut self, layer: ReasoningCapabilityLayer) -> Self {
        self.layer = layer;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Stable machine-matchable reasoning failure code.
pub enum ReasoningFailureCode {
    /// Token budget is outside the provider-neutral range.
    BudgetOutOfRange,
    /// Effort and budget were supplied together.
    MutuallyExclusiveControls,
    /// Mode and control combination is invalid.
    InvalidModeControlCombination,
    /// Handling policy is invalid for the reasoning source.
    InvalidHandlingPolicy,
    /// Required reasoning field is blank.
    BlankField,
    /// Requested capability is unsupported.
    UnsupportedCapability,
    /// Requested capability is unknown.
    UnknownCapability,
}

fn unsupported_or_ignored(
    intent: &ReasoningIntent,
    capability: ReasoningCapability,
    reason: &str,
) -> Result<ReasoningCompatibilityOutcome, CoreError> {
    if matches!(intent.handling, ReasoningHandlingPolicy::Permissive)
        && !intent.source.is_explicit()
    {
        Ok(ReasoningCompatibilityOutcome::Ignored {
            intent: intent.clone(),
            capability,
            layer: ReasoningCapabilityLayer::Unknown,
            reason: reason.to_string(),
        })
    } else {
        Err(CoreError::ReasoningUnsupportedCapability {
            failure: Box::new(ReasoningCapabilityFailure {
                code: ReasoningFailureCode::UnsupportedCapability,
                intent: intent.clone(),
                capability,
                layer: ReasoningCapabilityLayer::Unknown,
                message: reason.to_string(),
            }),
        })
    }
}

fn unknown_or_ignored(
    intent: &ReasoningIntent,
    capability: ReasoningCapability,
) -> Result<ReasoningCompatibilityOutcome, CoreError> {
    if matches!(intent.handling, ReasoningHandlingPolicy::Permissive)
        && !intent.source.is_explicit()
    {
        Ok(ReasoningCompatibilityOutcome::Ignored {
            intent: intent.clone(),
            capability,
            layer: ReasoningCapabilityLayer::Unknown,
            reason: "target reasoning capability is unknown".to_string(),
        })
    } else {
        Err(CoreError::ReasoningUnsupportedCapability {
            failure: Box::new(ReasoningCapabilityFailure {
                code: ReasoningFailureCode::UnknownCapability,
                intent: intent.clone(),
                capability,
                layer: ReasoningCapabilityLayer::Unknown,
                message: "target reasoning capability is unknown".to_string(),
            }),
        })
    }
}

fn reasoning_validation_error(
    code: ReasoningFailureCode,
    field: &'static str,
    message: impl Into<String>,
) -> CoreError {
    CoreError::ReasoningValidation {
        failure: Box::new(ReasoningValidationFailure {
            code,
            field,
            message: message.into(),
        }),
    }
}

fn validate_required_reasoning_text(field: &'static str, value: &str) -> Result<(), CoreError> {
    if value.trim().is_empty() {
        return Err(reasoning_validation_error(
            ReasoningFailureCode::BlankField,
            field,
            "value must not be blank",
        ));
    }
    Ok(())
}

fn validate_optional_reasoning_text(
    field: &'static str,
    value: Option<&str>,
) -> Result<(), CoreError> {
    if matches!(value, Some(value) if value.trim().is_empty()) {
        return Err(reasoning_validation_error(
            ReasoningFailureCode::BlankField,
            field,
            "value must not be blank when present",
        ));
    }
    Ok(())
}
