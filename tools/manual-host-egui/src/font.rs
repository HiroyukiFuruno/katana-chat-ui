use eframe::egui;
use std::fs;
use std::sync::Arc;

const FONT_NAME: &str = "kcu-japanese";
const FONT_PATHS: [&str; 3] = [
    "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
    "/System/Library/Fonts/AppleSDGothicNeo.ttc",
    "/System/Library/Fonts/CJKSymbolsFallback.ttc",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManualFontStatus {
    Installed,
    Unavailable,
}

pub struct ManualFontInstaller;

impl ManualFontInstaller {
    pub fn install(context: &egui::Context) -> ManualFontStatus {
        match Self::load_font() {
            Some((path, bytes)) => {
                context.set_fonts(Self::font_definitions(bytes));
                let _ = path;
                ManualFontStatus::Installed
            }
            None => ManualFontStatus::Unavailable,
        }
    }

    fn load_font() -> Option<(String, Vec<u8>)> {
        for path in FONT_PATHS {
            if let Ok(bytes) = fs::read(path) {
                return Some((path.to_string(), bytes));
            }
        }
        None
    }

    fn font_definitions(bytes: Vec<u8>) -> egui::FontDefinitions {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            FONT_NAME.to_string(),
            Arc::new(egui::FontData::from_owned(bytes)),
        );
        Self::install_family(&mut fonts, egui::FontFamily::Proportional);
        Self::install_family(&mut fonts, egui::FontFamily::Monospace);
        fonts
    }

    fn install_family(fonts: &mut egui::FontDefinitions, family: egui::FontFamily) {
        if let Some(entries) = fonts.families.get_mut(&family) {
            entries.insert(0, FONT_NAME.to_string());
        }
    }
}
