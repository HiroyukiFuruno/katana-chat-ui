use super::types::{AcpConfigValue, AcpSessionConfig, AcpSessionConfigCategory};
use agent_client_protocol::schema::v1::{
    SessionConfigKind, SessionConfigOption, SessionConfigOptionCategory, SessionConfigSelectOption,
    SessionConfigSelectOptions,
};

pub(crate) fn project_config_options(options: Vec<SessionConfigOption>) -> Vec<AcpSessionConfig> {
    options.into_iter().map(project_config_option).collect()
}

fn project_config_option(option: SessionConfigOption) -> AcpSessionConfig {
    let (current_value, values) = match option.kind {
        SessionConfigKind::Select(select) => (
            Some(select.current_value.to_string()),
            select_options(select.options),
        ),
        SessionConfigKind::Boolean(_) | _ => (None, Vec::new()),
    };
    let category = if matches!(option.category, Some(SessionConfigOptionCategory::Model)) {
        AcpSessionConfigCategory::Model
    } else {
        AcpSessionConfigCategory::Other
    };
    AcpSessionConfig {
        id: option.id.to_string(),
        name: option.name,
        category,
        current_value,
        values,
    }
}

fn select_options(options: SessionConfigSelectOptions) -> Vec<AcpConfigValue> {
    match options {
        SessionConfigSelectOptions::Ungrouped(items) => project_values(items),
        SessionConfigSelectOptions::Grouped(groups) => groups
            .into_iter()
            .flat_map(|group| project_values(group.options))
            .collect(),
        _ => Vec::new(),
    }
}

fn project_values(values: Vec<SessionConfigSelectOption>) -> Vec<AcpConfigValue> {
    values
        .into_iter()
        .map(|value| AcpConfigValue {
            id: value.value.to_string(),
            name: value.name,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::project_config_options;
    use agent_client_protocol::schema::v1::{
        SessionConfigOption, SessionConfigOptionCategory, SessionConfigSelectOption,
    };

    #[test]
    fn keeps_advertised_current_value_when_it_is_not_first() {
        let option = SessionConfigOption::select(
            "model",
            "Model",
            "second",
            vec![
                SessionConfigSelectOption::new("first", "First"),
                SessionConfigSelectOption::new("second", "Second"),
            ],
        )
        .category(SessionConfigOptionCategory::Model);
        let configs = project_config_options(vec![option]);
        assert_eq!(configs[0].current_value.as_deref(), Some("second"));
    }
}
