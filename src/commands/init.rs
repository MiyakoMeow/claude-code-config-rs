//! Init 命令
//!
//! 初始化 CCC 配置管理

use crate::{
    config::ensure_ccc_config_exists,
    output::{info, success},
};

/// 初始化命令
///
/// 创建 `~/.claude/cccrs-config.json` 配置文件
pub fn execute() -> anyhow::Result<()> {
    ensure_ccc_config_exists()?;

    info("正在初始化 CCC 配置管理...");

    success("CCC 配置管理已初始化");
    println!();
    println!("下一步:");
    println!("  1. 添加配置档案 (两种方式):");
    println!("     - 交互式: ccc add <name>    # 手动输入配置信息");
    println!("     - 导入式: ccc import <name> # 从当前 settings.json 导入");
    println!("  2. 使用 'ccc list' 查看所有配置");
    println!("  3. 使用 'ccc use <name>' 切换配置");
    println!();
    println!("示例:");
    println!("  ccc add kimi       # 交互式添加 kimi 配置");
    println!("  ccc import work    # 导入当前配置为 work");
    println!("  ccc use kimi       # 切换到 kimi 配置");

    Ok(())
}
