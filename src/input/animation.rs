use tokio::time::{Duration, sleep};

use super::thread::InputHandle;
use crate::error::ToolError;

/// Speed in pixels per second for animated mouse moves.
const MOVE_SPEED_PX_PER_SEC: f64 = 2000.0;

/// Maximum animation duration in seconds.
const MAX_DURATION_SEC: f64 = 0.5;

/// Target framerate for animation.
const FPS: f64 = 60.0;

/// HID settle time after final frame.
const MOVE_SETTLE_MS: u64 = 50;

/// Ease-out-cubic: `1 - (1 - t)^3`
fn ease_out_cubic(t: f64) -> f64 {
    let inv = 1.0 - t;
    1.0 - inv * inv * inv
}

/// Animate mouse movement from current position to target using ease-out-cubic.
///
/// Falls back to instant move if distance is too small for meaningful animation.
pub async fn animated_move(
    input: &InputHandle,
    start_x: i32,
    start_y: i32,
    target_x: i32,
    target_y: i32,
) -> Result<(), ToolError> {
    let dx = (target_x - start_x) as f64;
    let dy = (target_y - start_y) as f64;
    let distance = (dx * dx + dy * dy).sqrt();

    let duration_sec = (distance / MOVE_SPEED_PX_PER_SEC).min(MAX_DURATION_SEC);

    // If distance is tiny, skip animation
    let min_frames = 2.0;
    if duration_sec < min_frames / FPS {
        input.move_mouse(target_x, target_y).await?;
        sleep(Duration::from_millis(MOVE_SETTLE_MS)).await;
        return Ok(());
    }

    let total_frames = (duration_sec * FPS).floor() as u32;
    let frame_interval = Duration::from_secs_f64(1.0 / FPS);

    for frame in 1..=total_frames {
        let t = frame as f64 / total_frames as f64;
        let eased = ease_out_cubic(t);

        let x = start_x as f64 + dx * eased;
        let y = start_y as f64 + dy * eased;

        input.move_mouse(x.round() as i32, y.round() as i32).await?;

        if frame < total_frames {
            sleep(frame_interval).await;
        }
    }

    sleep(Duration::from_millis(MOVE_SETTLE_MS)).await;
    Ok(())
}
