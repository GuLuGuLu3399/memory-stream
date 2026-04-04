# JWT Proactive Refresh — Rust 层定时刷新

## 目标

在 Rust 层实现 JWT proactive refresh：access_token 过期前 5 分钟自动刷新，前端完全无感。

## 现状

- `AuthState = Arc<Mutex<Option<String>>>`，纯同步，无时间感知
- 只有 401 被动刷新（`api.rs` + `ws_client.rs` 各自调用 `try_refresh_token`）
- 无 token 持久化 — 每次启动都要重新登录
- Go 后端：access_token 2h，refresh_token 7d

## 设计

### 1. AuthState 改造（auth.rs）

**保持 `Mutex` 不变**（不用 RwLock），内部扩展字段：

```rust
pub struct AuthState {
    access_token: Mutex<Option<String>>,
    refresh_token: Mutex<Option<String>>,
    created_at: Mutex<Option<Instant>>,   // token 获取时间
    expires_in: Duration,                  // access_token 有效期（2h）
    persist_path: PathBuf,                 // 磁盘持久化路径
}
```

- **Mutex 不变**：避免 Tauri State 类型不兼容问题。`Mutex<Option<Instant>>` 同步锁足够。
- **created_at + expires_in**：计算 token 剩余寿命，驱动 proactive timer。
- **persist_path**：token 写入 `$APP_DATA/tokens.json`，重启自动恢复。

### 2. Token 持久化（auth.rs）

- `save_to_disk()`: 序列化 `{ access_token, refresh_token }` 到 JSON 文件
- `load_from_disk()`: 启动时读取，`created_at = Instant::now()`（保守策略）
- 登录/刷新/清除时自动 save

### 3. Proactive Refresh Timer（auth.rs）

```rust
pub fn spawn_proactive_refresh(auth: Arc<AuthState>, client: reqwest::Client) {
    tauri::async_runtime::spawn(async move {
        loop {
            let sleep_duration = {
                let state = auth.access_token.lock().ok();
                let created = auth.created_at.lock().ok();
                // 无 token → sleep 30s 重检
                // 有 token，剩余 > 5min → sleep (剩余 - 5min)
                // 有 token，剩余 <= 5min → 立即刷新
            };

            tokio::time::sleep(sleep_duration).await;

            // 执行刷新
            let refreshed = do_refresh(&client, &auth).await;
            if !refreshed {
                // 刷新失败 → sleep 30s 后重试（最多 3 次）
            }
        }
    });
}
```

- 使用 `tauri::async_runtime::spawn`（不是 `tokio::spawn`），因为从 sync `setup()` 调用
- 刷新失败指数退避：30s → 60s → 120s
- 无 token 时 sleep 30s 重检（等待用户登录）

### 4. 401 Interceptor 优化（api.rs）

保持现有逻辑不变：401 → `try_refresh_token` → 重试。

改进：加 `refreshing` Mutex flag 防并发刷新（thundering herd）：

```rust
auth.refreshing.lock().unwrap() = true;
// ... do refresh ...
auth.refreshing.lock().unwrap() = false;
```

其他并发 401 请求发现 `refreshing == true`，等 5s 后直接用新 token 重试。

### 5. WS Auth 失败处理（ws_client.rs）

现有逻辑基本正确，改进：
- WS auth rejected → `try_refresh_token` → 重连时用新 token
- 刷新失败 → emit `auth_expired` 给前端

### 6. lib.rs 注册

```rust
// setup() 中
let auth_arc = Arc::new(AuthState::new(data_dir.clone()));
let http_client = reqwest::Client::new(); // 给 proactive timer 用
auth::spawn_proactive_refresh(auth_arc.clone(), http_client);
app.manage(auth_arc);
```

**不改变 `.manage()` 类型** — 仍然是 `Arc<AuthState>`，所有 command 签名不变。

### 7. 前端改动

无。前端仍然 `invoke("login", ...)` 登录，完全透明。

## 不做的事

- **不引入 RwLock** — Mutex 够用，避免 Tauri State 类型不兼容
- **不改变 command 签名** — `State<'_, Arc<AuthState>>` 保持不变
- **不改 useAuth.ts** — 前端逻辑不变
- **不传 AppHandle 给 timer** — 刷新失败不需要 emit event，静默重试即可

## 文件改动清单

| 文件 | 改动 |
|------|------|
| `auth.rs` | 扩展 AuthState 字段 + 持久化 + proactive timer + thundering herd flag |
| `api.rs` | 401 interceptor 加并发刷新保护 |
| `ws_client.rs` | 微调：用共享 Client 替代 inline `Client::new()` |
| `lib.rs` | `AuthState::new(data_dir)` + spawn proactive timer |

## 验证标准

1. `cargo check` 0 errors
2. 启动应用 → 登录 → 关闭 → 重启 → token 自动恢复，无需重新登录
3. 保持应用打开 2h+ → 观察日志 proactive refresh 成功
4. 模拟 token 过期（Go 重启）→ 下次 401 或 WS 重连时 refresh 失败 → 前端收到错误提示
