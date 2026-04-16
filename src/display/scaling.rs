use crate::types::{DisplayGeometry, LogicalCoord, ScreenCoord, TargetDims};

/// Maximum screenshot dimension sent to the model.
/// Matches the reference implementation's API resize params.
const MAX_LONG_SIDE: u32 = 1280;
const MAX_SHORT_SIDE: u32 = 768;

/// Compute the target dimensions for a screenshot image.
/// Takes physical pixel dimensions and scales down to fit within API limits.
pub fn compute_target_dims(pixel_width: u32, pixel_height: u32) -> TargetDims {
    let (long, short) = if pixel_width >= pixel_height {
        (pixel_width, pixel_height)
    } else {
        (pixel_height, pixel_width)
    };

    let scale = f64::min(
        MAX_LONG_SIDE as f64 / long as f64,
        MAX_SHORT_SIDE as f64 / short as f64,
    )
    .min(1.0); // never upscale

    let target_w = (pixel_width as f64 * scale).round() as u32;
    let target_h = (pixel_height as f64 * scale).round() as u32;

    TargetDims {
        width: target_w.max(1),
        height: target_h.max(1),
    }
}

/// Convert a coordinate from screenshot image space to macOS logical points.
///
/// The model sees a resized screenshot (target_dims). It sends coordinates in
/// that pixel space. We need to map back to logical points for enigo/CGEvent.
///
/// Flow: screen_coord → scale up to physical → scale down to logical
/// Simplified: screen_coord * (logical_dim / target_dim)
pub fn screen_to_logical(
    coord: ScreenCoord,
    display: &DisplayGeometry,
    target: &TargetDims,
) -> LogicalCoord {
    let scale_x = display.width as f64 / target.width as f64;
    let scale_y = display.height as f64 / target.height as f64;

    LogicalCoord {
        x: coord.x * scale_x + display.origin_x as f64,
        y: coord.y * scale_y + display.origin_y as f64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_target_dims_retina() {
        // 2880x1800 Retina display (physical pixels)
        let dims = compute_target_dims(2880, 1800);
        // Should scale to fit within 1280x768
        assert!(dims.width <= MAX_LONG_SIDE);
        assert!(dims.height <= MAX_SHORT_SIDE);
        // Aspect ratio preserved
        let original_aspect = 2880.0 / 1800.0;
        let target_aspect = dims.width as f64 / dims.height as f64;
        assert!((original_aspect - target_aspect).abs() < 0.02);
    }

    #[test]
    fn test_compute_target_dims_small() {
        // Already smaller than limits — no scaling
        let dims = compute_target_dims(800, 600);
        assert_eq!(dims.width, 800);
        assert_eq!(dims.height, 600);
    }

    #[test]
    fn test_screen_to_logical_retina() {
        let display = DisplayGeometry {
            display_id: 1,
            width: 1440,
            height: 900,
            pixel_width: 2880,
            pixel_height: 1800,
            scale_factor: 2.0,
            origin_x: 0,
            origin_y: 0,
        };
        // Target dims after resizing 2880x1800
        let target = compute_target_dims(2880, 1800);

        // Center of the screenshot image
        let screen = ScreenCoord {
            x: target.width as f64 / 2.0,
            y: target.height as f64 / 2.0,
        };
        let logical = screen_to_logical(screen, &display, &target);

        // Should map to center of logical display
        assert!((logical.x - 720.0).abs() < 1.0);
        assert!((logical.y - 450.0).abs() < 1.0);
    }
}
