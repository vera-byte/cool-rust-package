.PHONY: help tag create-tag version

help: ## 显示帮助信息
	@echo "可用命令:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

version: ## 显示当前版本号
	@cd "$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))" && \
	grep -A 10 "\[workspace.package\]" Cargo.toml | grep "version" | head -1 | sed -E 's/.*version[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/'

sync-versions: ## 同步 workspace 版本号到所有内部依赖
	@./scripts/sync-versions.sh

tag: create-tag ## 创建 git tag（别名）

create-tag: ## 根据 Cargo.toml 版本号自动创建 git tag
	@./scripts/create-tag.sh

