use super::{AccountUsageSnapshot, ContextUsageSnapshot, UsageStatus};

#[test]
fn context_usage_calculates_percentage_and_warning_status() {
    let usage = ContextUsageSnapshot::new(750, 1000);

    assert_eq!(usage.percentage, 75);
    assert_eq!(usage.status, UsageStatus::Warning);
}

#[test]
fn account_usage_has_explicit_unavailable_state() {
    let usage = AccountUsageSnapshot::unavailable("provider does not expose usage");

    assert!(!usage.is_available());
    assert_eq!(usage.unavailable_reason(), "provider does not expose usage");
}
