#!/bin/bash

# CCC (Claude Code Config) - Claude 配置管理工具
# 用于管理 ~/.claude/settings.json 中的 API 配置切换

set -euo pipefail

readonly SCRIPT_NAME="ccc"
readonly CLAUDE_SETTINGS="$HOME/.claude/settings.json"
readonly CCC_CONFIG="$HOME/.claude/ccc-config.json"

# 颜色定义
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m' # No Color

# 错误处理
error() {
    echo -e "${RED}错误: $1${NC}" >&2
    exit 1
}

info() {
    echo -e "${BLUE}信息: $1${NC}"
}

success() {
    echo -e "${GREEN}成功: $1${NC}"
}

warn() {
    echo -e "${YELLOW}警告: $1${NC}"
}

# 检查依赖
check_dependencies() {
    local missing_deps=()

    if ! command -v jq &> /dev/null; then
        missing_deps+=("jq")
    fi

    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        error "缺少依赖工具: ${missing_deps[*]}

安装方法:
  macOS: brew install ${missing_deps[*]}
  Ubuntu/Debian: sudo apt-get install ${missing_deps[*]}
  CentOS/RHEL: sudo yum install ${missing_deps[*]}"
    fi

    # 检查 Claude settings 目录
    if [[ ! -d "$(dirname "$CLAUDE_SETTINGS")" ]]; then
        error "Claude 配置目录不存在: $(dirname "$CLAUDE_SETTINGS")

请先安装并运行 Claude Code。"
    fi
}

# 备份 Claude settings 文件
backup_claude_settings() {
    local backup_file="${CLAUDE_SETTINGS}.backup.$(date +%Y%m%d_%H%M%S)"
    cp "$CLAUDE_SETTINGS" "$backup_file"
    info "已备份当前配置: $backup_file"
}

# 验证 JSON 格式
validate_json() {
    local file="$1"
    if ! jq empty "$file" &>/dev/null; then
        error "JSON 格式错误: $file"
    fi
}

# 显示帮助信息
show_help() {
    cat << EOF
CCC (Claude Code Config) - Claude 配置管理工具

用法:
    $SCRIPT_NAME <命令> [参数]

命令:
    install              安装脚本到系统路径
    init                 初始化配置管理
    list                 查看当前配置和所有可用配置
    add <name>          交互式添加新的配置档案
    import <name>       从当前 settings.json 导入配置档案
    rm <name>           删除指定配置档案
    use <name>          切换到指定配置

示例:
    $SCRIPT_NAME install
    $SCRIPT_NAME init
    $SCRIPT_NAME add kimi
    $SCRIPT_NAME import current
    $SCRIPT_NAME list
    $SCRIPT_NAME use kimi
    $SCRIPT_NAME rm old-config

注意:
    - 配置文件存储在 ~/.claude/ccc-config.json
    - 只会修改 apiKeyHelper 和 env.ANTHROPIC_* 字段
    - 其他 settings.json 字段保持不变
EOF
}

