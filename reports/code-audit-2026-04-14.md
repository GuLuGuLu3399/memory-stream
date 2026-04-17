# Memory Stream 全量代码审计报告

审计日期：2026-04-14

审计范围：Go 后端、Tauri 桌面端、Vue 前端、Rust 核心、脚本与配置。

## 结论

项目整体已经能通过现有测试和类型检查，但仍有 3 个需要优先处理的问题，其中 1 个会直接污染导入数据结构，1 个会让“停止监听”按钮失效，1 个会扩大 WebSocket 的攻击面。

## 发现

### 1. 导入时写入了错误的 AST JSON 形态

位置：[frontend-workspace/apps/admin-tauri/src-tauri/src/vault_sync.rs](frontend-workspace/apps/admin-tauri/src-tauri/src/vault_sync.rs#L829)

`ast_data` 当前被写成了字符串 "{}"，而不是 JSON 对象 {}。这会让新导入的卡片在后端存成 JSON 字符串值，后续依赖 AST 结构的流程会拿到错误类型，属于数据层污染。

影响：导入卡片的 AST 结构不正确，后续目录提取、渲染、检索或再导出都可能出现兼容性问题。

建议：把 `ast_data` 改为真正的 JSON 对象，或者显式复用前端解析后的 AST 结果再上传。

### 2. “停止监听”只停了前端轮询，没有停 Rust 侧 watcher

位置：[frontend-workspace/apps/admin-tauri/src/composables/useVaultSync.ts](frontend-workspace/apps/admin-tauri/src/composables/useVaultSync.ts#L10)

位置：[frontend-workspace/apps/admin-tauri/src/composables/useVaultSync.ts](frontend-workspace/apps/admin-tauri/src/composables/useVaultSync.ts#L30)

`stopWatcher()` 只清掉了 `setInterval` 和前端状态，但没有调用 Rust 端的停止命令。后端的文件监听器仍然保持在 `AppState` 中运行，因此 UI 上看起来已经停了，实际 watcher 还在收事件。

影响：停止按钮语义不成立，后台 watcher 会继续占用资源，并可能在重新启用时一次性积累大量未处理事件。

建议：补一个 Rust 停止命令，把 `AppState.watcher` 显式 `unwatch` 并清空，再让前端 stop 操作同步调用它。

### 3. WebSocket 原始来源检查被完全关闭

位置：[go-server/internal/handlers/ws.go](go-server/internal/handlers/ws.go#L19)

`CheckOrigin` 现在无条件返回 `true`，等于放弃了浏览器侧的来源防护。虽然消息层仍有认证，但任何站点都可以尝试建立 WebSocket 连接，扩大了跨站 WebSocket 风险面。

影响：攻击面扩大，尤其是在浏览器端持有令牌或存在自动认证场景时，风险会更高。

建议：恢复基于配置的允许来源名单，或至少只对明确的桌面/本地域名单放行。

## 验证情况

- 已确认当前工作区里的测试与类型检查能通过。
- `make test` 通过。
- 前端 `vue-tsc --noEmit` 通过。

## 建议优先级

1. 先修复导入 AST 结构错误，避免继续写入坏数据。
2. 再修复 watcher 停止语义，确保 UI 和后端一致。
3. 评估 WebSocket 来源策略，恢复最小可用的来源限制。
