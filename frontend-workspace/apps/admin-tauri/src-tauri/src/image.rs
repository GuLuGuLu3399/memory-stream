//! # 图片压缩 + S3 上传管道
//!
//! 将现有的 `image-compressor`（WebP 压缩）与 `ms-storage`（S3 上传）串联，
//! 通过 Tauri IPC 暴露给前端，实现一键"压缩 → 上传 → 获取 CDN URL"。

use base64::{engine::general_purpose::STANDARD, Engine as _};
use ms_storage::{StorageConfig, S3Backend, StorageProvider};
use serde::Serialize;
use std::io::Cursor;
use tauri::Manager;
use ts_rs::TS;

/// 最大允许的图片文件大小（20 MB）
const MAX_IMAGE_SIZE: u64 = 20 * 1024 * 1024;

/// S3 上传超时时间（30 秒）
const S3_UPLOAD_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

/// 上传结果，返回给前端
#[derive(Serialize, TS)]
#[ts(export_to = ".")]
pub struct ImageUploadResult {
    /// S3 对象 key（相对路径）
    pub key: String,
    /// CDN 公开访问 URL
    pub url: String,
    /// 压缩后的文件大小（字节）
    #[ts(type = "number")]
    pub size_bytes: u64,
    /// 原始文件大小（字节）
    #[ts(type = "number")]
    pub original_size_bytes: u64,
}

/// 从环境变量加载 S3 配置
///
/// 需要以下环境变量：
/// - `S3_ENDPOINT` — S3 兼容端点（如 MinIO: `http://localhost:9000`）
/// - `S3_REGION` — 区域（默认 `us-east-1`）
/// - `S3_BUCKET` — 存储桶名称
/// - `S3_ACCESS_KEY` — Access Key
/// - `S3_SECRET_KEY` — Secret Key
/// - `S3_PUBLIC_URL_BASE`（可选）— CDN 基础 URL
fn load_storage_config() -> Result<StorageConfig, String> {
    // 从 .env 文件加载环境变量（开发环境友好，生产环境由系统 env 提供）
    // Tauri dev 的 CWD 可能在 src-tauri/ 或项目根，所以两个位置都尝试
    for dir in &[".", ".."] {
        for name in &[".env", ".env.development", ".env.local"] {
            let path = std::path::Path::new(dir).join(name);
            let _ = dotenv::from_path(&path);
        }
    }

    Ok(StorageConfig {
        endpoint: std::env::var("S3_ENDPOINT")
            .map_err(|_| "S3_ENDPOINT not set (check .env file or system env)".to_string())?,
        region: std::env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
        bucket: std::env::var("S3_BUCKET")
            .map_err(|_| "S3_BUCKET environment variable not set".to_string())?,
        access_key: std::env::var("S3_ACCESS_KEY")
            .map_err(|_| "S3_ACCESS_KEY environment variable not set".to_string())?,
        secret_key: std::env::var("S3_SECRET_KEY")
            .map_err(|_| "S3_SECRET_KEY environment variable not set".to_string())?,
        public_url_base: std::env::var("S3_PUBLIC_URL_BASE").ok(),
        use_path_style: std::env::var("S3_USE_PATH_STYLE")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false),
    })
}

/// 从全局可重载状态获取 S3 配置，如未加载则回退到环境变量
pub fn get_storage_config_from_state(
    app: &tauri::AppHandle,
) -> Result<StorageConfig, String> {
    let state = app.state::<crate::ReloadableState>();
    let guard = state.s3_config.lock().map_err(|e| e.to_string())?;
    match guard.as_ref() {
        Some(config) => Ok(config.clone()),
        None => load_storage_config(),
    }
}

/// 生成唯一 S3 key：`images/{year}/{month}/{uuid}.webp`
fn generate_key(original_path: &str) -> String {
    let now = chrono_available_or_fallback();
    let ext = std::path::Path::new(original_path)
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_else(|| "bin".to_string());
    let uuid = uuid_fallback();
    format!("images/{}/{}/{}.{}.webp", now.year, now.month, uuid, ext)
}

/// 时间辅助（不引入 chrono 依赖，用 SystemTime）
struct TimeInfo {
    year: i32,
    month: u32,
}

fn chrono_available_or_fallback() -> TimeInfo {
    // 使用系统时间推算 year/month
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    // 简化计算：从 1970 年起算
    let days = secs / 86400;
    let (year, month) = estimate_year_month(days);
    TimeInfo { year, month }
}

/// 从 Unix 天数估算年份和月份（足够用于路径生成）
fn estimate_year_month(days_since_epoch: u64) -> (i32, u32) {
    let mut year = 1970i32;
    let mut remaining = days_since_epoch;
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        year += 1;
    }
    let leap = is_leap_year(year);
    let month_days: [u64; 12] = if leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month = 1u32;
    for &days in &month_days {
        if remaining < days {
            break;
        }
        remaining -= days;
        month += 1;
    }
    (year, month)
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// 简易 UUID（不引入 uuid crate，用随机数模拟）
fn uuid_fallback() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    // 用纳秒 + 线程 ID 生成伪唯一字符串
    let nanos = t.subsec_nanos();
    let secs = t.as_secs();
    format!("{:08x}{:04x}{:04x}{:08x}", secs & 0xFFFFFFFF, (nanos >> 16) & 0xFFFF, nanos & 0xFFFF, std::process::id())
}

