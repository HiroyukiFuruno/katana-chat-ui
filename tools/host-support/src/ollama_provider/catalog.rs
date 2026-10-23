use katana_acp_client::{AcpError, AiModel};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KatanAgentOllamaModel {
    pub id: String,
    pub name: String,
}

pub(super) fn normalize(models: Vec<AiModel>) -> Result<Vec<KatanAgentOllamaModel>, AcpError> {
    let mut ids = BTreeSet::new();
    let mut catalog = Vec::new();
    for model in models {
        if model.id.trim().is_empty() {
            return Err(AcpError::Protocol(
                "Ollama model catalog contains an empty id".to_string(),
            ));
        }
        if ids.insert(model.id.clone()) {
            catalog.push(KatanAgentOllamaModel {
                id: model.id,
                name: model.name,
            });
        }
    }
    if catalog.is_empty() {
        return Err(AcpError::Protocol(
            "Ollama model catalog is empty".to_string(),
        ));
    }
    Ok(catalog)
}

pub(super) fn model_id(models: &[KatanAgentOllamaModel], id: &str) -> Result<String, AcpError> {
    if models.is_empty() {
        return Err(AcpError::Protocol(
            "Ollama model catalog has not been loaded".to_string(),
        ));
    }
    models
        .iter()
        .find(|model| model.id == id)
        .map(|model| model.id.clone())
        .ok_or_else(|| AcpError::Protocol(format!("KatanAgent model is not available: {id}")))
}

#[cfg(test)]
mod tests {
    use super::normalize;
    use katana_acp_client::{AcpError, AiModel};

    #[test]
    fn catalog_preserves_first_unique_model() -> Result<(), AcpError> {
        let catalog = normalize(vec![
            AiModel {
                id: "llama3".to_string(),
                name: "Llama 3".to_string(),
            },
            AiModel {
                id: "llama3".to_string(),
                name: "Duplicate".to_string(),
            },
            AiModel {
                id: "gemma".to_string(),
                name: "Gemma".to_string(),
            },
        ])?;
        assert_eq!(catalog.len(), 2);
        assert_eq!(catalog[0].name, "Llama 3");
        Ok(())
    }

    #[test]
    fn empty_catalog_is_explicitly_rejected() {
        assert!(matches!(normalize(Vec::new()), Err(AcpError::Protocol(_))));
    }
}
