//! CLI 测试模块
//!
//! 使用 `assert_cmd` 进行 CLI 行为测试

use anyhow::{Context, Result};
use assert_cmd::{Command, cargo_bin};
use predicates::prelude::*;
use serial_test::serial;
use std::path::PathBuf;
use tempfile::TempDir;

/// 设置临时 HOME 目录
fn setup_temp_home() -> Result<(TempDir, PathBuf, PathBuf)> {
    let temp_dir = TempDir::new().context("创建临时目录失败")?;
    let temp_home = temp_dir.path().join("test_home");
    std::fs::create_dir_all(&temp_home).context("创建临时 HOME 目录失败")?;

    use claude_code_config_rs::config::paths::{CCC_CONFIG_FILE, CLAUDE_DIR, SETTINGS_FILE};
    let claude_dir = temp_home.join(CLAUDE_DIR);
    std::fs::create_dir_all(&claude_dir).context("创建 Claude 目录失败")?;

    let settings_path = claude_dir.join(SETTINGS_FILE);
    let ccc_config_path = claude_dir.join(CCC_CONFIG_FILE);

    // 设置 HOME 环境变量 (Unix/Linux)
    unsafe {
        std::env::set_var("HOME", temp_home.to_string_lossy().as_ref());
    }

    // 设置 USERPROFILE 环境变量 (Windows)
    unsafe {
        std::env::set_var("USERPROFILE", temp_home.to_string_lossy().as_ref());
    }

    Ok((temp_dir, settings_path, ccc_config_path))
}

/// 创建初始 settings.json
fn create_initial_settings(path: &PathBuf) -> Result<()> {
    std::fs::write(
        path,
        r#"{
    "otherField": "should remain",
    "env": {
        "ANTHROPIC_BASE_URL": "https://api.anthropic.com",
        "ANTHROPIC_AUTH_TOKEN": "sk-test-key"
    }
}"#,
    )
    .context("写入初始 settings 失败")
}

#[test]
#[serial]
fn test_help_command() -> Result<()> {
    let mut cmd = Command::new(cargo_bin!("cccrs"));
    cmd.arg("help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Claude Code 配置管理工具"));
    Ok(())
}

#[test]
#[serial]
fn test_init_command() -> Result<()> {
    let (_temp_dir, settings_path, ccc_config_path) = setup_temp_home()?;

    // 创建初始 settings.json
    create_initial_settings(&settings_path)?;

    let mut cmd = Command::new(cargo_bin!("cccrs"));
    cmd.arg("init").assert().success();

    // 验证配置文件创建
    if !ccc_config_path.exists() {
        anyhow::bail!("配置文件不存在");
    }
    let content = std::fs::read_to_string(&ccc_config_path).context("读取配置文件失败")?;
    if !content.contains("profiles") {
        anyhow::bail!("配置文件不包含 profiles");
    }
    if !content.contains("current") {
        anyhow::bail!("配置文件不包含 current");
    }
    Ok(())
}

#[test]
#[serial]
fn test_list_empty() -> Result<()> {
    let (_temp_dir, settings_path, _ccc_config_path) = setup_temp_home()?;

    // 创建初始 settings.json
    create_initial_settings(&settings_path)?;

    // 先初始化
    let mut cmd = Command::new(cargo_bin!("cccrs"));
    cmd.arg("init").assert().success();

    // list 应该显示无配置档案
    let mut cmd2 = Command::new(cargo_bin!("cccrs"));
    cmd2.arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("可用配置档案"))
        .stdout(predicate::str::contains("无配置档案"));
    Ok(())
}

#[test]
#[serial]
fn test_import_and_list() -> Result<()> {
    let (_temp_dir, settings_path, ccc_config_path) = setup_temp_home()?;

    // 创建初始 settings.json
    create_initial_settings(&settings_path)?;

    // 初始化
    let mut cmd = Command::new(cargo_bin!("cccrs"));
    cmd.arg("init").assert().success();

    // 导入当前配置
    let mut cmd2 = Command::new(cargo_bin!("cccrs"));
    cmd2.arg("import")
        .arg("anthropic")
        .assert()
        .success()
        .stdout(predicate::str::contains("已从当前 settings.json 导入"));

    // 验证配置已添加
    let config_content = std::fs::read_to_string(&ccc_config_path).context("读取配置文件失败")?;
    let config: serde_json::Value =
        serde_json::from_str(&config_content).context("解析 JSON 失败")?;
    let profiles = config
        .get("profiles")
        .and_then(|p| p.as_object())
        .context("获取 profiles 失败")?;
    let anthropic = profiles
        .get("anthropic")
        .and_then(|p| p.as_object())
        .context("获取 anthropic 配置失败")?;

    let env = anthropic
        .get("env")
        .and_then(|e| e.as_object())
        .context("获取 env 失败")?;
    let base_url = env
        .get("ANTHROPIC_BASE_URL")
        .and_then(|v| v.as_str())
        .context("获取 Base URL 失败")?;
    let auth_token = env
        .get("ANTHROPIC_AUTH_TOKEN")
        .and_then(|v| v.as_str())
        .context("获取 Auth Token 失败")?;

    if base_url != "https://api.anthropic.com" {
        anyhow::bail!("Base URL 不匹配");
    }
    if auth_token != "sk-test-key" {
        anyhow::bail!("Auth Token 不匹配");
    }

    // list 应该显示配置
    let mut cmd3 = Command::new(cargo_bin!("cccrs"));
    cmd3.arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("anthropic"))
        .stdout(predicate::str::contains("当前活跃配置"));
    Ok(())
}

