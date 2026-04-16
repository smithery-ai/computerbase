use enigo::Axis;

use super::mouse::move_and_settle;
use super::thread::InputHandle;
use crate::error::ToolError;
use crate::types::ScrollDirection;

/// Scroll at the given coordinates.
///
/// Moves mouse to position first, then scrolls.
/// Vertical-first: the common axis. A horizontal failure shouldn't lose the vertical.
pub async fn scroll_at(
    input: &InputHandle,
    x: i32,
    y: i32,
    direction: ScrollDirection,
    amount: u32,
) -> Result<(), ToolError> {
    move_and_settle(input, x, y).await?;

    let (scroll_amount, axis) = match direction {
        ScrollDirection::Up => (amount as i32, Axis::Vertical),
        ScrollDirection::Down => (-(amount as i32), Axis::Vertical),
        ScrollDirection::Left => (-(amount as i32), Axis::Horizontal),
        ScrollDirection::Right => (amount as i32, Axis::Horizontal),
    };

    input.scroll(scroll_amount, axis).await
}
