use enigo::{Button, Direction};
use tokio::time::{Duration, sleep};

use super::thread::InputHandle;
use crate::error::ToolError;

/// HID round-trip settle time after mouse move (matches reference: 50ms).
const MOVE_SETTLE_MS: u64 = 50;

/// Move mouse and wait for settle.
pub async fn move_and_settle(input: &InputHandle, x: i32, y: i32) -> Result<(), ToolError> {
    input.move_mouse(x, y).await?;
    sleep(Duration::from_millis(MOVE_SETTLE_MS)).await;
    Ok(())
}

/// Click at coordinates with optional button and count.
pub async fn click_at(
    input: &InputHandle,
    x: i32,
    y: i32,
    button: Button,
    count: u32,
) -> Result<(), ToolError> {
    move_and_settle(input, x, y).await?;

    // enigo doesn't have a multi-click API — send rapid sequential clicks.
    // macOS tracks click timing internally and computes clickCount.
    for _ in 0..count {
        input.click(button, Direction::Click).await?;
    }

    Ok(())
}

/// Press the left mouse button at current position.
pub async fn mouse_down(input: &InputHandle) -> Result<(), ToolError> {
    input.click(Button::Left, Direction::Press).await
}

/// Release the left mouse button at current position.
pub async fn mouse_up(input: &InputHandle) -> Result<(), ToolError> {
    input.click(Button::Left, Direction::Release).await
}

/// Get current cursor position.
pub async fn cursor_position(input: &InputHandle) -> Result<(i32, i32), ToolError> {
    input.cursor_position().await
}
