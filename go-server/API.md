# Memory Stream Go-Server API Reference

**Base URL**: `http://localhost:8080/api/v1`

> One-shot cutover note: legacy admin write routes (`/cards`, `/categories`, `/edges`, `/cards/merge`) are retired and now return `410 Gone`. Use `/sync/*` for machine sync writes and local IPC flow for admin editing.

---

## Cards 卡片

| #   | Method   | Path               | Auth | 说明                                        |
| --- | -------- | ------------------ | ---- | ------------------------------------------- |
| 1   | `GET`    | `/cards`           | No   | 获取最近 20 条卡片（轻量，不含 raw_md）     |
| 2   | `GET`    | `/cards/discover`  | No   | 获取无连线的孤立卡片，支持分页排序          |
| 3   | `GET`    | `/cards/:id`       | No   | 获取单张卡片完整内容（含 raw_md、ast_data） |
| 4   | `GET`    | `/cards/:id/graph` | No   | 获取以某卡片为中心的子图                    |
| 5   | `POST`   | `/cards`           | Yes  | 创建卡片（可选附带 parent_id 自动建边）     |
| 6   | `PUT`    | `/cards/:id`       | Yes  | 更新卡片内容                                |
| 7   | `DELETE` | `/cards/:id`       | Yes  | 硬删除卡片及所有关联数据（事务）            |
| 8   | `POST`   | `/cards/:id/view`  | Yes  | 兼容端点（无持久化副作用）                  |

### `GET /cards`

获取最近 20 条卡片（轻量，不含 raw_md、ast_data）。

**Response** (200):

```json
{
  "cards": [
    {
      "id": "uuid",
      "title": "标题",
      "excerpt": "摘要",
      "category_id": 1,
      "created_at": "2026-03-29T10:00:00Z",
      "updated_at": "2026-03-29T10:00:00Z",
      "category": { "id": 1, "name": "Rust" }
    }
  ]
}
```

### `GET /cards/discover`

获取无连线的孤立卡片（card_edges 中无记录），支持分页和排序。

**Query Parameters**:

| 参数        | 类型   | 默认值     | 说明                                                                    |
| ----------- | ------ | ---------- | ----------------------------------------------------------------------- |
| `sort`      | string | `"latest"` | `"latest"`（按 updated_at DESC）；`"hot"` 目前兼容保留，行为同 `latest` |
| `page`      | int    | `1`        | 页码                                                                    |
| `page_size` | int    | `20`       | 每页条数（最大 100）                                                    |

**Response** (200):

```json
{
  "cards": [
    {
      "id": "uuid",
      "title": "标题",
      "excerpt": "摘要",
      "category_id": 2,
      "created_at": "2026-03-29T10:00:00Z",
      "updated_at": "2026-03-29T10:00:00Z",
      "category": { "id": 2, "name": "Vue" }
    }
  ]
}
```

### `GET /cards/:id`

获取单张卡片完整内容。

**Path Parameters**: `id` — UUID 或 `"root"`（自动解析为根节点）

**Response** (200):

```json
{
  "id": "uuid",
  "title": "标题",
  "raw_md": "# Markdown 内容",
  "excerpt": "摘要",
  "ast_data": { "type": "Root", "children": [] },
  "category_id": null,
  "created_at": "2026-03-29T10:00:00Z",
  "updated_at": "2026-03-29T10:00:00Z",
  "category": null
}
```

**Error** (404): `{ "error": "卡片未找到: record not found" }`

### `GET /cards/:id/graph`

获取以某卡片为中心的子图，深度受限遍历。

**Path Parameters**: `id` — UUID 或 `"root"`

**Query Parameters**:

| 参数    | 类型 | 默认值 | 说明              |
| ------- | ---- | ------ | ----------------- |
| `depth` | int  | `2`    | 图遍历深度（1-5） |

**Response** (200):

```json
{
  "nodes": [
    { "id": "uuid-1", "title": "卡片A" },
    { "id": "uuid-2", "title": "卡片B" }
  ],
  "edges": [{ "source": "uuid-1", "target": "uuid-2", "relation": "sequence" }]
}
```

### `POST /cards`

创建新卡片。可选附带 `parent_id` 自动创建边。

**Request Body**:

