use super::OllamaProvider;
use crate::AcpError;

#[test]
fn test_endpoint_normalization() -> Result<(), AcpError> {
    let default_provider = OllamaProvider::new(None, None)?;
    assert_eq!(default_provider.endpoint, "http://localhost:11434");

    let custom_provider = OllamaProvider::new(Some("http://127.0.0.1:11434/".to_string()), None)?;
    assert_eq!(custom_provider.endpoint, "http://127.0.0.1:11434");
    Ok(())
}