# 主函数
main() {
    check_dependencies

    if [[ $# -eq 0 ]]; then
        show_help
        exit 0
    fi

    case "${1:-}" in
        import)
            if [[ $# -ne 2 ]]; then
                error "import 命令需要指定配置名称: $SCRIPT_NAME import <name>"
            fi
            cmd_import "$2"
            ;;
        install)
            cmd_install
            ;;
        init)
            cmd_init
            ;;
        list)
            cmd_list
            ;;
        add)
            if [[ $# -ne 2 ]]; then
                error "add 命令需要指定配置名称: $SCRIPT_NAME add <name>"
            fi
            cmd_add "$2"
            ;;
        rm|remove)
            if [[ $# -ne 2 ]]; then
                error "rm 命令需要指定配置名称: $SCRIPT_NAME rm <name>"
            fi
            cmd_remove "$2"
            ;;
        use|switch)
            if [[ $# -ne 2 ]]; then
                error "use 命令需要指定配置名称: $SCRIPT_NAME use <name>"
            fi
            cmd_use "$2"
            ;;
        help|-h|--help)
            show_help
            ;;
        *)
            error "未知命令: $1。使用 '$SCRIPT_NAME help' 查看帮助"
            ;;
    esac
}

# 确保配置文件存在
ensure_config_file() {
    if [[ ! -f "$CCC_CONFIG" ]]; then
        info "初始化配置文件: $CCC_CONFIG"
        mkdir -p "$(dirname "$CCC_CONFIG")"
        echo '{"profiles": {}, "current": null}' > "$CCC_CONFIG"
    fi
}

# 确保 Claude settings 文件存在
ensure_claude_settings() {
    if [[ ! -f "$CLAUDE_SETTINGS" ]]; then
        error "Claude settings 文件不存在: $CLAUDE_SETTINGS"
    fi
}

# 读取配置文件
read_config() {
    ensure_config_file
    jq -r "$1" "$CCC_CONFIG" 2>/dev/null || echo "null"
}

# 写入配置文件
write_config() {
    local filter="$1"
    local temp_file
    temp_file=$(mktemp)

    ensure_config_file
    jq "$filter" "$CCC_CONFIG" > "$temp_file" && mv "$temp_file" "$CCC_CONFIG" || {
        rm -f "$temp_file"
        error "写入配置文件失败"
    }
}

# 获取当前活跃的配置名称
get_current_profile() {
    read_config '.current // empty'
}

# 智能检测当前活跃的配置
detect_active_profile() {
    ensure_claude_settings
    ensure_config_file

    # 获取当前 settings.json 中的配置
    local current_api_key_helper current_base_url current_api_key
    current_api_key_helper=$(jq -r '.apiKeyHelper // empty' "$CLAUDE_SETTINGS" 2>/dev/null || echo "")
    current_base_url=$(jq -r '.env.ANTHROPIC_BASE_URL // empty' "$CLAUDE_SETTINGS" 2>/dev/null || echo "")
    current_api_key=$(jq -r '.env.ANTHROPIC_API_KEY // empty' "$CLAUDE_SETTINGS" 2>/dev/null || echo "")

    # 遍历所有配置档案进行比较
    local profiles
    profiles=$(get_all_profiles)

    if [[ -z "$profiles" ]]; then
        echo ""
        return
    fi

    while IFS= read -r profile_name; do
        if [[ -z "$profile_name" ]]; then
            continue
        fi

        # 获取档案中的配置
        local profile_details
        profile_details=$(get_profile_details "$profile_name")

        local profile_api_key_helper profile_base_url profile_api_key
        profile_api_key_helper=$(echo "$profile_details" | jq -r '.apiKeyHelper // empty' 2>/dev/null || echo "")
        profile_base_url=$(echo "$profile_details" | jq -r '.env.ANTHROPIC_BASE_URL // empty' 2>/dev/null || echo "")
        profile_api_key=$(echo "$profile_details" | jq -r '.env.ANTHROPIC_API_KEY // empty' 2>/dev/null || echo "")

        # 比较核心字段是否匹配（base_url 和 api_key 是必须的）
        if [[ "${current_base_url:-}" == "${profile_base_url:-}" ]] && \
           [[ "${current_api_key:-}" == "${profile_api_key:-}" ]]; then
            echo "$profile_name"
            return
        fi
    done <<< "$profiles"

    # 没有找到匹配的配置
    echo ""
}

# 获取当前活跃的配置名称（智能检测版本）
get_current_profile_smart() {
    local detected_profile stored_profile

    # 智能检测当前配置
    detected_profile=$(detect_active_profile)

    # 获取存储的当前配置
    stored_profile=$(get_current_profile)

    # 如果检测到配置且与存储的不同，自动更新
    if [[ -n "$detected_profile" && "$detected_profile" != "$stored_profile" ]]; then
        write_config ".current = \"$detected_profile\""
        echo "$detected_profile"
    elif [[ -n "$detected_profile" ]]; then
        echo "$detected_profile"
    else
        # 没有检测到匹配的配置，返回空
        echo ""
    fi
}

# 获取所有配置档案名称
get_all_profiles() {
    read_config '.profiles | keys[]'
}

# 检查配置档案是否存在
profile_exists() {
    local name="$1"
    local exists
    exists=$(read_config ".profiles | has(\"$name\")")
    [[ "$exists" == "true" ]]
}

# 获取指定配置档案的详细信息
get_profile_details() {
    local name="$1"
    read_config ".profiles[\"$name\"]"
}

# 从当前 Claude settings 中提取相关配置
extract_current_settings() {
    ensure_claude_settings
    local apiKeyHelper baseUrl apiKey

    apiKeyHelper=$(jq -r '.apiKeyHelper // empty' "$CLAUDE_SETTINGS")
    baseUrl=$(jq -r '.env.ANTHROPIC_BASE_URL // empty' "$CLAUDE_SETTINGS")
    apiKey=$(jq -r '.env.ANTHROPIC_API_KEY // empty' "$CLAUDE_SETTINGS")

    jq -n \
        --arg apiKeyHelper "$apiKeyHelper" \
        --arg baseUrl "$baseUrl" \
        --arg apiKey "$apiKey" \
        '{
            apiKeyHelper: (if $apiKeyHelper == "" then null else $apiKeyHelper end),
            env: {
                ANTHROPIC_BASE_URL: (if $baseUrl == "" then null else $baseUrl end),
                ANTHROPIC_API_KEY: (if $apiKey == "" then null else $apiKey end)
            }
        }'
}

