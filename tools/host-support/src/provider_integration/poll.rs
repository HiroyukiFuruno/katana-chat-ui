use crate::{ChatHostError, ChatHostState};

use super::AcpHostBridge;

impl AcpHostBridge {
    pub async fn poll(&mut self, host: &mut ChatHostState) -> Result<bool, ChatHostError> {
        let Some(projection) = self.executor.next_projection().await else {
            return Ok(false);
        };
        self.apply_projection(host, projection)?;
        host.persist()?;
        Ok(true)
    }

    pub fn try_poll(&mut self, host: &mut ChatHostState) -> Result<bool, ChatHostError> {
        let Some(projection) = self.try_next_projection() else {
            return Ok(false);
        };
        self.apply_projection(host, projection)?;
        host.persist()?;
        Ok(true)
    }
}
