//! CCCRS (Claude Code Config Rust) - Claude 配置管理工具
//!
//! 用于管理 `~/.claude/settings.json` 中的 API 配置切换

use clap::{Parser, Subcommand};

/// CLI 参数解析
#[derive(Parser, Debug)]
#[command(name = "cccrs")]
#[command(author = "MiyakoMeow")]
#[command(version = "0.1.0")]
#[command(about = "Claude Code 配置管理工具", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

/// 可用子命令
#[derive(Subcommand, Debug)]
enum Commands {
    /// 安装脚本到系统路径
    Install,
    /// 初始化配置管理
    Init,
    /// 查看当前配置和所有可用配置
    List,
    /// 交互式添加新的配置档案
    Add {
        /// 配置名称
        name: String,
    },
    /// 从当前 settings.json 导入配置档案
    Import {
        /// 配置名称
        name: String,
    },
    /// 删除指定配置档案
    #[command(alias = "del", alias = "rm")]
    Remove {
        /// 配置名称
        name: String,
    },
    /// 切换到指定配置
    Use {
        /// 配置名称
        name: String,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    use claude_code_config_rs::commands::{add, import, init, install, list, remove, use_cmd};

    match args.command {
        Commands::Install => install(),
        Commands::Init => init(),
        Commands::List => list(),
        Commands::Add { name } => add(&name),
        Commands::Import { name } => import(&name),
        Commands::Remove { name } => remove(&name),
        Commands::Use { name } => use_cmd(&name),
    }
}
