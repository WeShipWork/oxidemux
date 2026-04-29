//! Usage and quota summary contracts for subscription-aware state.
//!
//! Usage values distinguish known, zero, unknown, unavailable, and degraded meter
//! states so consumers can surface quota pressure without treating absent data as
//! successful measurement.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Marker for usage and quota summary ownership in the headless core.
pub struct UsageBoundary;

#[derive(Clone, Debug, Eq, PartialEq)]
/// Aggregated usage meters for requests, tokens, models, providers, and accounts.
pub struct UsageSummary {
    /// Request usage or quota state.
    pub requests: MeteredValue,
    /// Input token usage meter.
    pub input_tokens: MeteredValue,
    /// Output token usage meter.
    pub output_tokens: MeteredValue,
    /// Usage totals grouped by model.
    pub model_totals: MeteredValue,
    /// Usage totals grouped by provider.
    pub provider_totals: MeteredValue,
    /// Usage totals grouped by account.
    pub account_totals: MeteredValue,
}

impl UsageSummary {
    /// Returns a usage summary where all meters are known zero.
    pub fn zero() -> Self {
        Self {
            requests: MeteredValue::Zero,
            input_tokens: MeteredValue::Zero,
            output_tokens: MeteredValue::Zero,
            model_totals: MeteredValue::Zero,
            provider_totals: MeteredValue::Zero,
            account_totals: MeteredValue::Zero,
        }
    }

    /// Returns a summary where usage or quota values are unknown.
    pub fn unknown() -> Self {
        Self {
            requests: MeteredValue::Unknown,
            input_tokens: MeteredValue::Unknown,
            output_tokens: MeteredValue::Unknown,
            model_totals: MeteredValue::Unknown,
            provider_totals: MeteredValue::Unknown,
            account_totals: MeteredValue::Unknown,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Aggregated request and token quota states.
pub struct QuotaSummary {
    /// Request usage or quota state.
    pub requests: QuotaState,
    /// Token quota state.
    pub tokens: QuotaState,
}

impl QuotaSummary {
    /// Returns a summary where usage or quota values are unknown.
    pub fn unknown() -> Self {
        Self {
            requests: QuotaState::Unknown,
            tokens: QuotaState::Unknown,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Known, unknown, unavailable, or degraded usage meter value.
pub enum MeteredValue {
    /// Meter is known to be zero.
    Zero,
    /// Meter has a known numeric value.
    Known(u64),
    /// State is not known to the core.
    Unknown,
    /// Meter value cannot currently be measured.
    Unavailable {
        /// Human-readable reason for this state.
        reason: String,
    },
    /// Meter value is partial, stale, or otherwise lower confidence.
    Degraded {
        /// Metered value when known.
        value: Option<u64>,
        /// Human-readable reason for this state.
        reason: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Known, unknown, unavailable, degraded, limited, or unlimited quota state.
pub enum QuotaState {
    /// State is not known to the core.
    Unknown,
    /// Quota reading cannot currently be obtained.
    Unavailable {
        /// Human-readable reason for this state.
        reason: String,
    },
    /// Quota reading is partial, stale, or otherwise lower confidence.
    Degraded {
        /// Remaining quota units when known.
        remaining: Option<u64>,
        /// Human-readable reason for this state.
        reason: String,
    },
    /// Quota is limited with remaining and total values.
    Limited {
        /// Remaining quota units when known.
        remaining: u64,
        /// Quota limit associated with the remaining value.
        limit: u64,
    },
    /// Quota is unlimited.
    Unlimited,
}
