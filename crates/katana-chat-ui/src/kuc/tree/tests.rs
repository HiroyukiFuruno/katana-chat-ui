use super::{ChatUiTreeBuilder, ChatUiTreeViewport};
use crate::{
    ChatOutputKind, ChatSession, ChatSessionError, ChatUiAction, ChatUiActionMapper,
    ChatUiOutputSurface, ChatUiSurface, CommandLaunchEntry, CommandLaunchKind,
    ContextUsageSnapshot, HostActionIntent, HostActionKind, TextOutput, VendorUiState,
};
use katana_ui_core::render_model::{
    UiHostActionPlan, UiJustifyContent, UiLayoutAxis, UiNode, UiNodeKind, UiPosition,
};
use std::collections::HashSet;

#[test]
fn tree_preserves_surface_and_projects_input_contract() -> Result<(), Box<dyn std::error::Error>> {
    let mut surface = sample_surface()?;
    surface.composer.send_available = false;
    surface.composer.send_enabled = false;
    surface.composer.stop_enabled = true;
    assert!(surface.composer.slash_launcher.visible);
    let tree = ChatUiTreeBuilder::build(&surface)?;

    assert_eq!(&surface, tree.source());
    assert_eq!("chat.root", tree.tree().root().id().as_str());
    assert_eq!(
        UiLayoutAxis::Vertical,
        tree.tree().root().props().common.layout_axis
    );
    assert_input_contract(&tree, &surface)?;
    assert_message_alignment(&tree, &surface)?;
    assert_message_card_contract(&tree, &surface)?;
    Ok(())
}

fn assert_message_card_contract(
    tree: &super::ChatUiTree,
    surface: &ChatUiSurface,
) -> Result<(), Box<dyn std::error::Error>> {
    for message in &surface.messages {
        let node_id = format!("chat.message.{}", message.id);
        let message = find_node(tree.tree().root(), &node_id)
            .ok_or_else(|| format!("message root must be rendered: {node_id}"))?;
        assert_eq!(UiNodeKind::Card, message.kind());
        assert_eq!(Some(16), message.props().common.child_indent);
    }
    Ok(())
}

#[test]
fn tree_disables_control_options_and_keeps_node_ids_unique()
-> Result<(), Box<dyn std::error::Error>> {
    let mut surface = sample_surface()?;
    disable_first_control(&mut surface)?;
    add_duplicate_output_locations(&mut surface)?;
    let tree = ChatUiTreeBuilder::build_for_viewport(
        &surface,
        Some("chat.composer.control.0"),
        ChatUiTreeViewport {
            scroll_y: 0,
            hide_thread: false,
            history_scroll_y: 0,
            history_content_height: 0,
            history_row_heights: &[],
            permission_scroll_y: 0,
            permission_content_height: 0,
            message_heights: &[],
            width: 640,
            height: 480,
        },
    )?;
    assert_control_option_is_disabled(tree.tree().root())?;
    assert_node_ids_are_unique(tree.tree().root())?;
    Ok(())
}

fn assert_input_contract(
    tree: &super::ChatUiTree,
    surface: &ChatUiSurface,
) -> Result<(), Box<dyn std::error::Error>> {
    let input = find_node(tree.tree().root(), "chat.composer.input")
        .ok_or("composer input must be rendered")?;
    assert!(input.props().text_area.ime_enabled);
    assert_eq!(surface.composer.text, input.props().interaction.value);
    let composer =
        find_node(tree.tree().root(), "chat.composer").ok_or("composer panel must be rendered")?;
    assert_eq!(UiPosition::Sticky, composer.props().common.position);
    assert!(find_node(tree.tree().root(), "chat.composer.stop").is_some());
    assert!(find_node(tree.tree().root(), "chat.composer.submit").is_none());
    assert_standard_tree_omits_slash_launcher(tree.tree().root());
    assert_eq!("claude-code", surface.vendor_bar.active_vendor_id);
    Ok(())
}

