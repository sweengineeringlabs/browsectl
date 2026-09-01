# launch

Minimal end-to-end usage: launch a browser, evaluate JavaScript, read
a computed CSS property, and query the viewport size -- the four most
common `CdpClient`/`PageEvaluator` calls, in one runnable file.

Run it:

```
cargo run --example launch
```

Requires a Chromium-based browser installed on the machine (set
`CHROME_PATH` to override auto-discovery) -- this example actually
launches one, it isn't mocked.

See [Architecture](../../../../../docs/3-design/architecture.md) for
why the client is synchronous with no async runtime.
