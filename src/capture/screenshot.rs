use base64::Engine;
use image::codecs::jpeg::JpegEncoder;
use image::DynamicImage;

use crate::display::geometry::primary_display;
use crate::display::scaling::compute_target_dims;
use crate::error::ToolError;
use crate::types::ScreenshotResult;

/// JPEG quality (1-100 scale, matching image crate's encoder).
const JPEG_QUALITY: u8 = 75;

/// Capture the primary display and return a JPEG base64-encoded screenshot.
pub fn capture_screenshot() -> Result<ScreenshotResult, ToolError> {
    let display = primary_display()
        .map_err(|e| ToolError::ScreenshotFailed(format!("display info: {e}")))?;

    let monitors = xcap::Monitor::all()
        .map_err(|e| ToolError::ScreenshotFailed(format!("enumerate monitors: {e}")))?;

    let monitor = monitors
        .into_iter()
        .find(|m| m.id().ok() == Some(display.display_id))
        .ok_or_else(|| ToolError::ScreenshotFailed("primary monitor not found".into()))?;

    // Capture returns RgbaImage at physical pixel resolution
    let rgba = monitor
        .capture_image()
        .map_err(|e| ToolError::ScreenshotFailed(format!("capture: {e}")))?;

    let captured_width = rgba.width();
    let captured_height = rgba.height();

    // Compute target dimensions (scale down to fit API limits)
    let target = compute_target_dims(captured_width, captured_height);

    // Resize if needed
    let resized = if captured_width != target.width || captured_height != target.height {
        DynamicImage::ImageRgba8(rgba)
            .resize_exact(target.width, target.height, image::imageops::FilterType::Lanczos3)
    } else {
        DynamicImage::ImageRgba8(rgba)
    };

    // Encode to JPEG
    let mut jpeg_buf = Vec::new();
    let encoder = JpegEncoder::new_with_quality(&mut jpeg_buf, JPEG_QUALITY);
    resized
        .to_rgb8()
        .write_with_encoder(encoder)
        .map_err(|e| ToolError::ScreenshotFailed(format!("JPEG encode: {e}")))?;

    // Base64 encode
    let base64_image = base64::engine::general_purpose::STANDARD.encode(&jpeg_buf);

    Ok(ScreenshotResult {
        base64_image,
        width: target.width,
        height: target.height,
    })
}

/// Capture a region of the primary display at higher resolution.
pub fn capture_region(
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Result<ScreenshotResult, ToolError> {
    let display = primary_display()
        .map_err(|e| ToolError::ScreenshotFailed(format!("display info: {e}")))?;

    let monitors = xcap::Monitor::all()
        .map_err(|e| ToolError::ScreenshotFailed(format!("enumerate monitors: {e}")))?;

    let monitor = monitors
        .into_iter()
        .find(|m| m.id().ok() == Some(display.display_id))
        .ok_or_else(|| ToolError::ScreenshotFailed("primary monitor not found".into()))?;

    let rgba = monitor
        .capture_region(x, y, width, height)
        .map_err(|e| ToolError::ScreenshotFailed(format!("region capture: {e}")))?;

    let w = rgba.width();
    let h = rgba.height();

    // Encode to JPEG
    let mut jpeg_buf = Vec::new();
    let encoder = JpegEncoder::new_with_quality(&mut jpeg_buf, JPEG_QUALITY);
    DynamicImage::ImageRgba8(rgba)
        .to_rgb8()
        .write_with_encoder(encoder)
        .map_err(|e| ToolError::ScreenshotFailed(format!("JPEG encode: {e}")))?;

    let base64_image = base64::engine::general_purpose::STANDARD.encode(&jpeg_buf);

    Ok(ScreenshotResult {
        base64_image,
        width: w,
        height: h,
    })
}
