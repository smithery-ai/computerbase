use enigo::Button;

use crate::capture::screenshot::capture_screenshot;
use crate::display::scaling::screen_to_logical;
use crate::error::ToolError;
use crate::input::drag::drag;
use crate::input::keyboard::{hold_key, press_key_combo, type_text};
use crate::input::mouse::{click_at, cursor_position, mouse_down, mouse_up, move_and_settle};
use crate::input::scroll::scroll_at;
use crate::input::thread::InputHandle;
use crate::types::{BatchAction, CoordPair, DisplayGeometry, LogicalCoord, TargetDims};

/// Convert a coordinate pair using display info.
fn convert_coord(coord: CoordPair, display: &DisplayGeometry, target: &TargetDims) -> LogicalCoord {
    screen_to_logical(coord.into(), display, target)
}

/// Execute a batch of actions sequentially. Stops on the first error.
/// Returns a description of the last successful action.
pub async fn execute_batch(
    actions: Vec<BatchAction>,
    input: &InputHandle,
    display: &DisplayGeometry,
    target: &TargetDims,
) -> Result<String, ToolError> {
    let mut last_result = "no actions executed".to_string();

    for (i, action) in actions.into_iter().enumerate() {
        let result = execute_single(action, input, display, target)
            .await
            .map_err(|e| ToolError::BatchActionFailed {
                index: i,
                source: Box::new(e),
            })?;
        last_result = result;
    }

    Ok(last_result)
}

async fn execute_single(
    action: BatchAction,
    input: &InputHandle,
    display: &DisplayGeometry,
    target: &TargetDims,
) -> Result<String, ToolError> {
    match action {
        BatchAction::Screenshot {} => {
            let result = tokio::task::spawn_blocking(capture_screenshot)
                .await
                .map_err(|e| ToolError::ScreenshotFailed(e.to_string()))??;
            Ok(format!(
                "screenshot captured ({}x{})",
                result.width, result.height
            ))
        }
        BatchAction::LeftClick { coordinate, text } => {
            let lc = convert_coord(coordinate, display, target);
            let (x, y) = (lc.x.round() as i32, lc.y.round() as i32);
            if let Some(mods) = text {
                crate::input::modifiers::with_modifiers(input, Some(&mods), || async {
                    click_at(input, x, y, Button::Left, 1).await
                })
                .await?;
            } else {
                click_at(input, x, y, Button::Left, 1).await?;
            }
            Ok(format!("clicked ({x}, {y})"))
        }
        BatchAction::RightClick { coordinate, text } => {
            let lc = convert_coord(coordinate, display, target);
            let (x, y) = (lc.x.round() as i32, lc.y.round() as i32);
            if let Some(mods) = text {
                crate::input::modifiers::with_modifiers(input, Some(&mods), || async {
                    click_at(input, x, y, Button::Right, 1).await
                })
                .await?;
            } else {
                click_at(input, x, y, Button::Right, 1).await?;
            }
            Ok(format!("right-clicked ({x}, {y})"))
        }
        BatchAction::MiddleClick { coordinate, text } => {
            let lc = convert_coord(coordinate, display, target);
            let (x, y) = (lc.x.round() as i32, lc.y.round() as i32);
            if let Some(mods) = text {
                crate::input::modifiers::with_modifiers(input, Some(&mods), || async {
                    click_at(input, x, y, Button::Middle, 1).await
                })
                .await?;
            } else {
                click_at(input, x, y, Button::Middle, 1).await?;
            }
            Ok(format!("middle-clicked ({x}, {y})"))
        }
        BatchAction::DoubleClick { coordinate, text } => {
            let lc = convert_coord(coordinate, display, target);
            let (x, y) = (lc.x.round() as i32, lc.y.round() as i32);
            if let Some(mods) = text {
                crate::input::modifiers::with_modifiers(input, Some(&mods), || async {
                    click_at(input, x, y, Button::Left, 2).await
                })
                .await?;
            } else {
                click_at(input, x, y, Button::Left, 2).await?;
            }
            Ok(format!("double-clicked ({x}, {y})"))
        }
        BatchAction::TripleClick { coordinate, text } => {
            let lc = convert_coord(coordinate, display, target);
            let (x, y) = (lc.x.round() as i32, lc.y.round() as i32);
            if let Some(mods) = text {
                crate::input::modifiers::with_modifiers(input, Some(&mods), || async {
                    click_at(input, x, y, Button::Left, 3).await
                })
                .await?;
            } else {
                click_at(input, x, y, Button::Left, 3).await?;
            }
            Ok(format!("triple-clicked ({x}, {y})"))
        }
        BatchAction::MouseMove { coordinate } => {
            let lc = convert_coord(coordinate, display, target);
            move_and_settle(input, lc.x.round() as i32, lc.y.round() as i32).await?;
            Ok(format!("moved to ({}, {})", lc.x.round(), lc.y.round()))
        }
        BatchAction::LeftClickDrag {
            coordinate,
            start_coordinate,
        } => {
            let to = convert_coord(coordinate, display, target);
            let from = start_coordinate.map(|c| {
                let lc = convert_coord(c, display, target);
                (lc.x.round() as i32, lc.y.round() as i32)
            });
            drag(input, from, (to.x.round() as i32, to.y.round() as i32)).await?;
            Ok("dragged".to_string())
        }
        BatchAction::Scroll {
            coordinate,
            scroll_direction,
            scroll_amount,
        } => {
            let lc = convert_coord(coordinate, display, target);
            scroll_at(
                input,
                lc.x.round() as i32,
                lc.y.round() as i32,
                scroll_direction,
                scroll_amount,
            )
            .await?;
            Ok("scrolled".to_string())
        }
        BatchAction::Key { text, repeat } => {
            press_key_combo(input, &text, repeat).await?;
            Ok(format!("pressed {text}"))
        }
        BatchAction::Type { text } => {
            type_text(input, &text).await?;
            Ok("typed text".to_string())
        }
        BatchAction::HoldKey { text, duration } => {
            hold_key(input, &text, duration).await?;
            Ok(format!("held {text} for {duration}s"))
        }
        BatchAction::CursorPosition {} => {
            let (x, y) = cursor_position(input).await?;
            Ok(format!("cursor at ({x}, {y})"))
        }
        BatchAction::LeftMouseDown {} => {
            mouse_down(input).await?;
            Ok("mouse down".to_string())
        }
        BatchAction::LeftMouseUp {} => {
            mouse_up(input).await?;
            Ok("mouse up".to_string())
        }
        BatchAction::Wait { duration } => {
            tokio::time::sleep(tokio::time::Duration::from_secs_f64(duration)).await;
            Ok(format!("waited {duration}s"))
        }
    }
}
