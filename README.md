# Claude Code Config RS (cccrs)

用于管理 Claude Code 配置的命令行工具，支持在多个 API 配置档案之间快速切换。

## 功能特性

- **多配置管理**: 创建和管理多个 API 配置档案
- **智能检测**: 自动检测当前活跃配置
- **安全备份**: 切换配置时自动备份当前设置
- **交互式添加**: 提供友好的交互式配置添加流程
- **配置导入**: 可从当前 settings.json 导入配置

## 安装

```bash
# 从源码构建
cargo build --release

# 安装到系统路径（需要管理员权限）
./target/release/cccrs install
# 安装位置: /usr/local/bin/cccrs
```

## 使用方法

### 初始化

```bash
cccrs init
```

### 添加配置

**交互式添加**

```bash
cccrs add <配置名称>
# 按提示输入 Base URL 和 API Key
```

**从当前设置导入**

```bash
cccrs import <配置名称>
# 从 ~/.claude/settings.json 导入当前配置
```

### 查看配置

```bash
cccrs list
# 显示当前活跃配置和所有可用配置档案
```

### 切换配置

```bash
cccrs use <配置名称>
# 切换到指定配置（自动备份当前配置）
```

### 删除配置

```bash
cccrs remove <配置名称>
# 别名: cccrs rm / cccrs del
```

## 配置文件

### cccrs 配置文件

位置：`~/.claude/cccrs-config.json`

```json
{
  "profiles": {
    "配置名称": {
      "api_key_helper": "命令（可选）",
      "env": {
        "ANTHROPIC_BASE_URL": "https://api.example.com",
        "ANTHROPIC_AUTH_TOKEN": "sk-xxx"
      }
    }
  },
  "current": "当前配置名称"
}
```

### Claude Settings 文件

位置：`~/.claude/settings.json`

cccrs 只会修改以下字段，其他字段保持不变：

- `apiKeyHelper`
- `env.ANTHROPIC_BASE_URL`
- `env.ANTHROPIC_AUTH_TOKEN`

## 使用示例

```bash
# 1. 初始化
cccrs init

# 2. 添加配置档案
cccrs add kimi
# 输入 Base URL: https://api.moonshot.cn
# 输入 API Key: sk-xxx

# 3. 添加另一个配置
cccrs add anthropic
# 输入 Base URL: https://api.anthropic.com
# 输入 API Key: sk-xxx

# 4. 查看所有配置
cccrs list

# 5. 切换配置
cccrs use kimi

# 6. 删除配置
cccrs rm old-config
```

## 备份机制

每次使用 `cccrs use` 切换配置时，会自动创建 settings.json 的备份：

- 备份位置：`~/.claude/settings.json.backup.{时间戳}`
- 格式：`YYYYMMDD_HHMMSS`

## 智能检测

`cccrs list` 命令会自动检测当前活跃配置：

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
cargo test
```

### 代码检查

```bash
cargo clippy
cargo fmt
```

## 许可证

Apache-2.0
