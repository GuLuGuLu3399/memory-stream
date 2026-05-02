# 🐹 Go Server — Memory Stream API 后端

> 提供 RESTful API + WebSocket 实时通信，管理知识卡片的 CRUD、图谱边关系、分类等核心业务。
>
> **路线图**：[CHECKLIST.md](../CHECKLIST.md) · **当前版本**：V3.4

## 架构总览

```
go-server/
├── cmd/api/main.go        # 入口：路由注册 + 中间件 + 服务启动
├── internal/
│   ├── handlers/           # HTTP 处理器（Cards, Edges, Categories, Graph, Auth）
│   ├── services/           # 业务逻辑层（事务管理、图遍历、热度计算）
│   ├── middleware/         # CORS、Auth、日志中间件
│   └── models/             # GORM 模型（Card, Edge, Category 等）
├── migration/              # 数据库迁移脚本
├── internalerrors/         # 统一错误定义
└── internalpkg/logger/     # 结构化日志
```

## 快速开始

```bash
# 1. 配置环境变量
cp .env.example .env
# 编辑 .env 设置 DATABASE_URL、JWT_SECRET 等

# 2. 启动服务
go run cmd/api/main.go

# 服务默认监听 :8080
```

## 环境变量

| 变量           | 必填 | 默认值    | 说明                                                     |
| -------------- | ---- | --------- | -------------------------------------------------------- |
| `DATABASE_URL` | ✅   | —         | PostgreSQL 连接串                                        |
| `REDIS_ADDR`   | ✅   | —         | Redis 连接地址 (host:port)                               |
| `JWT_SECRET`   | ✅   | —         | JWT 签名密钥（≥32 字节，生产环境必填）                   |
| `PORT`         | ❌   | `8080`    | HTTP 监听端口                                            |
| `GO_ENV`       | ❌   | —         | 环境标识：`production` \| `development`                  |
| `GIN_MODE`     | ❌   | `release` | Gin 框架模式：`debug`（详细日志）\| `release` 精简日志） |
| `DEBUG`        | ❌   | —         | 快速启用调试模式：`true` 或 `1`                          |

### 环境变量优先级

**Gin 模式设置优先级**（从高到低）：

1. `GIN_MODE` 环境变量 - 最优先
2. `DEBUG` 环境变量 - 如果为 `true` 则启用 debug 模式
3. `GO_ENV=production` - 启用 release 模式
4. 默认 `release` 模式

**配置文件加载优先级**（按顺序尝试）：

1. `.env.production` - 生产环境推荐
2. `.env.local` - 本地开发（.gitignore 中）
3. `.env` - 开发模板
4. 系统环境变量 - Docker/K8s 环境

> **注意**：S3/MinIO 对象存储由 Tauri 桌面端直接调用，Go 后端不涉及图片存储。

## API 文档

详见 [API.md](./API.md) — 包含所有 19 个端点的完整文档。

### 核心端点速览

| 模块       | 端点数 | 说明                          |
| ---------- | ------ | ----------------------------- |
| Cards      | 8      | 卡片 CRUD + 图谱子图 + 浏览量 |
| Categories | 5      | 分类管理                      |
| Edges      | 3      | 有向边创建/删除/更新          |
| Graph      | 2      | 大纲视图 + 局部图             |
| WebSocket  | 1      | 实时边操作                    |

### 认证机制

- JWT Bearer Token（`Authorization: Bearer <token>`）
- 写操作需要认证，读操作公开
- Genesis Admin：首次启动自动创建管理员账户
- 登录端点：`POST /api/v1/auth/login`

## 数据库

使用 GORM + PostgreSQL，自动迁移。

### 核心表

- **cards** — 知识卡片（title, raw_md, excerpt, ast_data）
- **card_edges** — 有向边（source_id → target_id, relation_type）
- **categories** — 分类（name, description）
- **card_layouts** — 图谱布局坐标（x, y）

## WebSocket 协议

`/api/v1/ws` — 实时边操作 + 心跳检测。

### 认证流程（Auth-on-Connect）

连接时不带 Token，建立后 3 秒内必须发送 AUTH 消息：

```
客户端 → 服务端: { "action": "AUTH", "payload": { "token": "jwt..." } }
服务端 → 客户端: { "event": "AUTH_OK", "payload": { "user_id": "...", "role": "..." } }
```

超时未认证 → 服务端主动断开。

### 心跳检测（Ping/Pong）

认证成功后，客户端每 15s 发送 PING，服务端回送 PONG（客户端计算 RTT）：

```
客户端 → 服务端: { "action": "PING", "payload": {} }
服务端 → 客户端: { "event": "PONG" }
```

### 实时操作

```
客户端 → 服务端: { "action": "CREATE_EDGE", "payload": { "source_id": "...", "target_id": "...", "relation_type": "reference" } }
服务端 → 所有客户端: { "event": "EDGE_CREATED", "payload": {...} }

客户端 → 服务端: { "action": "DELETE_EDGE", "payload": { "source_id": "...", "target_id": "..." } }
服务端 → 所有客户端: { "event": "EDGE_DELETED", "payload": {...} }
```

## CORS 配置

允许来源：

- `http://localhost:5173`（Web Reader 开发服务器）
- `http://localhost:1420`（Tauri 开发服务器）
- `http://localhost:4173`（Vite 预览）
- `https://tauri.localhost`（Tauri 生产环境）

## 技术栈

- **Gin** — HTTP 框架
- **GORM** — ORM
- **PostgreSQL** — 主数据库
- **golang-jwt** — JWT 认证
- **gorilla/websocket** — WebSocket
