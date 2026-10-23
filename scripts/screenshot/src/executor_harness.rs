use crate::{fixture, hash::sha256_hex, request::Request};
use anyhow::{Context, Result};
use eframe::egui;
use egui_kittest::Harness;
use katana_chat_ui::ChatUiSurface;
use katana_chat_ui_egui::{EguiChatController, EguiKucView};
use std::path::Path;

const WARMUP_FRAMES: usize = 10;
const PIXELS_PER_POINT: f32 = 2.0;
const HEADLESS_MAX_TEXTURE_SIDE: usize = 4096;

pub fn run(request: &Request, output_dir: &Path) -> Result<()> {
    validate_texture_capacity(request)?;
    let surface = fixture::surface(&request.scenario)?;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(
            request.viewport.width as f32,
            request.viewport.height as f32,
        ))
        .with_pixels_per_point(PIXELS_PER_POINT)
        .build_eframe(|_context| ScreenshotApp::new(surface));
    harness.input_mut().max_texture_side = Some(HEADLESS_MAX_TEXTURE_SIDE);

    for _ in 0..WARMUP_FRAMES {
        harness.step();
    }

    if let Some(error) = &harness.state().render_error {
        anyhow::bail!("KUC rendering failed: {error}");
    }

    let output_path = output_dir.join(&request.output_name);
    let image = harness
        .render()
        .map_err(|error| anyhow::anyhow!("render failed: {error}"))?;
    image
        .save(&output_path)
        .with_context(|| format!("failed to save screenshot: {}", output_path.display()))?;
    validate_screenshot_hash(&output_path, &request.baseline_sha256)?;
    Ok(())
}

fn validate_texture_capacity(request: &Request) -> Result<()> {
    let scale = PIXELS_PER_POINT as u32;
    let width = request.viewport.width.checked_mul(scale);
    let height = request.viewport.height.checked_mul(scale);
    if width.is_none_or(|side| side as usize > HEADLESS_MAX_TEXTURE_SIDE)
        || height.is_none_or(|side| side as usize > HEADLESS_MAX_TEXTURE_SIDE)
    {
        anyhow::bail!("viewport exceeds the headless texture limit");
    }
    Ok(())
}

fn validate_screenshot_hash(output_path: &Path, expected_hash: &str) -> Result<()> {
    let image_bytes = std::fs::read(output_path)
        .with_context(|| format!("failed to read screenshot: {}", output_path.display()))?;
    let actual_hash = sha256_hex(&image_bytes);
    if actual_hash != expected_hash {
        anyhow::bail!(
            "screenshot baseline mismatch for {}: expected {}, actual {}",
            output_path.display(),
            expected_hash,
            actual_hash
        );
    }
    Ok(())
}

struct ScreenshotApp {
    surface: ChatUiSurface,
    controller: ScreenshotController,
    view: EguiKucView,
    render_error: Option<String>,
}

impl ScreenshotApp {
    fn new(surface: ChatUiSurface) -> Self {
        let composer_text = surface.composer.text.clone();
        Self {
            surface,
            controller: ScreenshotController { composer_text },
            view: EguiKucView::default(),
            render_error: None,
        }
    }
}

impl eframe::App for ScreenshotApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Harness の初回描画は build_eframe 内で走るため、そのフレームにも上限を渡す。
        ui.ctx().input_mut(|input| {
            input.max_texture_side = HEADLESS_MAX_TEXTURE_SIDE;
        });
        if let Err(error) = self.view.render(ui, &self.surface, &mut self.controller) {
            self.render_error = Some(error);
        }
    }
}

struct ScreenshotController {
    composer_text: String,
}

impl EguiChatController for ScreenshotController {
    fn composer_text_mut(&mut self) -> &mut String {
        &mut self.composer_text
    }

    fn attach(&mut self) {}

    fn submit(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn stop(&mut self) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::validate_texture_capacity;
    use crate::request::{Request, Scenario, Viewport};

    #[test]
    fn standard_hidpi_viewport_fits_the_declared_headless_capability() {
        let mut request = request();
        request.viewport = Viewport {
            width: 1440,
            height: 900,
        };
        assert!(validate_texture_capacity(&request).is_ok());
    }

    #[test]
    fn physical_texture_overflow_is_rejected_before_harness_allocation() {
        let mut request = request();
        request.viewport = Viewport {
            width: 2049,
            height: 900,
        };
        assert!(validate_texture_capacity(&request).is_err());
    }

    fn request() -> Request {
        Request {
            schema_version: "1".into(),
            name: "texture-capacity".into(),
            viewport: Viewport::default(),
            scenario: Scenario::default(),
            output_name: "texture-capacity.png".into(),
            baseline_sha256: "0".repeat(64),
        }
    }
}
