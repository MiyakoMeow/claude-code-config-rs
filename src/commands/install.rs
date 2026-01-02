//! Install 命令
//!
//! 将脚本安装到系统路径

use std::path::{Path, PathBuf};

use anyhow::{Context, bail};

use crate::output::{error, info, success};

/// 安装命令
///
/// 将当前可执行文件复制到 `/usr/local/bin/cccrs`
///
/// # Errors
///
/// 返回权限不足、文件复制失败等错误
pub fn execute() -> anyhow::Result<()> {
    info("正在安装 CCCRS 到系统路径...");

    // 获取当前可执行文件路径
    let current_exe = std::env::current_exe().context("获取当前可执行文件路径失败")?;

    // 确定目标路径
    let target_dir = PathBuf::from("/usr/local/bin");
    let target_file = target_dir.join("cccrs");

    // 检查目标目录是否存在
    if !target_dir.exists() {
        error(&format!("目标目录不存在: {}", target_dir.display()));
        info("请手动安装到系统路径");
        bail!("目标目录不存在: {}", target_dir.display());
    }

    // 检查写入权限
    if !is_writable(target_dir.as_path()) {
        error(&format!("需要管理员权限安装到 {}", target_dir.display()));
        println!();
        println!("请运行:");
        println!(
            "  sudo cp \"{}\" \"{}\"",
            current_exe.display(),
            target_file.display()
        );
        println!("  sudo chmod +x \"{}\"", target_file.display());
        bail!("需要管理员权限");
    }

    // 复制文件
    std::fs::copy(&current_exe, &target_file).context("复制文件失败")?;

    // 设置可执行权限 (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&target_file)
            .context("获取文件元数据失败")?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&target_file, perms).context("设置权限失败")?;
    }

    success(&format!("CCCRS 已成功安装到: {}", target_file.display()));
    info("现在可以在任何位置使用 'cccrs' 命令");
    println!();
    println!("建议下一步:");
    println!("  ccc init           # 初始化配置管理");
    println!("  ccc add default    # 保存当前配置");

    Ok(())
}

/// 检查目录是否可写
#[must_use]
fn is_writable(path: &Path) -> bool {
    // 简化实现：检查当前用户是否有写权限
    path.parent()
        .and_then(|p| std::fs::metadata(p).ok())
        .map(|m| -> bool { !m.permissions().readonly() })
        .unwrap_or_default()
}
