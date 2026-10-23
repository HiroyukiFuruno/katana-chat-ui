use katana_acp_client::AcpContentBlock;
use katana_acp_client::protocol::{
    AcpPermissionDecision, AcpStdioTransport, AcpStdioTransportConfig,
};
use std::{collections::BTreeMap, path::PathBuf};

mod projection;

use projection::project;
pub use projection::{AcpProviderModel, AcpProviderProjection};

pub struct AcpProviderProfile {
    pub vendor_id: String,
    pub command: PathBuf,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub cwd: PathBuf,
}

impl AcpProviderProfile {
    pub fn new(
        vendor_id: impl Into<String>,
        command: impl Into<PathBuf>,
        cwd: impl Into<PathBuf>,
    ) -> Self {
        Self {
            vendor_id: vendor_id.into(),
            command: command.into(),
            args: Vec::new(),
            env: BTreeMap::new(),
            cwd: cwd.into(),
        }
    }

    #[must_use]
    pub fn args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.args = args.into_iter().map(Into::into).collect();
        self
    }

    #[must_use]
    pub fn envs(
        mut self,
        env: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        self.env.extend(
            env.into_iter()
                .map(|(name, value)| (name.into(), value.into())),
        );
        self
    }
}

pub struct AcpProviderExecutor {
    transport: AcpStdioTransport,
    pub profile: AcpProviderProfile,
    available: bool,
    models: Vec<AcpProviderModel>,
}

impl AcpProviderExecutor {
    pub fn connect(profile: AcpProviderProfile) -> Result<Self, katana_acp_client::AcpError> {
        let config = AcpStdioTransportConfig::new(profile.command.clone(), profile.cwd.clone())
            .args(profile.args.clone())
            .envs(profile.env.clone());
        Ok(Self {
            transport: AcpStdioTransport::spawn(config)?,
            profile,
            available: false,
            models: Vec::new(),
        })
    }

    #[must_use]
    pub fn is_available(&self) -> bool {
        self.available
    }

    pub async fn prompt(
        &self,
        blocks: Vec<AcpContentBlock>,
    ) -> Result<(), katana_acp_client::AcpError> {
        self.transport.prompt(blocks).await
    }

    pub async fn cancel(&self) -> Result<(), katana_acp_client::AcpError> {
        self.transport.cancel().await
    }

    pub async fn select_model(
        &self,
        model: &AcpProviderModel,
    ) -> Result<(), katana_acp_client::AcpError> {
        if !self.available || !self.models.contains(model) {
            return Err(katana_acp_client::AcpError::Protocol(
                "ACP model is not available in this session".to_string(),
            ));
        }
        self.transport
            .set_config(model.config_id.clone(), model.id.clone())
            .await
    }

    pub async fn select_model_id(&self, id: &str) -> Result<(), katana_acp_client::AcpError> {
        let model = self
            .models
            .iter()
            .find(|model| model.id == id)
            .ok_or_else(|| katana_acp_client::AcpError::Protocol("Unknown ACP model".into()))?;
        self.select_model(model).await
    }

    pub async fn choose_permission(
        &self,
        request_id: u64,
        decision: AcpPermissionDecision,
    ) -> Result<(), katana_acp_client::AcpError> {
        self.transport
            .respond_permission(request_id, decision)
            .await
    }

    pub async fn next_projection(&mut self) -> Option<AcpProviderProjection> {
        let projection = self.transport.recv().await.map(project)?;
        self.record(&projection);
        Some(projection)
    }

    pub fn try_next_projection(&mut self) -> Option<AcpProviderProjection> {
        let projection = self.transport.try_recv().map(project)?;
        self.record(&projection);
        Some(projection)
    }

    fn record(&mut self, projection: &AcpProviderProjection) {
        match projection {
            AcpProviderProjection::Ready { models } => {
                self.available = true;
                self.models = models.clone();
            }
            AcpProviderProjection::Unavailable(_) | AcpProviderProjection::Failed(_) => {
                self.available = false;
                self.models.clear();
            }
            _ => {}
        }
    }
}