| 字段            | 类型   | 必填    | 默认值       | 说明                          |
| --------------- | ------ | ------- | ------------ | ----------------------------- |
| `title`         | string | No      | `""`         | 卡片标题                      |
| `raw_md`        | string | **Yes** | —            | Markdown 原始内容             |
| `excerpt`       | string | No      | `""`         | 纯文本摘要                    |
| `ast_data`      | json   | No      | `{}`         | AST 结构化 JSON               |
| `category_id`   | \*uint | No      | `null`       | 分类 ID，null 表示未分类      |
| `parent_id`     | string | No      | —            | 父卡片 UUID，若提供则自动建边 |
| `relation_type` | string | No      | `"sequence"` | 自动建边的关系类型            |

```json
{
  "title": "New Card",
  "raw_md": "# Hello\n\nSome content",
  "excerpt": "A summary",
  "ast_data": { "type": "Root", "children": [] },
  "category_id": 3,
  "parent_id": "uuid-parent",
  "relation_type": "reference"
}
```

**Response** (200): `{ "message": "卡片已存入记忆流", "card_id": "uuid" }`

**Error** (400): `{ "error": "参数解析失败: ..." }`

### `PUT /cards/:id`

更新卡片内容。

**Request Body**:

| 字段          | 类型   | 必填    | 说明                      |
| ------------- | ------ | ------- | ------------------------- |
| `title`       | string | **Yes** | 新标题                    |
| `raw_md`      | string | **Yes** | 新 Markdown 内容          |
| `excerpt`     | string | No      | 新摘要                    |
| `ast_data`    | string | **Yes** | 新 AST JSON（字符串形式） |
| `category_id` | \*uint | No      | 分类 ID，null 清除分类    |

```json
{
  "title": "Updated Title",
  "raw_md": "# Updated Content",
  "excerpt": "New summary",
  "ast_data": "{\"type\":\"Root\",\"children\":[]}",
  "category_id": 3
}
```

**Response** (200): `{ "message": "卡片已更新", "card_id": "uuid" }`

### `DELETE /cards/:id`

硬删除卡片及所有关联数据（边、布局），单事务。

**Response** (200): `{ "message": "卡片已删除", "card_id": "uuid" }`

### `POST /cards/:id/view`

历史兼容端点：本地优先模式下不再持久化热度指标，调用返回成功但不写入统计。

**Path Parameters**: `id` — UUID 或 `"root"`

**Response** (200): `{ "message": "已记录（兼容模式）", "card_id": "uuid" }`

**实现**: 兼容 No-Op（不更新数据库统计）

---

## Categories 分类

| #   | Method   | Path                       | Auth | 说明                                  |
| --- | -------- | -------------------------- | ---- | ------------------------------------- |
| 9   | `GET`    | `/categories`              | No   | 获取所有分类                          |
| 10  | `GET`    | `/categories/:id/clusters` | No   | 获取某分类下的卡片（按更新时间排序）  |
| 11  | `POST`   | `/categories`              | Yes  | 创建分类                              |
| 12  | `PUT`    | `/categories/:id`          | Yes  | 更新分类                              |
| 13  | `DELETE` | `/categories/:id`          | Yes  | 删除分类（关联卡片 category_id 置空） |

### `GET /categories`

**Response** (200):

```json
{
  "categories": [
    {
      "id": 1,
      "name": "Rust",
      "description": "Rust 语言生态与最佳实践",
      "created_at": "..."
    }
  ]
}
```

### `GET /categories/:id/clusters`

获取某分类下的卡片，按 updated_at DESC 排序，最多 20 条。

**Response** (200):

```json
{
  "clusters": [
    {
      "card_id": "uuid",
      "title": "标题",
      "updated_at": "..."
    }
  ]
}
```

### `POST /categories`

**Request Body**: `{ "name": "DevOps"（必填）, "description": "..." }`

**Response** (200): `{ "category": { "id": 7, "name": "DevOps", "description": "...", "created_at": "..." } }`

### `PUT /categories/:id`

**Request Body**: `{ "name": "新名称"（必填）, "description": "..." }`

**Response** (200): `{ "message": "分类已更新" }`

### `DELETE /categories/:id`

删除分类，将关联卡片的 `category_id` 置为 NULL。

**Response** (200): `{ "message": "分类已删除" }`

---

## Edges 边

| #   | Method   | Path     | Auth | 说明             |
| --- | -------- | -------- | ---- | ---------------- |
| 14  | `POST`   | `/edges` | Yes  | 创建有向边       |
| 15  | `DELETE` | `/edges` | Yes  | 删除边           |
| 16  | `PATCH`  | `/edges` | Yes  | 更新边的关系类型 |

