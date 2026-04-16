use std::sync::{Arc, Mutex};

use enigo::Button;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager,
    tower::StreamableHttpService,
};
use rmcp::transport::StreamableHttpServerConfig;
use rmcp::{tool, tool_router, ServiceExt};
use serde::Deserialize;
use tokio_util::sync::CancellationToken;

use crate::apps;
use crate::batch;
use crate::capture::screenshot::{capture_region, capture_screenshot};
use crate::capture::zoom::capture_zoom;
use crate::clipboard;
use crate::display::geometry::primary_display;
use crate::display::scaling::{compute_target_dims, screen_to_logical};
use crate::input::drag::drag;
use crate::input::keyboard::{hold_key, press_key_combo, type_text};
use crate::input::modifiers::with_modifiers;
use crate::input::mouse::{click_at, cursor_position, mouse_down, mouse_up, move_and_settle};
use crate::input::scroll::scroll_at;
use crate::input::thread::InputHandle;
use crate::types::{
    BatchAction, CoordPair, DisplayGeometry, LogicalCoord, RegionRect, ScrollDirection, TargetDims,
};

// ── Tool parameter types ────────────────────────────────────────────

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ScreenshotParams {
    /// If set to true takes a screenshot of the full page instead of the currently visible viewport.
    #[serde(default)]
    pub full_page: Option<bool>,
    /// Image format: png, jpeg, or webp. Default is "png".
    #[serde(default)]
    pub format: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ClickParams {
    /// (x, y) coordinate to click.
    pub coordinate: CoordPair,
    /// Modifier keys to hold during the click (e.g. "shift", "ctrl+shift").
    #[serde(default)]
    pub text: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DragParams {
    /// (x, y) end point of the drag.
    pub coordinate: CoordPair,
    /// (x, y) start point. If omitted, drags from the current cursor position.
    #[serde(default)]
    pub start_coordinate: Option<CoordPair>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ScrollParams {
    /// (x, y) coordinate to scroll at.
    pub coordinate: CoordPair,
    /// Direction to scroll.
    pub scroll_direction: ScrollDirection,
    /// Number of scroll ticks.
    #[serde(default = "default_scroll_amount")]
    pub scroll_amount: u32,
}

fn default_scroll_amount() -> u32 {
    3
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MoveParams {
    /// (x, y) coordinate to move to.
    pub coordinate: CoordPair,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct KeyParams {
    /// Key or chord to press, e.g. "return", "cmd+a", "ctrl+shift+tab".
    pub text: String,
    /// Number of times to repeat the key press. Default is 1.
    #[serde(default = "default_repeat")]
    pub repeat: u32,
}

fn default_repeat() -> u32 {
    1
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TypeParams {
    /// Text to type.
    pub text: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct HoldKeyParams {
    /// Key or chord to hold, e.g. "space", "shift+down".
    pub text: String,
    /// Duration in seconds (0-100).
    pub duration: f64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct OpenAppParams {
    /// Application display name (e.g. "Slack") or bundle identifier.
    pub app: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WaitParams {
    /// Duration in seconds (0-100).
    pub duration: f64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ZoomParams {
    /// (x0, y0, x1, y1): Rectangle to zoom into in the coordinate space of the most recent screenshot.
    pub region: RegionRect,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct BatchParams {
    /// List of actions to execute sequentially.
    pub actions: Vec<BatchAction>,
}

// ── Server state ────────────────────────────────────────────────────

#[derive(Debug)]
pub struct ComputerUseMcp {
    input: InputHandle,
    /// Cached display + target dims for coordinate conversion.
    display_cache: Mutex<Option<(DisplayGeometry, TargetDims)>>,
}

impl Clone for ComputerUseMcp {
    fn clone(&self) -> Self {
        ComputerUseMcp {
            input: self.input.clone(),
            display_cache: Mutex::new(None),
        }
    }
}

impl ComputerUseMcp {
    pub fn new(input: InputHandle) -> Self {
        ComputerUseMcp {
            input,
            display_cache: Mutex::new(None),
        }
    }

    /// Get or refresh the display + target dims cache.
    fn get_display_info(&self) -> Result<(DisplayGeometry, TargetDims), String> {
        let display = primary_display().map_err(|e| e.to_string())?;
        let target = compute_target_dims(display.pixel_width, display.pixel_height);
        *self.display_cache.lock().unwrap() = Some((display.clone(), target));
        Ok((display, target))
    }

    /// Convert a coordinate pair from screenshot space to logical display space.
    fn to_logical(&self, coord: CoordPair) -> Result<LogicalCoord, String> {
        let (display, target) = self.get_display_info()?;
        Ok(screen_to_logical(coord.into(), &display, &target))
    }

    /// Convert and round to integer coords for enigo.
    fn to_logical_i32(&self, coord: CoordPair) -> Result<(i32, i32), String> {
        let lc = self.to_logical(coord)?;
        Ok((lc.x.round() as i32, lc.y.round() as i32))
    }

    /// Execute a click with optional modifiers.
    async fn do_click(
        &self,
        coord: CoordPair,
        button: Button,
        count: u32,
        modifiers: Option<String>,
    ) -> String {
        let (x, y) = match self.to_logical_i32(coord) {
            Ok(v) => v,
            Err(e) => return format!("error: {e}"),
        };

        let result = if let Some(mods) = modifiers {
            let input = &self.input;
            with_modifiers(input, Some(&mods), || async {
                click_at(input, x, y, button, count).await
            })
            .await
        } else {
            click_at(&self.input, x, y, button, count).await
        };

        match result {
            Ok(()) => format!("clicked ({x}, {y})"),
            Err(e) => format!("error: {e}"),
        }
    }
}

// ── Tool implementations ────────────────────────────────────────────

#[tool_router(server_handler)]
impl ComputerUseMcp {
    #[tool(
        name = "screenshot",
        description = "Take a screenshot of the primary display."
    )]
    async fn screenshot(&self, Parameters(_p): Parameters<ScreenshotParams>) -> String {
        match tokio::task::spawn_blocking(capture_screenshot).await {
            Ok(Ok(result)) => {
                // Return as MCP image content
                serde_json::json!({
                    "type": "image",
                    "data": result.base64_image,
                    "mimeType": "image/jpeg",
                    "width": result.width,
                    "height": result.height,
                })
                .to_string()
            }
            Ok(Err(e)) => format!("error: {e}"),
            Err(e) => format!("error: task join failed: {e}"),
        }
    }

    #[tool(
        name = "left_click",
        description = "Left-click at the given coordinates."
    )]
    async fn left_click(&self, Parameters(p): Parameters<ClickParams>) -> String {
        self.do_click(p.coordinate, Button::Left, 1, p.text).await
    }

    #[tool(
        name = "right_click",
        description = "Right-click at the given coordinates."
    )]
    async fn right_click(&self, Parameters(p): Parameters<ClickParams>) -> String {
        self.do_click(p.coordinate, Button::Right, 1, p.text).await
    }

    #[tool(
        name = "middle_click",
        description = "Middle-click at the given coordinates."
    )]
    async fn middle_click(&self, Parameters(p): Parameters<ClickParams>) -> String {
        self.do_click(p.coordinate, Button::Middle, 1, p.text).await
    }

    #[tool(
        name = "double_click",
        description = "Double-click at the given coordinates."
    )]
    async fn double_click(&self, Parameters(p): Parameters<ClickParams>) -> String {
        self.do_click(p.coordinate, Button::Left, 2, p.text).await
    }

    #[tool(
        name = "triple_click",
        description = "Triple-click at the given coordinates."
    )]
    async fn triple_click(&self, Parameters(p): Parameters<ClickParams>) -> String {
        self.do_click(p.coordinate, Button::Left, 3, p.text).await
    }

    #[tool(
        name = "left_click_drag",
        description = "Press, move to target, and release."
    )]
    async fn left_click_drag(&self, Parameters(p): Parameters<DragParams>) -> String {
        let to = match self.to_logical_i32(p.coordinate) {
            Ok(v) => v,
            Err(e) => return format!("error: {e}"),
        };
        let from = p.start_coordinate.map(|c| match self.to_logical_i32(c) {
            Ok(v) => Ok(v),
            Err(e) => Err(e),
        });
        let from = match from.transpose() {
            Ok(v) => v,
            Err(e) => return format!("error: {e}"),
        };

        match drag(&self.input, from, to).await {
            Ok(()) => "dragged".to_string(),
            Err(e) => format!("error: {e}"),
        }
    }

    #[tool(
        name = "scroll",
        description = "Scroll at the given coordinates."
    )]
    async fn scroll(&self, Parameters(p): Parameters<ScrollParams>) -> String {
        let (x, y) = match self.to_logical_i32(p.coordinate) {
            Ok(v) => v,
            Err(e) => return format!("error: {e}"),
        };

        match scroll_at(&self.input, x, y, p.scroll_direction, p.scroll_amount).await {
            Ok(()) => "scrolled".to_string(),
            Err(e) => format!("error: {e}"),
        }
    }

    #[tool(
        name = "mouse_move",
        description = "Move the mouse cursor without clicking."
    )]
    async fn mouse_move(&self, Parameters(p): Parameters<MoveParams>) -> String {
        let (x, y) = match self.to_logical_i32(p.coordinate) {
            Ok(v) => v,
            Err(e) => return format!("error: {e}"),
        };

        match move_and_settle(&self.input, x, y).await {
            Ok(()) => format!("moved to ({x}, {y})"),
            Err(e) => format!("error: {e}"),
        }
    }

    #[tool(
        name = "left_mouse_down",
        description = "Press the left mouse button at the current cursor position."
    )]
    async fn left_mouse_down(&self) -> String {
        match mouse_down(&self.input).await {
            Ok(()) => "mouse down".to_string(),
            Err(e) => format!("error: {e}"),
        }
    }

    #[tool(
        name = "left_mouse_up",
        description = "Release the left mouse button at the current cursor position."
    )]
    async fn left_mouse_up(&self) -> String {
        match mouse_up(&self.input).await {
            Ok(()) => "mouse up".to_string(),
            Err(e) => format!("error: {e}"),
        }
    }

    #[tool(
        name = "cursor_position",
        description = "Get the current mouse cursor position."
    )]
    async fn cursor_position(&self) -> String {
        match cursor_position(&self.input).await {
            Ok((x, y)) => format!("({x}, {y})"),
            Err(e) => format!("error: {e}"),
        }
    }

    #[tool(
        name = "key",
        description = "Press a key or key combination (e.g. \"return\", \"cmd+a\")."
    )]
    async fn key(&self, Parameters(p): Parameters<KeyParams>) -> String {
        match press_key_combo(&self.input, &p.text, p.repeat).await {
            Ok(()) => format!("pressed {}", p.text),
            Err(e) => format!("error: {e}"),
        }
    }

    #[tool(name = "type", description = "Type text into whatever currently has keyboard focus.")]
    async fn type_text(&self, Parameters(p): Parameters<TypeParams>) -> String {
        match type_text(&self.input, &p.text).await {
            Ok(()) => "typed".to_string(),
            Err(e) => format!("error: {e}"),
        }
    }

    #[tool(
        name = "hold_key",
        description = "Press and hold a key for the specified duration, then release."
    )]
    async fn hold_key(&self, Parameters(p): Parameters<HoldKeyParams>) -> String {
        match hold_key(&self.input, &p.text, p.duration).await {
            Ok(()) => format!("held {} for {}s", p.text, p.duration),
            Err(e) => format!("error: {e}"),
        }
    }

    #[tool(
        name = "read_clipboard",
        description = "Read the current clipboard contents as text."
    )]
    async fn read_clipboard(&self) -> String {
        match tokio::task::spawn_blocking(clipboard::read_clipboard).await {
            Ok(Ok(text)) => text,
            Ok(Err(e)) => format!("error: {e}"),
            Err(e) => format!("error: {e}"),
        }
    }

    #[tool(
        name = "write_clipboard",
        description = "Write text to the clipboard."
    )]
    async fn write_clipboard(&self, Parameters(p): Parameters<TypeParams>) -> String {
        let text = p.text;
        match tokio::task::spawn_blocking(move || clipboard::write_clipboard(&text)).await {
            Ok(Ok(())) => "clipboard written".to_string(),
            Ok(Err(e)) => format!("error: {e}"),
            Err(e) => format!("error: {e}"),
        }
    }

    #[tool(
        name = "open_application",
        description = "Bring an application to the front, launching it if necessary."
    )]
    async fn open_application(&self, Parameters(p): Parameters<OpenAppParams>) -> String {
        let app = p.app;
        match tokio::task::spawn_blocking(move || apps::open_application(&app)).await {
            Ok(Ok(())) => "opened".to_string(),
            Ok(Err(e)) => format!("error: {e}"),
            Err(e) => format!("error: {e}"),
        }
    }

    #[tool(
        name = "wait",
        description = "Wait for a specified duration."
    )]
    async fn wait(&self, Parameters(p): Parameters<WaitParams>) -> String {
        let duration = p.duration.clamp(0.0, 100.0);
        tokio::time::sleep(tokio::time::Duration::from_secs_f64(duration)).await;
        format!("waited {duration}s")
    }

    #[tool(
        name = "zoom",
        description = "Take a higher-resolution screenshot of a specific region."
    )]
    async fn zoom(&self, Parameters(p): Parameters<ZoomParams>) -> String {
        let region = p.region;
        match tokio::task::spawn_blocking(move || capture_zoom(&region)).await {
            Ok(Ok(result)) => {
                serde_json::json!({
                    "type": "image",
                    "data": result.base64_image,
                    "mimeType": "image/jpeg",
                    "width": result.width,
                    "height": result.height,
                })
                .to_string()
            }
            Ok(Err(e)) => format!("error: {e}"),
            Err(e) => format!("error: {e}"),
        }
    }

    #[tool(
        name = "computer_batch",
        description = "Execute a sequence of actions in one call. Actions execute sequentially and stop on the first error."
    )]
    async fn computer_batch(&self, Parameters(p): Parameters<BatchParams>) -> String {
        let (display, target) = match self.get_display_info() {
            Ok(info) => info,
            Err(e) => return format!("error: {e}"),
        };

        match batch::execute_batch(p.actions, &self.input, &display, &target).await {
            Ok(msg) => msg,
            Err(e) => format!("error: {e}"),
        }
    }
}

const DEFAULT_BIND: &str = "127.0.0.1:3100";

pub async fn run_http(bind_addr: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let addr = bind_addr.unwrap_or(DEFAULT_BIND);
    tracing::info!("starting streamable HTTP server on {addr}");

    let input = InputHandle::spawn()?;
    let ct = CancellationToken::new();

    let service: StreamableHttpService<ComputerUseMcp, LocalSessionManager> =
        StreamableHttpService::new(
            move || Ok(ComputerUseMcp::new(input.clone())),
            Default::default(),
            StreamableHttpServerConfig::default()
                .with_cancellation_token(ct.child_token()),
        );

    let router = axum::Router::new().nest_service("/mcp", service);
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!("listening on http://{addr}/mcp");

    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            tokio::signal::ctrl_c().await.ok();
            tracing::info!("shutting down");
            ct.cancel();
        })
        .await?;

    Ok(())
}
