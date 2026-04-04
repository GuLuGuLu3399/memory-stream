# SysConfig 锻造系统配置舱 + Singularity Merger 概念坍缩引擎

## TL;DR

> **Quick Summary**: 双功能迭代 — (A) 消除硬编码配置，凭证加密持久化，支持热重载；(B) 一键合并节点，原子边迁移，markdown wikilink 批量替换
> 
> **Deliverables**:
> - Rust config 模块 + keyring 集成 + 热重载命令
> - Go merge endpoint + 原子边事务
> - Vue Settings 面板 + Merge UI
> - 连通性测试 + 文件写入降级处理
>
> **Estimated Effort**: Large (双功能，跨 3 层)
> **Parallel Execution**: YES - 4 waves
> **Critical Path**: T1 → T4 → T7 → T10 → T13 → T16 → F1-F4

---

## Context

### Original Request
用户要求实现两个功能：
1. **SysConfig 锻造系统配置舱** — 消除硬编码，S3 密钥加密存储，运行时热重载
2. **Singularity Merger 概念坍缩引擎** — 一键合并重复节点，原子边迁移，markdown wikilink 替换

### Interview Summary
**Key Discussions**:
- 存储策略: keyring (OS native) 敏感凭证 + store (JSON) 非敏感配置
- 热重载: `Arc<Mutex<Option<Client>>>` 模式，`invoke("reload_sys_config")` 命令
- 合并执行顺序: Rust 预检(内存) → Go 事务(原子) → Rust 落盘(文件)
- 降级策略: 文件写入失败时 emit 事件，前端警告 + 重试按钮

**Research Findings**:
- 硬编码 WS URL: `ws_client.rs:10`
- S3 配置: `image.rs:39-61` via dotenv
- `tauri-plugin-store` 未安装
- 现有边基础设施: `SyncReferenceEdges()`, `FindOrCreateByTitle()`

### Metis Review
**Identified Gaps** (addressed):
- 首次运行 UX → 自动打开设置面板
- Keyring 不可用 → 降级到 store 加密模式
- WS URL 引导 → 保持硬编码 + env 覆盖（选项 A）
- 合并自引用 → 更新为 `[[Survivor]]`（选项 A）
- 边去重 → 保留所有边（不同类型）
- 代码块 wikilink → 使用 AST 跳过

---

## Work Objectives

### Core Objective
1. **SysConfig**: 安全存储 S3/API/WS 配置，支持运行时热重载，消除硬编码
2. **Singularity Merger**: 原子级节点合并，边迁移 + markdown wikilink 替换

### Concrete Deliverables
- `rust-workspace/admin-tauri/src-tauri/src/config/` — config 模块
- `go-server/internal/handlers/merge.go` — merge endpoint
- `frontend-workspace/apps/admin-tauri/src/views/Settings.vue` — 设置面板
- `frontend-workspace/apps/admin-tauri/src/components/MergePanel.vue` — 合并面板

### Definition of Done
- [ ] S3 密钥存储在 OS keyring，不再明文
- [ ] 修改 API/WS URL 后无需重启即可生效
- [ ] 合并两节点后所有边迁移到 Survivor
- [ ] 合并后 markdown 中 `[[Victim]]` 替换为 `[[Survivor]]`
- [ ] 所有 QA 场景通过

### Must Have
- keyring 集成（OS native）
- store 集成（非敏感配置）
- 热重载命令
- 原子边事务
- 连通性测试（ping + S3）
- 文件写入降级

### Must NOT Have (Guardrails)
- ❌ 不改动 Go server JWT_SECRET（保持 .env）
- ❌ 不改动 Go server CORS 配置
- ❌ 不重构 auth.rs token 持久化
- ❌ 不扩展 vault scanner 超出配置范围
- ❌ 不实现 markdown 流式重写（用内存缓冲）

---

## Verification Strategy

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed.

### Test Decision
- **Infrastructure exists**: YES (Rust cargo test, Go test, Vue vitest)
- **Automated tests**: Tests after (实现后补测试)
- **Framework**: cargo test / go test / vitest
- **Agent-Executed QA**: ALWAYS (Playwright for UI, Bash for API)

### QA Policy
Every task MUST include agent-executed QA scenarios.
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Rust Foundation — Config Module + Vault Scanner):
├── T1: tauri-plugin-store + keyring 依赖安装 [quick]
├── T2: config 模块 + SysConfig struct [quick]
├── T3: keyring wrapper + 降级逻辑 [unspecified-high]
├── T4: reload_sys_config 命令 + 热重载 [deep]
├── T5: vault scanner (config-only) [quick]
└── T6: markdown wikilink 替换器 (AST-aware) [deep]

Wave 2 (Go Backend — Merge Endpoint + Transaction):
├── T7: POST /cards/merge endpoint [unspecified-high]
├── T8: MergeCards() 服务 + 原子边事务 [deep]
├── T9: 边去重 + 自环处理 [quick]
└── T10: merge_warning WebSocket 事件 [quick]

Wave 3 (Vue Frontend — Settings + Merge UI):
├── T11: Pinia config store [quick]
├── T12: Settings.vue 面板 + 连通性测试 [visual-engineering]
├── T13: MergePanel.vue + 爆炸半径预览 [visual-engineering]
├── T14: 合并确认流程 + 重试降级 [unspecified-high]
└── T15: 首次运行引导 [quick]

Wave 4 (Integration — End-to-End):
├── T16: Rust ↔ Go 合并集成 [deep]
├── T17: 配置热重载端到端测试 [unspecified-high]
└── T18: 合并流程端到端测试 [unspecified-high]

Wave FINAL (After ALL tasks — 4 parallel reviews):
├── F1: Plan compliance audit (oracle)
├── F2: Code quality review (unspecified-high)
├── F3: Real manual QA (unspecified-high + playwright)
└── F4: Scope fidelity check (deep)
→ Present results → Get explicit user okay

