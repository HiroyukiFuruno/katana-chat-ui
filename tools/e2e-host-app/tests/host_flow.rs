use std::process::Command;

const EXPECTED_OUTPUT: &[&str] = &[
    "render:panel",
    "standard-ui:surface:composer",
    "standard-ui:widget:floem",
    "attachment:count:1",
    "attachment:path_drop:file:///tmp/kcu-fixture.md",
    "path-drop:file:///tmp/kcu-fixture.md",
    "send-before-submit:enabled:true",
    "stop:visible:true",
    "usage:75",
    "handoff-json:outputs:2",
    "handoff-json:actions:2",
    "svg-standard:send:<svg",
    "data-kcu-icon=\"send\"",
    "vendor:active:claude-code",
    "vendor-control:endpoint:",
    "vendor-affordance:model:true",
    "vendor-affordance:mode:false",
    "vendor-affordance:thinking:true",
    "vendor-affordance:permission:true",
    "vendor-affordance:tool-approval:false",
    "vendor-affordance:web-search:false",
    "vendor-control:usage:false",
    "vendor-control:account-usage:false",
];

#[test]
fn external_host_exercises_basic_chat_flow() -> Result<(), Box<dyn std::error::Error>> {
    let stdout = run_host()?;
    assert_expected_output(&stdout);
    Ok(())
}

fn run_host() -> Result<String, Box<dyn std::error::Error>> {
    let binary = std::env::var("CARGO_BIN_EXE_e2e-host-app")?;
    let output = Command::new(binary).output()?;
    assert!(output.status.success());
    Ok(String::from_utf8(output.stdout)?)
}

fn assert_expected_output(stdout: &str) {
    for expected in EXPECTED_OUTPUT {
        assert!(
            stdout.contains(expected),
            "host output must contain `{expected}`"
        );
    }
}
