<p align="center">
  <img src="https://raw.githubusercontent.com/smithery-ai/mouseless/master/assets/mouseless.png" alt="mouseless" width="120">
</p>

<h1 align="center">mouseless</h1>

<p align="center">
  Rust MCP server for macOS desktop control. Screenshots, mouse, keyboard, app management — over stdio or Streamable HTTP.
</p>

## Quick start

```bash
curl -fsSL https://raw.githubusercontent.com/smithery-ai/mouseless/master/scripts/install.sh | bash
mouseless
```

`mouseless` runs in stdio mode by default. Point any MCP client (Claude Code, Claude Desktop, Cursor, Continue, …) at it:

```json
{
  "mcpServers": {
    "mouseless": {
      "command": "mouseless"
    }
  }
}
```

The install script drops the latest release into `~/.local/bin/mouseless`. Override with `INSTALL_DIR=/usr/local/bin` or pin a version with `VERSION=v0.1.0`.

## Other install options

### Smithery

```bash
smithery install mouseless
smithery run mouseless
```

### Build from crates.io

```bash
cargo install mouseless
```

### Claude Desktop bundle

Download `mouseless-vX.Y.Z.mcpb` from the [latest release](https://github.com/smithery-ai/mouseless/releases/latest) and drop it into Claude Desktop. The bundle ships a universal macOS binary with the manifest wiring up all 21 tools over stdio.

## Running manually

stdio is the default — no flag needed.

```bash
mouseless                       # stdio (default)
mouseless --http                # HTTP on 127.0.0.1:3100
mouseless --http 0.0.0.0:8080   # HTTP on custom address
mouseless --help                # usage
mouseless --version
```

`RUST_LOG` controls log verbosity (default `info`), e.g. `RUST_LOG=debug mouseless`.

## MCP client configuration

**stdio (direct binary):**

```json
{ "mcpServers": { "mouseless": { "command": "mouseless" } } }
```

**stdio (via Smithery):**

```json
{ "mcpServers": { "mouseless": { "command": "smithery", "args": ["run", "mouseless"] } } }
```

**Streamable HTTP:**

```json
{ "mcpServers": { "mouseless": { "url": "http://127.0.0.1:3100/mcp" } } }
```

## Requirements

- macOS (aarch64 or x86_64)
- Accessibility permission (System Settings > Privacy & Security > Accessibility)
- Screen Recording permission (System Settings > Privacy & Security > Screen Recording)
- Rust 1.88+ (only if building from source)

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

## Build from source

```bash
git clone https://github.com/smithery-ai/mouseless
cd mouseless
cargo build --release
./target/release/mouseless
```

## Cutting a release (maintainers)

Tag + push — CI builds both darwin targets, attaches tarballs + a `.mcpb` bundle to the GitHub Release, and runs `cargo publish`:

```bash
git tag v0.1.2 && git push origin v0.1.2
```

Local dry-run of the release script:

```bash
scripts/release.sh --dry-run
```

## Architecture

```
src/
├── main.rs             # CLI arg parsing, startup banner, transport dispatch
├── server.rs           # MCP tool handlers, HTTP transport
├── display/            # Display geometry, Retina scaling, coordinate conversion
├── capture/            # xcap screenshots, JPEG encoding, zoom
├── input/
│   ├── thread.rs       # Dedicated enigo thread (mpsc channels)
│   ├── mouse.rs        # Click, move, cursor position
│   ├── keyboard.rs     # Key parsing, combos, hold
│   ├── drag.rs         # Drag with ease-out-cubic animation
│   ├── scroll.rs       # Directional scroll
│   ├── modifiers.rs    # Modifier bracket (press/release LIFO)
│   └── animation.rs    # 60fps ease-out-cubic mouse animation
├── clipboard.rs        # pbcopy/pbpaste with verification
├── apps.rs             # App launch via macOS `open` command
├── batch.rs            # Sequential action dispatcher
├── types.rs            # ScreenCoord, LogicalCoord, BatchAction enum
└── error.rs            # 3-tier error hierarchy
```

### Coordinate system

The model sees a resized screenshot (max 1280x768). Coordinates from tool calls are in that image space. The server converts them to macOS logical points for enigo/CGEvent:

```
ScreenCoord (model) → LogicalCoord (enigo) → PhysicalCoord (framebuffer)
```

Three distinct Rust types prevent mixing coordinate spaces at compile time.

### Input threading

Enigo runs on a dedicated OS thread (not tokio). Commands flow through an mpsc channel, responses through oneshot channels. This avoids CGEventSource thread-affinity issues and keeps enigo's stateful click-timing tracking consistent.

## License

MIT