Critical Path: T1 → T4 → T7 → T10 → T13 → T16 → F1-F4
Parallel Speedup: ~60% faster than sequential
Max Concurrent: 6 (Wave 1)
```

### Dependency Matrix

| Task | Depends On | Blocks |
|------|------------|--------|
| T1 | — | T2, T3, T4 |
| T2 | T1 | T4, T11 |
| T3 | T1 | T4, T11 |
| T4 | T2, T3 | T16, T17 |
| T5 | — | T6 |
| T6 | T5 | T16, T18 |
| T7 | — | T8, T10 |
| T8 | T7 | T16 |
| T9 | T8 | T16 |
| T10 | T7 | T14 |
| T11 | T2, T3 | T12, T15 |
| T12 | T11 | T17 |
| T13 | — | T14, T18 |
| T14 | T10, T13 | T18 |
| T15 | T11 | — |
| T16 | T4, T6, T8, T9 | T18 |
| T17 | T4, T12 | — |
| T18 | T6, T13, T14, T16 | — |

### Agent Dispatch Summary
- **Wave 1**: **6** — T1,T2,T5 → `quick`, T3,T6 → `deep`/`unspecified-high`, T4 → `deep`
- **Wave 2**: **4** — T7,T10 → `unspecified-high`/`quick`, T8 → `deep`, T9 → `quick`
- **Wave 3**: **5** — T11,T15 → `quick`, T12,T13 → `visual-engineering`, T14 → `unspecified-high`
- **Wave 4**: **3** — T16 → `deep`, T17,T18 → `unspecified-high`
- **FINAL**: **4** — F1 → `oracle`, F2,F3 → `unspecified-high`, F4 → `deep`

---

## TODOs

- [ ] 1. Install tauri-plugin-store + keyring Dependencies

  **What to do**:
  - Add `tauri-plugin-store = "2"` to `Cargo.toml` dependencies
  - Add `keyring = "3"` to `Cargo.toml` dependencies
  - Add npm package: `npm install @tauri-apps/plugin-store`
  - Register plugins in `lib.rs`: `.plugin(tauri_plugin_store::Builder::new().build())`
  - Add capability permission: `"store:default"` to `capabilities/default.json`

  **Must NOT do**:
  - Do NOT add tauri-plugin-stronghold (user chose keyring)
  - Do NOT modify Go server dependencies

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Dependency installation and registration, straightforward
  - **Skills**: []
  - **Skills Evaluated but Omitted**:
    - `git-master`: Not needed for dependency installation

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T2, T5)
  - **Parallel Group**: Wave 1 (with T2, T3, T4, T5, T6)
  - **Blocks**: T2, T3, T4, T11
  - **Blocked By**: None (can start immediately)

  **References**:
  - `frontend-workspace/apps/admin-tauri/src-tauri/Cargo.toml:16-26` — Existing Tauri plugins pattern
  - `frontend-workspace/apps/admin-tauri/src-tauri/src/lib.rs:530-553` — Plugin registration location
  - `frontend-workspace/apps/admin-tauri/src-tauri/capabilities/default.json` — Permission config

  **Acceptance Criteria**:
  - [ ] `cargo build` succeeds with new dependencies
  - [ ] `npm run tauri dev` starts without errors
  - [ ] Store plugin accessible via `app.store()` in Rust

  **QA Scenarios**:

  ```
  Scenario: Verify store plugin loads correctly
    Tool: Bash (cargo build)
    Preconditions: Cargo.toml updated with dependencies
    Steps:
      1. Run `cargo build --manifest-path frontend-workspace/apps/admin-tauri/src-tauri/Cargo.toml`
      2. Check exit code is 0
    Expected Result: Build succeeds, no compile errors
    Failure Indicators: Exit code != 0, missing crate errors
    Evidence: .sisyphus/evidence/task-01-store-plugin-build.txt

  Scenario: Verify npm package installs
    Tool: Bash (npm)
    Preconditions: package.json exists
    Steps:
      1. Run `cd frontend-workspace/apps/admin-tauri && npm install @tauri-apps/plugin-store`
      2. Run `npm ls @tauri-apps/plugin-store`
      3. Check package is listed
    Expected Result: Package appears in npm ls output
    Failure Indicators: npm ERR!, package not found
    Evidence: .sisyphus/evidence/task-01-npm-store-install.txt
  ```

  **Evidence to Capture**:
  - [ ] Build output showing successful compilation
  - [ ] npm ls output showing @tauri-apps/plugin-store

  **Commit**: YES
  - Message: `feat(sysconfig): add tauri-plugin-store and keyring dependencies`
  - Files: `Cargo.toml`, `package.json`, `lib.rs`, `capabilities/default.json`
  - Pre-commit: `cargo build`

- [ ] 2. Create Config Module + SysConfig Struct

  **What to do**:
  - Create `src-tauri/src/config/mod.rs` with `SysConfig` struct
  - Struct fields: `api_base_url: String`, `ws_url: String`, `s3_endpoint: String`, `s3_region: String`, `s3_bucket: String`, `s3_access_key: Option<String>` (sensitive), `s3_secret_key: Option<String>` (sensitive), `s3_public_url_base: String`
  - Implement `Default` trait with localhost fallback values
  - Implement `Serialize/Deserialize` for JSON compatibility
  - Add `get_config()` and `save_config()` functions using store

  **Must NOT do**:
  - Do NOT store sensitive fields in store (use keyring in T3)
  - Do NOT modify existing image.rs config loading yet

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Struct definition and basic CRUD, well-defined scope
  - **Skills**: [`coding-standards`]
    - `coding-standards`: Ensure proper Rust patterns, error handling

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T1, T5)
  - **Parallel Group**: Wave 1
  - **Blocks**: T4, T11
  - **Blocked By**: T1 (needs store plugin)

  **References**:
  - `frontend-workspace/apps/admin-tauri/src-tauri/src/image.rs:39-61` — Current S3 config loading pattern
  - `rust-workspace/ms-storage/src/lib.rs:11-19` — StorageConfig struct for reference
  - `tauri-plugin-store` docs: https://v2.tauri.app/plugin/store/

  **Acceptance Criteria**:
  - [ ] `config/mod.rs` created with SysConfig struct
  - [ ] `Default::default()` returns localhost values
  - [ ] `get_config()` reads from store
  - [ ] `save_config()` writes to store

  **QA Scenarios**:

  ```
  Scenario: Verify default config values
    Tool: Bash (cargo test)
    Preconditions: config/mod.rs created
    Steps:
      1. Run `cargo test --manifest-path frontend-workspace/apps/admin-tauri/src-tauri/Cargo.toml config::tests::test_default`
      2. Assert test passes
    Expected Result: Test passes, default values match expected
    Failure Indicators: Test fails, missing fields
    Evidence: .sisyphus/evidence/task-02-default-config.txt

  Scenario: Verify config save/load round-trip
    Tool: Bash (cargo test)
    Preconditions: get_config/save_config implemented
    Steps:
      1. Run test that saves config with custom values
      2. Load config and assert values match
    Expected Result: Loaded config matches saved config
    Failure Indicators: Values differ, serialization error
    Evidence: .sisyphus/evidence/task-02-config-roundtrip.txt
  ```

  **Evidence to Capture**:
  - [ ] Test output showing default values
  - [ ] Round-trip test passing

  **Commit**: YES
  - Message: `feat(sysconfig): add SysConfig struct with store persistence`
  - Files: `src-tauri/src/config/mod.rs`, `src-tauri/src/lib.rs` (mod declaration)
  - Pre-commit: `cargo test config::`

- [ ] 3. Implement Keyring Wrapper with Fallback

  **What to do**:
  - Create `src-tauri/src/config/keyring_wrapper.rs`
  - Implement `store_secret(key: &str, value: &str) -> Result<()>`
  - Implement `get_secret(key: &str) -> Result<Option<String>>`
  - Implement fallback logic: if keyring fails, use encrypted store via AES-256
  - Add platform detection: log warning on Linux if keyring unavailable
  - Define secret keys: `S3_ACCESS_KEY`, `S3_SECRET_KEY`

  **Must NOT do**:
  - Do NOT fall back to plaintext storage
  - Do NOT require user password (use OS keychain)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Security-critical code, needs careful error handling and cross-platform testing
  - **Skills**: [`coding-standards`]
    - `coding-standards`: Secure coding patterns, proper error handling

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T1, T5)
  - **Parallel Group**: Wave 1
  - **Blocks**: T4, T11, T12
  - **Blocked By**: T1 (needs keyring dependency)

  **References**:
  - `keyring` crate docs: https://docs.rs/keyring/latest/keyring/
  - `frontend-workspace/apps/admin-tauri/src-tauri/src/image.rs:39-61` — Current credential loading to replace
  - Metis research: Linux keyring may be memory-only

  **Acceptance Criteria**:
  - [ ] `store_secret()` successfully stores to OS keyring
  - [ ] `get_secret()` retrieves from keyring
  - [ ] Fallback to encrypted store works on keyring failure
  - [ ] Unit tests cover: success, keyring unavailable, fallback

  **QA Scenarios**:

  ```
  Scenario: Verify secret storage to keyring
    Tool: Bash (cargo test)
    Preconditions: keyring_wrapper.rs created
    Steps:
      1. Run `cargo test keyring_wrapper::tests::test_store_and_retrieve`
      2. Assert test passes
    Expected Result: Secret stored and retrieved successfully
    Failure Indicators: Keyring error, value mismatch
    Evidence: .sisyphus/evidence/task-03-keyring-store.txt

  Scenario: Verify fallback when keyring unavailable
    Tool: Bash (cargo test)
    Preconditions: Fallback logic implemented
    Steps:
      1. Run test that simulates keyring failure
      2. Assert fallback to encrypted store succeeds
    Expected Result: Secret stored in fallback without error
    Failure Indicators: Fallback fails, panic
    Evidence: .sisyphus/evidence/task-03-keyring-fallback.txt
  ```

  **Evidence to Capture**:
  - [ ] Test output for keyring store/retrieve
  - [ ] Test output for fallback scenario

  **Commit**: YES
  - Message: `feat(sysconfig): add keyring wrapper with encrypted fallback`
  - Files: `src-tauri/src/config/keyring_wrapper.rs`, `src-tauri/src/config/mod.rs`
  - Pre-commit: `cargo test keyring_wrapper::`

- [ ] 4. Implement reload_sys_config Command + Hot Reload

  **What to do**:
  - Add `#[tauri::command] fn reload_sys_config(app: AppHandle) -> Result<(), String>` in `lib.rs`
  - Wrap S3 client and HTTP client in `Arc<Mutex<Option<Client>>>` as global state
  - On reload: clear old clients, reload config from store/keyring, recreate clients
  - Update `image.rs` `load_storage_config()` to read from store instead of env vars
  - Update `ws_client.rs` to read WS URL from config instead of hardcoded value
  - Emit `config-reloaded` event on success
  - Emit `config-reload-failed` event with error message on failure
  - Register command in `lib.rs` invoke_handler

  **Must NOT do**:
  - Do NOT block main thread during client recreation
  - Do NOT restart entire app
  - Do NOT modify Go server configuration loading

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Global state management, async coordination, IPC design, multi-file refactoring
  - **Skills**: [`coding-standards`]
    - `coding-standards`: Proper async patterns, thread safety, Arc<Mutex> usage

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T5, T6)
  - **Parallel Group**: Wave 1
  - **Blocks**: T12, T16, T17
  - **Blocked By**: T2, T3 (needs config module and keyring)

  **References**:
  - `frontend-workspace/apps/admin-tauri/src-tauri/src/image.rs:39-61` — Current `load_storage_config()` to refactor (read from store instead of dotenv)
  - `frontend-workspace/apps/admin-tauri/src-tauri/src/image.rs:178-180` — Current S3Backend creation (wrap in Arc<Mutex>)
  - `frontend-workspace/apps/admin-tauri/src-tauri/src/ws_client.rs:10` — Hardcoded `ws://localhost:8080/api/v1/ws` to replace with config value
  - `frontend-workspace/apps/admin-tauri/src-tauri/src/lib.rs:530-553` — Plugin registration and command handler location

  **Acceptance Criteria**:
  - [ ] `reload_sys_config` command registered and callable from frontend
  - [ ] S3 client recreated with new credentials on reload
  - [ ] WS client reconnects to new URL on reload
  - [ ] `config-reloaded` event emitted on success
  - [ ] `config-reload-failed` event emitted on failure
  - [ ] `image.rs` reads S3 config from store (not dotenv)
  - [ ] `ws_client.rs` reads WS URL from config (not hardcoded)

  **QA Scenarios**:

  ```
  Scenario: Verify hot reload updates S3 client
    Tool: Bash (cargo test)
    Preconditions: reload_sys_config implemented
    Steps:
      1. Save initial config with localhost S3 endpoint
      2. Call reload_sys_config
      3. Save new config with different S3 endpoint
      4. Call reload_sys_config again
      5. Assert S3 client uses new endpoint
    Expected Result: Client recreated with new config
    Failure Indicators: Old config still used, panic
    Evidence: .sisyphus/evidence/task-04-hot-reload-s3.txt

  Scenario: Verify reload emits events
    Tool: Bash (cargo test)
    Preconditions: Event emission implemented
    Steps:
      1. Subscribe to `config-reloaded` event
      2. Call reload_sys_config with valid config
      3. Assert `config-reloaded` event received
      4. Call reload_sys_config with invalid config
      5. Assert `config-reload-failed` event received
    Expected Result: Correct event for success/failure
    Failure Indicators: Wrong event, no event, timeout
    Evidence: .sisyphus/evidence/task-04-reload-events.txt
  ```

  **Evidence to Capture**:
  - [ ] Test output showing client recreation
  - [ ] Event emission verification

  **Commit**: YES
  - Message: `feat(sysconfig): add reload_sys_config command with hot reload`
  - Files: `src-tauri/src/lib.rs`, `src-tauri/src/image.rs`, `src-tauri/src/ws_client.rs`
  - Pre-commit: `cargo test reload`

