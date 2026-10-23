use super::{AcpContentEncoder, AcpPromptCapabilities};
use crate::{AcpContentBlock, AcpEmbeddedResource, AcpImageContent, AcpTextContent};
use serde_json::{Value, from_str};

#[test]
fn wire_text_is_typed_without_changing_internal_storage() -> Result<(), Box<dyn std::error::Error>>
{
    let block = AcpContentBlock::Text(AcpTextContent::new("日本語"));
    let wire = AcpContentEncoder::encode(
        std::slice::from_ref(&block),
        AcpPromptCapabilities::default(),
    )?;
    assert_eq!(
        serde_json::to_value(wire)?,
        from_str::<Value>(r#"[{"type":"text","text":"日本語"}]"#)?
    );
    assert_eq!(
        serde_json::to_value(block)?,
        from_str::<Value>(r#"{"Text":{"text":"日本語"}}"#)?
    );
    Ok(())
}

#[test]
fn embedded_context_uses_official_nested_resource_schema() -> Result<(), Box<dyn std::error::Error>>
{
    let blocks = [AcpContentBlock::EmbeddedResource(AcpEmbeddedResource::new(
        "file:///workspace/tmp/note.md",
        "text/markdown",
        "# note",
    ))];
    assert!(AcpContentEncoder::encode(&blocks, AcpPromptCapabilities::default()).is_err());
    let capabilities = AcpPromptCapabilities {
        embedded_context: true,
        image: false,
    };
    let wire = AcpContentEncoder::encode(&blocks, capabilities)?;
    assert_eq!(
        serde_json::to_value(wire)?,
        from_str::<Value>(
            r##"[{
            "type":"resource", "resource": {
                "uri":"file:///workspace/tmp/note.md", "mimeType":"text/markdown", "text":"# note"
            }
        }]"##
        )?
    );
    Ok(())
}

#[test]
fn images_require_negotiation_and_real_base64_payload() -> Result<(), Box<dyn std::error::Error>> {
    let blocks = [AcpContentBlock::Image(AcpImageContent::new(
        "image/png",
        "data:image/png;base64,YQ==",
    ))];
    assert!(AcpContentEncoder::encode(&blocks, AcpPromptCapabilities::default()).is_err());
    let capabilities = AcpPromptCapabilities {
        image: true,
        embedded_context: false,
    };
    let wire = AcpContentEncoder::encode(&blocks, capabilities)?;
    assert_eq!(
        serde_json::to_value(wire)?,
        from_str::<Value>(r#"[{"type":"image","mimeType":"image/png","data":"YQ=="}]"#)?
    );
    for reference in [
        "file:///tmp/a.png",
        "data:image/jpeg;base64,YQ==",
        "data:image/png;base64,???",
    ] {
        let invalid = [AcpContentBlock::Image(AcpImageContent::new(
            "image/png",
            reference,
        ))];
        assert!(AcpContentEncoder::encode(&invalid, capabilities).is_err());
    }
    Ok(())
}
