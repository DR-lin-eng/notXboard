use std::path::{Path, PathBuf};

const DEFAULT_RUNTIME_ROOT: &str = "/app/runtime";
const DEFAULT_STATE_ROOT: &str = "/app/state";

pub(crate) fn runtime_root() -> PathBuf {
    std::env::var("RUST_RUNTIME_ROOT")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_RUNTIME_ROOT))
}

pub(crate) fn state_root() -> PathBuf {
    std::env::var("RUST_STATE_ROOT")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_STATE_ROOT))
}

pub(crate) fn runtime_path(relative: impl AsRef<Path>) -> PathBuf {
    runtime_root().join(relative)
}

pub(crate) fn state_path(relative: impl AsRef<Path>) -> PathBuf {
    state_root().join(relative)
}

pub(crate) fn resources_path(relative: impl AsRef<Path>) -> PathBuf {
    runtime_path(Path::new("resources").join(relative))
}

pub(crate) fn public_path(relative: impl AsRef<Path>) -> PathBuf {
    runtime_path(Path::new("public").join(relative))
}

pub(crate) fn themes_path(relative: impl AsRef<Path>) -> PathBuf {
    runtime_path(Path::new("theme").join(relative))
}

pub(crate) fn plugins_path(relative: impl AsRef<Path>) -> PathBuf {
    runtime_path(Path::new("plugins").join(relative))
}

pub(crate) fn state_plugins_path(relative: impl AsRef<Path>) -> PathBuf {
    state_path(Path::new("plugins").join(relative))
}
