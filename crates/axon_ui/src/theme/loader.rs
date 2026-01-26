//! 主题加载器
//!
//! 从文件系统加载 JSON 主题文件

use super::Theme;
use super::theme_serde::SerializableTheme;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info, warn};

/// 主题加载错误
#[derive(Debug)]
pub enum ThemeLoadError {
    /// 文件系统错误
    Io(std::io::Error),
    /// JSON 解析错误
    Json(serde_json::Error),
    /// 颜色解析错误
    ColorParse(super::theme_serde::ColorParseError),
}

impl std::fmt::Display for ThemeLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {}", e),
            Self::Json(e) => write!(f, "JSON parse error: {}", e),
            Self::ColorParse(e) => write!(f, "Color parse error: {}", e),
        }
    }
}

impl std::error::Error for ThemeLoadError {}

impl From<std::io::Error> for ThemeLoadError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for ThemeLoadError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<super::theme_serde::ColorParseError> for ThemeLoadError {
    fn from(e: super::theme_serde::ColorParseError) -> Self {
        Self::ColorParse(e)
    }
}

/// 主题加载器
pub struct ThemeLoader;

impl ThemeLoader {
    /// 从 JSON 文件加载主题
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Theme, ThemeLoadError> {
        let path = path.as_ref();
        debug!("Loading theme from: {}", path.display());

        let content = fs::read_to_string(path)?;
        let serializable: SerializableTheme = serde_json::from_str(&content)?;
        let theme = serializable.to_theme()?;

        info!(
            "Successfully loaded theme: {} from {}",
            theme.name(),
            path.display()
        );
        Ok(theme)
    }

    /// 从 JSON 字符串加载主题
    pub fn load_from_str(json: &str) -> Result<Theme, ThemeLoadError> {
        let serializable: SerializableTheme = serde_json::from_str(json)?;
        let theme = serializable.to_theme()?;
        Ok(theme)
    }

    /// 扫描目录并加载所有主题文件
    ///
    /// 只加载 .json 文件,忽略加载失败的文件并记录错误
    pub fn load_from_directory<P: AsRef<Path>>(dir: P) -> Vec<Theme> {
        let dir = dir.as_ref();

        if !dir.exists() {
            debug!("Theme directory does not exist: {}", dir.display());
            return Vec::new();
        }

        if !dir.is_dir() {
            warn!("Theme path is not a directory: {}", dir.display());
            return Vec::new();
        }

        info!("Scanning theme directory: {}", dir.display());

        let mut themes = Vec::new();

        match fs::read_dir(dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();

                    // 只处理 .json 文件
                    if path.extension().and_then(|s| s.to_str()) != Some("json") {
                        continue;
                    }

                    match Self::load_from_file(&path) {
                        Ok(theme) => {
                            info!("Loaded theme: {} from {}", theme.name(), path.display());
                            themes.push(theme);
                        }
                        Err(e) => {
                            error!("Failed to load theme from {}: {}", path.display(), e);
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to read theme directory {}: {}", dir.display(), e);
            }
        }

        info!("Loaded {} themes from {}", themes.len(), dir.display());
        themes
    }

    /// 获取默认主题目录路径
    ///
    /// - Linux/macOS: `~/.config/zterm/themes/`
    /// - Windows: `%APPDATA%\zterm\themes\`
    pub fn default_theme_directory() -> Option<PathBuf> {
        let config_dir = dirs::config_dir()?;
        Some(config_dir.join("zterm").join("themes"))
    }

    /// 确保主题目录存在
    pub fn ensure_theme_directory() -> Result<PathBuf, std::io::Error> {
        let dir = Self::default_theme_directory().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not determine config directory",
            )
        })?;

        if !dir.exists() {
            info!("Creating theme directory: {}", dir.display());
            fs::create_dir_all(&dir)?;
        }

        Ok(dir)
    }

    /// 将主题导出为 JSON 字符串
    pub fn theme_to_json(theme: &Theme) -> Result<String, serde_json::Error> {
        let serializable = SerializableTheme::from_theme(theme);
        serde_json::to_string_pretty(&serializable)
    }

    /// 将主题导出到文件
    pub fn export_theme<P: AsRef<Path>>(theme: &Theme, path: P) -> Result<(), ThemeLoadError> {
        let json = Self::theme_to_json(theme)?;
        fs::write(path, json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_theme_directory() {
        let dir = ThemeLoader::default_theme_directory();
        assert!(dir.is_some());
    }

    #[test]
    fn test_theme_roundtrip() {
        use crate::theme::builtin;
        let original = builtin::default_dark();

        // 转换为 JSON
        let json = ThemeLoader::theme_to_json(&original).unwrap();

        // 从 JSON 加载回来
        let loaded = ThemeLoader::load_from_str(&json).unwrap();

        assert_eq!(original.name(), loaded.name());
        assert_eq!(original.appearance(), loaded.appearance());
    }
}
