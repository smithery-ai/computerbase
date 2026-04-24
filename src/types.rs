use serde::{Deserialize, Serialize};

/// Coordinates in screenshot image space (what the model sees and sends back).
/// These are the raw [x, y] values from tool input.
#[derive(Debug, Clone, Copy)]
pub struct ScreenCoord {
    pub x: f64,
    pub y: f64,
}

/// Coordinates in macOS logical space (points, not pixels).
/// What CGEvent and enigo use.
#[derive(Debug, Clone, Copy)]
pub struct LogicalCoord {
    pub x: f64,
    pub y: f64,
}

/// Coordinates in physical pixel space (logical * scale_factor).
/// What the raw framebuffer and xcap captures produce.
#[derive(Debug, Clone, Copy)]
pub struct PhysicalCoord {
    pub x: f64,
    pub y: f64,
}

/// A coordinate pair as it arrives from MCP JSON: a 2-element array.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CoordPair(pub [f64; 2]);

impl From<CoordPair> for ScreenCoord {
    fn from(c: CoordPair) -> Self {
        ScreenCoord {
            x: c.0[0],
            y: c.0[1],
        }
    }
}

/// A region rectangle: [x0, y0, x1, y1] in screenshot pixel space.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RegionRect(pub [i32; 4]);

/// Display geometry information.
#[derive(Debug, Clone)]
pub struct DisplayGeometry {
    pub display_id: u32,
    pub width: u32,        // logical points
    pub height: u32,       // logical points
    pub pixel_width: u32,  // physical pixels
    pub pixel_height: u32, // physical pixels
    pub scale_factor: f64, // pixel_width / width (2.0 on Retina)
    pub origin_x: i32,     // multi-monitor offset
    pub origin_y: i32,
}

/// The dimensions we resize the screenshot to before sending.
#[derive(Debug, Clone, Copy)]
pub struct TargetDims {
    pub width: u32,
    pub height: u32,
}

/// Result from a screenshot capture.
#[derive(Debug, Clone, Serialize)]
pub struct ScreenshotResult {
    pub base64_image: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    Key,
    Type,
    MouseMove,
    LeftClick,
    LeftClickDrag,
    RightClick,
    MiddleClick,
    DoubleClick,
    TripleClick,
    Scroll,
    HoldKey,
    Screenshot,
    CursorPosition,
    LeftMouseDown,
    LeftMouseUp,
    Wait,
}

/// A single action inside a `computer_batch` call.
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum BatchAction {
    Key {
        text: String,
        #[serde(default = "default_repeat")]
        repeat: u32,
    },
    Type {
        text: String,
    },
    MouseMove {
        coordinate: CoordPair,
    },
    LeftClick {
        coordinate: CoordPair,
        #[serde(default)]
        text: Option<String>,
    },
    RightClick {
        coordinate: CoordPair,
        #[serde(default)]
        text: Option<String>,
    },
    MiddleClick {
        coordinate: CoordPair,
        #[serde(default)]
        text: Option<String>,
    },
    DoubleClick {
        coordinate: CoordPair,
        #[serde(default)]
        text: Option<String>,
    },
    TripleClick {
        coordinate: CoordPair,
        #[serde(default)]
        text: Option<String>,
    },
    LeftClickDrag {
        coordinate: CoordPair,
        #[serde(default)]
        start_coordinate: Option<CoordPair>,
    },
    Scroll {
        coordinate: CoordPair,
        scroll_direction: ScrollDirection,
        #[serde(default = "default_scroll_amount")]
        scroll_amount: u32,
    },
    HoldKey {
        text: String,
        duration: f64,
    },
    Screenshot {},
    CursorPosition {},
    LeftMouseDown {},
    LeftMouseUp {},
    Wait {
        duration: f64,
    },
}

fn default_repeat() -> u32 {
    1
}

fn default_scroll_amount() -> u32 {
    3
}
