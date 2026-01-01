//! CLI 测试模块
//!
//! 使用 assert_cmd 进行 CLI 行为测试

#[allow(deprecated)]
use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;
use tempfile::TempDir;

/// 设置临时 HOME 目录
fn setup_temp_home() -> (TempDir, PathBuf, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let temp_home = temp_dir.path().join("test_home");
    std::fs::create_dir_all(&temp_home).unwrap();

    let claude_dir = temp_home.join(".claude");
    std::fs::create_dir_all(&claude_dir).unwrap();

    let settings_path = claude_dir.join("settings.json");
    let ccc_config_path = claude_dir.join("ccc-config.json");

    // 设置 HOME 环境变量
    unsafe {
        std::env::set_var("HOME", temp_home.to_string_lossy().as_ref());
    }

    (temp_dir, settings_path, ccc_config_path)
}

/// 创建初始 settings.json
fn create_initial_settings(path: &PathBuf) {
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
    .unwrap();
}

#[test]
#[allow(deprecated)]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("claude-code-config-rs").unwrap();
    cmd.arg("help")
        .assert()
        .success()
        .stdout(predicate::str::contains("CCC (Claude Code Config)"));
}

#[test]
#[allow(deprecated)]
fn test_init_command() {
    let (_temp_dir, settings_path, ccc_config_path) = setup_temp_home();

    // 创建初始 settings.json
    create_initial_settings(&settings_path);

    let mut cmd = Command::cargo_bin("claude-code-config-rs").unwrap();
    cmd.arg("init").assert().success();

    // 验证配置文件创建
    assert!(ccc_config_path.exists());
    let content = std::fs::read_to_string(&ccc_config_path).unwrap();
    assert!(content.contains("profiles"));
    assert!(content.contains("current"));
}

#[test]
#[allow(deprecated)]
fn test_list_empty() {
    let (_temp_dir, settings_path, _ccc_config_path) = setup_temp_home();

    // 创建初始 settings.json
    create_initial_settings(&settings_path);

    // 先初始化
    let mut cmd = Command::cargo_bin("claude-code-config-rs").unwrap();
    cmd.arg("init").assert().success();

    // list 应该显示无配置档案
    let mut cmd = Command::cargo_bin("claude-code-config-rs").unwrap();
    cmd.arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("可用配置档案"))
        .stdout(predicate::str::contains("无配置档案"));
}

#[test]
#[allow(deprecated)]
fn test_import_and_list() {
    let (_temp_dir, settings_path, ccc_config_path) = setup_temp_home();

    // 创建初始 settings.json
    create_initial_settings(&settings_path);

    // 初始化
    let mut cmd = Command::cargo_bin("claude-code-config-rs").unwrap();
    cmd.arg("init").assert().success();

    // 导入当前配置
    let mut cmd = Command::cargo_bin("claude-code-config-rs").unwrap();
    cmd.arg("import")
        .arg("anthropic")
        .assert()
        .success()
        .stdout(predicate::str::contains("已从当前 settings.json 导入"));

    // 验证配置已添加
    let config_content = std::fs::read_to_string(&ccc_config_path).unwrap();
    let config: serde_json::Value = serde_json::from_str(&config_content).unwrap();
    assert!(config["profiles"]["anthropic"].is_object());
    assert_eq!(
        config["profiles"]["anthropic"]["env"]["ANTHROPIC_BASE_URL"],
        "https://api.anthropic.com"
    );
    assert_eq!(
        config["profiles"]["anthropic"]["env"]["ANTHROPIC_AUTH_TOKEN"],
        "sk-test-key"
    );

    // list 应该显示配置
    let mut cmd = Command::cargo_bin("claude-code-config-rs").unwrap();
    cmd.arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("anthropic"))
        .stdout(predicate::str::contains("当前活跃配置"));
}

