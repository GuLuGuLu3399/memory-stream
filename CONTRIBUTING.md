# Contributing to Memory Stream

## 开发环境搭建

### 前置依赖

- **Node.js** ≥ 20 + pnpm ≥ 9
- **Rust** stable (rustup)
- **Go** ≥ 1.23
- **PostgreSQL** 16（或通过 Docker Compose）
- **Tauri CLI** v2（桌面端开发）

### 一键启动基础设施

```bash
docker compose up -d
```

### 安装依赖

```bash
make install
```

## 开发流程

### 1. 创建分支

```bash
git checkout -b feat/your-feature
```

分支命名规范：
- `feat/` — 新功能
- `fix/` — Bug 修复
- `refactor/` — 重构
- `docs/` — 文档

### 2. 开发 & 测试

```bash
# 启动开发服务
make dev

# 运行全部测试
make test

# 快速检查（lint + type check）
make check
```

### 3. 单模块测试

```bash
# Rust
cd rust-workspace && cargo test
cd rust-workspace && cargo clippy -- -D warnings

# Go
cd go-server && go test -race ./...
cd go-server && go vet ./...

# Frontend
cd frontend-workspace && pnpm test
cd frontend-workspace && npx vue-tsc --noEmit
```

### 4. 提交规范

使用 [Conventional Commits](https://www.conventionalcommits.org/)：

```
feat(graph): add spotlight mode N-degree neighbor highlight
fix(ws): resolve reconnection loop on auth failure
refactor(layout): extract potpack box typing
docs(readme): update architecture diagram
```

## 代码规范

### TypeScript / Vue 3
- 使用 `<script setup>` Composition API
- Pinia store 严格类型化
- 消除 `any`，使用 `unknown` 或具体类型
- Tauri `invoke` 调用必须 try-catch

### Rust
- 库 crate 禁止 `.unwrap()`，使用 `Result` + `thiserror`
- 通过 `cargo clippy -- -D warnings`
- 避免 unnecessary `.clone()`

### Go
- 错误包装：`fmt.Errorf("failed to X: %w", err)`
- 结构化日志：`logger.Log.*`
- Handler 函数添加 godoc 注释
- 数据库操作使用事务保护关键 mutations

## PR 检查清单

- [ ] 通过 `make check` 和 `make test`
- [ ] 新功能有对应测试
- [ ] 无 `any` 类型（TypeScript）
- [ ] 无 `.unwrap()`（Rust 库 crate）
- [ ] Conventional Commits 格式
