use std::path::Path;

use image::imageops::FilterType;
use image::GenericImageView;

use crate::error::AppError;

const MAX_WIDTH: u32 = 200;
const MAX_HEIGHT: u32 = 280;
const WEBP_QUALITY: f32 = 65.0;

pub fn generate_thumbnail(image_path: &Path) -> Result<Vec<u8>, AppError> {
    let img = image::open(image_path)?;
    let (orig_w, orig_h) = img.dimensions();

    let scale = (MAX_WIDTH as f64 / orig_w as f64).min(MAX_HEIGHT as f64 / orig_h as f64);
    let resized = if scale < 1.0 {
        let new_w = (orig_w as f64 * scale).round() as u32;
        let new_h = (orig_h as f64 * scale).round() as u32;
        img.resize_exact(new_w, new_h, FilterType::Triangle)
    } else {
        img.clone()
    };

    let rgba = resized.to_rgba8();
    let (w, h) = rgba.dimensions();
    let encoder = webp::Encoder::from_rgba(&rgba, w, h);
    let mem = encoder.encode(WEBP_QUALITY);

    if mem.is_empty() {
        return Err(AppError::WebpEncode);
    }

    Ok(mem.to_vec())
}
