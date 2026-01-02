# Claude Code Config (ccc) - 使用说明

## 概述

CCC 是一个用于管理 Claude Code 配置的命令行工具，支持在多个配置档案之间快速切换。

## 功能特性

- **多配置管理**: 支持创建多个 API 配置档案
- **智能检测**: 自动检测当前活跃配置
- **安全备份**: 切换配置时自动备份当前设置
- **交互式添加**: 提供友好的交互式配置添加流程
- **配置导入**: 可从当前 settings.json 导入配置

## 安装

```bash
# 从源码构建
cargo build --release

# 安装到系统路径（需要管理员权限）
./target/release/claude-code-config-rs install
```

## 使用方法

### 初始化

```bash
# 初始化配置管理
ccc init
```

### 添加配置

**方式一：交互式添加**
```bash
ccc add <配置名称>
# 提示输入 Base URL 和 API Key
```

**方式二：从当前设置导入**
```bash
ccc import <配置名称>
# 从 ~/.claude/settings.json 导入当前配置
```

### 查看配置

```bash
ccc list
# 显示当前活跃配置和所有可用配置档案
```

### 切换配置

```bash
ccc use <配置名称>
# 切换到指定配置（自动备份当前配置）
```

### 删除配置

```bash
ccc rm <配置名称>
# 删除指定配置档案
```

## 配置文件

### CCC 配置文件
位置：`~/.claude/cccrs-config.json`

```json
{
  "profiles": {
    "配置名称": {
      "apiKeyHelper": "命令（可选）",
      "env": {
        "ANTHROPIC_BASE_URL": "Base URL",
        "ANTHROPIC_AUTH_TOKEN": "API Key"
      }
    }
  },
  "current": "当前配置名称"
}
```

### Claude Settings 文件
位置：`~/.claude/settings.json`

CCC 只会修改以下字段，其他字段保持不变：
- `apiKeyHelper`
- `env.ANTHROPIC_BASE_URL`
- `env.ANTHROPIC_AUTH_TOKEN`

## 使用示例

```bash
# 1. 初始化
ccc init

# 2. 添加配置档案
ccc add kimi
# 输入 Base URL: https://api.moonshot.cn
# 输入 API Key: sk-xxx

# 3. 添加另一个配置
ccc add anthropic
# 输入 Base URL: https://api.anthropic.com
# 输入 API Key: sk-xxx

# 4. 查看所有配置
ccc list

# 5. 切换配置
ccc use kimi

# 6. 删除配置
ccc rm old-config
```

## 备份机制

每次使用 `ccc use` 切换配置时，会自动创建 settings.json 的备份：
- 备份位置：`~/.claude/settings.json.backup.{时间戳}`
- 格式：`YYYYMMDD_HHMMSS`

## 智能检测

`ccc list` 命令会自动检测当前活跃配置：
- 通过比较 settings.json 和 cccrs-config.json 中的配置
- 如果检测到匹配的配置，会自动更新 `current` 字段

## 开发

### 构建

```bash
# Debug 构建
cargo build

# Release 构建
cargo build --release
```

### 测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_help_command
```

### 代码检查

```bash
# 检查代码警告
cargo clippy

# 修复可自动修复的警告
cargo clippy --fix

# 格式化代码
cargo fmt
```

## 许可证

MIT
