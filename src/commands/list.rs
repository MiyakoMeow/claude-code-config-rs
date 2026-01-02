//! List 命令
//!
//! 查看当前配置和所有可用配置

use anyhow::Context;
use colored::Colorize;

use crate::{
    config::{CccConfig, ClaudeSettings},
    output::{info, success, warn},
};

/// 列出所有配置
///
/// 显示当前活跃配置（智能检测）和所有可用配置档案
///
/// # Errors
///
/// 返回配置文件读取错误
pub fn execute() -> anyhow::Result<()> {
    println!("=== Claude Code 配置管理 ===");
    println!();

    // 加载配置
    let mut config = CccConfig::load().context("加载配置失败")?;

    // 尝试加载 Claude settings
    let settings_result = ClaudeSettings::load();

    // 智能检测当前配置
    let detected_profile = settings_result
        .as_ref()
        .map_or(None, |settings| detect_active_profile(&config, settings));

    // 如果检测到配置且与存储的不同，更新存储的当前配置
    let current_profile = if let Some(detected) = &detected_profile {
        if config.current.as_deref() != Some(detected) {
            info(&format!("检测到当前配置: {}", detected));
            config.current = Some(detected.clone());
            let _ = config.save();
        }
        Some(detected.clone())
    } else {
        config.current.clone()
    };

    // 显示当前活跃配置
    if let Some(profile_name) = &current_profile {
        if let Some(profile) = config.profiles.get(profile_name) {
            success(&format!("当前活跃配置: {} (智能检测)", profile_name));
            println!();

            // API Key Helper
            if profile.api_key_helper.is_some() {
                println!("  API Key Helper: [已配置]");
            } else {
                println!("  API Key Helper: 未设置");
            }

            // Base URL
            let base_url = profile
                .env
                .anthropic_base_url
                .as_deref()
                .unwrap_or("未设置");
            println!("  Base URL: {}", base_url);

            // API Key (部分显示)
            let api_key = profile.env.anthropic_api_key.as_deref().unwrap_or("未设置");
            let masked_key = if api_key.starts_with("sk-") {
                "sk-***[已配置]".to_string()
            } else if api_key != "未设置" {
                "***[已配置]".to_string()
            } else {
                "未设置".to_string()
            };
            println!("  API Key: {}", masked_key);
            println!();
        }
    } else {
        // 检查是否有当前配置但未匹配任何档案
        if let Ok(ref settings) = settings_result {
            let has_api_key = settings
                .env
                .as_ref()
                .and_then(|e| e.anthropic_api_key.as_ref())
                .is_some();

            if has_api_key {
                warn("当前配置未保存为档案");
                println!("  提示: 使用 'ccc import <name>' 保存当前配置");
            } else {
                warn("当前无活跃配置");
            }
        } else {
            warn("当前无活跃配置");
        }
        println!();
    }

    // 显示所有配置档案
    println!("可用配置档案:");

    if config.profiles.is_empty() {
        println!("  (无配置档案)");
    } else {
        for name in config.profiles.keys() {
            if current_profile.as_deref() == Some(name) {
                println!("  * {} (当前)", name.green());
            } else {
                println!("  {}", name);
            }
        }
    }

    Ok(())
}

/// 智能检测当前活跃的配置
///
/// 通过比较 settings.json 和 cccrs-config.json 中的配置来检测
#[must_use]
fn detect_active_profile(config: &CccConfig, settings: &ClaudeSettings) -> Option<String> {
    // 获取当前 settings 中的配置
    let settings_url = settings.env.as_ref()?.anthropic_base_url.as_ref()?;
    let settings_key = settings.env.as_ref()?.anthropic_api_key.as_ref()?;

    // 遍历所有配置档案进行比较
    for (name, profile) in &config.profiles {
        let profile_url = profile.env.anthropic_base_url.as_ref()?;
        let profile_key = profile.env.anthropic_api_key.as_ref()?;

        // 比较核心字段是否匹配
        if profile_url == settings_url && profile_key == settings_key {
            return Some(name.clone());
        }
    }

    None
}
