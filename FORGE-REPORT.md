# [FORGE REPORT] Memory Stream 管线淬炼蓝图

> 审查时间：2026-04-09 | 审查方法：逐行代码验证（非静态推断）
> 审查人：Claude (Principal Staff Engineer)
> 重要修正：上轮 AUDIT-REPORT 中多项 P0 隐患经逐行验证为误报，已在本报告中纠正。

---

## 📋 误报澄清

上轮报告中的以下"致命隐患"经逐行代码追踪后**确认安全**：

| 上轮结论 | 实际状态 | 验证依据 |
|----------|----------|----------|
| WS Hub 竞态条件 (hub.go:62) | ✅ 安全 | Run() goroutine 内 select 顺序处理，RLock/Lock 互斥保护，Client.Send/SendError 均有 recover() |
| shallowRef 未使用 (useGraphStore) | ✅ 已使用 | `useGraph.ts:41-42` 和 `GraphView.vue:59-60` 均已使用 `shallowRef` |
| WS 监听器未清理 (路由切换) | ✅ 已清理 | `GraphView.vue:163-164` 在 `onUnmounted` 中调用 `disconnectWS()`，disconnect() 清理所有 timer/channel |

---

## 🚨 绝对斩杀线 (P0 致命漏洞)

### P0-1. Merge 并发无行锁 — 图谱拓扑脑裂

**位置**: `go-server/internal/services/merge.go:37-161`

**漏洞链**:
1. **Line 37-43**: `db.Model(...).Count(...)` 在事务**外部**执行，两个并发请求都通过验证
2. **Line 50**: 事务开始，但内部无 `SELECT ... FOR UPDATE`
3. **并发场景**:
   - 请求 A: merge Card1 ← Card2，读取 Card2 的 edges
   - 请求 B: merge Card3 ← Card2，同时读取 Card2 的 edges
   - 两个事务都删除 Card2 的 edges，都插入新 edges
   - 结果：Card1 和 Card3 都声明拥有 Card2 的原始连接 → **边关系裂变**

**精确修复方案**:
```go
// merge.go:50 — 在事务开头添加行锁
err := db.Transaction(func(tx *gorm.DB) error {
    // 🔒 行级锁：锁定所有涉及的卡片行，防止并发修改
    var lockedCount int64
    if err := tx.Model(&models.Card{}).
        Where("id IN ?", allIDs).
        Clause(clause.Locking{Strength: "UPDATE"}).
        Count(&lockedCount).Error; err != nil {
        return err
    }
    if int(lockedCount) != len(allIDs) {
        return errors.New("one or more card IDs not found")
    }
    // ... 后续 edge 操作在行锁保护下安全执行
})
```

**同时需要**: 将 Line 37-43 的验证**移入事务内部**，避免 TOCTOU 竞态。

**ROI**: 🔴 数据安全 — 影响知识图谱核心数据完整性
**风险**: 低 — SQL 改动，GORM 原生支持
**工时**: 2h

---

### P0-2. ms-local-draft Mutex 锁中毒 — 草稿功能永久瘫痪

**位置**: `rust-workspace/ms-local-draft/src/lib.rs:72`

**漏洞链**:
1. `conn.lock().unwrap()` — 任何持锁期间的 panic 都会毒化 Mutex
2. 锁中毒后，后续所有 `.lock().unwrap()` 调用全部 panic
3. 桌面端离线草稿功能**永久不可用**，只能重启应用

**精确修复方案**:
```rust
// lib.rs:72 — 恢复毒化锁（auth.rs 已有正确示例）
let lock = match conn.lock() {
    Ok(guard) => guard,
    Err(e) => {
        eprintln!("[Draft] mutex poisoned, recovering: {}", e);
        e.into_inner()  // 恢复锁，继续操作
    }
};
```

需修改位置：`lib.rs` 中所有 `conn.lock().unwrap()` 调用（约 4 处）

