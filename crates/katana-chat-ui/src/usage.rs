use serde::{Deserialize, Serialize};

const CRITICAL_USAGE_PERCENTAGE: u8 = 90;
const WARNING_USAGE_PERCENTAGE: u8 = 70;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextUsageSnapshot {
    pub used_tokens: u64,
    pub max_tokens: u64,
    pub percentage: u8,
    pub status: UsageStatus,
}

impl ContextUsageSnapshot {
    pub fn new(used_tokens: u64, max_tokens: u64) -> Self {
        let percentage = Self::percentage(used_tokens, max_tokens);
        Self {
            used_tokens,
            max_tokens,
            percentage,
            status: UsageStatus::from_percentage(percentage),
        }
    }

    pub fn unavailable() -> Self {
        Self {
            used_tokens: 0,
            max_tokens: 0,
            percentage: 0,
            status: UsageStatus::Unavailable,
        }
    }

    fn percentage(used_tokens: u64, max_tokens: u64) -> u8 {
        if max_tokens == 0 {
            return 0;
        }
        let percentage = used_tokens.saturating_mul(100) / max_tokens;
        if percentage > 100 {
            return 100;
        }
        percentage as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UsageStatus {
    Normal,
    Warning,
    Critical,
    Unavailable,
}

impl UsageStatus {
    pub fn from_percentage(percentage: u8) -> Self {
        if percentage >= CRITICAL_USAGE_PERCENTAGE {
            return Self::Critical;
        }
        if percentage >= WARNING_USAGE_PERCENTAGE {
            return Self::Warning;
        }
        Self::Normal
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountUsageSnapshot {
    pub state: AccountUsageState,
}

impl AccountUsageSnapshot {
    pub fn available(details: AccountUsageDetails) -> Self {
        Self {
            state: AccountUsageState::Available(details),
        }
    }

    pub fn unavailable(reason: impl Into<String>) -> Self {
        Self {
            state: AccountUsageState::Unavailable(reason.into()),
        }
    }

    pub fn is_available(&self) -> bool {
        matches!(self.state, AccountUsageState::Available(_))
    }

    pub fn unavailable_reason(&self) -> &str {
        match &self.state {
            AccountUsageState::Available(_) => "",
            AccountUsageState::Unavailable(reason) => reason.as_str(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountUsageState {
    Available(AccountUsageDetails),
    Unavailable(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountUsageDetails {
    pub auth_method: String,
    pub account_label: String,
    pub organization_label: String,
    pub plan_label: String,
    pub quota_rows: Vec<QuotaRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuotaRow {
    pub label: String,
    pub used: u64,
    pub limit: u64,
    pub status: UsageStatus,
}

#[cfg(test)]
mod tests;
