use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct SyncProgressEvent {
    pub status: String,
    pub progress: u32,
}

#[derive(Serialize, Clone)]
pub struct FileChangeEvent {
    pub path: String,
    pub kind: String,
}