/// 压缩图片并上传到 S3
///
/// 完整管道：
/// 1. 读取源图片文件
/// 2. 压缩为 WebP（内存中操作，不写临时文件）
/// 3. 上传到 S3
/// 4. 返回 CDN URL
///
/// # 参数
/// - `file_path` — 源图片的本地文件路径
///
/// # 返回
/// `ImageUploadResult` 包含 CDN URL、key 和文件大小信息
#[tauri::command]
pub async fn compress_and_upload_image(
    app: tauri::AppHandle,
    file_path: String,
) -> Result<ImageUploadResult, String> {
    let original_size = std::fs::metadata(&file_path)
        .map_err(|e| format!("无法读取文件信息: {}", e))?
        .len();

    if original_size > MAX_IMAGE_SIZE {
        return Err(format!(
            "图片文件过大: {} MB（上限 {} MB）",
            original_size / (1024 * 1024),
            MAX_IMAGE_SIZE / (1024 * 1024)
        ));
    }

    // 1. 读取并压缩图片为 WebP
    let img = ::image::open(&file_path)
        .map_err(|e| format!("打开图片失败: {}", e))?;

    let mut webp_buf = Cursor::new(Vec::new());
    let encoder = ::image::codecs::webp::WebPEncoder::new_lossless(&mut webp_buf);
    img.write_with_encoder(encoder)
        .map_err(|e| format!("WebP 编码失败: {}", e))?;

    let compressed_data = webp_buf.into_inner();
    let compressed_size = compressed_data.len() as u64;

    // 2. 加载 S3 配置并上传
    let config = get_storage_config_from_state(&app)?;
    let backend = S3Backend::new(&config)
        .map_err(|e| format!("S3 初始化失败: {}", e))?;

    let key = generate_key(&file_path);
    let url = tokio::time::timeout(
        S3_UPLOAD_TIMEOUT,
        backend.upload(&key, &compressed_data, "image/webp"),
    )
    .await
    .map_err(|_| "S3 上传超时（30s）".to_string())?
    .map_err(|e| format!("S3 上传失败: {}", e))?;

    Ok(ImageUploadResult {
        key,
        url,
        size_bytes: compressed_size,
        original_size_bytes: original_size,
    })
}

/// 从剪贴板 Base64 数据压缩图片并上传到 S3
///
/// 完整管道（内存直传，无磁盘 I/O）：
/// 1. Base64 解码为原始字节
/// 2. 从内存加载图片
/// 3. 压缩为 WebP（内存中操作）
/// 4. 上传到 S3
/// 5. 返回 CDN URL
///
/// # 参数
/// - `base64_data` — 纯 Base64 字符串（不含 `data:image/...;base64,` 前缀）
#[tauri::command]
pub async fn upload_clipboard_image(
    app: tauri::AppHandle,
    base64_data: String,
) -> Result<ImageUploadResult, String> {
    // 1. Base64 解码
    let image_bytes = STANDARD
        .decode(&base64_data)
        .map_err(|e| format!("Base64 解码失败: {}", e))?;
    let original_size = image_bytes.len() as u64;

    if original_size > MAX_IMAGE_SIZE {
        return Err(format!(
            "剪贴板图片过大: {} MB（上限 {} MB）",
            original_size / (1024 * 1024),
            MAX_IMAGE_SIZE / (1024 * 1024)
        ));
    }

    // 2. 从内存加载图片
    let img = ::image::load_from_memory(&image_bytes)
        .map_err(|e| format!("图像解析失败: {}", e))?;

    // 3. 在内存中压缩为 WebP
    let mut webp_buf = Cursor::new(Vec::new());
    let encoder = ::image::codecs::webp::WebPEncoder::new_lossless(&mut webp_buf);
    img.write_with_encoder(encoder)
        .map_err(|e| format!("WebP 编码失败: {}", e))?;

    let compressed_data = webp_buf.into_inner();
    let compressed_size = compressed_data.len() as u64;

    // 4. 加载 S3 配置并上传
    let config = get_storage_config_from_state(&app)?;
    let backend = S3Backend::new(&config)
        .map_err(|e| format!("S3 初始化失败: {}", e))?;

    let key = generate_key("clipboard.png");
    let url = tokio::time::timeout(
        S3_UPLOAD_TIMEOUT,
        backend.upload(&key, &compressed_data, "image/webp"),
    )
    .await
    .map_err(|_| "S3 上传超时（30s）".to_string())?
    .map_err(|e| format!("S3 上传失败: {}", e))?;

    Ok(ImageUploadResult {
        key,
        url,
        size_bytes: compressed_size,
        original_size_bytes: original_size,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2000));
        assert!(!is_leap_year(1900));
        assert!(is_leap_year(2004));
        assert!(!is_leap_year(2100));
    }

    #[test]
    fn test_estimate_year_month() {
        assert_eq!(estimate_year_month(0), (1970, 1));
        assert_eq!(estimate_year_month(364), (1970, 12));
        assert_eq!(estimate_year_month(365), (1971, 1));
        assert_eq!(estimate_year_month(730), (1972, 1));
    }

    #[test]
    fn test_generate_key_with_png() {
        let key = generate_key("photo.png");
        assert!(key.starts_with("images/"));
        assert!(key.ends_with(".png.webp"));
    }

    #[test]
    fn test_generate_key_no_extension() {
        let key = generate_key("photo");
        assert!(key.ends_with(".bin.webp"));
    }
}
