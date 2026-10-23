use crate::{fixture, request};
use anyhow::{Context, Result, ensure};
use std::path::{Path, PathBuf};

const MAX_NATIVE_SIDE: u32 = 4096;

pub struct PreparedNativeFixture {
    pub surface_path: PathBuf,
    pub viewport: request::Viewport,
}

pub fn prepare(
    request_path: Option<&Path>,
    output_dir: &Path,
) -> Result<Option<PreparedNativeFixture>> {
    let Some(request_path) = request_path else {
        return Ok(None);
    };
    let request = request::load(request_path)?;
    validate_viewport(&request.viewport)?;
    let surface = fixture::surface(&request.scenario)?;
    let path = output_dir.join("native-surface.json");
    let bytes = serde_json::to_vec(&surface).context("cannot serialize visual fixture")?;
    std::fs::write(&path, bytes).context("cannot save native visual fixture")?;
    Ok(Some(PreparedNativeFixture {
        surface_path: path.canonicalize()?,
        viewport: request.viewport,
    }))
}

fn validate_viewport(viewport: &request::Viewport) -> Result<()> {
    ensure!(
        viewport.width >= 320 && viewport.height >= 240,
        "native viewport is too small"
    );
    ensure!(
        viewport.width <= MAX_NATIVE_SIDE && viewport.height <= MAX_NATIVE_SIDE,
        "native viewport exceeds {MAX_NATIVE_SIDE}px"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_viewport;
    use crate::request::Viewport;
    #[test]
    fn accepts_requested_sizes() {
        assert!(
            validate_viewport(&Viewport {
                width: 430,
                height: 900
            })
            .is_ok()
        );
    }
    #[test]
    fn rejects_zero_and_huge_sizes() {
        assert!(
            validate_viewport(&Viewport {
                width: 0,
                height: 900
            })
            .is_err()
        );
        assert!(
            validate_viewport(&Viewport {
                width: 4097,
                height: 900
            })
            .is_err()
        );
    }
}
