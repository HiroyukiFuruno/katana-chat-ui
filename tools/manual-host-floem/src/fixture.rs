use std::{env, fs, path::PathBuf};

use floem::{
    AnyView, IntoView,
    peniko::Color,
    reactive::{RwSignal, SignalUpdate},
    views::{Decorators, container, label, stack},
};
use katana_chat_ui::ChatUiSurface;
use katana_chat_ui_floem::{FloemChatView, FloemKucCallbacks};

pub(crate) struct Fixture {
    pub(crate) surface: ChatUiSurface,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

pub(crate) fn load_from_args() -> Result<Option<Fixture>, String> {
    fixture_path().and_then(|path| path.map(load_fixture).transpose())
}

fn fixture_path() -> Result<Option<(PathBuf, u32, u32)>, String> {
    let mut args = env::args_os().skip(1);
    let Some(flag) = args.next() else {
        return Ok(None);
    };
    let path = args.next().ok_or_else(usage)?;
    if flag != "--kcu-surface" || !PathBuf::from(&path).is_absolute() {
        return Err(usage());
    }
    let dimensions = match args.next() {
        None => (1280, 900),
        Some(flag) if flag == "--kcu-viewport" => {
            let width = dimension(args.next().ok_or_else(usage)?, 320)?;
            let height = dimension(args.next().ok_or_else(usage)?, 240)?;
            (width, height)
        }
        Some(_) => return Err(usage()),
    };
    if args.next().is_some() {
        return Err(usage());
    }
    Ok(Some((PathBuf::from(path), dimensions.0, dimensions.1)))
}

fn dimension(value: std::ffi::OsString, minimum: u32) -> Result<u32, String> {
    let value = value
        .to_string_lossy()
        .parse::<u32>()
        .map_err(|_| usage())?;
    if !(minimum..=4096).contains(&value) {
        return Err(usage());
    }
    Ok(value)
}

fn usage() -> String {
    "usage: manual-host-floem --kcu-surface <absolute JSON> [--kcu-viewport <width> <height>]"
        .into()
}

fn load_fixture((path, width, height): (PathBuf, u32, u32)) -> Result<Fixture, String> {
    let path = path.canonicalize().map_err(|error| error.to_string())?;
    let source = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let surface = serde_json::from_str(&source).map_err(|error| error.to_string())?;
    Ok(Fixture {
        surface,
        width,
        height,
    })
}

pub(crate) fn view(surface: ChatUiSurface) -> Result<AnyView, String> {
    let draft = RwSignal::new(surface.composer.text.clone());
    let surface = RwSignal::new(surface);
    FloemChatView::mount(
        surface,
        draft,
        1.0,
        FloemKucCallbacks {
            on_action: |action| eprintln!("UI fixture action unsupported: {action:?}"),
            on_draft: move |text| draft.set(text),
        },
    )
    .map(|view| container(view).style(style).into_any())
    .map_err(|error| error.to_string())
}

pub(crate) fn renderer_control_enabled() -> Result<bool, String> {
    renderer_control_policy(
        env::var("KCU_IME_FLOEM_RENDERER_CONTROL").ok(),
        env::var("GITHUB_ACTIONS").ok(),
        env::var("RUNNER_OS").ok(),
    )
}

fn renderer_control_policy(
    control: Option<String>,
    github_actions: Option<String>,
    runner_os: Option<String>,
) -> Result<bool, String> {
    match control.as_deref() {
        None => Ok(false),
        Some("1")
            if github_actions.as_deref() == Some("true")
                && runner_os.as_deref() == Some("macOS") =>
        {
            Ok(true)
        }
        Some("1") => Err(
            "KCU_IME_FLOEM_RENDERER_CONTROL=1 requires GITHUB_ACTIONS=true and RUNNER_OS=macOS"
                .into(),
        ),
        Some(value) => Err(format!(
            "KCU_IME_FLOEM_RENDERER_CONTROL must be 1 when set, got {value:?}"
        )),
    }
}

pub(crate) fn with_renderer_control(view: AnyView) -> AnyView {
    stack((
        view,
        label(|| "FLOEM RENDERER CONTROL").style(|style| {
            style
                .absolute()
                .inset_left(20.0)
                .inset_top(100.0)
                .padding(8.0)
                .color(Color::BLACK)
                .background(Color::rgb8(248, 248, 248))
                .z_index(10)
        }),
    ))
    .style(|style| style.size_full().min_width(0.0).min_height(0.0))
    .into_any()
}

fn style(style: floem::style::Style) -> floem::style::Style {
    style.size_full().min_width(0.0).min_height(0.0)
}

#[cfg(test)]
mod tests {
    use super::{dimension, renderer_control_policy};
    use std::ffi::OsString;

    #[test]
    fn dimensions_match_request_viewport_bounds() -> Result<(), String> {
        if dimension(OsString::from("320"), 320)? != 320
            || dimension(OsString::from("240"), 240)? != 240
        {
            return Err("accepted dimensions changed".into());
        }
        for (value, minimum) in [
            ("0", 320),
            ("-1", 320),
            ("NaN", 320),
            ("inf", 320),
            ("1.5", 320),
            ("4097", 320),
            ("319", 320),
            ("239", 240),
        ] {
            if dimension(OsString::from(value), minimum).is_ok() {
                return Err(format!("accepted invalid dimension: {value}"));
            }
        }
        Ok(())
    }

    #[test]
    fn renderer_control_requires_github_macos_context() {
        assert_eq!(renderer_control_policy(None, None, None), Ok(false));
        assert_eq!(
            renderer_control_policy(Some("1".into()), Some("true".into()), Some("macOS".into())),
            Ok(true)
        );
        assert!(
            renderer_control_policy(Some("1".into()), Some("true".into()), Some("Linux".into()))
                .is_err()
        );
        assert!(
            renderer_control_policy(Some("0".into()), Some("true".into()), Some("macOS".into()))
                .is_err()
        );
    }
}