> `relation_type` 仅支持 `"sequence"` 或 `"reference"`

### `POST /edges`

```json
{ "source_id": "uuid", "target_id": "uuid", "relation_type": "reference" }
```

**Response** (200): `{ "message": "连线已创建" }`

### `DELETE /edges`

```json
{ "source_id": "uuid", "target_id": "uuid" }
```

**Response** (200): `{ "message": "连线已删除" }`

### `PATCH /edges`

```json
{ "source_id": "uuid", "target_id": "uuid", "relation_type": "sequence" }
```

**Response** (200): `{ "message": "连线已更新" }`

---

## Graph 图

| #   | Method | Path                | Auth | 说明                                   |
| --- | ------ | ------------------- | ---- | -------------------------------------- |
| 17  | `GET`  | `/graph/outline`    | No   | 获取大纲视图（Topic/Cluster 层级结构） |
| 18  | `GET`  | `/graph/detail/:id` | No   | 获取以某卡片为中心的局部图             |

### `GET /graph/outline`

获取大纲视图。Topic 对应分类，Cluster 为最近 50 条卡片。

**Query Parameters**: `category_id`（可选，筛选分类）

**Response** (200):

```json
{
  "topics": [{ "id": "1", "label": "Rust", "card_count": 1 }],
  "clusters": [
    { "id": "uuid", "title": "卡片标题", "topic_id": "1", "created_at": "..." }
  ]
}
```

### `GET /graph/detail/:id`

获取以某卡片为中心的局部图（逻辑同 `GET /cards/:id/graph`）。

**Path Parameters**: `id` — UUID 或 `"root"`

**Query Parameters**: `depth`（默认 2，范围 1-5）

**Response** (200): 同 `GET /cards/:id/graph`

---

## Sync 与观测

| Method   | Path                 | Auth | 说明                                                      |
| -------- | -------------------- | ---- | --------------------------------------------------------- |
| `PUT`    | `/sync/card/:id`     | Yes  | 机器同步写入（upsert）                                    |
| `DELETE` | `/sync/card/:id`     | Yes  | 机器同步删除                                              |
| `GET`    | `/sync/changes`      | Yes  | 增量同步拉取（支持 `since` + `limit`）                    |
| `GET`    | `/ops/route-metrics` | Yes  | 迁移观测端点：返回 `legacy_write/sync_write` 路由命中计数 |

### `GET /ops/route-metrics`

**Response** (200)：

```json
{
  "code": 0,
  "message": "success",
  "data": {
    "metrics": {
      "class.legacy_write": 12,
      "class.sync_write": 340,
      "route.cards.update": 2,
      "route.sync.card.upsert": 330
    }
  }
}
```

---

## WebSocket

| #   | Path  | 说明               |
| --- | ----- | ------------------ |
| 19  | `/ws` | WebSocket 实时协作 |

### 客户端 → 服务端

**创建边**:

```json
{
  "action": "CREATE_EDGE",
  "payload": {
    "source_id": "uuid",
    "target_id": "uuid",
    "relation_type": "reference"
  }
}
```

**删除边**:

```json
{
  "action": "DELETE_EDGE",
  "payload": { "source_id": "uuid", "target_id": "uuid" }
}
```

### 服务端 → 客户端

**边已创建**:

```json
{
  "event": "EDGE_CREATED",
  "payload": {
    "source_id": "uuid",
    "target_id": "uuid",
    "relation_type": "..."
  }
}
```

**边已删除**:

```json
{
  "event": "EDGE_DELETED",
  "payload": { "source_id": "uuid", "target_id": "uuid" }
}
```

**布局更新**:

```json
{
  "event": "LAYOUT_UPDATED",
  "payload": [{ "id": "uuid", "x": 100.0, "y": 200.0 }]
}
```

**错误**:

```json
{ "event": "ERROR", "payload": { "message": "错误信息" } }
```

---

## 通用说明

- **写接口** Auth 中间件为 passthrough，暂无鉴权
- **错误响应统一格式**: `{ "error": "中文描述" }`
- **HTTP 状态码**: 200 成功 / 204 OPTIONS 预检 / 400 参数错误 / 404 未找到 / 500 服务器错误
- **环境变量**: `DATABASE_URL`（必填）、`PORT`（默认 8080）
- **CORS 允许来源**: `localhost:5173`、`localhost:1420`、`localhost:4173`、`tauri.localhost`