- [ ] 5. Implement Vault Scanner (Config-Only)

  **What to do**:
  - Create `src-tauri/src/vault_scanner.rs`
  - Implement `scan_for_missing_config() -> Vec<ConfigIssue>`
  - Check for: missing S3 keys, invalid URLs, unreachable endpoints
  - Return issues with severity: `Critical`, `Warning`, `Info`
  - Integrate with first-run logic: auto-open settings if `Critical` issues

  **Must NOT do**:
  - Do NOT scan for wikilink consistency
  - Do NOT scan for orphan files
  - Do NOT modify any files (read-only scan)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Read-only scanning, straightforward logic
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T1-T4, T6)
  - **Parallel Group**: Wave 1
  - **Blocks**: T17
  - **Blocked By**: T2 (needs SysConfig struct)

  **References**:
  - `frontend-workspace/apps/admin-tauri/src-tauri/src/image.rs:39-61` — Config fields to validate
  - `frontend-workspace/apps/admin-tauri/src-tauri/src/ws_client.rs:10` — Hardcoded WS URL to detect

  **Acceptance Criteria**:
  - [ ] `scan_for_missing_config()` returns empty vec when config valid
  - [ ] Returns `ConfigIssue` for missing S3 keys
  - [ ] Returns `ConfigIssue` for invalid URL format

  **QA Scenarios**:

  ```
  Scenario: Verify scanner detects missing S3 keys
    Tool: Bash (cargo test)
    Preconditions: vault_scanner.rs created
    Steps:
      1. Create config with missing s3_access_key
      2. Run scan_for_missing_config()
      3. Assert returns Critical issue
    Expected Result: Issue detected with severity Critical
    Failure Indicators: Empty vec returned
    Evidence: .sisyphus/evidence/task-05-scan-missing-s3.txt

  Scenario: Verify scanner passes valid config
    Tool: Bash (cargo test)
    Preconditions: Valid config available
    Steps:
      1. Create config with all fields valid
      2. Run scan_for_missing_config()
      3. Assert returns empty vec
    Expected Result: No issues returned
    Failure Indicators: False positive issues
    Evidence: .sisyphus/evidence/task-05-scan-valid.txt
  ```

  **Evidence to Capture**:
  - [ ] Test output for missing S3 keys
  - [ ] Test output for valid config

  **Commit**: YES
  - Message: `feat(sysconfig): add vault scanner for config validation`
  - Files: `src-tauri/src/vault_scanner.rs`, `src-tauri/src/lib.rs`
  - Pre-commit: `cargo test vault_scanner`

