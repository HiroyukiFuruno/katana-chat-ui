use katana_chat_ui::{AccountUsageSnapshot, ContextUsageSnapshot, UsageStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageMeterView {
    pub context_percentage: u8,
    pub context_status: UsageStatus,
    pub account_available: bool,
    pub account_reason: String,
}

impl UsageMeterView {
    pub fn from_model(context: &ContextUsageSnapshot, account: &AccountUsageSnapshot) -> Self {
        Self {
            context_percentage: context.percentage,
            context_status: context.status,
            account_available: account.is_available(),
            account_reason: account.unavailable_reason().to_string(),
        }
    }
}
