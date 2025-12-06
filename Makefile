# Claude Code Statusline Pro - 开发工具链
# 用法: make <target>

.PHONY: all fix fmt clippy check test build release clean ci bump

# 默认目标：完整 CI 流程
all: ci

# 1. 自动修复编译器建议
fix:
	cargo fix --workspace --all-features --allow-dirty

# 2. 格式化代码
fmt:
	cargo fmt --all

# 3. Clippy 自动修复
clippy-fix:
	cargo clippy --fix --workspace --all-features --allow-dirty -- -D warnings

# 4. Clippy 检查（严格模式）
clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

# 5. 编译检查
check:
	cargo check --workspace --all-targets --all-features

# 6. 运行测试
test:
	cargo test --workspace --all-targets --all-features -- --nocapture

# 7. Release 构建
build:
	cargo build --release

# 完整 CI 流程（按顺序执行所有步骤）
ci: fix fmt clippy-fix clippy check test build
	@echo "✅ All checks passed!"

# 快速检查（跳过自动修复）
quick: fmt clippy check test
	@echo "✅ Quick check passed!"

# 清理构建产物
clean:
	cargo clean

# 版本更新 (用法: make bump V=3.0.4)
bump:
ifndef V
	$(error 请指定版本号，例如: make bump V=3.0.4)
endif
	sed -i '' 's/^version = "[^"]*"/version = "$(V)"/' Cargo.toml
	find npm -name "package.json" -exec sed -i '' 's/"version": "[^"]*"/"version": "$(V)"/g' {} \;
	cargo generate-lockfile
	@echo "✅ Version bumped to $(V)"

# 帮助信息
help:
	@echo "可用命令:"
	@echo "  make ci      - 完整 CI 流程（推荐提交前执行）"
	@echo "  make quick   - 快速检查（跳过自动修复）"
	@echo "  make fix     - 自动修复编译器建议"
	@echo "  make fmt     - 格式化代码"
	@echo "  make clippy  - Clippy 静态检查"
	@echo "  make check   - 编译检查"
	@echo "  make test    - 运行测试"
	@echo "  make build   - Release 构建"
	@echo "  make clean   - 清理构建产物"
	@echo "  make bump V=x.x.x - 更新版本号"
