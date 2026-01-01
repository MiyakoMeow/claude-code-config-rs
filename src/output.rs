//! 输出模块
//!
//! 提供带颜色的格式化输出函数

use colored::Colorize;

/// 错误输出 (红色)
pub fn error(msg: &str) {
    eprintln!("{} {}", "错误:".red().bold(), msg);
}

/// 信息输出 (蓝色)
pub fn info(msg: &str) {
    println!("{} {}", "信息:".blue().bold(), msg);
}

/// 成功输出 (绿色)
pub fn success(msg: &str) {
    println!("{} {}", "成功:".green().bold(), msg);
}

/// 警告输出 (黄色)
pub fn warn(msg: &str) {
    println!("{} {}", "警告:".yellow().bold(), msg);
}
