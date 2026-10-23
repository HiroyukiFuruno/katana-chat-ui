use std::{
    collections::VecDeque,
    sync::{Arc, Weak},
};

use katana_chat_ui::{ChatSession, ChatUiSurface};
use kcu_host_support::{ChatHostEffect, ManagedHostUpdate};

use super::synchronization::{drain_bounded, update_plan, upgrade_host};

#[test]
fn stale_draft_ack_keeps_the_newer_composer_text() {
    let plan = update_plan(8, &update(7, "old", None, None));
    assert_eq!(plan.draft, None);
}

#[test]
fn update_plan_preserves_error_and_effect_without_clearing_them() {
    let plan = update_plan(
        8,
        &update(
            8,
            "current",
            Some("provider stopped"),
            Some(ChatHostEffect::Submitted { message_id: 3 }),
        ),
    );
    assert_eq!(plan.draft.as_deref(), Some("current"));
    assert_eq!(plan.error.as_deref(), Some("provider stopped"));
    assert_eq!(
        plan.effect,
        Some(ChatHostEffect::Submitted { message_id: 3 })
    );
    assert_eq!(
        update_plan(8, &update(8, "current", None, None)).error,
        None
    );
}

#[test]
fn update_drain_stops_after_the_tick_limit() {
    let mut pending: VecDeque<_> = (0..33).collect();
    let mut applied = Vec::new();
    drain_bounded(
        || Ok::<_, ()>(pending.pop_front()),
        |value| applied.push(value),
        |_| unreachable!("the queue does not fail"),
    );
    assert_eq!(applied.len(), 32);
    assert_eq!(pending.into_iter().collect::<Vec<_>>(), vec![32]);
}

#[test]
fn scheduling_stops_when_the_host_is_dropped() {
    let host = Arc::new(());
    let weak: Weak<()> = Arc::downgrade(&host);
    assert!(upgrade_host(&weak).is_some());
    drop(host);
    assert!(upgrade_host(&weak).is_none());
}

fn update(
    ack_seq: u64,
    draft: &str,
    error: Option<&str>,
    effect: Option<ChatHostEffect>,
) -> ManagedHostUpdate {
    let mut session = ChatSession::new();
    session.draft_mut().set_text(draft);
    ManagedHostUpdate {
        ack_seq,
        surface: ChatUiSurface::from_render_model(&session.render_model()),
        effect,
        error: error.map(str::to_string),
    }
}