# 应用配置到 Claude settings
apply_profile_to_settings() {
    local profile_data="$1"
    local temp_file
    temp_file=$(mktemp)

    ensure_claude_settings
    validate_json "$CLAUDE_SETTINGS"

    # 创建备份
    backup_claude_settings

    # 使用 jq 合并配置，只更新指定字段
    jq \
        --argjson profile "$profile_data" \
        '
        .apiKeyHelper = ($profile.apiKeyHelper // .apiKeyHelper) |
        .env.ANTHROPIC_BASE_URL = ($profile.env.ANTHROPIC_BASE_URL // .env.ANTHROPIC_BASE_URL) |
        .env.ANTHROPIC_API_KEY = ($profile.env.ANTHROPIC_API_KEY // .env.ANTHROPIC_API_KEY)
        ' \
        "$CLAUDE_SETTINGS" > "$temp_file" && mv "$temp_file" "$CLAUDE_SETTINGS" || {
        rm -f "$temp_file"
        error "应用配置到 Claude settings 失败"
    }

    # 验证生成的 JSON
    validate_json "$CLAUDE_SETTINGS"
}

# 命令实现
cmd_import() {
    local name="$1"

    # 验证配置名称
    if [[ ! "$name" =~ ^[a-zA-Z0-9_-]+$ ]]; then
        error "配置名称只能包含字母、数字、下划线和连字符"
    fi

    if profile_exists "$name"; then
        error "配置档案 '$name' 已存在"
    fi

    # 从当前 Claude settings 提取配置
    local current_settings
    current_settings=$(extract_current_settings)

    # 保存到配置文件
    write_config ".profiles[\"$name\"] = $current_settings"

    success "配置档案 '$name' 已从当前 settings.json 导入"
    info "提示: 使用 'ccc use $name' 切换到此配置"
}

cmd_install() {
    local target_dir="/usr/local/bin"
    local target_file="$target_dir/ccc"
    local script_path
    script_path="$(readlink -f "$0" 2>/dev/null || realpath "$0" 2>/dev/null || echo "$0")"

    info "正在安装 CCE 到系统路径..."

    # 检查目标目录是否存在
    if [[ ! -d "$target_dir" ]]; then
        error "目标目录不存在: $target_dir"
    fi

    # 检查是否有写权限
    if [[ ! -w "$target_dir" ]]; then
        echo "需要管理员权限安装到 $target_dir"
        echo "请运行: sudo cp \"$script_path\" \"$target_file\""
        echo "然后运行: sudo chmod +x \"$target_file\""
        exit 1
    fi

    # 复制文件
    cp "$script_path" "$target_file" || error "复制文件失败"
    chmod +x "$target_file" || error "设置可执行权限失败"

    success "CCC 已成功安装到: $target_file"
    info "现在可以在任何位置使用 'ccc' 命令"
    echo
    echo "建议下一步:"
    echo "  ccc init           # 初始化配置管理"
    echo "  ccc add default    # 保存当前配置"
}

cmd_init() {
    ensure_config_file

    info "正在初始化 CCC 配置管理..."

    success "CCC 配置管理已初始化"
    echo
    echo "下一步:"
    echo "  1. 添加配置档案 (两种方式):"
    echo "     - 交互式: ccc add <name>    # 手动输入配置信息"
    echo "     - 导入式: ccc import <name> # 从当前 settings.json 导入"
    echo "  2. 使用 'ccc list' 查看所有配置"
    echo "  3. 使用 'ccc use <name>' 切换配置"
    echo
    echo "示例:"
    echo "  ccc add kimi       # 交互式添加 kimi 配置"
    echo "  ccc import work    # 导入当前配置为 work"
    echo "  ccc use kimi       # 切换到 kimi 配置"
}

cmd_list() {
    ensure_config_file

    local current_profile detected_profile
    detected_profile=$(detect_active_profile)
    current_profile="$detected_profile"

    echo "=== Claude Code 配置管理 ==="
    echo

    if [[ -n "$current_profile" ]]; then
        echo -e "${GREEN}当前活跃配置: $current_profile${NC} (智能检测)"
        local details
        details=$(get_profile_details "$current_profile")

        local api_key_helper base_url api_key
        api_key_helper=$(echo "$details" | jq -r '.apiKeyHelper // "未设置"')
        base_url=$(echo "$details" | jq -r '.env.ANTHROPIC_BASE_URL // "未设置"')
        api_key=$(echo "$details" | jq -r '.env.ANTHROPIC_API_KEY // "未设置"')

        # 隐藏 API Key Helper 的具体内容
        if [[ "$api_key_helper" != "未设置" ]]; then
            echo "  API Key Helper: [已配置]"
        else
            echo "  API Key Helper: 未设置"
        fi

        echo "  Base URL: $base_url"

        # 隐藏 API Key 的具体内容，只显示前缀和后缀
        if [[ "$api_key" != "未设置" && "$api_key" =~ ^sk-.+ ]]; then
            echo "  API Key: sk-***[已配置]"
        elif [[ "$api_key" != "未设置" ]]; then
            echo "  API Key: ***[已配置]"
        else
            echo "  API Key: 未设置"
        fi

        # 自动更新存储的当前配置
        local stored_profile
        stored_profile=$(get_current_profile)
        if [[ "$current_profile" != "$stored_profile" ]]; then
            write_config ".current = \"$current_profile\""
        fi
    else
        # 检查是否有当前配置但未匹配任何档案
        ensure_claude_settings
        local has_config
        has_config=$(jq -r '.env.ANTHROPIC_API_KEY // empty' "$CLAUDE_SETTINGS" 2>/dev/null)

        if [[ -n "$has_config" ]]; then
            echo -e "${YELLOW}当前配置未保存为档案${NC}"
            echo "  提示: 使用 'ccc import <name>' 保存当前配置"
        else
            echo -e "${YELLOW}当前无活跃配置${NC}"
        fi
    fi

    echo
    echo "可用配置档案:"

    local profiles
    profiles=$(get_all_profiles)

    if [[ -z "$profiles" ]]; then
        echo "  (无配置档案)"
    else
        while IFS= read -r profile; do
            if [[ "$profile" == "$current_profile" ]]; then
                echo -e "  ${GREEN}* $profile${NC} (当前)"
            else
                echo "  $profile"
            fi
        done <<< "$profiles"
    fi
}

cmd_add() {
    local name="$1"

    # 验证配置名称
    if [[ ! "$name" =~ ^[a-zA-Z0-9_-]+$ ]]; then
        error "配置名称只能包含字母、数字、下划线和连字符"
    fi

    if profile_exists "$name"; then
        error "配置档案 '$name' 已存在"
    fi

    echo "创建配置档案: $name"
    echo

    # 交互式获取配置信息
    echo -n "请输入 Base URL (例如: https://api.anthropic.com): "
    read -r base_url

    if [[ -z "$base_url" ]]; then
        error "Base URL 不能为空"
    fi

    echo -n "请输入 API Key (例如: sk-xxx): "
    read -r api_key

    if [[ -z "$api_key" ]]; then
        error "API Key 不能为空"
    fi

    # 询问是否需要 API Key Helper
    echo -n "是否需要 API Key Helper? (y/N): "
    read -r need_helper

    local api_key_helper=""
    if [[ "$need_helper" =~ ^[Yy]$ ]]; then
        echo -n "请输入 API Key Helper 命令 (例如: echo 'sk-xxx'): "
        read -r api_key_helper
    fi

    # 构建配置对象
    local config_data
    if [[ -n "$api_key_helper" ]]; then
        config_data=$(jq -n \
            --arg apiKeyHelper "$api_key_helper" \
            --arg baseUrl "$base_url" \
            --arg apiKey "$api_key" \
            '{
                apiKeyHelper: $apiKeyHelper,
                env: {
                    ANTHROPIC_BASE_URL: $baseUrl,
                    ANTHROPIC_API_KEY: $apiKey
                }
            }')
    else
        config_data=$(jq -n \
            --arg baseUrl "$base_url" \
            --arg apiKey "$api_key" \
            '{
                env: {
                    ANTHROPIC_BASE_URL: $baseUrl,
                    ANTHROPIC_API_KEY: $apiKey
                }
            }')
    fi

    # 保存到配置文件
    write_config ".profiles[\"$name\"] = $config_data"

    echo
    success "配置档案 '$name' 已创建"
    info "提示: 使用 'ccc use $name' 切换到此配置"
}

cmd_remove() {
    local name="$1"

    if ! profile_exists "$name"; then
        error "配置档案 '$name' 不存在"
    fi

    local current_profile
    current_profile=$(get_current_profile)

    # 如果删除的是当前配置，需要清除当前配置
    if [[ "$name" == "$current_profile" ]]; then
        write_config '.current = null | del(.profiles["'"$name"'"])'
        warn "已删除当前活跃配置，请使用 'ccc use <name>' 切换到其他配置"
    else
        write_config 'del(.profiles["'"$name"'"])'
    fi

    success "配置档案 '$name' 已删除"
}

cmd_use() {
    local name="$1"

    if ! profile_exists "$name"; then
        error "配置档案 '$name' 不存在"
    fi

    # 获取配置详情并应用
    local profile_data
    profile_data=$(get_profile_details "$name")

    apply_profile_to_settings "$profile_data"

    # 更新当前配置
    write_config ".current = \"$name\""

    success "已切换到配置档案: $name"
}

# 运行主函数
main "$@"