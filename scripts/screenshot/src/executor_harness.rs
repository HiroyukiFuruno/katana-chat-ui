use crate::{fixture, request::Request};
use anyhow::{Context, Result};
use eframe::egui;
use egui_kittest::Harness;
use katana_chat_ui::ChatUiSurface;
use katana_chat_ui_egui::{EguiChatController, EguiChatView};
use sha2::{Digest, Sha256};
use std::path::Path;

const WARMUP_FRAMES: usize = 10;
const PIXELS_PER_POINT: f32 = 2.0;

pub fn run(request: &Request, output_dir: &Path) -> Result<()> {
    let surface = fixture::surface(&request.scenario)?;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(
            request.viewport.width as f32,
            request.viewport.height as f32,
        ))
        .with_pixels_per_point(PIXELS_PER_POINT)
        .build_eframe(|_context| ScreenshotApp::new(surface));

    for _ in 0..WARMUP_FRAMES {
        harness.step();
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

fn validate_screenshot_hash(output_path: &Path, expected_hash: &str) -> Result<()> {
    let image_bytes = std::fs::read(output_path)
        .with_context(|| format!("failed to read screenshot: {}", output_path.display()))?;
    let actual_hash = format!("{:x}", Sha256::digest(&image_bytes));
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
}

impl ScreenshotApp {
    fn new(surface: ChatUiSurface) -> Self {
        let composer_text = surface.composer.text.clone();
        Self {
            surface,
            controller: ScreenshotController { composer_text },
        }
    }
}

impl eframe::App for ScreenshotApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        EguiChatView::render(ui, &self.surface, &mut self.controller);
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
