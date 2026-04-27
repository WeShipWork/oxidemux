#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UsageBoundary;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UsageSummary {
    pub requests: MeteredValue,
    pub input_tokens: MeteredValue,
    pub output_tokens: MeteredValue,
    pub model_totals: MeteredValue,
    pub provider_totals: MeteredValue,
    pub account_totals: MeteredValue,
}

impl UsageSummary {
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
pub struct QuotaSummary {
    pub requests: QuotaState,
    pub tokens: QuotaState,
}

impl QuotaSummary {
    pub fn unknown() -> Self {
        Self {
            requests: QuotaState::Unknown,
            tokens: QuotaState::Unknown,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MeteredValue {
    Zero,
    Known(u64),
    Unknown,
    Unavailable { reason: String },
    Degraded { value: Option<u64>, reason: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QuotaState {
    Unknown,
    Unavailable {
        reason: String,
    },
    Degraded {
        remaining: Option<u64>,
        reason: String,
    },
    Limited {
        remaining: u64,
        limit: u64,
    },
    Unlimited,
}