- [ ] 6. Implement Markdown Wikilink Replacer (AST-Aware)

  **What to do**:
  - Create `src-tauri/src/wikilink_replacer.rs`
  - Implement `scan_vault_for_wikilinks(vault_path: &Path, victim_title: &str) -> Vec<PathBuf>` — find all .md files containing `[[victim_title]]`
  - Implement `replace_wikilinks_in_memory(files: &mut HashMap<PathBuf, String>, victim_title: &str, survivor_title: &str) -> Result<()>`
  - Use pulldown-cmark AST parsing to SKIP wikilinks inside code blocks (``` `` ` `` and `` ` ``)
  - Only replace exact title matches: `[[Victim]]` → `[[Survivor]]`, NOT `[[Victim Extended]]`
  - Modified content stays in memory HashMap, NOT written to disk yet
  - Add Tauri command `#[tauri::command] fn preview_merge_impact(vault_path: String, victim_titles: Vec<String>) -> Result<MergePreview>` returning file list + count

  **Must NOT do**:
  - Do NOT write files to disk (T16 handles write-back after Go transaction)
  - Do NOT use regex-only replacement (must be AST-aware to skip code blocks)
  - Do NOT handle `[[title|display]]` syntax (out of scope per MVP wikilink spec)

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: AST parsing, string manipulation, edge cases (code blocks, partial matches, unicode)
  - **Skills**: [`coding-standards`]
    - `coding-standards`: Proper Rust patterns, error handling

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T1-T5)
  - **Parallel Group**: Wave 1
  - **Blocks**: T13, T16, T18
  - **Blocked By**: None (independent Rust module)

  **References**:
  - `rust-workspace/md-parser/src/lib.rs:20-35` — Existing `extract_wikilinks()` regex pattern
  - `rust-workspace/md-parser/src/lib.rs` — pulldown-cmark usage for AST parsing
  - `rust-workspace/ast-renderer/src/lib.rs` — AST event iteration pattern
  - `go-server/internal/services/card.go:43-75` — `FindOrCreateByTitle()` for title matching rules

  **Acceptance Criteria**:
  - [ ] `scan_vault_for_wikilinks()` finds all .md files with matching title
  - [ ] `replace_wikilinks_in_memory()` replaces exact `[[Victim]]` → `[[Survivor]]`
  - [ ] Wikilinks inside code blocks (``` fenced and inline) are NOT replaced
  - [ ] `[[Victim]]` replaced but `[[Victim Extended]]` is NOT replaced
  - [ ] Unicode titles handled correctly
  - [ ] `preview_merge_impact` command callable from frontend

  **QA Scenarios**:

  ```
  Scenario: Verify exact title replacement
    Tool: Bash (cargo test)
    Preconditions: wikilink_replacer.rs created
    Steps:
      1. Create test markdown: "See [[Alpha]] and [[Alpha Beta]]"
      2. Run replace_wikilinks_in_memory with victim="Alpha", survivor="Gamma"
      3. Assert result: "See [[Gamma]] and [[Alpha Beta]]"
    Expected Result: Only exact [[Alpha]] replaced, [[Alpha Beta]] untouched
    Failure Indicators: Both replaced, or neither replaced
    Evidence: .sisyphus/evidence/task-06-exact-replace.txt

  Scenario: Verify code block skipping
    Tool: Bash (cargo test)
    Preconditions: AST-aware parsing implemented
    Steps:
      1. Create test markdown with wikilink inside ```code block``` and `inline code`
      2. Run replace_wikilinks_in_memory
      3. Assert wikilinks in code blocks are unchanged
    Expected Result: Only wikilinks outside code blocks replaced
    Failure Indicators: Code block content modified
    Evidence: .sisyphus/evidence/task-06-codeblock-skip.txt

  Scenario: Verify unicode title handling
    Tool: Bash (cargo test)
    Preconditions: Unicode support implemented
    Steps:
      1. Create test markdown: "链接到 [[中文节点]] 的内容"
      2. Run replace_wikilinks_in_memory with victim="中文节点"
      3. Assert correct replacement
    Expected Result: Unicode title replaced correctly
    Failure Indicators: Garbled text, panic on unicode
    Evidence: .sisyphus/evidence/task-06-unicode.txt
  ```

  **Evidence to Capture**:
  - [ ] Test output for exact title replacement
  - [ ] Test output for code block skipping
  - [ ] Test output for unicode handling

  **Commit**: YES
  - Message: `feat(merger): add AST-aware wikilink replacer with code block skipping`
  - Files: `src-tauri/src/wikilink_replacer.rs`, `src-tauri/src/lib.rs`
  - Pre-commit: `cargo test wikilink_replacer`

- [ ] 7. Implement POST /cards/merge Endpoint

  **What to do**:
  - Create `go-server/internal/handlers/merge.go`
  - Define `MergeRequest` struct: `SurvivorID uuid.UUID`, `VictimIDs []uuid.UUID`
  - Define `MergeResponse` struct: `EdgesMigrated int`, `NodesDeleted int`, `Warnings []string`
  - Implement `MergeCards` handler: validate input (no self-merge, survivor not in victims), call service, return JSON
  - Register route: `r.POST("/cards/merge", h.MergeCards)` in router setup
  - Add auth middleware to route (require JWT)

  **Must NOT do**:
  - Do NOT implement business logic here (delegate to T8 service)
  - Do NOT modify existing card/edge handlers
  - Do NOT add CORS changes

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: API design, input validation, error response formatting
  - **Skills**: [`coding-standards`]
    - `coding-standards`: Go handler patterns, error handling

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T5, T6, T8)
  - **Parallel Group**: Wave 2
  - **Blocks**: T8, T10, T13
  - **Blocked By**: None (can start immediately, calls T8 service)

  **References**:
  - `go-server/internal/handlers/card.go:182-200` — Existing handler pattern (Update with extracted_links)
  - `go-server/internal/handlers/card.go` — Route registration pattern
  - `go-server/internal/middleware/auth.go` — Auth middleware for JWT validation
  - `go-server/cmd/api/main.go` — Router setup location

  **Acceptance Criteria**:
  - [ ] `POST /cards/merge` returns 400 for self-merge
  - [ ] `POST /cards/merge` returns 400 for empty victim_ids
  - [ ] `POST /cards/merge` returns 401 without JWT
  - [ ] `POST /cards/merge` returns 200 with MergeResponse on success

  **QA Scenarios**:

  ```
  Scenario: Verify self-merge rejection
    Tool: Bash (curl)
    Preconditions: Go server running with test data
    Steps:
      1. curl -X POST http://localhost:8080/api/v1/cards/merge \
           -H "Authorization: Bearer $TOKEN" \
           -d '{"survivor_id":"aaa","victim_ids":["aaa"]}'
      2. Assert HTTP 400
      3. Assert body contains "Cannot merge node into itself"
    Expected Result: 400 Bad Request with clear error message
    Failure Indicators: 200 OK, or 500 Internal Server Error
    Evidence: .sisyphus/evidence/task-07-self-merge-reject.txt

  Scenario: Verify empty victims rejection
    Tool: Bash (curl)
    Preconditions: Go server running
    Steps:
      1. curl -X POST http://localhost:8080/api/v1/cards/merge \
           -H "Authorization: Bearer $TOKEN" \
           -d '{"survivor_id":"aaa","victim_ids":[]}'
      2. Assert HTTP 400
    Expected Result: 400 Bad Request
    Failure Indicators: 200 OK
    Evidence: .sisyphus/evidence/task-07-empty-victims.txt
  ```

  **Evidence to Capture**:
  - [ ] curl output for self-merge rejection
  - [ ] curl output for empty victims rejection

  **Commit**: YES
  - Message: `feat(merger): add POST /cards/merge endpoint with validation`
  - Files: `go-server/internal/handlers/merge.go`, `go-server/cmd/api/main.go`
  - Pre-commit: `go build ./...`

- [ ] 8. Implement MergeCards() Service + Atomic Edge Transaction

  **What to do**:
  - Create `MergeCards(survivorID uuid.UUID, victimIDs []uuid.UUID) (*MergeResult, error)` in `go-server/internal/services/card.go` (or new `merge.go`)
  - Execute in a single PostgreSQL transaction:
    1. `BEGIN`
    2. `UPDATE card_edges SET target_id = $survivor WHERE target_id IN ($victims)` — redirect incoming edges
    3. `UPDATE card_edges SET source_id = $survivor WHERE source_id IN ($victors)` — redirect outgoing edges
    4. Handle duplicate edges: `ON CONFLICT (source_id, target_id, relation_type) DO NOTHING` (dedup by composite key)
    5. Delete any self-loop edges created (source_id = target_id = survivor_id AND relation_type = 'sequence') — keep reference self-loops
    6. `DELETE FROM cards WHERE id IN ($victims)`
    7. `COMMIT`
  - Return `MergeResult`: edges migrated count, nodes deleted count, any warnings

  **Must NOT do**:
  - Do NOT modify existing `SyncReferenceEdges()` — this is a new flow
  - Do NOT delete survivor node
  - Do NOT touch sequence edges unless they become self-loops

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Complex SQL transaction, edge deduplication logic, self-loop handling, error recovery
  - **Skills**: [`coding-standards`]
    - `coding-standards`: Transaction patterns, error handling

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on T7 handler)
  - **Parallel Group**: Wave 2 (sequential after T7)
  - **Blocks**: T9, T16
  - **Blocked By**: T7 (handler calls this service)

  **References**:
  - `go-server/internal/services/edge.go:130-196` — `SyncReferenceEdges()` transaction pattern (MUST follow this pattern)
  - `go-server/internal/services/edge.go:144-152` — Getting current edges for a card
  - `go-server/internal/models/edge.go` — CardEdge model, composite key structure
  - `go-server/internal/services/card.go:43-75` — `FindOrCreateByTitle()` for ghost card handling
  - PostgreSQL schema: `card_edges` table with `(source_id, target_id, relation_type)` unique constraint

  **Acceptance Criteria**:
  - [ ] All edge updates in single PostgreSQL transaction
  - [ ] Duplicate edges handled via `ON CONFLICT DO NOTHING`
  - [ ] Sequence self-loops removed (source_id = target_id = survivor)
  - [ ] Reference self-loops preserved (allowed)
  - [ ] Victim nodes deleted after edge migration
  - [ ] Transaction rolls back on ANY error
  - [ ] `go test ./internal/services/...` passes

  **QA Scenarios**:

  ```
  Scenario: Verify atomic edge migration
    Tool: Bash (go test)
    Preconditions: Test database with cards and edges
    Steps:
      1. Create cards A (survivor), B, C (victims)
      2. Create edges: X→B, B→Y, Z→C, C→W
      3. Run MergeCards(A, [B, C])
      4. Assert edges now: X→A, A→Y, Z→A, A→W
      5. Assert cards B, C deleted
    Expected Result: All edges migrated, victims deleted
    Failure Indicators: Missing edges, victims still exist
    Evidence: .sisyphus/evidence/task-08-atomic-merge.txt

  Scenario: Verify transaction rollback on failure
    Tool: Bash (go test)
    Preconditions: Force error during merge (e.g., invalid victim ID)
    Steps:
      1. Create cards A, B with edges
      2. Run MergeCards with one valid and one non-existent victim
      3. Assert NO changes persisted (edges unchanged, no deletions)
    Expected Result: Full rollback, zero side effects
    Failure Indicators: Partial state change
    Evidence: .sisyphus/evidence/task-08-rollback.txt

  Scenario: Verify duplicate edge dedup
    Tool: Bash (go test)
    Preconditions: A→Z and B→Z edges exist (same relation_type)
    Steps:
      1. Merge B into A
      2. Assert only ONE A→Z edge remains (not two)
    Expected Result: Single edge after merge, no duplicates
    Failure Indicators: Duplicate edge created
    Evidence: .sisyphus/evidence/task-08-dedup.txt
  ```

  **Evidence to Capture**:
  - [ ] Test output for atomic merge
  - [ ] Test output for rollback
  - [ ] Test output for dedup

  **Commit**: YES
  - Message: `feat(merger): implement MergeCards service with atomic edge transaction`
  - Files: `go-server/internal/services/merge.go`
  - Pre-commit: `go test ./internal/services/...`

- [ ] 9. Handle Edge Deduplication + Self-Loop Cleanup

  **What to do**:
  - Add `deduplicateEdges(tx *gorm.DB, survivorID uuid.UUID) error` helper in `go-server/internal/services/merge.go`
  - Query for duplicate edges after migration: `SELECT source_id, target_id, relation_type, COUNT(*) FROM card_edges WHERE source_id = $survivor OR target_id = $survivor GROUP BY source_id, target_id, relation_type HAVING COUNT(*) > 1`
  - Delete duplicates, keeping the earliest created edge
  - Add `removeSequenceSelfLoops(tx *gorm.DB, survivorID uuid.UUID) error`
  - Delete edges where `source_id = target_id = survivor_id AND relation_type = 'sequence'`
  - Keep reference self-loops (user explicitly wrote `[[Self]]`)

  **Must NOT do**:
  - Do NOT delete reference self-loops
  - Do NOT create new edges (only dedup and clean)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Focused SQL operations, clear scope
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on T8)
  - **Parallel Group**: Wave 2 (sequential after T8)
  - **Blocks**: T16
  - **Blocked By**: T8

  **References**:
  - `go-server/internal/services/merge.go` — Created in T8, add helpers here
  - `go-server/internal/models/edge.go` — Edge model with relation_type field

  **Acceptance Criteria**:
  - [ ] Duplicate edges removed after merge
  - [ ] Sequence self-loops removed
  - [ ] Reference self-loops preserved
  - [ ] Unit tests pass

  **QA Scenarios**:

  ```
  Scenario: Verify sequence self-loop removal
    Tool: Bash (go test)
    Steps:
      1. After merge, if survivor has sequence edge to itself, it's deleted
      2. Assert no sequence self-loops remain
    Expected Result: Sequence self-loops deleted
    Evidence: .sisyphus/evidence/task-09-self-loop-cleanup.txt

  Scenario: Verify reference self-loop preservation
    Tool: Bash (go test)
    Steps:
      1. Create survivor with `[[Survivor]]` wikilink in content
      2. Merge victim into survivor
      3. Assert reference self-loop (survivor → survivor, type=reference) still exists
    Expected Result: Reference self-loop preserved
    Evidence: .sisyphus/evidence/task-09-ref-self-loop.txt
  ```

  **Evidence to Capture**:
  - [ ] Test output for self-loop handling

  **Commit**: YES
  - Message: `feat(merger): add edge deduplication and self-loop cleanup`
  - Files: `go-server/internal/services/merge.go`
  - Pre-commit: `go test ./internal/services/...`

- [ ] 10. Broadcast Merge WebSocket Event

  **What to do**:
  - In `MergeCards()` service, after successful transaction COMMIT, broadcast WebSocket event
  - Event type: `cards_merged`
  - Payload: `{ "survivor_id": "...", "victim_ids": ["..."], "edges_migrated": 5, "nodes_deleted": 2 }`
  - Also broadcast individual `card_deleted` events for each victim (so frontend removes from graph)
  - Broadcast `card_updated` for survivor (so frontend refreshes edges)
  - Use existing WebSocket hub broadcast mechanism

  **Must NOT do**:
  - Do NOT create new WebSocket infrastructure (use existing hub)
  - Do NOT broadcast before transaction COMMIT

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Simple event broadcast using existing infrastructure
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T8)
  - **Parallel Group**: Wave 2
  - **Blocks**: T14
  - **Blocked By**: T7 (needs handler/route context)

  **References**:
  - `go-server/internal/handlers/ws.go` — WebSocket hub and broadcast mechanism
  - `go-server/internal/services/edge.go` — Existing event broadcast patterns (if any)

  **Acceptance Criteria**:
  - [ ] `cards_merged` event broadcast after successful merge
  - [ ] `card_deleted` event broadcast for each victim
  - [ ] `card_updated` event broadcast for survivor
  - [ ] Events only sent after COMMIT (not before)

  **QA Scenarios**:

  ```
  Scenario: Verify merge event broadcast
    Tool: Bash (wscat or curl websocket)
    Preconditions: WebSocket connected, merge endpoint available
    Steps:
      1. Connect to ws://localhost:8080/api/v1/ws with JWT
      2. Trigger merge via REST API
      3. Assert `cards_merged` event received
      4. Assert `card_deleted` events received for each victim
    Expected Result: All 3 event types received in order
    Failure Indicators: Missing events, events before commit
    Evidence: .sisyphus/evidence/task-10-ws-events.txt
  ```

  **Evidence to Capture**:
  - [ ] WebSocket event capture

  **Commit**: YES (groups with T8)
  - Message: `feat(merger): broadcast cards_merged WebSocket event`
  - Files: `go-server/internal/services/merge.go`
  - Pre-commit: `go test ./internal/services/...`

- [ ] 11. Create Pinia Config Store

  **What to do**:
  - Create `frontend-workspace/apps/admin-tauri/src/stores/sysconfig.ts`
  - Define TypeScript interfaces: `SysConfig { api_base_url, ws_url, s3_endpoint, s3_region, s3_bucket, s3_access_key?, s3_secret_key?, s3_public_url_base }`
  - Implement Pinia store with actions:
    - `loadConfig()` — read from Tauri store via `@tauri-apps/plugin-store`
    - `saveConfig(config)` — write to store + save secrets to keyring
    - `testConnection()` — call Tauri command that pings API + tests S3 list_buckets
    - `reloadConfig()` — invoke `reload_sys_config` Tauri command
  - State: `config: SysConfig | null`, `loading: boolean`, `error: string | null`, `connectionStatus: 'idle' | 'testing' | 'ok' | 'failed'`
  - Use `defineStore('sysconfig', ...)` pattern matching existing stores

  **Must NOT do**:
  - Do NOT store secrets in Pinia persisted state
  - Do NOT call Go API directly (use Tauri IPC commands)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Well-defined Pinia store pattern, follow existing stores
  - **Skills**: [`coding-standards`]
    - `coding-standards`: TypeScript patterns, Pinia conventions

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T10)
  - **Parallel Group**: Wave 3
  - **Blocks**: T12, T15
  - **Blocked By**: T2, T3 (needs Rust config module + keyring wrapper)

  **References**:
  - `frontend-workspace/apps/admin-tauri/src/stores/knowledge.ts` — Existing Pinia store pattern, API call conventions
  - `frontend-workspace/apps/admin-tauri/src/stores/knowledge.ts:543-553` — Example async action pattern
  - `@tauri-apps/plugin-store` API: `load()`, `store.set()`, `store.get()`

  **Acceptance Criteria**:
  - [ ] Store loads config from Tauri store on app start
  - [ ] `saveConfig()` writes non-sensitive to store, sensitive to keyring
  - [ ] `testConnection()` returns success/failure status
  - [ ] `reloadConfig()` invokes Tauri command

  **QA Scenarios**:

  ```
  Scenario: Verify config store loads
    Tool: Playwright
    Preconditions: Tauri dev running with valid config
    Steps:
      1. Navigate to settings page
      2. Assert config fields populated with stored values
    Expected Result: Fields show stored config values
    Failure Indicators: Empty fields, error toast
    Evidence: .sisyphus/evidence/task-11-store-load.png

  Scenario: Verify config save round-trip
    Tool: Playwright
    Preconditions: Settings page open
    Steps:
      1. Change API URL to "http://new-api:9090/api/v1"
      2. Click Save
      3. Reload app
      4. Assert new URL persisted
    Expected Result: URL persists after reload
    Failure Indicators: Reverts to old value
    Evidence: .sisyphus/evidence/task-11-save-roundtrip.png
  ```

  **Evidence to Capture**:
  - [ ] Screenshot of loaded config
  - [ ] Screenshot after save round-trip

  **Commit**: YES
  - Message: `feat(sysconfig): add Pinia sysconfig store with keyring integration`
  - Files: `frontend-workspace/apps/admin-tauri/src/stores/sysconfig.ts`
  - Pre-commit: `npm run type-check`

- [ ] 12. Build Settings.vue Panel + Connectivity Test

  **What to do**:
  - Create `frontend-workspace/apps/admin-tauri/src/views/Settings.vue`
  - Full-screen black overlay panel (position: fixed, z-index: 50, bg: ms-deep #0d0d0d)
  - Two sections with monospace headers:
    - `[ NETWORK ]` — api_base_url (text input), ws_url (text input)
    - `[ STORAGE ]` — s3_endpoint, s3_region, s3_bucket, s3_access_key (password), s3_secret_key (password), s3_public_url_base
  - `[ TEST CONNECTION ]` button next to each section — calls `testConnection()`, shows green ✓ or red ✗
  - `[ SAVE ]` button — calls `saveConfig()` + `reloadConfig()`, only enabled after test passes
  - Sharp corners (no rounded-lg/xl), monospace fonts, neon accents (#00e5ff)
  - Transitions ≤ 250ms
  - Close button (top-right X) emits `close` event
  - Open from sidebar gear icon

  **Must NOT do**:
  - Do NOT use rounded corners (design system: sharp corners only)
  - Do NOT allow save without passing connectivity test
  - Do NOT use generic UI component library (custom build)

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: Industrial console aesthetic, precise visual design, Tailwind styling
  - **Skills**: [`frontend-design`, `coding-standards`]
    - `frontend-design`: Industrial dark theme, neon accents, monospace design
    - `coding-standards`: Vue 3 composition API patterns

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T13)
  - **Parallel Group**: Wave 3
  - **Blocks**: T17
  - **Blocked By**: T11 (needs Pinia store)

  **References**:
  - `frontend-workspace/apps/admin-tauri/src/components/TheForge.vue` — Existing panel layout, dark theme patterns
  - `frontend-workspace/apps/admin-tauri/src/components/RightAstrolabe.vue` — Section styling, sharp corners
  - Design tokens: ms-deep #0d0d0d, ms-panel #1a1a1a, ms-border #333, neon #00e5ff
  - `frontend-workspace/apps/admin-tauri/src/stores/sysconfig.ts` — Store created in T11

  **Acceptance Criteria**:
  - [ ] Panel renders with black background, two sections
  - [ ] Input fields bound to Pinia store state
  - [ ] TEST CONNECTION button shows status indicator
  - [ ] SAVE button disabled until test passes
  - [ ] Close button hides panel
  - [ ] Sharp corners, monospace font, neon accents

  **QA Scenarios**:

  ```
  Scenario: Verify settings panel renders
    Tool: Playwright
    Preconditions: Tauri dev running
    Steps:
      1. Click gear icon in sidebar
      2. Assert panel visible with z-index 50
      3. Assert "[ NETWORK ]" section visible
      4. Assert "[ STORAGE ]" section visible
      5. Screenshot panel
    Expected Result: Full black overlay with two labeled sections
    Failure Indicators: Panel not visible, missing sections
    Evidence: .sisyphus/evidence/task-12-panel-render.png

  Scenario: Verify connectivity test blocks save
    Tool: Playwright
    Preconditions: Settings panel open, no test run yet
    Steps:
      1. Fill API URL field
      2. Assert SAVE button is disabled
      3. Click TEST CONNECTION
      4. Assert status indicator appears
      5. If test passes, assert SAVE button enabled
    Expected Result: SAVE disabled before test, enabled after pass
    Failure Indicators: SAVE enabled without test
    Evidence: .sisyphus/evidence/task-12-test-blocks-save.png
  ```

  **Evidence to Capture**:
  - [ ] Screenshot of full panel
  - [ ] Screenshot of test/save button states

  **Commit**: YES
  - Message: `feat(sysconfig): add Settings panel with connectivity test`
  - Files: `frontend-workspace/apps/admin-tauri/src/views/Settings.vue`
  - Pre-commit: `npm run type-check`

- [ ] 13. Build MergePanel.vue + Explosion Radius Preview

  **What to do**:
  - Create `frontend-workspace/apps/admin-tauri/src/components/MergePanel.vue`
  - Multi-select card list: checkboxes for Victims, radio for Survivor
  - "Calculate Impact" button → calls `preview_merge_impact` Tauri command
  - Display explosion radius: number of edges affected, number of markdown files to modify
  - Show list of affected files with wikilink counts
  - Industrial aesthetic matching Settings panel
  - Disabled state when < 2 cards selected
  - Clear selection button

  **Must NOT do**:
  - Do NOT execute merge (T14 handles confirmation + execution)
  - Do NOT modify any data (read-only preview)

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: Complex UI component, multi-select interaction, impact visualization
  - **Skills**: [`frontend-design`, `coding-standards`]
    - `frontend-design`: Industrial dark theme, list styling
    - `coding-standards`: Vue 3 composition API

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T12)
  - **Parallel Group**: Wave 3
  - **Blocks**: T14, T18
  - **Blocked By**: None (can start immediately, reads from existing card store)

  **References**:
  - `frontend-workspace/apps/admin-tauri/src/stores/knowledge.ts` — Card list data source
  - `frontend-workspace/apps/admin-tauri/src/views/TheForge.vue` — Existing card list UI patterns
  - `src-tauri/src/wikilink_replacer.rs` — `preview_merge_impact` command (T6)

  **Acceptance Criteria**:
  - [ ] Multi-select for Victims, single-select for Survivor
  - [ ] "Calculate Impact" shows edge count and file count
  - [ ] Affected files listed with wikilink counts
  - [ ] Panel disabled when < 2 cards selected
  - [ ] Sharp corners, monospace font, neon accents

  **QA Scenarios**:

  ```
  Scenario: Verify explosion radius calculation
    Tool: Playwright
    Preconditions: Cards exist with edges and wikilinks
    Steps:
      1. Open merge panel
      2. Select 2 victim cards, 1 survivor
      3. Click "Calculate Impact"
      4. Assert edge count displayed
      5. Assert file count displayed
      6. Assert file list shown
    Expected Result: Impact numbers and file list visible
    Failure Indicators: No results, error message
    Evidence: .sisyphus/evidence/task-13-explosion-radius.png

  Scenario: Verify disabled state with < 2 cards
    Tool: Playwright
    Preconditions: Merge panel open
    Steps:
      1. Select only 1 card
      2. Assert "Calculate Impact" button is disabled
      3. Assert "Execute Merge" button is disabled
    Expected Result: Buttons disabled
    Failure Indicators: Buttons enabled
    Evidence: .sisyphus/evidence/task-13-disabled-state.png
  ```

  **Evidence to Capture**:
  - [ ] Screenshot of explosion radius
  - [ ] Screenshot of disabled state

  **Commit**: YES
  - Message: `feat(merger): add MergePanel with explosion radius preview`
  - Files: `frontend-workspace/apps/admin-tauri/src/components/MergePanel.vue`
  - Pre-commit: `npm run type-check`

- [ ] 14. Implement Merge Confirmation Flow + Retry Degradation

  **What to do**:
  - In MergePanel.vue, add confirmation flow after explosion radius shown:
    - Modal requiring user to type `CONFIRM` (exact match) to proceed
    - Warning text: "This operation will permanently delete N nodes and modify N files"
  - On confirm:
    1. Call `preview_merge_impact` to get file list (Rust reads files into memory + replaces)
    2. Call `POST /cards/merge` (Go atomic transaction)
    3. If Go returns 200, call Tauri command `write_back_merged_files` (Rust writes to disk)
    4. Listen for `merge_file_write_failed` event from Rust
  - On `merge_file_write_failed` event:
    - Show orange toast: "数据库合并成功，但文件 [X] 本地覆写受阻，请手动更正该文件内的文本链接。"
    - Show "Retry" button for each failed file
  - On full success: show green toast "合并完成" + refresh graph

  **Must NOT do**:
  - Do NOT allow merge without CONFIRM text entry
  - Do NOT skip Go transaction if Rust pre-check succeeds
  - Do NOT hide errors from user

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Complex async flow, error handling, event listening, state management
  - **Skills**: [`coding-standards`]
    - `coding-standards`: Async patterns, error handling

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on T10, T13)
  - **Parallel Group**: Wave 3 (sequential after T10, T13)
  - **Blocks**: T18
  - **Blocked By**: T10 (WS events), T13 (MergePanel UI)

  **References**:
  - `frontend-workspace/apps/admin-tauri/src/components/MergePanel.vue` — Created in T13
  - `go-server/internal/handlers/merge.go` — POST /cards/merge (T7)
  - `src-tauri/src/wikilink_replacer.rs` — Rust write-back (T6)
  - `frontend-workspace/apps/admin-tauri/src/stores/knowledge.ts` — Existing toast pattern

  **Acceptance Criteria**:
  - [ ] CONFIRM text required before merge execution
  - [ ] Execution order: Rust pre-check → Go transaction → Rust write-back
  - [ ] Orange toast + retry button on file write failure
  - [ ] Green toast on full success
  - [ ] Graph refreshes after merge

  **QA Scenarios**:

  ```
  Scenario: Verify CONFIRM requirement
    Tool: Playwright
    Preconditions: Merge panel open, explosion radius calculated
    Steps:
      1. Click "Execute Merge"
      2. Assert modal appears with CONFIRM input
      3. Type "confirm" (wrong case)
      4. Assert merge does NOT proceed
      5. Type "CONFIRM" (exact)
      6. Assert merge proceeds
    Expected Result: Only exact "CONFIRM" triggers merge
    Failure Indicators: Merge runs without confirmation
    Evidence: .sisyphus/evidence/task-14-confirm-flow.png

  Scenario: Verify graceful degradation on file write failure
    Tool: Playwright (mock)
    Preconditions: Merge executing, mock file write failure
    Steps:
      1. Execute merge
      2. Simulate file write failure event
      3. Assert orange toast appears
      4. Assert "Retry" button visible for failed file
      5. Click retry
      6. Assert retry attempted
    Expected Result: Warning shown with retry option
    Failure Indicators: Silent failure, no toast, app crash
    Evidence: .sisyphus/evidence/task-14-degradation.png
  ```

  **Evidence to Capture**:
  - [ ] Screenshot of CONFIRM modal
  - [ ] Screenshot of degradation toast

  **Commit**: YES
  - Message: `feat(merger): add merge confirmation flow with graceful degradation`
  - Files: `frontend-workspace/apps/admin-tauri/src/components/MergePanel.vue`
  - Pre-commit: `npm run type-check`

- [ ] 15. Implement First-Run Configuration Guide

  **What to do**:
  - Add first-run detection in App.vue or main layout:
    - On mount, call `vault_scanner.scan_for_missing_config()` via Tauri IPC
    - If `Critical` issues found, auto-open Settings panel
    - Show dismissible banner: "首次使用 — 请完成系统配置" with "前往配置" button
  - Settings panel auto-opens on first launch with empty config
  - After user saves valid config, banner dismissed permanently

  **Must NOT do**:
  - Do NOT block app usage if config is partially set
  - Do NOT show banner on every launch (only first time or when config invalid)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Simple detection logic, banner component, auto-open behavior
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T12, T13)
  - **Parallel Group**: Wave 3
  - **Blocks**: None
  - **Blocked By**: T11 (needs Pinia store)

  **References**:
  - `src-tauri/src/vault_scanner.rs` — Scanner created in T5
  - `frontend-workspace/apps/admin-tauri/src/App.vue` — App entry point

  **Acceptance Criteria**:
  - [ ] First launch with empty config auto-opens Settings
  - [ ] Banner shown on top with "前往配置" button
  - [ ] After config saved, banner dismissed
  - [ ] Subsequent launches do NOT auto-open Settings

  **QA Scenarios**:

  ```
  Scenario: Verify first-run auto-opens settings
    Tool: Playwright
    Preconditions: Fresh app with no saved config
    Steps:
      1. Launch app
      2. Assert Settings panel auto-opens
      3. Assert banner visible
    Expected Result: Settings panel open, banner shown
    Failure Indicators: Settings not open, no banner
    Evidence: .sisyphus/evidence/task-15-first-run.png
  ```

  **Evidence to Capture**:
  - [ ] Screenshot of first-run experience

  **Commit**: YES
  - Message: `feat(sysconfig): add first-run configuration guide`
  - Files: `frontend-workspace/apps/admin-tauri/src/App.vue`
  - Pre-commit: `npm run type-check`

- [ ] 16. Rust ↔ Go Merge Integration (End-to-End Pipeline)

  **What to do**:
  - Wire the complete merge pipeline across Rust → Go → Rust:
    1. Frontend calls Tauri command `prepare_merge(vault_path, victim_titles, survivor_title)`
    2. Rust `wikilink_replacer.rs` scans vault, loads files into memory HashMap, performs replacements
    3. Rust returns `MergePreview { files_modified: usize, wikilinks_replaced: usize }` to frontend
    4. Frontend calls Go `POST /cards/merge` with survivor_id + victim_ids
    5. Go executes atomic transaction, returns 200 OK or error
    6. If 200 OK: Frontend calls Tauri command `write_back_merged_files()`
    7. Rust writes modified content from memory HashMap to disk files
    8. On file write failure: Rust emits `merge_file_write_failed` event with file path + error
  - Add `write_back_merged_files` Tauri command that iterates the in-memory HashMap and writes each file

  **Must NOT do**:
  - Do NOT write files before Go transaction succeeds
  - Do NOT skip any step in the pipeline
  - Do NOT lose in-memory buffer if Go takes time to respond

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Cross-layer integration, state management between Rust/Go/Vue, error handling at boundaries
  - **Skills**: [`coding-standards`]
    - `coding-standards`: Integration patterns, error propagation

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on T4, T6, T8, T9)
  - **Parallel Group**: Wave 4 (sequential after all Wave 2 tasks)
  - **Blocks**: T18
  - **Blocked By**: T4 (hot reload), T6 (wikilink replacer), T8 (merge service), T9 (dedup)

  **References**:
  - `src-tauri/src/wikilink_replacer.rs` — Created in T6 (scan + replace in memory)
  - `src-tauri/src/lib.rs` — Tauri command registration
  - `go-server/internal/handlers/merge.go` — POST /cards/merge (T7)
  - `go-server/internal/services/merge.go` — MergeCards service (T8)
  - `frontend-workspace/apps/admin-tauri/src/components/MergePanel.vue` — Frontend integration (T14)

  **Acceptance Criteria**:
  - [ ] Complete pipeline: prepare → merge API → write-back
  - [ ] Files NOT modified if Go transaction fails
  - [ ] `merge_file_write_failed` event emitted on write failure
  - [ ] Memory HashMap cleared after write-back (success or failure)

  **QA Scenarios**:

  ```
  Scenario: Verify complete merge pipeline
    Tool: Bash (curl) + Playwright
    Preconditions: App running, test cards with edges and wikilinks
    Steps:
      1. Select Victim card (has [[Victim]] in another card's markdown)
      2. Select Survivor card
      3. Execute merge via UI
      4. Assert Go API returns 200
      5. Assert .md file updated: [[Victim]] → [[Survivor]]
      6. Assert edges migrated in DB
      7. Assert Victim card deleted from DB
    Expected Result: DB + files updated atomically
    Failure Indicators: Partial state, files not updated
    Evidence: .sisyphus/evidence/task-16-e2e-merge.txt

  Scenario: Verify pipeline stops on Go failure
    Tool: Bash (curl)
    Preconditions: Go server returns error (e.g., non-existent victim)
    Steps:
      1. Trigger merge with invalid victim ID
      2. Assert Go returns 4xx/5xx
      3. Assert NO .md files modified
      4. Assert NO edges changed in DB
    Expected Result: Zero side effects on failure
    Failure Indicators: Files modified despite DB failure
    Evidence: .sisyphus/evidence/task-16-pipeline-stop.txt
  ```

  **Evidence to Capture**:
  - [ ] E2E merge test output
  - [ ] Pipeline stop test output

  **Commit**: YES
  - Message: `feat(merger): wire end-to-end merge pipeline (Rust → Go → Rust)`
  - Files: `src-tauri/src/wikilink_replacer.rs`, `src-tauri/src/lib.rs`
  - Pre-commit: `cargo test && go test ./...`

- [ ] 17. Config Hot Reload End-to-End Test

  **What to do**:
  - Test the complete config change → hot reload flow:
    1. App running with localhost config
    2. Open Settings panel
    3. Change API URL to different value
    4. Run connectivity test (should fail for invalid URL)
    5. Revert to localhost
    6. Save + reload
    7. Verify WS reconnects with new URL
    8. Verify S3 client uses new credentials
  - Add integration test in `src-tauri/tests/` for `reload_sys_config` command

  **Must NOT do**:
  - Do NOT test against production endpoints (use localhost only)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Cross-layer testing, async coordination, state verification
  - **Skills**: [`coding-standards`]

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T16, T18)
  - **Parallel Group**: Wave 4
  - **Blocks**: None
  - **Blocked By**: T4 (reload command), T12 (Settings UI)

  **References**:
  - `src-tauri/src/config/mod.rs` — Config module (T2)
  - `src-tauri/src/config/keyring_wrapper.rs` — Keyring (T3)
  - `frontend-workspace/apps/admin-tauri/src/views/Settings.vue` — Settings UI (T12)

  **Acceptance Criteria**:
  - [ ] Config change triggers hot reload without restart
  - [ ] WS client reconnects to new URL
  - [ ] S3 client reinitializes with new credentials
  - [ ] Invalid config rejected with clear error

  **QA Scenarios**:

  ```
  Scenario: Verify hot reload without restart
    Tool: Playwright
    Preconditions: App running with localhost config
    Steps:
      1. Open Settings
      2. Change WS URL to "ws://localhost:9999/api/v1/ws"
      3. Test connection (expect fail — no server on 9999)
      4. Revert to "ws://localhost:8080/api/v1/ws"
      5. Test connection → pass
      6. Save
      7. Assert WS reconnects (check WS status indicator)
    Expected Result: WS reconnects without app restart
    Failure Indicators: App crashes, old URL still used
    Evidence: .sisyphus/evidence/task-17-hot-reload-e2e.png

  Scenario: Verify invalid config rejected
    Tool: Playwright
    Preconditions: Settings open
    Steps:
      1. Enter invalid S3 endpoint "not-a-url"
      2. Click TEST CONNECTION
      3. Assert red ✗ indicator
      4. Assert SAVE button remains disabled
    Expected Result: Test fails, save blocked
    Failure Indicators: Save allowed with invalid config
    Evidence: .sisyphus/evidence/task-17-invalid-config.png
  ```

  **Evidence to Capture**:
  - [ ] Screenshot of hot reload success
  - [ ] Screenshot of invalid config rejection

  **Commit**: YES
  - Message: `test(sysconfig): add hot reload end-to-end integration test`
  - Files: `src-tauri/tests/config_hot_reload.rs`
  - Pre-commit: `cargo test --test config_hot_reload`

- [ ] 18. Merge Flow End-to-End Test

  **What to do**:
  - Test complete merge flow from UI to database to filesystem:
    1. Create test cards: Survivor, Victim A, Victim B
    2. Create edges: Victim A → Card X, Card Y → Victim B
    3. Add `[[Victim A]]` and `[[Victim B]]` wikilinks in Card Z's markdown
    4. Execute merge: Victim A + Victim B → Survivor
    5. Verify: edges now Survivor → Card X, Card Y → Survivor
    6. Verify: Card Z markdown shows `[[Survivor]]` instead of victim names
    7. Verify: Victim A, B deleted from DB
    8. Verify: Ghost cards cleaned up (if any)
  - Test edge cases: merge with self-reference, merge with code blocks containing victim name
  - Test degradation: simulate file write failure, verify orange toast + retry

  **Must NOT do**:
  - Do NOT use production data
  - Do NOT skip cleanup after test

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Complex E2E scenario, multi-layer verification, edge case coverage
  - **Skills**: [`coding-standards`]

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T16, T17)
  - **Parallel Group**: Wave 4
  - **Blocks**: None
  - **Blocked By**: T6 (wikilink replacer), T13 (MergePanel), T14 (confirmation flow), T16 (integration)

  **References**:
  - All prior tasks — this is the final integration test

  **Acceptance Criteria**:
  - [ ] Complete merge flow works from UI to DB to filesystem
  - [ ] Edge cases handled: self-reference, code blocks, unicode titles
  - [ ] Degradation works: file write failure shows orange toast + retry
  - [ ] All evidence captured

  **QA Scenarios**:

  ```
  Scenario: Complete merge flow verification
    Tool: Playwright + Bash (curl + psql)
    Preconditions: Test data seeded, app running
    Steps:
      1. Open merge panel
      2. Select "Alpha" and "Beta" as Victims, "Gamma" as Survivor
      3. Calculate impact → verify counts
      4. Type CONFIRM → execute merge
      5. Assert green toast "合并完成"
      6. Query DB: SELECT * FROM cards WHERE title IN ('Alpha','Beta') → 0 rows
      7. Query DB: SELECT * FROM card_edges WHERE target_id IN (alpha_id, beta_id) → 0 rows
      8. Check .md file: grep "Alpha\|Beta" → only in non-wikilink context
      9. Assert graph updated (no Alpha/Beta nodes)
    Expected Result: Full pipeline success — DB + files + graph
    Failure Indicators: Any step fails
    Evidence: .sisyphus/evidence/task-18-merge-e2e.txt

  Scenario: Code block wikilinks preserved during merge
    Tool: Bash
    Preconditions: Card has wikilink inside ```code block```
    Steps:
      1. Merge victim node
      2. Read .md file
      3. Assert wikilink inside code block is UNCHANGED
    Expected Result: Code block content preserved
    Failure Indicators: Code block wikilink modified
    Evidence: .sisyphus/evidence/task-18-codeblock-preserve.txt
  ```

  **Evidence to Capture**:
  - [ ] E2E merge verification output
  - [ ] Code block preservation test output

  **Commit**: YES
  - Message: `test(merger): add merge flow end-to-end integration test`
  - Files: `frontend-workspace/apps/admin-tauri/tests/merge-e2e.ts`
  - Pre-commit: `npm test`

---

## Final Verification Wave

> 4 review agents run in PARALLEL. ALL must APPROVE.

- [ ] F1. **Plan Compliance Audit** — `oracle`
  Read the plan end-to-end. For each "Must Have": verify implementation exists. For each "Must NOT Have": search codebase for forbidden patterns. Check evidence files exist in .sisyphus/evidence/.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [ ] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo test` + `go test` + `npm test`. Review all changed files for: `as any`, empty catches, console.log in prod.
  Output: `Rust [PASS/FAIL] | Go [PASS/FAIL] | Vue [PASS/FAIL] | Files [N clean/N issues] | VERDICT`

- [ ] F3. **Real Manual QA** — `unspecified-high` + `playwright`
  Start from clean state. Execute EVERY QA scenario from EVERY task. Test cross-task integration. Save to `.sisyphus/evidence/final-qa/`.
  Output: `Scenarios [N/N pass] | Integration [N/N] | VERDICT`

- [ ] F4. **Scope Fidelity Check** — `deep`
  For each task: read "What to do", read actual diff. Verify 1:1. Check "Must NOT do" compliance.
  Output: `Tasks [N/N compliant] | Contamination [CLEAN/N issues] | VERDICT`

---

## Commit Strategy

- **Each task**: Single atomic commit
- **Format**: `feat(sysconfig): add keyring wrapper` or `feat(merger): add merge endpoint`
- **Pre-commit**: Run relevant tests

---

## Success Criteria

### Verification Commands
```bash
# Rust tests
cargo test --manifest-path rust-workspace/admin-tauri/Cargo.toml

# Go tests
cd go-server && go test ./internal/services/... ./internal/handlers/...

# Vue tests
cd frontend-workspace/apps/admin-tauri && npm test

# E2E: Save config
curl -X POST http://localhost:8080/api/v1/config -d '{"ws_url":"ws://test"}'
# Expected: 200 OK

# E2E: Merge nodes
curl -X POST http://localhost:8080/api/v1/cards/merge -d '{"survivor_id":"...","victim_ids":["..."]}'
# Expected: 200 OK, edges migrated
```

### Final Checklist
- [ ] All "Must Have" present
- [ ] All "Must NOT Have" absent
- [ ] All tests pass
- [ ] Evidence files exist
