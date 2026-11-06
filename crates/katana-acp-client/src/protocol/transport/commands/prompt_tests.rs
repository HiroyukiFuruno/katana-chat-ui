use super::*;

async fn next_raw_frame(peer: &mut Channel) -> Result<TransportFrame, std::io::Error> {
    tokio::time::timeout(Duration::from_secs(1), async {
        loop {
            match peer.rx.try_recv() {
                Ok(frame) => return Ok(frame),
                Err(error) if error.is_empty() => tokio::task::yield_now().await,
                Err(error) => return Err(std::io::Error::other(error)),
            }
        }
    })
    .await
    .map_err(std::io::Error::other)?
}

async fn next_method(peer: &mut Channel) -> Result<String, std::io::Error> {
    let frame = next_raw_frame(peer).await?;
    match frame {
        TransportFrame::Single(RawJsonRpcMessage::Request(request)) => {
            Ok(request.method.to_string())
        }
        TransportFrame::Single(RawJsonRpcMessage::Notification(notification)) => {
            Ok(notification.method.to_string())
        }
        _ => Err(std::io::Error::other("unexpected raw frame")),
    }
}

async fn observe_prompt_cancel(
    mut peer: Channel,
    release: tokio::sync::oneshot::Sender<()>,
) -> Result<[String; 2], std::io::Error> {
    let methods = [next_method(&mut peer).await?, next_method(&mut peer).await?];
    release
        .send(())
        .map_err(|_| std::io::Error::other("client stopped"))?;
    Ok(methods)
}

fn dispatch_prompt_cancel(context: &super::super::CommandContext<'_>) {
    assert!(
        super::super::dispatch(
            context,
            crate::protocol::transport::types::AcpTransportCommand::Prompt(Vec::new())
        )
        .is_ok()
    );
    assert!(
        super::super::dispatch(
            context,
            crate::protocol::transport::types::AcpTransportCommand::Cancel
        )
        .is_ok()
    );
}

async fn prompt_publication_precedes_cancel_case() -> TestResult {
    let (client_transport, peer) = Channel::duplex();
    let (release, release_rx) = tokio::sync::oneshot::channel();
    let observer = tokio::spawn(observe_prompt_cancel(peer, release));
    let pending = super::super::super::state::PendingPermissions::default();
    let busy = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let (events, _received) = prompt_events();
    let (config_queue, _config_requests) = prompt_config_queue();
    let session = prompt_session();
    Client
        .builder()
        .connect_with(client_transport, async move |connection| {
            let context = prompt_context(
                &connection,
                &pending,
                &session,
                &busy,
                &events,
                &config_queue,
            );
            dispatch_prompt_cancel(&context);
            release_rx
                .await
                .map_err(|_| agent_client_protocol::Error::internal_error())
        })
        .await?;
    let methods = observer.await??;
    assert_eq!(methods, ["session/prompt", "session/cancel"]);
    Ok(())
}

fn prompt_events() -> (
    tokio::sync::mpsc::Sender<AcpTransportEvent>,
    tokio::sync::mpsc::Receiver<AcpTransportEvent>,
) {
    tokio::sync::mpsc::channel(2)
}

fn prompt_config_queue() -> (
    tokio::sync::mpsc::Sender<super::super::ConfigRequest>,
    tokio::sync::mpsc::Receiver<super::super::ConfigRequest>,
) {
    tokio::sync::mpsc::channel(1)
}

fn prompt_session() -> (
    String,
    super::super::super::config_state::SessionConfigState,
) {
    (
        "session-1".to_string(),
        super::super::super::config_state::SessionConfigState::default(),
    )
}

#[test]
fn prompt_publication_precedes_cancel_notification() -> TestResult {
    run_async(prompt_publication_precedes_cancel_case())
}

async fn prompt_resets_permission_cancellation_case() -> TestResult {
    let (client_transport, _peer) = Channel::duplex();
    let pending = super::super::super::state::PendingPermissions::default();
    super::super::super::state::cancel_permissions(&pending);
    let busy = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let (events, _received) = tokio::sync::mpsc::channel(2);
    let (config_queue, _config_requests) = tokio::sync::mpsc::channel(1);
    let session = (
        "session-1".to_string(),
        super::super::super::config_state::SessionConfigState::default(),
    );
    Client
        .builder()
        .connect_with(client_transport, async move |connection| {
            let context = prompt_context(
                &connection,
                &pending,
                &session,
                &busy,
                &events,
                &config_queue,
            );
            assert_busy_prompt_keeps_cancellation(&context, &pending);
            assert_accepted_prompt_resets_cancellation(&context, &pending, &busy);
            Ok(())
        })
        .await?;
    Ok(())
}

fn assert_busy_prompt_keeps_cancellation(
    context: &super::super::CommandContext<'_>,
    pending: &super::super::super::state::PendingPermissions,
) {
    assert!(super::super::start_prompt(context, Vec::new()).is_err());
    assert_late_permission_cancelled(pending);
}

fn assert_accepted_prompt_resets_cancellation(
    context: &super::super::CommandContext<'_>,
    pending: &super::super::super::state::PendingPermissions,
    busy: &Arc<std::sync::atomic::AtomicBool>,
) {
    busy.store(false, std::sync::atomic::Ordering::Release);
    assert!(super::super::start_prompt(context, Vec::new()).is_ok());
    assert_permission_registered(pending);
}

fn prompt_context<'a>(
    connection: &'a ConnectionTo<Agent>,
    pending: &'a super::super::super::state::PendingPermissions,
    session: &'a (
        String,
        super::super::super::config_state::SessionConfigState,
    ),
    busy: &'a Arc<std::sync::atomic::AtomicBool>,
    events: &'a tokio::sync::mpsc::Sender<AcpTransportEvent>,
    config_queue: &'a tokio::sync::mpsc::Sender<super::super::ConfigRequest>,
) -> super::super::CommandContext<'a> {
    super::super::CommandContext {
        connection,
        pending,
        capabilities: AcpPromptCapabilities {
            image: false,
            embedded_context: false,
        },
        session,
        busy,
        events,
        config_queue,
    }
}

fn assert_late_permission_cancelled(pending: &super::super::super::state::PendingPermissions) {
    let (sender, mut receiver) = tokio::sync::oneshot::channel();
    assert!(super::super::super::state::insert_permission(pending, 1, sender).is_ok());
    assert!(matches!(
        receiver.try_recv(),
        Ok(crate::protocol::transport::AcpPermissionDecision::Cancelled)
    ));
}

fn assert_permission_registered(pending: &super::super::super::state::PendingPermissions) {
    let (sender, _receiver) = tokio::sync::oneshot::channel();
    assert!(matches!(
        super::super::super::state::insert_permission(pending, 2, sender),
        Ok(super::super::super::state::PermissionRegistration::Registered)
    ));
}

#[test]
fn accepted_prompt_resets_permission_cancellation_but_busy_prompt_does_not() -> TestResult {
    run_async(prompt_resets_permission_cancellation_case())
}
