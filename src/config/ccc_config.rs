//! CCC 配置文件模块
//!
//! 定义 CCC 配置数据结构并提供读写功能

use std::collections::HashMap;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::paths::{ensure_ccc_config_exists, get_ccc_config_path};

/// CCC 主配置文件结构
///
/// 存储在 `~/.claude/ccc-config.json`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CccConfig {
    /// 所有配置档案
    pub profiles: HashMap<String, Profile>,
    /// 当前活跃配置名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<String>,
}

/// 单个配置档案
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    /// API Key Helper 命令
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key_helper: Option<String>,
    /// 环境变量配置
    pub env: EnvConfig,
}

/// 环境变量配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct EnvConfig {
    /// Anthropic API Base URL
    #[serde(skip_serializing_if = "Option::is_none", rename = "ANTHROPIC_BASE_URL")]
    pub anthropic_base_url: Option<String>,
    /// Anthropic API Key
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "ANTHROPIC_AUTH_TOKEN"
    )]
    pub anthropic_api_key: Option<String>,
}

impl CccConfig {
    /// 从文件加载配置
    ///
    /// 如果配置文件不存在，则先创建再返回空配置
    ///
    /// # Errors
    ///
    /// 返回文件读取错误或 JSON 解析错误
    pub fn load() -> Result<Self> {
        ensure_ccc_config_exists().ok();
        let path = get_ccc_config_path();
        let content = std::fs::read_to_string(&path).context("读取配置文件失败")?;
        serde_json::from_str(&content).context("解析配置文件失败")
    }

    /// 保存配置到文件
    ///
    /// # Errors
    ///
    /// 返回文件写入错误
    pub fn save(&self) -> Result<()> {
        let path = get_ccc_config_path();
        let content = serde_json::to_string_pretty(self).context("序列化配置失败")?;
        std::fs::write(&path, content).context("写入配置文件失败")
    }

    /// 检查配置档案是否存在
    #[must_use]
    pub fn has_profile(&self, name: &str) -> bool {
        self.profiles.contains_key(name)
    }

    /// 获取指定配置档案
    #[must_use]
    pub fn get_profile(&self, name: &str) -> Option<&Profile> {
        self.profiles.get(name)
    }

    /// 添加或更新配置档案
    pub fn insert_profile(&mut self, name: String, profile: Profile) {
        self.profiles.insert(name, profile);
    }

    /// 删除配置档案
    ///
    /// 返回被删除的配置档案（如果存在）
    #[must_use]
    pub fn remove_profile(&mut self, name: &str) -> Option<Profile> {
        self.profiles.remove(name)
    }
}

impl Profile {
    /// 创建新的配置档案
    #[must_use]
    pub fn new(api_key_helper: Option<String>, env: EnvConfig) -> Self {
        Self {
            api_key_helper,
            env,
        }
    }
}

impl EnvConfig {
    /// 创建新的环境配置
    #[must_use]
    pub fn new(anthropic_base_url: Option<String>, anthropic_api_key: Option<String>) -> Self {
        Self {
            anthropic_base_url,
            anthropic_api_key,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_temp_config() -> (TempDir, CccConfig) {
        let temp_dir = TempDir::new().unwrap();
        let temp_home = temp_dir.path().join("home");
        std::fs::create_dir_all(&temp_home).unwrap();
        unsafe {
            std::env::set_var("HOME", temp_home.to_string_lossy().as_ref());
        }

        let config = CccConfig::default();
        (temp_dir, config)
    }

    #[test]
    fn test_empty_config() {
        let (_temp_dir, config) = setup_temp_config();
        assert!(config.profiles.is_empty());
        assert!(config.current.is_none());
    }

    #[test]
    fn test_profile_insert_and_get() {
        let (_temp_dir, mut config) = setup_temp_config();
        let profile = Profile::new(
            Some("echo test".to_string()),
            EnvConfig::new(
                Some("https://api.test.com".to_string()),
                Some("sk-test".to_string()),
            ),
        );
        config.insert_profile("test".to_string(), profile.clone());
        assert!(config.has_profile("test"));
        assert_eq!(config.get_profile("test"), Some(&profile));
    }

    #[test]
    fn test_profile_remove() {
        let (_temp_dir, mut config) = setup_temp_config();
        let profile = Profile::new(
            None,
            EnvConfig::new(
                Some("https://api.test.com".to_string()),
                Some("sk-test".to_string()),
            ),
        );
        config.insert_profile("test".to_string(), profile.clone());
        assert!(config.has_profile("test"));

        let removed = config.remove_profile("test");
        assert_eq!(removed, Some(profile));
        assert!(!config.has_profile("test"));
    }

    #[test]
    fn test_save_and_load() {
        let (_temp_dir, mut config) = setup_temp_config();
        let profile = Profile::new(
            Some("echo test".to_string()),
            EnvConfig::new(
                Some("https://api.test.com".to_string()),
                Some("sk-test".to_string()),
            ),
        );
        config.insert_profile("test".to_string(), profile);
        config.current = Some("test".to_string());

        config.save().unwrap();

        let loaded = CccConfig::load().unwrap();
        assert_eq!(loaded, config);
    }
}
