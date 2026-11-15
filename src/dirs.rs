//! Where Kadence keeps its files, in each system's usual place.

use std::path::PathBuf;

/// The settings folder: `~/.config/kadence` (or `$XDG_CONFIG_HOME`), `%APPDATA%\kadence`,
/// or `~/Library/Application Support/kadence`.
pub fn config() -> Option<PathBuf> {
    base("APPDATA", "Library/Application Support", "XDG_CONFIG_HOME", ".config")
}

/// The cache folder: `~/.cache/kadence` (or `$XDG_CACHE_HOME`), `%LOCALAPPDATA%\kadence`,
/// or `~/Library/Caches/kadence`.
pub fn cache() -> Option<PathBuf> {
    base("LOCALAPPDATA", "Library/Caches", "XDG_CACHE_HOME", ".cache")
}

fn base(windows: &str, macos: &str, xdg: &str, home: &str) -> Option<PathBuf> {
    let env = |k: &str| std::env::var_os(k).filter(|v| !v.is_empty()).map(PathBuf::from);
    let base = if cfg!(windows) {
        env(windows)
    } else if cfg!(target_os = "macos") {
        env("HOME").map(|h| h.join(macos))
    } else {
        env(xdg).or_else(|| env("HOME").map(|h| h.join(home)))
    }?;
    Some(base.join("kadence"))
}
