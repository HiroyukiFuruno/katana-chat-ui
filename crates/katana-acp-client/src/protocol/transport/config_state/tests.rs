use super::SessionConfigState;
use crate::protocol::{AcpSessionConfig, AcpSessionConfigCategory, AcpTransportEvent};
use tokio::sync::mpsc;

type TestResult = Result<(), Box<dyn std::error::Error>>;

use std::future::Future;

fn run_async(future: impl Future<Output = TestResult>) -> TestResult {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(future)
}

fn config(id: &str) -> AcpSessionConfig {
    AcpSessionConfig {
        id: id.to_string(),
        name: id.to_string(),
        category: AcpSessionConfigCategory::Other,
        current_value: Some(format!("{id}-value")),
        values: Vec::new(),
    }
}

fn update(session_id: &str, ids: &[&str]) -> AcpTransportEvent {
    AcpTransportEvent::SessionConfigOptionsUpdated {
        session_id: session_id.to_string(),
        config_options: ids.iter().map(|id| config(id)).collect(),
    }
}

#[test]
fn published_update_replaces_ids_and_emits_the_same_ui_event() -> TestResult {
    run_async(async {
        let state = SessionConfigState::default();
        let (events, mut received) = mpsc::channel(4);
        let first = update("session-1", &["old"]);
        state.publish(first.clone(), &events).await?;
        assert_eq!(received.recv().await, Some(first));
        assert!(state.contains("session-1", "old")?);

        let replacement = update("session-1", &["new"]);
        state.publish(replacement.clone(), &events).await?;
        assert_eq!(received.recv().await, Some(replacement));
        assert!(!state.contains("session-1", "old")?);
        assert!(state.contains("session-1", "new")?);
        Ok(())
    })
}

#[test]
fn config_ids_are_isolated_between_sessions() -> TestResult {
    run_async(async {
        let state = SessionConfigState::default();
        let (events, mut received) = mpsc::channel(4);
        state
            .publish(update("session-a", &["model-a"]), &events)
            .await?;
        state
            .publish(update("session-b", &["model-b"]), &events)
            .await?;
        let _ = received.recv().await.ok_or("missing first session event")?;
        let _ = received
            .recv()
            .await
            .ok_or("missing second session event")?;

        assert!(state.contains("session-a", "model-a")?);
        assert!(!state.contains("session-a", "model-b")?);
        assert!(state.contains("session-b", "model-b")?);
        assert!(!state.contains("session-b", "model-a")?);
        Ok(())
    })
}

#[test]
fn closed_event_sender_does_not_change_config_ids() -> TestResult {
    run_async(async {
        let state = SessionConfigState::default();
        let (events, received) = mpsc::channel(1);
        drop(received);

        assert!(
            state
                .publish(update("session-1", &["new"]), &events)
                .await
                .is_err()
        );
        assert!(!state.contains("session-1", "new")?);
        Ok(())
    })
}
