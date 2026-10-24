const STANDARD: &str = include_str!("../../../../scripts/screenshot/examples/standard-chat.json");
const EMPTY: &str = include_str!("../../../../scripts/screenshot/examples/empty-chat.json");
const THINKING: &str = include_str!("../../../../scripts/screenshot/examples/thinking-chat.json");
const NARROW: &str = include_str!("../../../../scripts/screenshot/examples/narrow-chat.json");
const TALL: &str = include_str!("../../../../scripts/screenshot/examples/tall-chat.json");
const JUSTFILE: &str = include_str!("../../../../justfile");

#[test]
fn screenshot_requests_keep_exact_visual_baselines() -> Result<(), String> {
    for request in requests()? {
        let hash = request
            .get("baseline_sha256")
            .and_then(serde_json::Value::as_str)
            .ok_or("screenshot request has no baseline hash")?;
        if hash.len() != 64 || !hash.chars().all(|value| value.is_ascii_hexdigit()) {
            return Err("screenshot baseline hash is not SHA-256".into());
        }
    }
    Ok(())
}

#[test]
fn screenshot_matrix_covers_required_visual_states() -> Result<(), String> {
    let names = requests()?
        .iter()
        .map(request_name)
        .collect::<Result<Vec<_>, _>>()?;
    for expected in [
        "empty-chat",
        "standard-chat",
        "thinking-chat",
        "narrow-chat",
        "tall-chat",
    ] {
        if !names.iter().any(|name| name == expected) {
            return Err(format!("screenshot matrix omits {expected}"));
        }
    }
    Ok(())
}

#[test]
fn visible_native_capture_remains_opt_in() -> Result<(), String> {
    let headless = recipe_body("harness-native-screenshot-check")?;
    let visible = recipe_body("harness-visible-native-screenshot-check")?;
    if !headless.contains("harness-screenshot-matrix-check") || headless.contains("--native-host") {
        return Err("default native screenshot check is not headless".into());
    }
    if !visible.contains("KCU_ALLOW_VISIBLE_WINDOWS") || !visible.contains("--native-host") {
        return Err("visible native screenshot check is not explicitly gated".into());
    }
    Ok(())
}

fn requests() -> Result<Vec<serde_json::Value>, String> {
    [STANDARD, EMPTY, THINKING, NARROW, TALL]
        .into_iter()
        .map(|source| serde_json::from_str(source).map_err(|error| error.to_string()))
        .collect()
}

fn request_name(request: &serde_json::Value) -> Result<String, String> {
    request
        .get("name")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| "screenshot request has no name".into())
}

fn recipe_body(name: &str) -> Result<String, String> {
    let mut lines = Vec::new();
    let mut active = false;
    for line in JUSTFILE.lines() {
        if line.starts_with(name) && line.contains(':') {
            active = true;
        } else if active && is_recipe_header(line) {
            break;
        }
        if active {
            lines.push(line);
        }
    }
    if lines.is_empty() {
        return Err(format!("missing just recipe: {name}"));
    }
    Ok(lines.join("\n"))
}

fn is_recipe_header(line: &str) -> bool {
    !line.is_empty() && !line.starts_with(' ') && !line.starts_with('\t')
}
