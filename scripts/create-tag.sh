#!/bin/bash
# 根据 Cargo.toml 中的版本号自动创建 git tag

set -e

# 获取脚本所在目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CARGO_TOML="$PROJECT_ROOT/Cargo.toml"

# 检查 Cargo.toml 是否存在
if [ ! -f "$CARGO_TOML" ]; then
    echo "❌ 错误: 找不到 Cargo.toml 文件: $CARGO_TOML"
    exit 1
fi

# 从 Cargo.toml 中提取版本号
# 查找 [workspace.package] 下的 version = "x.y.z"
VERSION=$(grep -A 10 "\[workspace.package\]" "$CARGO_TOML" | grep "version" | head -1 | sed -E 's/.*version[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/')

if [ -z "$VERSION" ]; then
    echo "❌ 错误: 无法从 Cargo.toml 中提取版本号"
    echo "请确保 Cargo.toml 中包含:"
    echo "  [workspace.package]"
    echo "  version = \"x.y.z\""
    exit 1
fi

# 创建 tag 名称 (格式: v0.1.0)
TAG_NAME="v${VERSION}"

echo "📦 从 Cargo.toml 提取的版本号: $VERSION"
echo "🏷️  将要创建的 tag: $TAG_NAME"

# 检查 tag 是否已存在
if git rev-parse "$TAG_NAME" >/dev/null 2>&1; then
    echo "⚠️  警告: tag $TAG_NAME 已存在"
    read -p "是否要删除并重新创建? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "🗑️  删除现有 tag..."
        git tag -d "$TAG_NAME" 2>/dev/null || true
        git push origin ":refs/tags/$TAG_NAME" 2>/dev/null || true
    else
        echo "❌ 取消操作"
        exit 1
    fi
fi

# 检查工作区是否有未提交的更改
if ! git diff-index --quiet HEAD --; then
    echo "⚠️  警告: 工作区有未提交的更改"
    read -p "是否继续创建 tag? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "❌ 取消操作"
        exit 1
    fi
fi

# 创建带注释的 tag
echo "🏷️  创建 tag: $TAG_NAME"
git tag -a "$TAG_NAME" -m "Release version $VERSION"

echo "✅ Tag 创建成功: $TAG_NAME"

# 询问是否推送
read -p "是否推送到远程仓库? (Y/n): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Nn]$ ]]; then
    echo "📤 推送 tag 到远程仓库..."
    git push origin "$TAG_NAME"
    echo "✅ Tag 已推送到远程仓库"
else
    echo "💡 提示: 使用以下命令手动推送 tag:"
    echo "   git push origin $TAG_NAME"
fi

echo ""
echo "🎉 完成! tag $TAG_NAME 已创建"
echo "💡 提示: 推送 tag 后，GitHub Actions 会自动触发发布流程"

