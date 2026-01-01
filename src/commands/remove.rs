//! Remove 命令
//!
//! 删除指定配置档案

use anyhow::Context;

use crate::{
    config::CccConfig,
    output::{success, warn},
};

/// 删除配置档案
///
/// 如果删除的是当前活跃配置，会提示用户重新选择
///
/// # Errors
///
/// 返回配置档案不存在等错误
pub fn execute(name: &str) -> anyhow::Result<()> {
    let mut config = CccConfig::load().context("加载配置失败")?;

    if !config.has_profile(name) {
        anyhow::bail!("配置档案 '{}' 不存在", name);
    }

    // 检查是否是当前配置
    let is_current = config.current.as_deref() == Some(name);

    // 删除配置
    let _ = config.remove_profile(name);

    // 如果删除的是当前配置，清除当前配置记录
    if is_current {
        config.current = None;
    }

    config.save().context("保存配置失败")?;

    if is_current {
        warn("已删除当前活跃配置，请使用 'ccc use <name>' 切换到其他配置");
    }

    success(&format!("配置档案 '{}' 已删除", name));

    Ok(())
}
