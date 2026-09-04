# browsectl-bin

Installs the `browse` command — a CLI exposing the [`browsectl`](https://crates.io/crates/browsectl) library over the command line for shell scripts, CI, and non-Rust callers.

```sh
cargo install browsectl-bin

browse launch --url https://example.com --port 9222
browse eval --port 9222 --script "document.title"
browse screenshot --port 9222 --output page.png
browse stop --port 9222
```

Launch/attach, eval, screenshot, click, input, file-input injection, network mocking, orphaned-session cleanup. Run `browse` with no arguments for the full command reference.

Requires a Chromium-based browser installed on the machine (set `CHROME_PATH` to override auto-discovery).

## Further reading

See [`scm/README.md`](https://github.com/sweengineeringlabs/browsectl/blob/main/scm/README.md) in the repo for the full CLI command reference, exit codes, and known limitations.

## License

MIT
