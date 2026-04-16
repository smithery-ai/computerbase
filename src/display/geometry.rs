use crate::error::SystemError;
use crate::types::DisplayGeometry;

/// Get the primary display geometry.
pub fn primary_display() -> Result<DisplayGeometry, SystemError> {
    let monitors = xcap::Monitor::all()
        .map_err(|e| SystemError::CoreGraphics(format!("failed to enumerate monitors: {e}")))?;

    let primary = monitors
        .into_iter()
        .find(|m| m.is_primary().unwrap_or(false))
        .ok_or_else(|| SystemError::CoreGraphics("no primary display found".into()))?;

    monitor_to_geometry(&primary)
}

/// List all connected displays.
pub fn all_displays() -> Result<Vec<DisplayGeometry>, SystemError> {
    let monitors = xcap::Monitor::all()
        .map_err(|e| SystemError::CoreGraphics(format!("failed to enumerate monitors: {e}")))?;

    monitors.iter().map(monitor_to_geometry).collect()
}

fn monitor_to_geometry(m: &xcap::Monitor) -> Result<DisplayGeometry, SystemError> {
    let map_err = |e: xcap::XCapError| SystemError::CoreGraphics(e.to_string());

    let width = m.width().map_err(map_err)?;
    let height = m.height().map_err(map_err)?;
    let scale = m.scale_factor().map_err(map_err)?;

    Ok(DisplayGeometry {
        display_id: m.id().map_err(map_err)?,
        width,
        height,
        pixel_width: (width as f64 * scale as f64).round() as u32,
        pixel_height: (height as f64 * scale as f64).round() as u32,
        scale_factor: scale as f64,
        origin_x: m.x().map_err(map_err)?,
        origin_y: m.y().map_err(map_err)?,
    })
}
