use enigo::{Direction, Key};
use tokio::time::{Duration, sleep};

use super::thread::InputHandle;
use crate::error::ToolError;

/// Delay between key repeats (matches 125Hz USB polling cadence).
const KEY_REPEAT_DELAY_MS: u64 = 8;

/// Parse an xdotool-style key sequence into enigo Key values.
///
/// Examples: "return", "cmd+a", "ctrl+shift+tab"
pub fn parse_key_sequence(sequence: &str) -> Result<Vec<Key>, ToolError> {
    sequence
        .split('+')
        .map(|part| parse_single_key(part.trim()))
        .collect()
}

/// Parse a single key name to an enigo Key.
fn parse_single_key(name: &str) -> Result<Key, ToolError> {
    let key = match name.to_lowercase().as_str() {
        // Modifiers
        "cmd" | "command" | "super" | "meta" => Key::Meta,
        "ctrl" | "control" => Key::Control,
        "alt" | "option" => Key::Alt,
        "shift" => Key::Shift,

        // Navigation
        "return" | "enter" => Key::Return,
        "tab" => Key::Tab,
        "escape" | "esc" => Key::Escape,
        "space" => Key::Space,
        "backspace" | "back_space" => Key::Backspace,
        "delete" | "del" => Key::Delete,
        "home" => Key::Home,
        "end" => Key::End,
        "pageup" | "page_up" => Key::PageUp,
        "pagedown" | "page_down" => Key::PageDown,

        // Arrows
        "up" | "uparrow" => Key::UpArrow,
        "down" | "downarrow" => Key::DownArrow,
        "left" | "leftarrow" => Key::LeftArrow,
        "right" | "rightarrow" => Key::RightArrow,

        // Function keys
        "f1" => Key::F1,
        "f2" => Key::F2,
        "f3" => Key::F3,
        "f4" => Key::F4,
        "f5" => Key::F5,
        "f6" => Key::F6,
        "f7" => Key::F7,
        "f8" => Key::F8,
        "f9" => Key::F9,
        "f10" => Key::F10,
        "f11" => Key::F11,
        "f12" => Key::F12,

        // Caps lock
        "capslock" | "caps_lock" => Key::CapsLock,

        // Single character
        s if s.len() == 1 => Key::Unicode(s.chars().next().unwrap()),

        _ => return Err(ToolError::UnknownKey(name.to_string())),
    };

    Ok(key)
}

/// Press a key combination (all keys in the sequence pressed, then released in LIFO order).
pub async fn press_key_combo(
    input: &InputHandle,
    sequence: &str,
    repeat: u32,
) -> Result<(), ToolError> {
    let keys = parse_key_sequence(sequence)?;

    for iteration in 0..repeat {
        if iteration > 0 {
            sleep(Duration::from_millis(KEY_REPEAT_DELAY_MS)).await;
        }

        // Press all keys in order
        let mut pressed: Vec<Key> = Vec::new();
        let press_result = async {
            for key in &keys {
                input.key(*key, Direction::Press).await?;
                pressed.push(*key);
            }
            Ok::<(), ToolError>(())
        }
        .await;

        // Release in LIFO order — always, even if press failed partway
        while let Some(key) = pressed.pop() {
            let _ = input.key(key, Direction::Release).await;
        }

        press_result?;
    }

    Ok(())
}

/// Type text directly via enigo.
pub async fn type_text(input: &InputHandle, text: &str) -> Result<(), ToolError> {
    input.type_text(text.to_string()).await
}

/// Hold a key for the specified duration.
pub async fn hold_key(
    input: &InputHandle,
    sequence: &str,
    duration_secs: f64,
) -> Result<(), ToolError> {
    let keys = parse_key_sequence(sequence)?;

    // Press all keys
    let mut pressed: Vec<Key> = Vec::new();
    for key in &keys {
        input.key(*key, Direction::Press).await?;
        pressed.push(*key);
    }

    // Hold for duration
    sleep(Duration::from_secs_f64(duration_secs)).await;

    // Release in LIFO order
    while let Some(key) = pressed.pop() {
        let _ = input.key(key, Direction::Release).await;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_keys() {
        assert!(matches!(parse_single_key("return"), Ok(Key::Return)));
        assert!(matches!(parse_single_key("cmd"), Ok(Key::Meta)));
        assert!(matches!(parse_single_key("a"), Ok(Key::Unicode('a'))));
        assert!(matches!(parse_single_key("f12"), Ok(Key::F12)));
        assert!(parse_single_key("nonexistent").is_err());
    }

    #[test]
    fn test_parse_combo() {
        let keys = parse_key_sequence("cmd+shift+a").unwrap();
        assert_eq!(keys.len(), 3);
        assert!(matches!(keys[0], Key::Meta));
        assert!(matches!(keys[1], Key::Shift));
        assert!(matches!(keys[2], Key::Unicode('a')));
    }
}
