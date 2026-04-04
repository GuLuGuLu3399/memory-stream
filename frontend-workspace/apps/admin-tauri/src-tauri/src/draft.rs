//! # 本地草稿管理命令
//!
//! 对接 `ms-local-draft` Crate，提供离线草稿的增删查功能。
//! 草稿数据存储在独立的 SQLite 数据库中，与图谱缓存物理隔离。

use ms_local_draft::DraftDb;
use serde::Serialize;
use std::sync::Arc;
use tauri::State;

/// 草稿管理器，封装在 `Arc` 中以支持跨线程共享
pub struct DraftManager {
    db: Arc<DraftDb>,
}

impl DraftManager {
    /// 初始化草稿数据库
    ///
    /// 在 Tauri `setup()` 中调用，使用 AppData 目录下的独立 DB 文件。
    pub async fn new(db_path: &std::path::Path) -> Result<Self, String> {
        let db = DraftDb::new(db_path)
            .await
            .map_err(|e| format!("Draft DB 初始化失败: {:?}", e))?;
        Ok(Self {
            db: Arc::new(db),
        })
    }

    /// 获取内部 DB 的 Arc 引用（用于 Tauri State 注入）
    pub fn into_inner(self) -> Arc<DraftDb> {
        self.db
    }
}

/// 草稿数据传输对象（返回给前端）
#[derive(Serialize)]
pub struct DraftDto {
    pub card_id: String,
    pub raw_md: String,
    pub ast_data: Option<String>,
    pub updated_at: i64,
}

impl From<ms_local_draft::Draft> for DraftDto {
    fn from(d: ms_local_draft::Draft) -> Self {
        Self {
            card_id: d.card_id,
            raw_md: d.raw_md,
            ast_data: d.ast_data,
            updated_at: d.updated_at,
        }
    }
}

/// 保存草稿到本地 SQLite
///
/// 支持离线编辑，用户关闭应用后草稿依然保留。
/// 下次打开时可通过 `load_draft` 恢复。
#[tauri::command]
pub async fn save_draft(
    db: State<'_, Arc<DraftDb>>,
    card_id: String,
    raw_md: String,
    ast_data: Option<String>,
) -> Result<(), String> {
    db.save_draft(&card_id, &raw_md, ast_data.as_deref())
        .await
        .map_err(|e| format!("保存草稿失败: {:?}", e))
}

/// 加载指定卡片的本地草稿
///
/// 返回 `null`（前端 `null`）如果该卡片没有本地草稿。
#[tauri::command]
pub async fn load_draft(
    db: State<'_, Arc<DraftDb>>,
    card_id: String,
) -> Result<Option<DraftDto>, String> {
    let draft = db
        .load_draft(&card_id)
        .await
        .map_err(|e| format!("加载草稿失败: {:?}", e))?;
    Ok(draft.map(DraftDto::from))
}

/// 列出所有本地草稿
///
/// 用于在应用启动时恢复未同步的编辑内容。
#[tauri::command]
pub async fn list_drafts(
    db: State<'_, Arc<DraftDb>>,
) -> Result<Vec<DraftDto>, String> {
    let drafts = db
        .list_all()
        .await
        .map_err(|e| format!("列出草稿失败: {:?}", e))?;
    Ok(drafts.into_iter().map(DraftDto::from).collect())
}

/// 删除指定卡片的本地草稿
///
/// 通常在成功同步到服务端后调用。
#[tauri::command]
pub async fn delete_draft(
    db: State<'_, Arc<DraftDb>>,
    card_id: String,
) -> Result<(), String> {
    db.delete_draft(&card_id)
        .await
        .map_err(|e| format!("删除草稿失败: {:?}", e))
}