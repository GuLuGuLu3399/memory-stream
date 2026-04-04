# Memory Stream V3.4 — 全栈深度代码审查综合报告

**审查日期**: 2026-04-01  
**审查范围**: Go Server / Vue Web Reader / Vue Admin Tauri / Rust Workspace (10 crates)  
**审查维度**: 架构模式 / 性能优化 / 并发安全 / 错误处理 / 安全 / 类型安全 / 可扩展性

---

## 一、总体评价

Memory Stream V3.4 展现了一个**架构成熟度较高**的 polyglot 系统。四端职责划分清晰（Go 纯数据、Tauri 本地能力、Rust 计算引擎、Vue 展示层），handler→service→storage 三层架构严格执行，Rust workspace 10 crate DAG 依赖无循环，前端 composable 职责边界清晰。

**主要优势**:
- ✅ Go Server handler→service→storage 零越级调用，中间件链顺序正确
- ✅ WebSocket Hub 单线程事件循环设计，无 data race
- ✅ Rust 零 unsafe，thiserror 统一错误处理，Cow<'a, str> 零拷贝
- ✅ Vue 10 composable 职责分明，Zod runtime validation，WASM 懒加载 + LRU + IndexedDB 三级缓存
- ✅ 所有 raw SQL 参数化，无 SQL 注入风险

**需关注领域**:
- 🔴 4 个 P0 问题（安全漏洞 + 数据丢失）
- 🟠 15 个 P1 问题（性能瓶颈 + 安全风险 + 可靠性）
- 🟡 20+ 个 P2 问题（改进建议）

---

## 二、P0 关键发现（必须立即修复）

### P0-1: 🔴 Genesis 端点 Login 错误被忽略 — 安全漏洞
**模块**: Go Server | **文件**: `internal/handlers/auth.go:123`

```go
// 当前代码 — 第4个返回值 error 被完全忽略
accessToken, refreshToken, _, _ := h.authSvc.Login(req.Username, req.Password)
```

**风险**: Login 失败时（密码错误、DB 错误、JWT 签名失败），响应仍返回空 token + 201 状态码。用户以为创世成功但实际无法使用。

**修复**:
```go
accessToken, refreshToken, _, loginErr := h.authSvc.Login(req.Username, req.Password)
if loginErr != nil {
    appErr.Respond(c, appErr.Wrap(loginErr, http.StatusInternalServerError, 50001,
        "创世成功但自动登录失败，请手动登录"))
    return
}
```

---

### P0-2: 🔴 Rust AST 管线 — Table/Strikethrough/TaskList 静默丢弃
**模块**: Rust Core | **文件**: `md-parser/src/lib.rs:86`, `ast-core/src/lib.rs`

**问题**: `pulldown-cmark` 启用了 `ENABLE_TABLES`, `ENABLE_STRIKETHROUGH`, `ENABLE_TASKLISTS` 选项，但 `AstNode` enum 没有对应的 variant。所有 Table/Strikethrough/TaskList 事件被 `_ => continue` 静默丢弃。

**影响**: 含表格、删除线、任务列表的 Markdown 内容**数据丢失**，渲染时完全缺失。

**修复范围**: 3 个文件（ast-core + md-parser + ast-renderer），约 150 行代码。

---

### P0-3: 🔴 Tauri Token 不持久化 — 每次启动需重新登录
**模块**: Admin Tauri | **文件**: `src-tauri/src/auth.rs`

**问题**: JWT token 仅存储在 `Mutex<Option<String>>` 内存中，应用重启即丢失。

**修复**: 使用 `tauri-plugin-store` 或 OS keychain 持久化。

---

### P0-4: 🔴 Tauri start_watcher 路径遍历漏洞
**模块**: Admin Tauri | **文件**: `src-tauri/src/lib.rs:408-417`

**问题**: `start_watcher` 命令未对路径做规范化/验证，允许遍历任意目录。

**修复**: 使用 `std::fs::canonicalize()` 并验证路径在允许范围内。

---

## 三、P1 高优先级发现（本迭代修复）

### Go Server (6 个 P1)

| # | 问题 | 文件 | 影响 |
|---|------|------|------|
| 1 | **Edge 操作未刷新 graph 缓存** | `edge.go:32-82` | 边增删后图谱显示陈旧 |
| 2 | **FindRoot N+1 顺序查询** | `edge.go:84-96` | 100 卡片链 = 100 次 DB 查询 |
| 3 | **Redis SCAN 野生键失效效率低** | `card.go:320-340` | 10万+ key 时 SCAN 阻塞秒级 |
| 4 | **Hub.Run 无 panic recovery** | `ws/hub.go:42` | 单次 panic = 全部 WebSocket 宕机 |
| 5 | **JWT Secret 无最小长度校验** | `services/auth.go:21` | 短 secret 可暴力破解 |
| 6 | **Genesis Admin TOCTOU 竞态** | `services/auth.go:76-81` | 多实例可创建多个 admin |

### Web Reader (4 个 P1)

| # | 问题 | 文件 | 影响 |
|---|------|------|------|
| 7 | **AUTH 超时前后端不一致** (前端5s vs 后端3s) | `useGraphSync.ts:98` | 慢网络重连循环 |
| 8 | **重连后无事件补偿** | `useGraphSync.ts` | 断连期间事件永久丢失 |
| 9 | **BFS 聚光灯无提前终止** | `graphLayout.ts:149-186` | 1000+ 节点不必要的全图遍历 |
| 10 | **脉冲环动画 width/height 触发 reflow** | `EntranceAnimation.vue:182` | 动画卡顿 |

