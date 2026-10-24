use crate::AcpProviderProjection;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ProjectionRoute {
    Ready,
    ApplyActive,
    IgnoreInactive,
    CancelInactivePermission,
}

pub(super) fn route(
    active_vendor_id: Option<&str>,
    source_vendor_id: &str,
    projection: &AcpProviderProjection,
) -> ProjectionRoute {
    if matches!(projection, AcpProviderProjection::Ready { .. }) {
        return ProjectionRoute::Ready;
    }
    if matches!(projection, AcpProviderProjection::Permission { .. })
        && active_vendor_id != Some(source_vendor_id)
    {
        return ProjectionRoute::CancelInactivePermission;
    }
    if active_vendor_id == Some(source_vendor_id) {
        ProjectionRoute::ApplyActive
    } else {
        ProjectionRoute::IgnoreInactive
    }
}

#[cfg(test)]
mod tests {
    use super::{ProjectionRoute, route};
    use crate::AcpProviderProjection;
    use katana_acp_client::protocol::{AcpPermissionContext, AcpPermissionOption};

    #[test]
    fn non_active_permission_is_cancelled_instead_of_reaching_the_chat() {
        let projection = AcpProviderProjection::Permission {
            request_id: 7,
            context: AcpPermissionContext {
                tool_call_id: "tool-7".into(),
                title: None,
                details: Vec::new(),
            },
            options: vec![AcpPermissionOption {
                option_id: "allow".into(),
                name: "Allow".into(),
            }],
        };

        assert_eq!(
            route(Some("codex-cli"), "claude-code", &projection),
            ProjectionRoute::CancelInactivePermission
        );
    }

    #[test]
    fn readiness_is_recorded_even_before_the_vendor_is_selected() {
        let projection = AcpProviderProjection::Ready { models: Vec::new() };

        assert_eq!(
            route(Some("codex-cli"), "claude-code", &projection),
            ProjectionRoute::Ready
        );
    }
}
