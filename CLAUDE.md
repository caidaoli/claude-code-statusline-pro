# Claude Code Statusline Pro - 开发指引

## 开发流程

在完成任何开发任务并提交修改之前，执行完整 CI 检查：

```bash
make ci
```

这将按顺序执行以下步骤：

1. `make fix` - 自动应用编译器建议
2. `make fmt` - 格式化代码
3. `make clippy-fix` - Clippy 自动修复
4. `make clippy` - Clippy 严格检查
5. `make check` - 编译检查
6. `make test` - 运行测试
7. `make build` - Release 构建

若某一步失败，应先修复问题并重新执行 `make ci`，直至全部通过。

## 常用命令

| 命令 | 说明 |
|------|------|
| `make ci` | 完整 CI 流程（推荐提交前执行） |
| `make quick` | 快速检查（跳过自动修复） |
| `make test` | 仅运行测试 |
| `make bump V=x.x.x` | 更新版本号 |

## 发布流程

```bash
# 1. 运行完整检查
make ci

# 2. 更新版本号
make bump V=3.0.4

# 3. 提交并打 tag
git add -A && git commit -m "chore: 更新版本号至 3.0.4"
git tag v3.0.4

# 4. 推送（触发 CI 构建和 npm 发布）
git push origin main --tags
```

## 项目结构

- `src/` - Rust 源码
- `npm/` - npm 包配置
- `tests/` - 集成测试
- `.github/workflows/` - CI 配置
