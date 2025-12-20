# 发布流程说明

## 前置条件

1. **设置 crates.io Token**
   - 访问 https://crates.io/settings/tokens
   - 创建一个新的 API token（需要 `publish` 权限）
   - 在 GitHub 仓库设置中添加 Secret：`CRATES_IO_TOKEN`

2. **确保所有包已准备好发布**
   - 检查所有 `Cargo.toml` 中的版本号
   - 确保所有依赖都已正确配置
   - 运行 `cargo publish --dry-run` 验证

## 发布步骤

### 1. 更新版本号

在 `Cargo.toml` 中更新 workspace 版本：

```toml
[workspace.package]
version = "0.1.0"  # 更新为新版本
```

### 2. 提交更改

```bash
git add .
git commit -m "chore: bump version to 0.1.0"
git push
```

### 3. 创建版本标签

```bash
# 创建并推送标签
git tag v0.1.0
git push origin v0.1.0
```

或者使用带注释的标签：

```bash
git tag -a v0.1.0 -m "Release version 0.1.0"
git push origin v0.1.0
```

### 4. 自动发布

推送标签后，GitHub Actions 会自动：

1. ✅ 运行构建和测试
2. ✅ 按依赖顺序发布所有包到 crates.io
3. ✅ 创建 GitHub Release

## 发布顺序

包会按以下顺序发布（确保依赖关系正确）：

1. **第一层（无内部依赖）**
   - `cool-macros`
   - `cool-core`

2. **第二层（依赖 cool-core）**
   - `cool-task`
   - `cool-es`
   - `cool-plugin`
   - `cool-rpc`

## 手动触发

如果需要在 GitHub Actions 中手动触发工作流：

1. 进入仓库的 Actions 页面
2. 选择 "Rust CI" 工作流
3. 点击 "Run workflow"
4. 选择分支并运行

## 故障排查

### 发布失败

- 检查 `CRATES_IO_TOKEN` 是否正确设置
- 确认版本号已更新且未重复
- 查看 GitHub Actions 日志获取详细错误信息

### 依赖问题

- 确保所有依赖的包都已成功发布
- 等待 crates.io 索引更新（通常需要几分钟）

### 重试发布

如果某个包发布失败，可以：

1. 修复问题后重新推送标签
2. 或者手动发布单个包：`cargo publish -p <package-name>`

## 注意事项

- ⚠️ 发布到 crates.io 是**不可逆**的，确保版本号正确
- ⚠️ 发布前务必运行完整的测试和 lint 检查
- ⚠️ 确保所有包的 `Cargo.toml` 中的元数据（description, license 等）都已正确填写

