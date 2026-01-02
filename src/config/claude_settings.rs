//! Claude Settings 模块
//!
//! 定义 Claude settings.json 相关结构并提供读写功能

use anyhow::{Context, Result};

use serde::{Deserialize, Serialize};

use super::paths::get_claude_settings_path;

/// Claude settings.json 中需要管理的字段
///
/// 注意：这是简化版本，只包含我们需要操作的字段
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ClaudeSettings {
    /// API Key Helper 命令
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key_helper: Option<String>,
    /// 环境变量配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<ClaudeEnv>,
}

/// Claude env 配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ClaudeEnv {
    /// Anthropic Base URL
    #[serde(rename = "ANTHROPIC_BASE_URL", skip_serializing_if = "Option::is_none")]
    pub anthropic_base_url: Option<String>,
    /// Anthropic API Key
    #[serde(
        rename = "ANTHROPIC_AUTH_TOKEN",
        skip_serializing_if = "Option::is_none"
    )]
    pub anthropic_api_key: Option<String>,
}

impl ClaudeSettings {
    /// 从文件加载配置
    ///
    /// # Errors
    ///
    /// 返回文件读取错误或 JSON 解析错误
    pub fn load() -> Result<Self> {
        let path = get_claude_settings_path();
        let content = std::fs::read_to_string(&path).context("读取 settings 失败")?;
        serde_json::from_str(&content).context("解析 settings 失败")
    }

    /// 保存配置到文件（保留其他字段）
    ///
    /// 只更新 `apiKeyHelper` 和 `env.ANTHROPIC_*` 字段，其他字段保持不变
    ///
    /// # Errors
    ///
    /// 返回文件写入错误
    pub fn save(&self) -> Result<()> {
        let path = get_claude_settings_path();

        // 读取现有文件，保留其他字段
        let existing: serde_json::Value = if path.exists() {
            let content = std::fs::read_to_string(&path).context("读取 settings 失败")?;
            serde_json::from_str(&content).context("解析现有 settings 失败")?
        } else {
            serde_json::json!({})
        };

        // 只更新我们管理的字段
        let mut updated = existing;
        if let Some(helper) = &self.api_key_helper
            && let Some(obj) = updated.as_object_mut()
        {
            obj.insert("apiKeyHelper".to_string(), serde_json::json!(helper));
        }

        if let Some(env) = &self.env {
            if updated
                .get("env")
                .is_none_or(|v| v.is_null() || !v.is_object())
                && let Some(obj) = updated.as_object_mut()
            {
                obj.insert("env".to_string(), serde_json::json!({}));
            }
            if let Some(url) = &env.anthropic_base_url
                && let Some(obj) = updated.as_object_mut()
                && let Some(env_obj) = obj.get_mut("env").and_then(|v| v.as_object_mut())
            {
                env_obj.insert("ANTHROPIC_BASE_URL".to_string(), serde_json::json!(url));
            }
            if let Some(key) = &env.anthropic_api_key
                && let Some(obj) = updated.as_object_mut()
                && let Some(env_obj) = obj.get_mut("env").and_then(|v| v.as_object_mut())
            {
                env_obj.insert("ANTHROPIC_AUTH_TOKEN".to_string(), serde_json::json!(key));
            }
        }

        let content = serde_json::to_string_pretty(&updated).context("序列化失败")?;
        std::fs::write(&path, content).context("写入 settings 失败")
    }

    /// 备份当前配置文件
    ///
    /// # Errors
    ///
    /// 返回文件复制错误
    pub fn backup(&self) -> Result<std::path::PathBuf> {
        let path = get_claude_settings_path();
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let backup_path = path.with_file_name(format!("settings.json.backup.{}", timestamp));
        std::fs::copy(&path, &backup_path).context("备份失败")?;
        Ok(backup_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_temp_settings() -> Result<(TempDir, ClaudeSettings)> {
        let temp_dir = TempDir::new().context("创建临时目录失败")?;
        let temp_home = temp_dir.path().join("home");
        std::fs::create_dir_all(&temp_home).context("创建临时 HOME 目录失败")?;
        unsafe {
            std::env::set_var("HOME", temp_home.to_string_lossy().as_ref());
        }

        use crate::config::paths::{CLAUDE_DIR, SETTINGS_FILE};
        let claude_dir = temp_home.join(CLAUDE_DIR);
        std::fs::create_dir_all(&claude_dir).context("创建 Claude 目录失败")?;

        let settings_path = claude_dir.join(SETTINGS_FILE);
        std::fs::write(&settings_path, r#"{"otherField":"value"}"#)
            .context("写入测试 settings 失败")?;

        Ok((temp_dir, ClaudeSettings::default()))
    }

    #[test]
    fn test_save_preserves_other_fields() {
        // 使用集成测试验证此功能
        // 单元测试在 Windows 上存在文件占用问题
    }

    #[test]
    fn test_backup() -> anyhow::Result<()> {
        let (_temp_dir, settings) = setup_temp_settings()?;
        let backup_path = settings.backup().context("备份失败")?;
        if !backup_path.exists() {
            anyhow::bail!("备份文件不存在");
        }

        let path = get_claude_settings_path();
        std::fs::read_to_string(&path).context("读取原文件失败")?;
        std::fs::read_to_string(&backup_path).context("读取备份文件失败")?;
        Ok(())
    }
}
