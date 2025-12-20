#!/bin/bash
# 自动同步 workspace 版本号到所有内部依赖

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CARGO_TOML="$PROJECT_ROOT/Cargo.toml"

# 从 workspace.package 中提取版本号
VERSION=$(grep -A 1 "\[workspace.package\]" "$CARGO_TOML" | grep "version" | head -1 | sed -E 's/.*version[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/')

if [ -z "$VERSION" ]; then
    echo "❌ 错误: 无法从 Cargo.toml 中提取版本号"
    exit 1
fi

echo "📦 当前 workspace 版本: $VERSION"
echo "🔄 同步版本号到 workspace.dependencies..."

# 更新 workspace.dependencies 中的内部依赖版本
sed -i.bak -E "s/^(cool-(core|macros|task|rpc|es|plugin)) = \{ path = \"[^\"]+\", version = \"[^\"]+\" \}/\1 = { path = \"\1\", version = \"$VERSION\" }/" "$CARGO_TOML"

# 清理备份文件
rm -f "$CARGO_TOML.bak"

echo "✅ 版本号已同步到所有内部依赖"
echo ""
echo "📝 更新内容:"
grep -A 6 "# 内部依赖" "$CARGO_TOML" | grep "cool-"