### Admin Tauri (5 个 P1)

| # | 问题 | 文件 | 影响 |
|---|------|------|------|
| 11 | **api_request 无端点白名单** | `api.rs:80` | 可访问任意 URL |
| 12 | **delete_card 无所有权验证** | `lib.rs:272-291` | 数据完整性风险 |
| 13 | **S3 上传无超时** | `image.rs:169-172` | 可无限挂起 |
| 14 | **图片无大小限制** | `image.rs:146` | OOM 风险 |
| 15 | **导出全量内存加载** | `export.rs:58-77` | 大知识库 OOM |

### Rust (1 个 P1)

| # | 问题 | 文件 | 影响 |
|---|------|------|------|
| 16 | **ms-kb-exporter 全量内存** | `ms-kb-exporter/src/lib.rs` | 大知识库内存溢出 |

---

## 四、P2 改进建议（下一迭代）

### Go Server
- DI 使用具体类型而非接口（可测试性 P2）
- Slow query 插件仅覆盖 SELECT，未覆盖 Raw/Exec
- Client.Send catch-all recover 掩盖真实 bug
- RateLimiter cleanup 持锁遍历高负载延迟
- Redis 客户端缺 pool/timeout 生产配置
- 错误处理 `Wrap()` 函数 nil panic 风险
- BroadcastEvent marshal 失败事件丢失

### Web Reader
- Spotlight mode 500+ 节点 blur() 性能差
- Store 中 list 专属状态（sortBy/density/categoryFilter）应局部化
- 401 handler retry 丢失 abort signal
- pendingQueue 无最大重试上限
- IndexedDB trim fire-and-forget 无错误处理
- WASM init 竞态（并发 loadDetail 触发多次 init）
- Zod 缺失 backlinks + discover endpoint schema

### Admin Tauri
- knowledge.ts 789 行/8+ 职责，应拆分 store
- sync_from_server 阻塞命令
- SQLite 缓存无失效策略
- WS 客户端仅处理 LAYOUT_UPDATED
- 文件监控无事件去重

### Rust
- WASM release profile 缺 `panic = "abort"`（5-10% 体积浪费）
- getter_with_clone 大文档开销
- URL sanitize 未处理控制字符前缀
- ms-toc-extractor 使用 ast-core 错误类型，命名不一致

### 类型安全
- 4 处 `catch (err: any)` 需改为 `unknown` + narrowing
- TocNode.vue inline type 应使用 `TocItem` 导入
- GraphEdge 字段名与 packages/types 不一致
- Tailwind TocNode padding 可用 JIT 替代 inline style

---

## 五、跨端一致性问题

### 1. WebSocket AUTH 超时不一致
- **后端**: `3 * time.Second` (ws.go:33)
- **前端 Web Reader**: `AUTH_TIMEOUT = 5000` (useGraphSync.ts:98)
- **前端 Admin Tauri**: `5 seconds` (ws_client.rs:11)
- **建议**: 统一为 3s（后端决定断开时间，前端应 ≤ 后端）

### 2. API 响应类型与共享包不一致
- `packages/types` 中 `GraphEdge` 使用 `source_id/target_id/relation_type`
- `web-reader/api/index.ts` 中 `GraphEdge` 使用 `source/target/relation`
- 需要对齐或在 API 层做转换

### 3. 跨端 WS 事件处理不对称
- Go Server 广播: CARD_CREATED, CARD_UPDATED, CARD_DELETED, EDGE_CREATED, EDGE_DELETED, EDGE_UPDATED, LAYOUT_UPDATED
- Web Reader 监听: 全部 7 种事件 ✅
- Admin Tauri WS: **仅处理 LAYOUT_UPDATED** ❌ — 其他 6 种事件被忽略

---

## 六、P0/P1 修复优先级排序

```
修复顺序（按影响×紧急度排序）:

第一批（安全漏洞 — 立即修复）:
  1. P0-1: auth.go Login 错误忽略          [30分钟]
  2. P0-4: Tauri 路径遍历                   [1小时]
  3. P1-5: JWT Secret 长度校验              [15分钟]
  4. P1-6: Genesis 竞态条件                 [1小时]
  5. P1-11: api_request 端点白名单           [2小时]

第二批（数据完整性 — 本周修复）:
  6. P0-2: Rust AST Table/Strikethrough     [4小时]
  7. P0-3: Tauri Token 持久化                [2小时]
  8. P1-1: Edge 缓存失效                    [2小时]
  9. P1-7: AUTH 超时对齐                    [30分钟]
  10. P1-12: delete_card 所有权验证          [1小时]

第三批（可靠性 — 下周修复）:
  11. P1-2: FindRoot CTE 替代               [1小时]
  12. P1-4: Hub.Run panic recovery           [1小时]
  13. P1-8: WS 重连事件补偿                  [3小时]
  14. P1-13: S3 上传超时                     [1小时]
  15. P1-14: 图片大小限制                    [30分钟]
  16. P1-15+16: 导出流式写入                 [4小时]

总预估工时: ~25 小时
```

---

## 七、模块评分

| 模块 | 架构 | 性能 | 并发安全 | 错误处理 | 安全 | 类型安全 | 综合 |
|------|------|------|----------|----------|------|----------|------|
| **Go Server** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | **B+** |
| **Web Reader** | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | **B+** |
| **Admin Tauri** | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ | **B** |
| **Rust Core** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | **A-** |
| **Rust Utils** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | **A-** |

**系统整体评分: B+** — 架构扎实，P0/P1 问题集中在安全加固和边缘情况处理，无根本性设计缺陷。
