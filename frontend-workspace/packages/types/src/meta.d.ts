export type SyncStatus = "synced" | "pending_push" | "pending_delete" | "conflict";
export interface CardIndex {
    uuid: string;
    file_path: string;
    file_hash: string;
    version: number;
    sync_status: SyncStatus;
    last_synced_hash: string | null;
}
export interface RelationRecord {
    source_uuid: string;
    target_uuid_or_tag: string;
    relation_type: "trunk" | "link" | "tag";
}
export interface AssetRef {
    local_path: string;
    cloud_url: string | null;
    ref_count: number;
}
export interface FtsHit {
    uuid: string;
    title: string;
    excerpt: string;
    rank: number;
}
export interface CardMeta {
    uuid: string;
    title: string;
    category: string;
    created_at: string;
    updated_at: string;
}
//# sourceMappingURL=meta.d.ts.map