#[test]
#[allow(deprecated)]
fn test_use_command_updates_settings() {
    let (temp_dir, settings_path, ccc_config_path) = setup_temp_home();

    // 创建初始 settings.json
    create_initial_settings(&settings_path);

    // 初始化并导入配置
    let mut cmd = Command::cargo_bin("claude-code-config-rs").unwrap();
    cmd.arg("init").assert().success();

    let mut cmd = Command::cargo_bin("claude-code-config-rs").unwrap();
    cmd.arg("import").arg("anthropic").assert().success();

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
    std::fs::write(&ccc_config_path, config).unwrap();

    // 切换到新配置
    let mut cmd = Command::cargo_bin("claude-code-config-rs").unwrap();
    cmd.arg("use")
        .arg("new-profile")
        .assert()
        .success()
        .stdout(predicate::str::contains("已切换到配置档案"));

    // 验证 settings.json 已更新，且保留其他字段
    let settings_content = std::fs::read_to_string(&settings_path).unwrap();
    let settings: serde_json::Value = serde_json::from_str(&settings_content).unwrap();

    assert_eq!(settings["env"]["ANTHROPIC_BASE_URL"], "https://api.new.com");
    assert_eq!(settings["env"]["ANTHROPIC_AUTH_TOKEN"], "sk-new-key");
    assert_eq!(settings["otherField"], "should remain");

    // 验证备份文件已创建
    let backups: Vec<_> = std::fs::read_dir(temp_dir.path().join("test_home/.claude"))
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.to_string_lossy().contains("backup"))
        .collect();
    assert!(!backups.is_empty(), "Backup file should be created");
}

#[test]
#[allow(deprecated)]
fn test_remove_command() {
    let (_temp_dir, settings_path, ccc_config_path) = setup_temp_home();

    // 创建初始 settings.json
    create_initial_settings(&settings_path);

    // 初始化并导入配置
    let mut cmd = Command::cargo_bin("claude-code-config-rs").unwrap();
    cmd.arg("init").assert().success();

    let mut cmd = Command::cargo_bin("claude-code-config-rs").unwrap();
    cmd.arg("import").arg("test-profile").assert().success();

    // 删除配置
    let mut cmd = Command::cargo_bin("claude-code-config-rs").unwrap();
    cmd.arg("rm")
        .arg("test-profile")
        .assert()
        .success()
        .stdout(predicate::str::contains("已删除"));

    // 验证配置已删除
    let config_content = std::fs::read_to_string(&ccc_config_path).unwrap();
    let config: serde_json::Value = serde_json::from_str(&config_content).unwrap();
    assert!(config["profiles"]["test-profile"].is_null());
}

#[test]
#[allow(deprecated)]
fn test_remove_current_profile() {
    let (_temp_dir, settings_path, ccc_config_path) = setup_temp_home();

    // 创建初始 settings.json
    create_initial_settings(&settings_path);

    // 初始化并导入配置
    let mut cmd = Command::cargo_bin("claude-code-config-rs").unwrap();
    cmd.arg("init").assert().success();

    let mut cmd = Command::cargo_bin("claude-code-config-rs").unwrap();
    cmd.arg("import").arg("current").assert().success();

    // 切换到当前配置
    let mut cmd = Command::cargo_bin("claude-code-config-rs").unwrap();
    cmd.arg("use").arg("current").assert().success();

    // 删除当前配置
    let mut cmd = Command::cargo_bin("claude-code-config-rs").unwrap();
    cmd.arg("rm")
        .arg("current")
        .assert()
        .success()
        .stdout(predicate::str::contains("已删除当前活跃配置"));

    // 验证当前配置已被清除
    let config_content = std::fs::read_to_string(&ccc_config_path).unwrap();
    let config: serde_json::Value = serde_json::from_str(&config_content).unwrap();
    assert!(config["profiles"]["current"].is_null());
    assert!(config["current"].is_null());
}

#[test]
#[allow(deprecated)]
fn test_import_nonexistent_profile() {
    let (_temp_dir, _settings_path, _ccc_config_path) = setup_temp_home();

    let mut cmd = Command::cargo_bin("claude-code-config-rs").unwrap();
    cmd.arg("rm")
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("不存在"));
}

#[test]
#[allow(deprecated)]
fn test_use_nonexistent_profile() {
    let (_temp_dir, _settings_path, _ccc_config_path) = setup_temp_home();

    let mut cmd = Command::cargo_bin("claude-code-config-rs").unwrap();
    cmd.arg("use")
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("不存在"));
}

#[test]
#[allow(deprecated)]
fn test_import_invalid_name() {
    let (_temp_dir, _settings_path, _ccc_config_path) = setup_temp_home();

    let mut cmd = Command::cargo_bin("claude-code-config-rs").unwrap();
    cmd.arg("import")
        .arg("invalid name with spaces")
        .assert()
        .failure();
}

#[test]
#[allow(deprecated)]
fn test_add_invalid_name() {
    let (_temp_dir, _settings_path, _ccc_config_path) = setup_temp_home();

    let mut cmd = Command::cargo_bin("claude-code-config-rs").unwrap();
    cmd.arg("add").arg("invalid@name").assert().failure();
}
