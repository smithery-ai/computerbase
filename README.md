<p align="center">
  <img src="https://raw.githubusercontent.com/smithery-ai/mouseless/master/assets/mouseless.png" alt="mouseless" width="120">
</p>

<h1 align="center">mouseless</h1>

<p align="center">
  Rust MCP server for macOS desktop control — screenshots, mouse, keyboard, apps. Over stdio or Streamable HTTP.
</p>

## Quick start

```bash
curl -fsSL https://raw.githubusercontent.com/smithery-ai/mouseless/master/scripts/install.sh | bash
```

Point any MCP client at it — `mouseless` runs stdio by default:

```json
{ "mcpServers": { "mouseless": { "command": "mouseless" } } }
```

Set `INSTALL_DIR` or `VERSION` to override the install script's defaults (`~/.local/bin`, latest).

## Other install options

[Smithery](https://smithery.ai/servers/smithery/mouseless):

```bash
smithery mcp add smithery/mouseless
```

From crates.io:

```bash
cargo install mouseless
```

Claude Desktop bundle — download `mouseless-vX.Y.Z.mcpb` from the [latest release](https://github.com/smithery-ai/mouseless/releases/latest) and drop it in.

## Usage

```bash
mouseless                       # stdio (default)
mouseless --http [ADDR]         # HTTP (default 127.0.0.1:3100)
mouseless --help | --version
```

HTTP client config: `{ "url": "http://127.0.0.1:3100/mcp" }`. `RUST_LOG` tunes verbosity (default `info`).

## Requirements

macOS (aarch64 / x86_64). Grant **Accessibility** and **Screen Recording** in System Settings > Privacy & Security.

## Tools (21)

| Tool | Description |
|------|-------------|
| `screenshot` | Capture the primary display (MCP ImageContent) |
| `zoom` | High-res capture of a screen region |
| `left_click` / `right_click` / `middle_click` | Click with optional modifiers |
| `double_click` / `triple_click` | Multi-click at coordinates |
| `left_click_drag` | Click-drag with animated move |
| `scroll` | Scroll up/down/left/right at coordinates |
| `mouse_move` | Move cursor without clicking |
| `left_mouse_down` / `left_mouse_up` | Press/release left button |
| `cursor_position` | Get current cursor coordinates |
| `key` / `hold_key` | Key combo (xdotool syntax) or hold for N seconds |
| `type` | Type text into focused element |
| `read_clipboard` / `write_clipboard` | pbpaste / pbcopy (write verifies) |
| `open_application` | Launch or focus an app by name or bundle ID |
| `wait` | Sleep for N seconds |
| `computer_batch` | Execute a sequence of actions in one call |

## Build from source

```bash
git clone https://github.com/smithery-ai/mouseless && cd mouseless
cargo build --release
```

## Release (maintainers)

Tag and push — CI builds both darwin targets, attaches tarballs + `.mcpb` to the GitHub Release, and runs `cargo publish`:

```bash
git tag v0.1.2 && git push origin v0.1.2
```

## Architecture

```
src/
├── main.rs              # arg parsing, startup banner, transport dispatch
├── server.rs            # MCP tool handlers, HTTP transport
├── display/ capture/    # display geometry, Retina scaling, xcap screenshots, zoom
├── input/               # dedicated enigo thread, mouse/keyboard/drag/scroll/animation
├── clipboard.rs apps.rs batch.rs   # pbcopy/pbpaste, app launch, batch dispatch
├── types.rs             # ScreenCoord / LogicalCoord / PhysicalCoord
└── error.rs             # 3-tier error hierarchy
```

Coordinates from the model are in the resized screenshot space (max 1280×768) and converted to macOS logical points — three distinct Rust types prevent mixing spaces at compile time. Enigo runs on a dedicated OS thread with mpsc/oneshot channels to avoid CGEventSource thread-affinity issues.

## License

MIT
