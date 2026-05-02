use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};

#[derive(Debug, Clone, PartialEq)]
pub enum SyncStatus {
    Synced,
    PendingPush,
    PendingDelete,
    Conflict,
}

impl SyncStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Synced => "synced",
            Self::PendingPush => "pending_push",
            Self::PendingDelete => "pending_delete",
            Self::Conflict => "conflict",
        }
    }
}

impl ToSql for SyncStatus {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        self.as_str().to_sql()
    }
}

impl FromSql for SyncStatus {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s = String::column_result(value)?;
        match s.as_str() {
            "synced" => Ok(Self::Synced),
            "pending_push" => Ok(Self::PendingPush),
            "pending_delete" => Ok(Self::PendingDelete),
            "conflict" => Ok(Self::Conflict),
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

pub struct CardIndex {
    pub uuid: String,
    pub file_path: String,
    pub file_hash: String,
    pub version: i64,
    pub sync_status: SyncStatus,
    pub last_synced_hash: Option<String>,
}

pub struct RelationRecord {
    pub source_uuid: String,
    pub target_uuid_or_tag: String,
    pub relation_type: String, // "trunk" | "link"
}

pub struct AssetRef {
    pub local_path: String,
    pub cloud_url: Option<String>,
    pub ref_count: i64,
}

pub struct FtsHit {
    pub uuid: String,
    pub title: String,
    pub excerpt: String,
    pub rank: f64,
}
