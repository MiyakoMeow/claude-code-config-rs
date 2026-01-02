//! Use 命令
//!
//! 切换到指定配置档案

use anyhow::Context;

use crate::{
    config::{CccConfig, ClaudeSettings},
    output::{info, success},
};

/// 切换到指定配置档案
///
/// 会自动备份当前的 settings.json，然后更新配置
///
/// # Errors
///
/// 返回配置档案不存在、文件操作失败等错误
pub fn execute(name: &str) -> anyhow::Result<()> {
    let config = CccConfig::load().context("加载配置失败")?;

    let profile = config
        .get_profile(name)
        .ok_or_else(|| anyhow::anyhow!("配置档案 '{}' 不存在", name))?
        .clone();

    // 加载并备份当前设置
    let mut settings = ClaudeSettings::load().context("加载 Claude settings 失败")?;

    // 备份
    let backup_path = settings.backup().context("备份失败")?;
    info(&format!("已备份当前配置: {}", backup_path.display()));

    // 只更新相关字段
    if let Some(helper) = &profile.api_key_helper {
        settings.api_key_helper = Some(helper.clone());
    }

    // 更新 env 字段
    if let Some(url) = &profile.env.anthropic_base_url {
        if settings.env.is_none() {
            settings.env = Some(crate::config::ClaudeEnv::default());
        }
        if let Some(env) = settings.env.as_mut() {
            env.anthropic_base_url = Some(url.clone());
        }
    }

    if let Some(key) = &profile.env.anthropic_api_key {
        if settings.env.is_none() {
            settings.env = Some(crate::config::ClaudeEnv::default());
        }
        if let Some(env) = settings.env.as_mut() {
            env.anthropic_api_key = Some(key.clone());
        }
    }

    settings.save().context("保存设置失败")?;

    // 更新当前配置记录
    let mut updated_config = CccConfig::load().context("加载配置失败")?;
    updated_config.current = Some(name.to_string());
    updated_config.save().context("保存配置失败")?;

    success(&format!("已切换到配置档案: {}", name));

    Ok(())
}
