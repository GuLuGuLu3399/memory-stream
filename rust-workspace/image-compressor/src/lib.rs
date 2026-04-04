pub mod error;

use error::{CompressError, CompressResult};
use image::GenericImageView;
use tokio::task;
use webp::Encoder;

pub struct CompressOptions {
    pub quality: f32,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
}

impl Default for CompressOptions {
    fn default() -> Self {
        Self {
            quality: 75.0,
            max_width: None,
            max_height: None,
        }
    }
}

impl CompressOptions {
    pub fn new(quality: f32) -> Self {
        Self {
            quality,
            ..Default::default()
        }
    }

    pub fn with_max_size(mut self, max_width: u32, max_height: u32) -> Self {
        self.max_width = Some(max_width);
        self.max_height = Some(max_height);
        self
    }
}

pub async fn compress_to_webp(
    raw_data: Vec<u8>,
    options: CompressOptions,
) -> CompressResult<Vec<u8>> {
    task::spawn_blocking(move || compress_sync(&raw_data, &options))
        .await
        .unwrap_or_else(|_| Err(CompressError::TaskPanic))
}

fn compress_sync(raw_data: &[u8], options: &CompressOptions) -> CompressResult<Vec<u8>> {
    let img = image::load_from_memory(raw_data)?;

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
        return Err(CompressError::EncodeError(
            "WebP 编码器返回空数据".to_string(),
        ));
    }

    Ok(webp_data.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_png(width: u32, height: u32) -> Vec<u8> {
        let img = image::RgbaImage::from_pixel(width, height, image::Rgba([255, 0, 0, 255]));
        let mut buf = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buf, image::ImageFormat::Png).unwrap();
        buf.into_inner()
    }

    #[tokio::test]
    async fn test_compress_png_to_webp() {
        let png_data = create_test_png(100, 100);
        let result = compress_to_webp(png_data, CompressOptions::default()).await;
        assert!(result.is_ok());
        let webp_data = result.unwrap();
        assert!(!webp_data.is_empty());
    }

    #[tokio::test]
    async fn test_compress_with_resize() {
        let png_data = create_test_png(2000, 2000);
        let options = CompressOptions::new(75.0).with_max_size(800, 800);
        let result = compress_to_webp(png_data, options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_compress_no_resize_when_small() {
        let png_data = create_test_png(50, 50);
        let options = CompressOptions::new(75.0).with_max_size(800, 800);
        let result = compress_to_webp(png_data, options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_compress_invalid_data() {
        let bad_data = vec![0u8; 16];
        let result = compress_to_webp(bad_data, CompressOptions::default()).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CompressError::DecodeError(_) => {}
            other => panic!("Expected DecodeError, got: {}", other),
        }
    }

    #[tokio::test]
    async fn test_compress_empty_data() {
        let result = compress_to_webp(vec![], CompressOptions::default()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_quality_affects_size() {
        let mut img = image::RgbaImage::new(500, 500);
        for y in 0..500 {
            for x in 0..500 {
                let r = ((x * 3 + y * 7) % 256) as u8;
                let g = ((x * 13 + y * 11) % 256) as u8;
                let b = ((x * 17 + y * 23) % 256) as u8;
                img.put_pixel(x, y, image::Rgba([r, g, b, 255]));
            }
        }
        let mut buf = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buf, image::ImageFormat::Png).unwrap();
        let png_data = buf.into_inner();

        let low = compress_to_webp(png_data.clone(), CompressOptions::new(5.0))
            .await
            .unwrap();
        let high = compress_to_webp(png_data, CompressOptions::new(95.0))
            .await
            .unwrap();
        assert!(low.len() < high.len(), "low={} high={}", low.len(), high.len());
    }
}
