# browsectl

A synchronous, zero-async-runtime Chromium DevTools Protocol (CDP) client for Rust. Works with any Chromium-based browser: Chrome, Edge, Brave, Arc, Vivaldi.

Launch or attach to a browser, evaluate JavaScript, read computed CSS, resize the viewport, navigate — over a plain WebSocket, no `tokio` required.

## Quick start

```rust
use browsectl::{CdpClient, PageEvaluator};

// Launch headless Chrome and connect
let mut client = CdpClient::launch("https://example.com").unwrap();

// Evaluate JavaScript
let title = client.evaluate("document.title").unwrap();

// Read computed CSS
let color = client.get_computed_style("h1", "color").unwrap();

// Resize the viewport (actually changes it — uses Emulation.setDeviceMetricsOverride)
client.set_viewport_width(375).unwrap();

// Navigate to a new page
client.navigate("https://example.com/other").unwrap();
```

For a CLI wrapping the same library — shell scripts, CI, non-Rust callers — see [`browsectl-bin`](https://crates.io/crates/browsectl-bin) (installs the `browse` command).

Requires a Chromium-based browser installed on the machine (set `CHROME_PATH` to override auto-discovery).

## Further reading

See [`scm/README.md`](https://github.com/sweengineeringlabs/browsectl/blob/main/scm/README.md) in the repo for the full crate layout, API surface, and design rationale ("minimal" describes deliberate restraint, not a limited feature set — hand-written calls for exactly the CDP methods its consumers need, nothing generated).

## License

MIT
