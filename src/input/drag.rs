use enigo::{Button, Direction};
use tokio::time::{Duration, sleep};

use super::animation::animated_move;
use super::mouse::move_and_settle;
use super::thread::InputHandle;
use crate::error::ToolError;

/// HID settle time after pressing mouse button (for pressedMouseButtons to register).
const PRESS_SETTLE_MS: u64 = 50;

/// Drag from a start point (or current cursor) to a target point.
///
/// Always releases the mouse button in the finally block, even if the move fails,
/// to prevent the user's left button from getting stuck pressed.
pub async fn drag(
    input: &InputHandle,
    from: Option<(i32, i32)>,
    to: (i32, i32),
) -> Result<(), ToolError> {
    // Move to start point if specified
    let start = if let Some((fx, fy)) = from {
        move_and_settle(input, fx, fy).await?;
        (fx, fy)
    } else {
        input.cursor_position().await?
    };

    // Press left button
    input.click(Button::Left, Direction::Press).await?;
    sleep(Duration::from_millis(PRESS_SETTLE_MS)).await;

    // Animate to target — always release button even on error
    let move_result = animated_move(input, start.0, start.1, to.0, to.1).await;

    // ALWAYS release
    let release_result = input.click(Button::Left, Direction::Release).await;

    // Return the first error
    move_result?;
    release_result?;

    Ok(())
}
