//! Per-user folders for FileWheel's settings, caches and logs.
//!
//! Linux follows the XDG base-directory variables; Windows uses
//! `%APPDATA%\FileWheel` for settings and `%LOCALAPPDATA%\FileWheel` for the
//! rest.
use std::path::PathBuf;

#[cfg(not(windows))]
fn xdg(variable: &str, fallback: &str) -> Option<PathBuf> {
    std::env::var_os(variable)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(fallback)))
        .map(|root| root.join("filewheel"))
}

#[cfg(windows)]
fn known(variable: &str) -> Option<PathBuf> {
    std::env::var_os(variable)
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .map(|root| root.join("FileWheel"))
}

/// Settings such as `appearance.conf`.
pub fn config_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    return known("APPDATA");
    #[cfg(not(windows))]
    return xdg("XDG_CONFIG_HOME", ".config");
}

/// Disposable data, such as the LibreOffice profile.
pub fn cache_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    return known("LOCALAPPDATA");
    #[cfg(not(windows))]
    return xdg("XDG_CACHE_HOME", ".cache");
}

/// The job log.
pub fn state_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    return known("LOCALAPPDATA");
    #[cfg(not(windows))]
    return xdg("XDG_STATE_HOME", ".local/state");
}
