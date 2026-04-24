use thiserror::Error;

/// Errors from executing a tool action (screenshot, click, type, etc.).
/// These become `isError: true` content in `tools/call` responses.
#[derive(Debug, Error)]
pub enum ToolError {
    #[error("screenshot capture failed: {0}")]
    ScreenshotFailed(String),

    #[error("display not found: {0}")]
    DisplayNotFound(String),

    #[error("mouse operation failed: {0}")]
    MouseFailed(String),

    #[error("keyboard operation failed: {0}")]
    KeyboardFailed(String),

    #[error("unknown key: '{0}'")]
    UnknownKey(String),

    #[error("clipboard operation failed: {0}")]
    ClipboardFailed(String),

    #[error("app operation failed: {0}")]
    AppFailed(String),

    #[error("image processing failed: {0}")]
    ImageFailed(#[from] image::ImageError),

    #[error("batch action {index} failed: {source}")]
    BatchActionFailed {
        index: usize,
        source: Box<ToolError>,
    },

    #[error("coordinate out of bounds: ({x}, {y})")]
    CoordinateOutOfBounds { x: f64, y: f64 },

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("not implemented: {0}")]
    NotImplemented(String),
}

/// Errors in the MCP protocol layer.
#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("unknown tool: {0}")]
    UnknownTool(String),

    #[error("invalid input for tool '{tool}': {reason}")]
    InvalidInput { tool: String, reason: String },

    #[error("deserialization failed: {0}")]
    DeserializeFailed(#[from] serde_json::Error),
}

/// System-level errors (display framework, permissions, etc.).
#[derive(Debug, Error)]
pub enum SystemError {
    #[error("core graphics error: {0}")]
    CoreGraphics(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Top-level error that maps to MCP responses.
#[derive(Debug, Error)]
pub enum ServerError {
    #[error(transparent)]
    Tool(#[from] ToolError),

    #[error(transparent)]
    Protocol(#[from] ProtocolError),

    #[error(transparent)]
    System(#[from] SystemError),
}
