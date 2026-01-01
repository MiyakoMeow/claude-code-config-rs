//! Add 命令
//!
//! 交互式添加新的配置档案

use anyhow::Context;
use dialoguer::{Confirm, Input};

use crate::{
    config::{CccConfig, EnvConfig, Profile},
    output::{error, info, success},
};

/// 交互式添加配置档案
///
/// 提示用户输入 Base URL、API Key 和可选的 API Key Helper
///
/// # Errors
///
/// 返回配置名称无效、配置已存在、用户取消等错误
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

    println!("创建配置档案: {}", name);
    println!();

    // 交互式获取配置信息
    let base_url: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("请输入 Base URL (例如: https://api.anthropic.com)")
        .validate_with(|input: &String| {
            if input.is_empty() {
                Err("Base URL 不能为空")
            } else {
                Ok(())
            }
        })
        .interact()
        .context("读取输入失败")?;

    let api_key: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("请输入 API Key (例如: sk-xxx)")
        .validate_with(|input: &String| {
            if input.is_empty() {
                Err("API Key 不能为空")
            } else {
                Ok(())
            }
        })
        .interact()
        .context("读取输入失败")?;

    // 询问是否需要 API Key Helper
    let need_helper = Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("是否需要 API Key Helper?")
        .default(false)
        .interact()
        .context("读取输入失败")?;

    let api_key_helper = if need_helper {
        Some(
            Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
                .with_prompt("请输入 API Key Helper 命令 (例如: echo 'sk-xxx')")
                .interact()
                .context("读取输入失败")?,
        )
    } else {
        None
    };

    // 创建配置
    let profile = Profile::new(
        api_key_helper,
        EnvConfig::new(Some(base_url), Some(api_key)),
    );

    config.insert_profile(name.to_string(), profile);
    config.save().context("保存配置失败")?;

    println!();
    success(&format!("配置档案 '{}' 已创建", name));
    info(&format!("提示: 使用 'ccc use {}' 切换到此配置", name));

    Ok(())
}
