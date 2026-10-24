use anyhow::{Context, Result};
use std::{path::Path, process::Command};

pub(super) fn visible_pixels(output_path: &Path) -> Result<bool> {
    let output = Command::new("identify")
        .args(["-format", "%k"])
        .arg(output_path)
        .output()
        .context("failed to launch ImageMagick identify")?;
    anyhow::ensure!(
        output.status.success(),
        "ImageMagick identify failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    let colors = unique_color_count(&output.stdout)?;
    Ok(is_nonuniform(colors))
}

fn unique_color_count(output: &[u8]) -> Result<u64> {
    String::from_utf8_lossy(output)
        .trim()
        .parse()
        .context("ImageMagick identify returned an invalid color count")
}

fn is_nonuniform(colors: u64) -> bool {
    colors > 1
}

#[cfg(test)]
mod tests {
    use super::is_nonuniform;

    #[test]
    fn single_color_frame_is_not_ready() {
        assert!(!is_nonuniform(1));
    }

    #[test]
    fn multiple_colors_make_a_frame_ready() {
        assert!(is_nonuniform(2));
    }
}
