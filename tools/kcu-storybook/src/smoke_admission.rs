const NON_LINUX_ERROR: &str = "automated Storybook smoke must run in the isolated Linux/Xvfb container; use `just storybook-window-smoke`";
const LINUX_DISPLAY_ERROR: &str = "automated Storybook smoke requires the isolated Linux/Xvfb display; use `just storybook-window-smoke`";

pub(crate) fn validate(smoke: bool) -> eframe::Result<()> {
    let rejection = rejection_reason(
        smoke,
        std::env::consts::OS,
        std::env::var("KCU_X11_HEADLESS").ok().as_deref(),
        std::env::var("DISPLAY").ok().as_deref(),
        std::env::var("WAYLAND_DISPLAY").ok().as_deref(),
    );
    match rejection {
        Some(message) => Err(eframe::Error::AppCreation(Box::new(std::io::Error::other(
            message,
        )))),
        None => Ok(()),
    }
}

fn rejection_reason(
    smoke: bool,
    target_os: &str,
    headless_display: Option<&str>,
    display: Option<&str>,
    wayland_display: Option<&str>,
) -> Option<&'static str> {
    if !smoke {
        return None;
    }
    if target_os != "linux" {
        return Some(NON_LINUX_ERROR);
    }
    if headless_display != Some("1")
        || !display.is_some_and(|value| !value.is_empty())
        || wayland_display.is_some()
    {
        return Some(LINUX_DISPLAY_ERROR);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{LINUX_DISPLAY_ERROR, NON_LINUX_ERROR, rejection_reason};

    #[test]
    fn interactive_storybook_is_not_restricted_by_smoke_environment() {
        assert_eq!(
            rejection_reason(false, "macos", None, None, Some("wayland-0")),
            None
        );
    }

    #[test]
    fn smoke_rejects_non_linux_before_creating_the_native_app() {
        assert_eq!(
            rejection_reason(true, "macos", Some("1"), Some(":99"), None),
            Some(NON_LINUX_ERROR)
        );
        assert_eq!(
            rejection_reason(true, "windows", Some("1"), Some(":99"), None),
            Some(NON_LINUX_ERROR)
        );
    }

    #[test]
    fn smoke_requires_the_isolated_x11_display_without_wayland() {
        for (headless_display, display, wayland_display) in [
            (None, Some(":99"), None),
            (Some("1"), None, None),
            (Some("1"), Some(""), None),
            (Some("1"), Some(":99"), Some("wayland-0")),
        ] {
            assert_eq!(
                rejection_reason(true, "linux", headless_display, display, wayland_display),
                Some(LINUX_DISPLAY_ERROR)
            );
        }
        assert_eq!(
            rejection_reason(true, "linux", Some("1"), Some(":99"), None),
            None
        );
    }
}
