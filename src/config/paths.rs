//! 配置文件路径模块
//!
//! 提供 Claude settings 和 CCCRS 配置文件的路径获取功能

use std::path::PathBuf;

use home::home_dir;

/// Claude 配置目录名称
pub const CLAUDE_DIR: &str = ".claude";

/// Claude settings.json 文件名
pub const SETTINGS_FILE: &str = "settings.json";

/// CCCRS 配置文件名
pub const CCC_CONFIG_FILE: &str = "cccrs-config.json";

/// 获取 Claude settings.json 的路径
///
/// 返回 `~/.claude/settings.json`
#[must_use]
pub fn get_claude_settings_path() -> PathBuf {
    home_dir()
        .unwrap_or_default()
        .join(CLAUDE_DIR)
        .join(SETTINGS_FILE)
}

/// 获取 CCCRS 配置文件的路径
///
/// 返回 `~/.claude/cccrs-config.json`
#[must_use]
pub fn get_ccc_config_path() -> PathBuf {
    home_dir()
        .unwrap_or_default()
        .join(CLAUDE_DIR)
        .join(CCC_CONFIG_FILE)
}

/// 确保 CCCRS 配置文件存在
///
/// 如果文件不存在，则创建包含初始配置的 JSON 文件
///
/// # Errors
///
/// 返回文件或目录创建错误
pub fn ensure_ccc_config_exists() -> std::io::Result<()> {
    let path = get_ccc_config_path();
    if !path.exists() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let initial_content = r#"{"profiles": {},"current": null}"#;
        std::fs::write(&path, initial_content)?;
    }
    Ok(())
}

/// 验证配置名称是否有效
///
/// 有效名称只能包含字母、数字、下划线和连字符
#[must_use]
pub fn validate_profile_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_profile_name_valid() {
        assert!(validate_profile_name("kimi"));
        assert!(validate_profile_name("my_config_1"));
        assert!(validate_profile_name("test-profile"));
        assert!(validate_profile_name("a1b2c3"));
    }

    #[test]
    fn test_validate_profile_name_invalid() {
        assert!(!validate_profile_name(""));
        assert!(!validate_profile_name("hello world"));
        assert!(!validate_profile_name("test@123"));
        assert!(!validate_profile_name("test.name"));
    }

    #[test]
    fn test_public_constants() {
        // 验证公开常量可以被访问
        assert_eq!(CLAUDE_DIR, ".claude");
        assert_eq!(SETTINGS_FILE, "settings.json");
        assert_eq!(CCC_CONFIG_FILE, "cccrs-config.json");
    }

    #[test]
    fn test_path_functions() {
        // 验证路径函数返回正确的路径
        let settings_path = get_claude_settings_path();
        assert!(settings_path.ends_with(".claude/settings.json"));

        let ccc_config_path = get_ccc_config_path();
        assert!(ccc_config_path.ends_with(".claude/cccrs-config.json"));
    }
}
