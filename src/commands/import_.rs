//! Import 命令
//!
//! 从当前 settings.json 导入配置档案

use anyhow::Context;

use crate::{
    config::{CccConfig, EnvConfig, Profile},
    output::{error, info, success},
};

/// 从当前 Claude settings 导入配置档案
///
/// # Errors
///
/// 返回配置名称无效、配置已存在、文件不存在等错误
pub fn execute(name: &str) -> anyhow::Result<()> {
    // 验证配置名称
    if !crate::config::validate_profile_name(name) {
        error("配置名称只能包含字母、数字、下划线和连字符");
        anyhow::bail!("无效的配置名称");
    }

    let mut config = CccConfig::load().context("加载配置失败")?;

    if config.has_profile(name) {
        error(&format!("配置档案 '{}' 已存在", name));
        anyhow::bail!("配置档案已存在");
    }

    // 从当前 Claude settings 读取配置
    let settings = crate::config::ClaudeSettings::load().context("加载 Claude settings 失败")?;

    let profile = Profile {
        api_key_helper: settings.api_key_helper,
        env: EnvConfig {
            anthropic_base_url: Some(
                settings
                    .env
                    .as_ref()
                    .and_then(|e| e.anthropic_base_url.clone())
                    .ok_or_else(|| anyhow::anyhow!("settings.json 中缺少 ANTHROPIC_BASE_URL"))?,
            ),
            anthropic_api_key: Some(
                settings
                    .env
                    .as_ref()
                    .and_then(|e| e.anthropic_api_key.clone())
                    .ok_or_else(|| anyhow::anyhow!("settings.json 中缺少 ANTHROPIC_API_KEY"))?,
            ),
        },
    };

    config.insert_profile(name.to_string(), profile);
    config.save().context("保存配置失败")?;

    success(&format!("配置档案 '{}' 已从当前 settings.json 导入", name));
    info(&format!("提示: 使用 'ccc use {}' 切换到此配置", name));

    Ok(())
}
