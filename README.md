# computerbase

Rust MCP server for macOS desktop control. Screenshots, mouse, keyboard, app management — over Streamable HTTP.

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/smithery-ai/computerbase/master/scripts/install.sh | bash
```

Installs the latest release to `~/.local/bin/computer-use-mcp`. Override with `INSTALL_DIR=/usr/local/bin` or pin a version with `VERSION=v0.1.0`.

## Quick start

```bash
# Run (default: 127.0.0.1:3100)
RUST_LOG=info computer-use-mcp

# Custom address
RUST_LOG=info computer-use-mcp 0.0.0.0:8080
```

## Build from source

```bash
cargo build --release
./target/release/computer-use-mcp
```

## Cutting a release (maintainers)

```bash
scripts/release.sh           # tag + build + upload to GitHub Releases
scripts/release.sh 0.2.0     # bump Cargo.toml version first
scripts/release.sh --dry-run # build tarballs only
```

## Add to Claude Code

```json
{
  "mcpServers": {
    "computerbase": {
      "url": "http://127.0.0.1:3100/mcp"
    }
  }
}
```

## Requirements

- macOS (aarch64 or x86_64)
- Rust 1.88+
- Accessibility permission (System Settings > Privacy > Accessibility)
- Screen Recording permission (System Settings > Privacy > Screen Recording)

## Tools (21)

| Tool | Description |
|------|-------------|
| `screenshot` | Capture the primary display (returns MCP ImageContent) |
| `zoom` | High-res capture of a screen region |
| `left_click` | Left-click with optional modifier keys |
| `right_click` | Right-click with optional modifier keys |
| `middle_click` | Middle-click with optional modifier keys |
| `double_click` | Double-click at coordinates |
| `triple_click` | Triple-click at coordinates |
| `left_click_drag` | Click-drag from start to end with animated move |
| `scroll` | Scroll up/down/left/right at coordinates |
| `mouse_move` | Move cursor without clicking |
| `left_mouse_down` | Press left button (hold) |
| `left_mouse_up` | Release left button |
| `cursor_position` | Get current cursor coordinates |
| `key` | Key combo in xdotool syntax (`cmd+shift+a`) |
| `type` | Type text into focused element |
| `hold_key` | Hold a key for N seconds |
| `read_clipboard` | Read clipboard via pbpaste |
| `write_clipboard` | Write clipboard via pbcopy (with read-back verify) |
| `open_application` | Launch or focus an app by name or bundle ID |
| `wait` | Sleep for N seconds |
| `computer_batch` | Execute a sequence of actions in one call |

## Architecture

```
src/
├── server.rs          # MCP tool handlers, HTTP transport
├── display/           # Display geometry, Retina scaling, coordinate conversion
├── capture/           # xcap screenshots, JPEG encoding, zoom
├── input/
│   ├── thread.rs      # Dedicated enigo thread (mpsc channels)
│   ├── mouse.rs       # Click, move, cursor position
│   ├── keyboard.rs    # Key parsing, combos, hold
│   ├── drag.rs        # Drag with ease-out-cubic animation
│   ├── scroll.rs      # Directional scroll
│   ├── modifiers.rs   # Modifier bracket (press/release LIFO)
│   └── animation.rs   # 60fps ease-out-cubic mouse animation
├── clipboard.rs       # pbcopy/pbpaste with verification
├── apps.rs            # App launch via macOS `open` command
├── batch.rs           # Sequential action dispatcher
├── types.rs           # ScreenCoord, LogicalCoord, BatchAction enum
└── error.rs           # 3-tier error hierarchy
```

### Coordinate system

The model sees a resized screenshot (max 1280x768). Coordinates from tool calls are in that image space. The server converts them to macOS logical points for enigo/CGEvent:

```
ScreenCoord (model) → LogicalCoord (enigo) → PhysicalCoord (framebuffer)
```

Three distinct Rust types prevent mixing coordinate spaces at compile time.

### Input threading

Enigo runs on a dedicated OS thread (not tokio). Commands flow through an mpsc channel, responses through oneshot channels. This avoids CGEventSource thread-affinity issues and keeps enigo's stateful click-timing tracking consistent.