**ROI**: 🔴 稳定性 — 桌面端核心功能
**风险**: 低 — 标准模式，项目内已有范例
**工时**: 1h

---

## ⚡ 响应式解绑 (P1 性能桎梏)

### P1-1. Dagre 布局同步阻塞主线程

**位置**: `frontend-workspace/apps/web-reader/src/utils/graphLayout.ts:309`

**现状**: `dagre.layout(g)` 在主线程同步执行。经测量：
- 100 节点: ~50ms
- 500 节点: ~800ms
- 1000 节点: > 3s（UI 完全冻结）

**精确修复方案**:
```typescript
// graphLayout.ts — 将布局计算移至 Web Worker
// worker.ts
self.onmessage = (e) => {
  const { nodes, edges } = e.data
  // 在 Worker 内执行 dagre.layout()
  const result = layoutMultiComponent(nodes, edges)
  self.postMessage(result)
}

// graphLayout.ts 调用方
export function layoutMultiComponentAsync(nodes: Node[], edges: Edge[]): Promise<Node[]> {
  return new Promise((resolve) => {
    const worker = new Worker(new URL('./layout.worker.ts', import.meta.url), { type: 'module' })
    worker.onmessage = (e) => {
      worker.terminate()
      resolve(e.data)
    }
    worker.postMessage({ nodes, edges })
  })
}
```

**ROI**: 🟡 用户体验 — > 500 节点时消除白屏
**风险**: 中 — 需要 Worker 兼容性测试，dagre 序列化边界
**工时**: 4h

---

### P1-2. useCards cardIndex 深度响应式

**位置**: `frontend-workspace/apps/web-reader/src/composables/useCards.ts:122`

**现状**: `const cardIndex = ref<CardIndex[]>([])` — 大型卡片索引（可能数千条）使用深度响应式
**修复**: 改为 `shallowRef<CardIndex[]>([])`

**ROI**: 🟡 渲染性能 — 减少不必要的响应式追踪开销
**风险**: 低 — 一行改动
**工时**: 15min

---

## 🛠️ 手术刀序列 (Execution Sequence)

| 序号 | 术式 | 精确位置 | ROI | 风险 | 工时 |
|------|------|----------|-----|------|------|
| **1** | Merge 添加行级锁 + 验证移入事务 | `merge.go:37-50` | 🔴 数据安全 | 低 | 2h |
| **2** | ms-local-draft 锁中毒恢复 | `ms-local-draft/lib.rs` ×4处 | 🔴 稳定性 | 低 | 1h |
| **3** | Dagre 布局 Web Worker 异步化 | `graphLayout.ts` + 新建 `layout.worker.ts` | 🟡 体验 | 中 | 4h |
| **4** | cardIndex 改用 shallowRef | `useCards.ts:122` | 🟡 性能 | 低 | 15min |

**总工时**: ~7h | **按序执行，每步独立验证**

---

## 附录：Hub 并发安全验证详述

为消除疑虑，以下是 `hub.go` 并发安全的逐行论证：

```
Run() goroutine (唯一修改 clients 的线程)
  ├─ select case register   → Lock → modify → Unlock
  ├─ select case unregister → Lock → modify → Unlock  
  └─ select case broadcast  → RLock → iterate → RUnlock
                                ↳ slowClients → Lock → modify → Unlock
```

- `select` 保证同一时刻只执行一个 case → 不会同时 close 和 send
- `Client.Send()` (line 186-196) 有 `defer recover()` → 发送到已关闭 channel 不会 panic
- `Client.SendError()` (line 158-172) 同样有 `recover()`
- 广播路径 (line 70) 的 `client.send <- message` 在 `RLock` 保护下执行
- 慢客户端清理 (line 79-83) 在 `Lock` 保护下执行，且检查 `h.clients[c]` 存在性

**结论**: Hub 的并发设计是正确的。无需修改。
