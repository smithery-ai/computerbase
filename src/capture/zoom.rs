use crate::display::geometry::primary_display;
use crate::display::scaling::compute_target_dims;
use crate::error::ToolError;
use crate::types::{RegionRect, ScreenshotResult, TargetDims};

/// Capture a sub-region of the screen at higher resolution.
///
/// The region is specified in screenshot pixel space (the coordinates the model sees).
/// We convert back to logical coordinates for xcap's capture_region.
pub fn capture_zoom(region: &RegionRect) -> Result<ScreenshotResult, ToolError> {
    let [x0, y0, x1, y1] = region.0;
    if x0 >= x1 || y0 >= y1 {
        return Err(ToolError::InvalidInput(format!(
            "invalid zoom region: [{x0}, {y0}, {x1}, {y1}] — x1 must be > x0 and y1 must be > y0"
        )));
    }

    let display =
        primary_display().map_err(|e| ToolError::ScreenshotFailed(format!("display info: {e}")))?;

    // The model's coordinates are in screenshot target space.
    // Convert back to logical display coordinates for xcap.
    let target = compute_target_dims(display.pixel_width, display.pixel_height);
    let scale_x = display.width as f64 / target.width as f64;
    let scale_y = display.height as f64 / target.height as f64;

    let logical_x = (x0 as f64 * scale_x).round() as u32;
    let logical_y = (y0 as f64 * scale_y).round() as u32;
    let logical_w = ((x1 - x0) as f64 * scale_x).round() as u32;
    let logical_h = ((y1 - y0) as f64 * scale_y).round() as u32;

    super::screenshot::capture_region(logical_x, logical_y, logical_w, logical_h)
}
