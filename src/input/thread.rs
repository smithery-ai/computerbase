// Dedicated thread for enigo input operations.
//
// Enigo is stateful (click timing, button tracking, event flags) and its
// macOS CGEventSource may have thread-affinity expectations. We own a single
// Enigo instance on a dedicated thread and communicate via channels.

use enigo::{Axis, Button, Coordinate, Direction, Enigo, Keyboard, Mouse, Settings};
use std::thread;
use tokio::sync::{mpsc, oneshot};

use crate::error::ToolError;

/// Commands sent to the enigo input thread.
pub enum InputCommand {
    MoveMouse {
        x: i32,
        y: i32,
        reply: oneshot::Sender<Result<(), ToolError>>,
    },
    Click {
        button: Button,
        direction: Direction,
        reply: oneshot::Sender<Result<(), ToolError>>,
    },
    Scroll {
        amount: i32,
        axis: Axis,
        reply: oneshot::Sender<Result<(), ToolError>>,
    },
    Key {
        key: enigo::Key,
        direction: Direction,
        reply: oneshot::Sender<Result<(), ToolError>>,
    },
    TypeText {
        text: String,
        reply: oneshot::Sender<Result<(), ToolError>>,
    },
    CursorPosition {
        reply: oneshot::Sender<Result<(i32, i32), ToolError>>,
    },
    MainDisplaySize {
        reply: oneshot::Sender<Result<(i32, i32), ToolError>>,
    },
}

/// Handle to communicate with the input thread.
#[derive(Clone, Debug)]
pub struct InputHandle {
    tx: mpsc::UnboundedSender<InputCommand>,
}

impl InputHandle {
    /// Spawn the dedicated input thread and return a handle.
    pub fn spawn() -> Result<Self, ToolError> {
        let (tx, mut rx) = mpsc::unbounded_channel::<InputCommand>();

        thread::Builder::new()
            .name("enigo-input".into())
            .spawn(move || {
                let mut enigo = match Enigo::new(&Settings::default()) {
                    Ok(e) => e,
                    Err(e) => {
                        tracing::error!("failed to create Enigo: {e}");
                        return;
                    }
                };

                // Block on receiving commands (not async — this is a plain thread)
                while let Some(cmd) = rx.blocking_recv() {
                    match cmd {
                        InputCommand::MoveMouse { x, y, reply } => {
                            let result = enigo
                                .move_mouse(x, y, Coordinate::Abs)
                                .map_err(|e| ToolError::MouseFailed(e.to_string()));
                            let _ = reply.send(result);
                        }
                        InputCommand::Click {
                            button,
                            direction,
                            reply,
                        } => {
                            let result = enigo
                                .button(button, direction)
                                .map_err(|e| ToolError::MouseFailed(e.to_string()));
                            let _ = reply.send(result);
                        }
                        InputCommand::Scroll {
                            amount,
                            axis,
                            reply,
                        } => {
                            let result = enigo
                                .scroll(amount, axis)
                                .map_err(|e| ToolError::MouseFailed(e.to_string()));
                            let _ = reply.send(result);
                        }
                        InputCommand::Key {
                            key,
                            direction,
                            reply,
                        } => {
                            let result = enigo
                                .key(key, direction)
                                .map_err(|e| ToolError::KeyboardFailed(e.to_string()));
                            let _ = reply.send(result);
                        }
                        InputCommand::TypeText { text, reply } => {
                            let result = enigo
                                .text(&text)
                                .map_err(|e| ToolError::KeyboardFailed(e.to_string()));
                            let _ = reply.send(result);
                        }
                        InputCommand::CursorPosition { reply } => {
                            let result = enigo
                                .location()
                                .map_err(|e| ToolError::MouseFailed(e.to_string()));
                            let _ = reply.send(result);
                        }
                        InputCommand::MainDisplaySize { reply } => {
                            let result = enigo
                                .main_display()
                                .map_err(|e| ToolError::MouseFailed(e.to_string()));
                            let _ = reply.send(result);
                        }
                    }
                }

                tracing::info!("enigo input thread shutting down");
            })
            .map_err(|e| ToolError::MouseFailed(format!("spawn input thread: {e}")))?;

        Ok(InputHandle { tx })
    }

    /// Send a command and await the response.
    async fn send<T>(
        &self,
        make_cmd: impl FnOnce(oneshot::Sender<Result<T, ToolError>>) -> InputCommand,
    ) -> Result<T, ToolError> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.tx
            .send(make_cmd(reply_tx))
            .map_err(|_| ToolError::MouseFailed("input thread died".into()))?;
        reply_rx
            .await
            .map_err(|_| ToolError::MouseFailed("input thread dropped reply".into()))?
    }

    // ── Public API ────────────────────────────────────────────────

    pub async fn move_mouse(&self, x: i32, y: i32) -> Result<(), ToolError> {
        self.send(|reply| InputCommand::MoveMouse { x, y, reply })
            .await
    }

    pub async fn click(&self, button: Button, direction: Direction) -> Result<(), ToolError> {
        self.send(|reply| InputCommand::Click {
            button,
            direction,
            reply,
        })
        .await
    }

    pub async fn scroll(&self, amount: i32, axis: Axis) -> Result<(), ToolError> {
        self.send(|reply| InputCommand::Scroll {
            amount,
            axis,
            reply,
        })
        .await
    }

    pub async fn key(&self, key: enigo::Key, direction: Direction) -> Result<(), ToolError> {
        self.send(|reply| InputCommand::Key {
            key,
            direction,
            reply,
        })
        .await
    }

    pub async fn type_text(&self, text: String) -> Result<(), ToolError> {
        self.send(|reply| InputCommand::TypeText { text, reply })
            .await
    }

    pub async fn cursor_position(&self) -> Result<(i32, i32), ToolError> {
        self.send(|reply| InputCommand::CursorPosition { reply })
            .await
    }
}
