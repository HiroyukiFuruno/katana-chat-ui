use agent_client_protocol::schema::v1::{
    SessionConfigOption, SessionConfigOptionCategory, SessionConfigSelectOption,
    SetSessionConfigOptionResponse,
};

use super::config_result_events;
use crate::protocol::{
    AcpConfigValue, AcpSessionConfig, AcpSessionConfigCategory, AcpTransportEvent,
};

#[test]
fn successful_model_change_projects_agent_confirmed_config_and_acknowledges() {
    let events = config_result_events(
        Ok(confirmed_model_response()),
        "session-1".into(),
        "model".into(),
    );

    assert_eq!(events, expected_confirmed_model_events());
}

fn confirmed_model_response() -> SetSessionConfigOptionResponse {
    let model = SessionConfigOption::select(
        "model",
        "Model",
        "model-b",
        vec![
            SessionConfigSelectOption::new("model-a", "Model A"),
            SessionConfigSelectOption::new("model-b", "Model B"),
        ],
    )
    .category(SessionConfigOptionCategory::Model);
    SetSessionConfigOptionResponse::new(vec![model])
}

fn expected_confirmed_model_events() -> Vec<AcpTransportEvent> {
    vec![
        AcpTransportEvent::SessionConfigOptionsUpdated {
            session_id: "session-1".into(),
            config_options: vec![confirmed_model_config()],
        },
        AcpTransportEvent::ConfigUpdated {
            config_id: "model".into(),
        },
    ]
}

fn confirmed_model_config() -> AcpSessionConfig {
    AcpSessionConfig {
        id: "model".into(),
        name: "Model".into(),
        category: AcpSessionConfigCategory::Model,
        current_value: Some("model-b".into()),
        values: vec![
            AcpConfigValue {
                id: "model-a".into(),
                name: "Model A".into(),
            },
            AcpConfigValue {
                id: "model-b".into(),
                name: "Model B".into(),
            },
        ],
    }
}

#[test]
fn failed_model_change_keeps_config_state_unchanged() {
    let events = config_result_events(
        Err(agent_client_protocol::Error::internal_error()),
        "session-1".into(),
        "model".into(),
    );

    assert!(matches!(
        events.as_slice(),
        [AcpTransportEvent::TransportError { .. }]
    ));
}

#[test]
fn rejected_model_change_is_distinct_from_transport_failure() {
    for error in [
        agent_client_protocol::Error::invalid_request(),
        agent_client_protocol::Error::method_not_found(),
        agent_client_protocol::Error::invalid_params(),
    ] {
        let events = config_result_events(Err(error), "session-1".into(), "model".into());

        assert!(matches!(
            events.as_slice(),
            [AcpTransportEvent::ConfigRejected {
                config_id,
                message: _
            }] if config_id == "model"
        ));
    }
}
