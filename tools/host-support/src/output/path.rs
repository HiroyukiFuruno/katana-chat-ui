use std::{
    fs,
    io::ErrorKind,
    path::{Component, Path, PathBuf},
};

pub(super) fn target_path(
    root: &Path,
    requested: &str,
    create_parents: bool,
) -> Result<PathBuf, String> {
    let root = canonical_root(root)?;
    let tmp = tmp_root(&root)?;
    let target = requested_target(&root, &tmp, requested)?;
    validate_target(&tmp, &target, create_parents)?;
    Ok(target)
}

fn canonical_root(root: &Path) -> Result<PathBuf, String> {
    fs::create_dir_all(root).map_err(|error| error.to_string())?;
    let metadata = fs::symlink_metadata(root).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!(
            "output root is not a directory: {}",
            root.display()
        ));
    }
    fs::canonicalize(root).map_err(|error| error.to_string())
}

fn tmp_root(root: &Path) -> Result<PathBuf, String> {
    let tmp = root.join("tmp");
    ensure_directory(&tmp, true)?;
    let tmp = fs::canonicalize(tmp).map_err(|error| error.to_string())?;
    tmp.starts_with(root)
        .then_some(tmp)
        .ok_or_else(|| "temporary output root escaped configured root".to_string())
}

fn requested_target(root: &Path, tmp: &Path, requested: &str) -> Result<PathBuf, String> {
    let requested = Path::new(requested);
    let joined = if requested.is_absolute() {
        requested.to_path_buf()
    } else {
        root.join(requested)
    };
    let target = normalize(&joined)?;
    if target == tmp || !target.starts_with(tmp) {
        return Err(format!(
            "files are allowed only under {}: {}",
            tmp.display(),
            requested.display()
        ));
    }
    Ok(target)
}

fn validate_target(tmp: &Path, target: &Path, create_parents: bool) -> Result<(), String> {
    let relative = target
        .strip_prefix(tmp)
        .map_err(|error| error.to_string())?;
    let parts = relative.components().collect::<Vec<_>>();
    let (_, parents) = parts
        .split_last()
        .ok_or_else(|| "output file name is missing".to_string())?;
    let mut current = tmp.to_path_buf();
    for parent in parents {
        current.push(component_name(*parent)?);
        ensure_directory(&current, create_parents)?;
    }
    ensure_not_symlink(target)
}

fn component_name(component: Component<'_>) -> Result<&std::ffi::OsStr, String> {
    match component {
        Component::Normal(part) => Ok(part),
        _ => Err("output path contains an unsafe component".to_string()),
    }
}

fn ensure_directory(path: &Path, create: bool) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => Err(format!(
            "output path is not a directory: {}",
            path.display()
        )),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound && create => {
            fs::create_dir(path).map_err(|create_error| create_error.to_string())
        }
        Err(error) => Err(error.to_string()),
    }
}

fn ensure_not_symlink(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            Err(format!("output target is a symlink: {}", path.display()))
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

fn normalize(path: &Path) -> Result<PathBuf, String> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(Path::new("/")),
            Component::CurDir => {}
            Component::ParentDir if !normalized.pop() => {
                return Err(format!("path escapes root: {}", path.display()));
            }
            Component::ParentDir => {}
            Component::Normal(part) => normalized.push(part),
        }
    }
    Ok(normalized)
}
