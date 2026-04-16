use enigo::Direction;

use super::keyboard::parse_key_sequence;
use super::thread::InputHandle;
use crate::error::ToolError;

/// Parse a modifier string (e.g. "shift", "ctrl+shift") and hold the modifiers
/// while executing a closure, then release in LIFO order.
///
/// If the modifier string is None or empty, executes the closure directly.
pub async fn with_modifiers<F, Fut>(
    input: &InputHandle,
    modifier_str: Option<&str>,
    action: F,
) -> Result<(), ToolError>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<(), ToolError>>,
{
    let modifier_str = match modifier_str {
        Some(s) if !s.is_empty() => s,
        _ => return action().await,
    };

    let keys = parse_key_sequence(modifier_str)?;
    let mut pressed = Vec::new();

    // Press modifiers
    let press_result = async {
        for key in &keys {
            input.key(*key, Direction::Press).await?;
            pressed.push(*key);
        }
        Ok::<(), ToolError>(())
    }
    .await;

    // Execute action (only if all modifiers pressed successfully)
    let action_result = if press_result.is_ok() {
        action().await
    } else {
        Err(press_result.unwrap_err())
    };

    // Release in LIFO order — always, even on error
    while let Some(key) = pressed.pop() {
        let _ = input.key(key, Direction::Release).await;
    }

    action_result
}