#[test]
#[serial]
fn test_use_command_updates_settings() -> Result<()> {
    let (temp_dir, settings_path, ccc_config_path) = setup_temp_home()?;

    // 创建初始 settings.json
    create_initial_settings(&settings_path)?;

    // 初始化并导入配置
    let mut cmd = Command::new(cargo_bin!("cccrs"));
    cmd.arg("init").assert().success();

    let mut cmd2 = Command::new(cargo_bin!("cccrs"));
    cmd2.arg("import").arg("anthropic").assert().success();

    // 准备另一个配置
    let config = r#"{
        "profiles": {
            "new-profile": {
                "env": {
                    "ANTHROPIC_BASE_URL": "https://api.new.com",
                    "ANTHROPIC_AUTH_TOKEN": "sk-new-key"
                }
            }
        },
        "current": null
    }"#;
    std::fs::write(&ccc_config_path, config).context("写入配置失败")?;

    // 切换到新配置
    let mut cmd3 = Command::new(cargo_bin!("cccrs"));
    cmd3.arg("use")
        .arg("new-profile")
        .assert()
        .success()
        .stdout(predicate::str::contains("已切换到配置档案"));

    // 验证 settings.json 已更新，且保留其他字段
    let settings_content =
        std::fs::read_to_string(&settings_path).context("读取 settings 文件失败")?;
    let settings: serde_json::Value =
        serde_json::from_str(&settings_content).context("解析 JSON 失败")?;

    let env = settings
        .get("env")
        .and_then(|e| e.as_object())
        .context("获取 env 失败")?;
    let base_url = env
        .get("ANTHROPIC_BASE_URL")
        .and_then(|v| v.as_str())
        .context("获取 Base URL 失败")?;
    let auth_token = env
        .get("ANTHROPIC_AUTH_TOKEN")
        .and_then(|v| v.as_str())
        .context("获取 Auth Token 失败")?;
    let other_field = settings
        .get("otherField")
        .and_then(|v| v.as_str())
        .context("获取 otherField 失败")?;

    if base_url != "https://api.new.com" {
        anyhow::bail!("Base URL 不匹配");
    }
    if auth_token != "sk-new-key" {
        anyhow::bail!("Auth Token 不匹配");
    }
    if other_field != "should remain" {
        anyhow::bail!("其他字段不匹配");
    }

    // 验证备份文件已创建
    use claude_code_config_rs::config::paths::CLAUDE_DIR;
    let backups: Vec<_> = std::fs::read_dir(temp_dir.path().join("test_home").join(CLAUDE_DIR))
        .context("读取备份目录失败")?
        .filter_map(std::result::Result::ok)
        .map(|e| e.path())
        .filter(|p| p.to_string_lossy().contains("backup"))
        .collect();
    if backups.is_empty() {
        anyhow::bail!("备份文件未创建");
    }
    Ok(())
}

#[test]
#[serial]
fn test_remove_command() -> Result<()> {
    let (_temp_dir, settings_path, ccc_config_path) = setup_temp_home()?;

    // 创建初始 settings.json
    create_initial_settings(&settings_path)?;

    // 初始化并导入配置
    let mut cmd = Command::new(cargo_bin!("cccrs"));
    cmd.arg("init").assert().success();

    let mut cmd2 = Command::new(cargo_bin!("cccrs"));
    cmd2.arg("import").arg("test-profile").assert().success();

    // 删除配置
    let mut cmd3 = Command::new(cargo_bin!("cccrs"));
    cmd3.arg("remove")
        .arg("test-profile")
        .assert()
        .success()
        .stdout(predicate::str::contains("已删除"));

    // 验证配置已删除
    let config_content = std::fs::read_to_string(&ccc_config_path).context("读取配置文件失败")?;
    let config: serde_json::Value =
        serde_json::from_str(&config_content).context("解析 JSON 失败")?;
    let profiles = config
        .get("profiles")
        .and_then(|p| p.as_object())
        .context("获取 profiles 失败")?;
    let test_profile = profiles
        .get("test-profile")
        .unwrap_or(&serde_json::Value::Null);
    if !test_profile.is_null() {
        anyhow::bail!("配置档案应已删除");
    }
    Ok(())
}

