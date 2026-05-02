//! 媒体瘦身 — WebP 压缩管线

use image::GenericImageView;
use tokio::task;
use webp::Encoder;

use crate::error::{IoError, IoResult};

/// 压缩选项
pub struct CompressOptions {
    pub quality: f32,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub max_input_bytes: usize,
}

impl Default for CompressOptions {
    fn default() -> Self {
        Self {
            quality: 75.0,
            max_width: None,
            max_height: None,
            max_input_bytes: 20 * 1024 * 1024,
        }
    }
}

impl CompressOptions {
    #[must_use]
    pub fn new(quality: f32) -> Self {
        Self {
            quality,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn with_max_size(mut self, max_width: u32, max_height: u32) -> Self {
        self.max_width = Some(max_width);
        self.max_height = Some(max_height);
        self
    }
}

/// 异步压缩图片为 WebP 格式（在 blocking 线程池执行）。
pub async fn compress_to_webp(raw_data: Vec<u8>, options: CompressOptions) -> IoResult<Vec<u8>> {
    if raw_data.len() > options.max_input_bytes {
        return Err(IoError::PayloadTooLarge(format!(
            "Image size {} bytes exceeds max {} bytes",
            raw_data.len(),
            options.max_input_bytes
        )));
    }

    task::spawn_blocking(move || {
        let img = image::load_from_memory(&raw_data)
            .map_err(|e| IoError::ImageDecodeFailed(e.to_string()))?;

        let final_img = if options.max_width.is_some() || options.max_height.is_some() {
            let (w, h) = img.dimensions();
            let max_w = options.max_width.unwrap_or(w);
            let max_h = options.max_height.unwrap_or(h);
            if w > max_w || h > max_h {
                image::DynamicImage::ImageRgba8(img.thumbnail(max_w, max_h).to_rgba8())
            } else {
                img
            }
        } else {
            img
        };

        let rgba = final_img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let encoder = Encoder::from_rgba(&rgba, width, height);
        let webp_data = encoder.encode(options.quality);

        if webp_data.is_empty() {
            return Err(IoError::ImageEncodeFailed(
                "WebP encoder returned empty data".into(),
            ));
        }

        Ok(webp_data.to_vec())
    })
    .await
    .map_err(extract_panic)?
}

fn extract_panic(join_err: tokio::task::JoinError) -> IoError {
    let reason = if join_err.is_panic() {
        let panic_err = join_err.into_panic();
        if let Some(s) = panic_err.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_err.downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic payload".to_string()
        }
    } else {
        "Task cancelled".to_string()
    };
    IoError::TaskPanic { reason }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_png(width: u32, height: u32) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let img = image::RgbaImage::from_pixel(width, height, image::Rgba([255, 0, 0, 255]));
        let mut buf = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buf, image::ImageFormat::Png)?;
        Ok(buf.into_inner())
    }

    #[tokio::test]
    async fn test_compress_png_to_webp() -> IoResult<()> {
        let png_data = create_test_png(100, 100).unwrap();
        let result = compress_to_webp(png_data, CompressOptions::default()).await?;
        assert!(!result.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_compress_with_resize() -> IoResult<()> {
        let png_data = create_test_png(2000, 2000).unwrap();
        let options = CompressOptions::new(75.0).with_max_size(800, 800);
        let result = compress_to_webp(png_data, options).await?;
        assert!(!result.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_compress_invalid_data() {
        let bad_data = vec![0u8; 16];
        let result = compress_to_webp(bad_data, CompressOptions::default()).await;
        assert!(matches!(result, Err(IoError::ImageDecodeFailed(_))));
    }

    #[tokio::test]
    async fn test_compress_empty_data() {
        let result = compress_to_webp(vec![], CompressOptions::default()).await;
        assert!(result.is_err());
    }
}
