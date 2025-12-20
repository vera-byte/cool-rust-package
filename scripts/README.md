# 脚本说明

## create-tag.sh

根据 `Cargo.toml` 中的版本号自动创建 git tag。

### 使用方法

```bash
# 在项目根目录执行
./scripts/create-tag.sh

# 或者从任何位置执行
cd cool-rust-package && ./scripts/create-tag.sh
```

### 功能

1. ✅ 自动从 `Cargo.toml` 的 `[workspace.package]` 中提取版本号
2. ✅ 创建格式为 `v{version}` 的 tag（如 `v0.1.0`）
3. ✅ 检查 tag 是否已存在，避免重复创建
4. ✅ 检查工作区是否有未提交的更改
5. ✅ 可选择是否推送到远程仓库

### 示例

```bash
# Cargo.toml 中版本为 0.1.0
[workspace.package]
version = "0.1.0"

# 执行脚本后会自动创建 tag: v0.1.0
./scripts/create-tag.sh
```

### 注意事项

- 脚本会创建带注释的 tag（annotated tag）
- 如果 tag 已存在，会询问是否删除并重新创建
- 如果有未提交的更改，会给出警告
- 推送 tag 后，GitHub Actions 会自动触发发布流程