#[test]
#[serial]
fn test_remove_current_profile() -> Result<()> {
    let (_temp_dir, settings_path, ccc_config_path) = setup_temp_home()?;

    // 创建初始 settings.json
    create_initial_settings(&settings_path)?;

    // 初始化并导入配置
    let mut cmd = Command::new(cargo_bin!("cccrs"));
    cmd.arg("init").assert().success();

    let mut cmd2 = Command::new(cargo_bin!("cccrs"));
    cmd2.arg("import").arg("current").assert().success();

    // 切换到当前配置
    let mut cmd3 = Command::new(cargo_bin!("cccrs"));
    cmd3.arg("use").arg("current").assert().success();

    // 删除当前配置
    let mut cmd4 = Command::new(cargo_bin!("cccrs"));
    cmd4.arg("remove")
        .arg("current")
        .assert()
        .success()
        .stdout(predicate::str::contains("已删除当前活跃配置"));

    // 验证当前配置已被清除
    let config_content = std::fs::read_to_string(&ccc_config_path).context("读取配置文件失败")?;
    let config: serde_json::Value =
        serde_json::from_str(&config_content).context("解析 JSON 失败")?;
    let profiles = config
        .get("profiles")
        .and_then(|p| p.as_object())
        .context("获取 profiles 失败")?;
    let current_profile = profiles.get("current").unwrap_or(&serde_json::Value::Null);
    let current_field = config.get("current").unwrap_or(&serde_json::Value::Null);
    if !current_profile.is_null() {
        anyhow::bail!("当前配置档案应已删除");
    }
    if !current_field.is_null() {
        anyhow::bail!("当前配置字段应已清除");
    }
    Ok(())
}

#[test]
#[serial]
fn test_import_nonexistent_profile() -> Result<()> {
    let (_temp_dir, _settings_path, _ccc_config_path) = setup_temp_home()?;

    let mut cmd = Command::new(cargo_bin!("cccrs"));
    cmd.arg("remove")
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("不存在"));
    Ok(())
}

#[test]
#[serial]
fn test_use_nonexistent_profile() -> Result<()> {
    let (_temp_dir, _settings_path, _ccc_config_path) = setup_temp_home()?;

    let mut cmd = Command::new(cargo_bin!("cccrs"));
    cmd.arg("use")
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("不存在"));
    Ok(())
}

#[test]
#[serial]
fn test_import_invalid_name() -> Result<()> {
    let (_temp_dir, _settings_path, _ccc_config_path) = setup_temp_home()?;

    let mut cmd = Command::new(cargo_bin!("cccrs"));
    cmd.arg("import")
        .arg("invalid name with spaces")
        .assert()
        .failure();
    Ok(())
}

#[test]
#[serial]
fn test_add_invalid_name() -> Result<()> {
    let (_temp_dir, _settings_path, _ccc_config_path) = setup_temp_home()?;

    let mut cmd = Command::new(cargo_bin!("cccrs"));
    cmd.arg("add").arg("invalid@name").assert().failure();
    Ok(())
}

#[test]
#[serial]
fn test_remove_command_del_alias() -> Result<()> {
    let (_temp_dir, settings_path, _ccc_config_path) = setup_temp_home()?;

    create_initial_settings(&settings_path)?;

    let mut cmd = Command::new(cargo_bin!("cccrs"));
    cmd.arg("init").assert().success();

    let mut cmd2 = Command::new(cargo_bin!("cccrs"));
    cmd2.arg("import").arg("test-alias").assert().success();

    // 使用 del 别名删除配置
    let mut cmd3 = Command::new(cargo_bin!("cccrs"));
    cmd3.arg("del")
        .arg("test-alias")
        .assert()
        .success()
        .stdout(predicate::str::contains("已删除"));
    Ok(())
}

#[test]
#[serial]
fn test_remove_command_rm_alias() -> Result<()> {
    let (_temp_dir, settings_path, _ccc_config_path) = setup_temp_home()?;

    create_initial_settings(&settings_path)?;

    let mut cmd = Command::new(cargo_bin!("cccrs"));
    cmd.arg("init").assert().success();

    let mut cmd2 = Command::new(cargo_bin!("cccrs"));
    cmd2.arg("import").arg("test-rm-alias").assert().success();

    // 使用 rm 别名删除配置
    let mut cmd3 = Command::new(cargo_bin!("cccrs"));
    cmd3.arg("rm")
        .arg("test-rm-alias")
        .assert()
        .success()
        .stdout(predicate::str::contains("已删除"));
    Ok(())
}