fn assert_standard_tree_omits_slash_launcher(root: &UiNode) {
    assert!(find_node(root, "chat.composer.slash.query").is_none());
    assert!(
        UiHostActionPlan::collect_from_root(root)
            .iter()
            .all(|plan| !matches!(
                ChatUiActionMapper::from_host_action(plan),
                Ok(ChatUiAction::SelectSlashCommand { .. })
            ))
    );
}

fn assert_message_alignment(
    tree: &super::ChatUiTree,
    surface: &ChatUiSurface,
) -> Result<(), Box<dyn std::error::Error>> {
    let message_id = surface
        .messages
        .first()
        .ok_or("sample must contain a user message")?
        .id;
    let row_id = format!("chat.message.{message_id}.row");
    let row = find_node(tree.tree().root(), &row_id).ok_or("message row must be rendered")?;
    assert_eq!(UiJustifyContent::End, row.props().common.justify_content);
    Ok(())
}

fn disable_first_control(surface: &mut ChatUiSurface) -> Result<(), Box<dyn std::error::Error>> {
    let control = surface
        .vendor_bar
        .controls
        .first_mut()
        .ok_or("sample must contain a vendor control")?;
    control.enabled = false;
    Ok(())
}

fn add_duplicate_output_locations(
    surface: &mut ChatUiSurface,
) -> Result<(), Box<dyn std::error::Error>> {
    let output = ChatUiOutputSurface {
        id: 7,
        source_message_id: 1,
        kind: ChatOutputKind::Text(TextOutput::new("Generated document")),
        status: crate::OutputStatus::Ready,
        actions: vec![HostActionIntent::new(7, HostActionKind::Copy)],
    };
    let message = surface
        .message_list
        .messages
        .first_mut()
        .ok_or("sample must contain a message list entry")?;
    message.outputs.push(output.clone());
    surface.output_handoff.outputs.push(output);
    Ok(())
}

fn assert_control_option_is_disabled(root: &UiNode) -> Result<(), Box<dyn std::error::Error>> {
    let plans = UiHostActionPlan::collect_from_root(root);
    let plan = plans
        .iter()
        .find(|plan| {
            matches!(
                ChatUiActionMapper::from_host_action(plan),
                Ok(ChatUiAction::SelectVendorControl { .. })
            )
        })
        .ok_or("tree must include a vendor control option")?;
    assert!(!plan.enabled);
    Ok(())
}

fn assert_node_ids_are_unique(root: &UiNode) -> Result<(), Box<dyn std::error::Error>> {
    let mut ids = HashSet::new();
    collect_node_ids(root, &mut ids)
}

fn collect_node_ids(
    node: &UiNode,
    ids: &mut HashSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !ids.insert(node.id().as_str().to_string()) {
        return Err(format!("duplicate node id: {}", node.id().as_str()).into());
    }
    for child in node.children() {
        collect_node_ids(child, ids)?;
    }
    Ok(())
}

fn sample_surface() -> Result<ChatUiSurface, ChatSessionError> {
    let mut session = ChatSession::new();
    session.set_provider_configured("Claude Code");
    session.set_context_usage(ContextUsageSnapshot::new(1, 100));
    session.set_command_entries(vec![CommandLaunchEntry::new(
        "review",
        "Review",
        CommandLaunchKind::Workflow,
    )]);
    session.draft_mut().set_text("Create a document");
    session.submit_draft()?;
    session.draft_mut().set_text("/re");
    session.set_vendor_ui_state(
        VendorUiState::for_vendor("claude-code")
            .with_available_vendors(vec!["claude-code".to_string(), "codex-cli".to_string()])
            .with_models(vec!["model".to_string()], "model"),
    );
    Ok(ChatUiSurface::from_render_model(&session.render_model()))
}

fn find_node<'a>(node: &'a UiNode, target: &str) -> Option<&'a UiNode> {
    if node.id().as_str() == target {
        return Some(node);
    }
    node.children()
        .iter()
        .find_map(|child| find_node(child, target))
}
