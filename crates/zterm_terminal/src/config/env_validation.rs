//! 环境变量验证和安全检查

use std::collections::HashMap;
use std::path::PathBuf;

/// 环境变量验证错误
#[derive(Debug, thiserror::Error)]
pub enum EnvValidationError {
    #[error("危险的环境变量值: {key} = {value}")]
    DangerousValue { key: String, value: String },

    #[error("路径不存在: {path}")]
    PathNotFound { path: PathBuf },

    #[error("路径不可访问: {path}")]
    PathNotAccessible { path: PathBuf },
}

/// 验证并清理环境变量
pub fn validate_env_vars(env: &mut HashMap<String, String>) -> Result<(), EnvValidationError> {
    // 移除危险的环境变量
    let dangerous_vars = [
        "LD_PRELOAD",     // 库预加载攻击
        "LD_LIBRARY_PATH", // 库路径劫持（可配置允许）
        "DYLD_INSERT_LIBRARIES", // macOS 库注入
        "DYLD_LIBRARY_PATH",
    ];

    for var in &dangerous_vars {
        if env.contains_key(*var) {
            tracing::warn!("移除危险环境变量: {}", var);
            env.remove(*var);
        }
    }

    // 验证 PATH
    if let Some(path) = env.get("PATH") {
        validate_path_var(path)?;
    }

    // 验证 HOME
    if let Some(home) = env.get("HOME") {
        let home_path = PathBuf::from(home);
        if !home_path.exists() {
            tracing::warn!("HOME 目录不存在: {}", home);
        }
    }

    Ok(())
}

/// 验证 PATH 环境变量
fn validate_path_var(path: &str) -> Result<(), EnvValidationError> {
    let paths: Vec<&str> = path.split(if cfg!(windows) { ';' } else { ':' }).collect();

    for p in paths {
        if p.is_empty() {
            continue;
        }

        // 检查是否包含危险字符
        if p.contains('\0') || p.contains('\n') {
            return Err(EnvValidationError::DangerousValue {
                key: "PATH".to_string(),
                value: p.to_string(),
            });
        }
    }

    Ok(())
}

/// 设置安全的默认环境变量
pub fn get_safe_default_env() -> HashMap<String, String> {
    let mut env = HashMap::new();

    // 设置 TERM
    env.insert("TERM".to_string(), "xterm-256color".to_string());

    // 设置 COLORTERM
    env.insert("COLORTERM".to_string(), "truecolor".to_string());

    // 设置 TERM_PROGRAM
    env.insert("TERM_PROGRAM".to_string(), "zterm".to_string());
    env.insert("TERM_PROGRAM_VERSION".to_string(), env!("CARGO_PKG_VERSION").to_string());

    env
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_removes_dangerous_vars() {
        let mut env = HashMap::new();
        env.insert("LD_PRELOAD".to_string(), "/evil/lib.so".to_string());
        env.insert("SAFE_VAR".to_string(), "value".to_string());

        validate_env_vars(&mut env).unwrap();

        assert!(!env.contains_key("LD_PRELOAD"));
        assert!(env.contains_key("SAFE_VAR"));
    }

    #[test]
    fn test_validate_path_with_null_byte() {
        let result = validate_path_var("/usr/bin\0/evil");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_safe_default_env() {
        let env = get_safe_default_env();

        assert_eq!(env.get("TERM"), Some(&"xterm-256color".to_string()));
        assert_eq!(env.get("COLORTERM"), Some(&"truecolor".to_string()));
        assert!(env.contains_key("TERM_PROGRAM"));
    }
}
