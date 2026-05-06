use std::process::Command;

#[test]
fn external_host_exercises_basic_chat_flow() -> Result<(), Box<dyn std::error::Error>> {
    let binary = std::env::var("CARGO_BIN_EXE_e2e-host-app")?;
    let output = Command::new(binary).output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("render:panel"));
    assert!(stdout.contains("standard-ui:surface:composer"));
    assert!(stdout.contains("standard-ui:widget:floem"));
    assert!(stdout.contains("attachment:count:1"));
    assert!(stdout.contains("path-drop:file:///tmp/kcu-fixture.md"));
    assert!(stdout.contains("send-before-submit:enabled:true"));
    assert!(stdout.contains("stop:visible:true"));
    assert!(stdout.contains("usage:75"));
    assert!(stdout.contains("handoff-json:outputs:2"));
    assert!(stdout.contains("handoff-json:actions:2"));
    assert!(stdout.contains("svg-standard:send:<svg"));
    assert!(stdout.contains("data-kcu-icon=\"send\""));
    assert!(stdout.contains("vendor:active:ollama"));
    assert!(stdout.contains("vendor-control:endpoint:http://localhost:11434"));
    assert!(stdout.contains("vendor-affordance:model:true"));
    assert!(stdout.contains("vendor-affordance:mode:false"));
    assert!(stdout.contains("vendor-affordance:thinking:true"));
    assert!(stdout.contains("vendor-affordance:permission:false"));
    assert!(stdout.contains("vendor-affordance:tool-approval:true"));
    assert!(stdout.contains("vendor-affordance:web-search:false"));
    assert!(stdout.contains("vendor-control:usage:true"));
    assert!(stdout.contains("vendor-control:account-usage:false"));
    Ok(())
}
