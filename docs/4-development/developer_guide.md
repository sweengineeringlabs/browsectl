# Developer Guide

## Prerequisites

| Requirement | Version |
|-------------|---------|
| Rust | **1.99.1**, pinned in `scm/rust-toolchain.toml` and as `rust-version` in `scm/Cargo.toml` (`rustup` selects it automatically) |
| Browser | Chrome, Edge, Brave or Chromium, for the browser-dependent tests and for using the CLI |
| curl | Any recent version (used by browser-discovery code to find the debugger URL) |

A `toolchain pins` CI job fails if a workflow's Rust pin differs from `rust-version`, and the
`docs` job fails if a document states a different Rust version.

## Build

The workspace manifest lives at `scm/Cargo.toml` — run Cargo from `scm/`:

```sh
cd scm
cargo build
```

No code generation, no proc-macros beyond `serde`. A cold build takes about a minute or two;
the release binary (`cargo build --release -p browsectl-bin`) is `target/release/browse`.

`scripts/dev/build.sh` runs what CI runs, without GitHub Actions and in an isolated build
directory that never waits on another cargo process's lock:

```sh
bash scripts/dev/build.sh all    # pins, fmt, clippy, tests, release build, verify
```

It builds into `.work/target` (git-ignored; `BROWSECTL_TARGET_DIR` overrides) with two parallel
jobs (`BROWSECTL_JOBS`). Other steps: `pins`, `fmt`, `clippy`, `test`, `build`, `release`,
`verify`. See [`scripts/dev/README.md`](../../scripts/dev/README.md). Documentation changes are
checked with `bash scripts/dev/check-docs.sh`.

## Tests

```sh
# Everything that runs in CI: unit tests, the help snapshots and the offline tests
cd scm && cargo test --workspace --locked

# Tests that drive a real browser (88 of them). They are #[ignore]d, do not run in CI, and need
# a Chromium-based browser installed. One thread: they share browser ports.
cargo test --workspace --locked -- --ignored --test-threads=1
```

Unit tests live in `#[cfg(test)]` blocks inside source files. Integration tests use the `_e2e_test.rs` suffix and live with whichever crate owns the binary/library surface they exercise: `scm/main/browsectl/tests/` for everything that only needs the library, and `scm/main/bin/tests/cli_e2e_test.rs` for the one suite that spawns the `browse` binary (it needs `CARGO_BIN_EXE_browse`, which is only set within `browse`'s own package). `scm/main/bin/tests/help_snapshot_test.rs` compares every `--help` text with `scm/main/bin/tests/snapshots/`.

To target a specific browser:

```sh
CHROME_PATH=/opt/chromium/chrome cargo test --workspace --locked -- --ignored --test-threads=1
```

## Continuous integration

| Workflow | Job | Runs |
|---|---|---|
| `ci.yml` | `toolchain pins` | workflow Rust pins and `rust-toolchain.toml` equal `rust-version` |
| `ci.yml` | `docs` | relative links resolve; Rust versions in docs equal `rust-version` |
| `ci.yml` | `test` | fmt, clippy `-D warnings` and `cargo test --workspace` on ubuntu-24.04, macos-15, macos-15-intel and windows-2022 |
| `release.yml` | `build`, `draft` | on a `v*` tag: test and build four platforms, create a draft release |

This repository is public, so Actions run normally. A macOS job that is *cancelled* for runner
capacity is not a failure (see the [runbook](../7-operations/runbook.md)).

## Releasing

Follow the [release playbook](../6-deployment/playbook.md).

## Adding a new CDP method

Most CDP methods are thin wrappers around `client.send()`. To expose one as a typed helper:

1. Add the method to the `PageEvaluator` trait in `scm/main/browsectl/src/api/traits/page_evaluator.rs` as a default implementation that calls `self.evaluate(js)`.
2. If the method needs a new return type (e.g. a parsed struct), define it in `scm/main/browsectl/src/api/types/`.
3. If the method cannot be expressed as a JS expression (e.g. `Network.enable`), implement it directly on `CdpClient` in `scm/main/browsectl/src/core/client.rs` via `self.send_cdp(method, params)`.

Example — wrapping `Page.getNavigationHistory`:

```rust
// In scm/main/browsectl/src/api/traits/page_evaluator.rs
fn get_navigation_history(&self) -> Result<serde_json::Value, String>;

// In scm/main/browsectl/src/core/client.rs (CdpClient impl)
fn get_navigation_history(&self) -> Result<serde_json::Value, String> {
    self.send("Page.getNavigationHistory", serde_json::json!({}))
}
```

## Environment variables

| Variable | Effect |
|----------|--------|
| `CHROME_PATH` | Override browser binary path used by `PlatformBrowserLocator::find()`. Must exist on disk — a nonexistent path is an error, not a fallback. |
| `BROWSECTL_SESSION_DIR` | Override where the CLI's session records (written by `launch`, read by `stop`/`reap`) live. Defaults to `<tmp>/browsectl/sessions`. Tests use this to avoid touching a real machine's session state. |
| `BROWSECTL_TARGET_DIR`, `BROWSECTL_JOBS`, `BROWSECTL_DIST_DIR` | Used by `scripts/dev/build.sh` and `ship.sh`: build directory (default `.work/target`), cargo parallelism (default 2), release staging directory (default `.work/dist`). |
| `BROWSE_UPDATE_SNAPSHOTS` | When set, the help snapshot test rewrites its snapshots instead of comparing. |

## Project structure walkthrough

Each crate's library/binary source lives at the plain, Cargo-standard `src/lib.rs`/`src/main.rs` (declared explicitly in `Cargo.toml` for clarity, though these now match Cargo's own default discovery paths). `examples/<name>/src/main.rs` has one extra `src/` level beyond what Cargo auto-discovers (`examples/<name>.rs` or `examples/<name>/main.rs`), so it still needs an explicit `[[example]]` entry. `tests/` is fully auto-discovered: test files sit directly under `tests/*.rs`, no explicit `[[test]]` entries — this also matters for `arch`'s test-coverage checks (`all_methods_tested`, `test_covers_annotation`), which key off Cargo's own test-target discovery.

Two workspace members, both published: `browsectl` (the library) and `bin` (package `browsectl-bin`, the CLI — installs `browse`). `browsectl-bin` depends on `browsectl`, so it publishes second, after `browsectl` is live on crates.io. Each e2e test lives with whichever crate owns the `CARGO_BIN_EXE_*`/library surface it needs — see [`scm/README.md`](../../scm/README.md) for the top-level layout.

```
scm/
├── Cargo.toml                  Workspace root (members: main/browsectl, main/bin)
├── Cargo.lock
├── deny.toml                   cargo-deny config (cargo deny check --config deny.toml)
│
└── main/
    ├── browsectl/               Package "browsectl" — the published library
    │   ├── Cargo.toml
    │   ├── src/
    │   │   ├── lib.rs              Public surface — re-exports from api/ and saf/
    │   │   ├── api/
    │   │   │   ├── types/cdp/
    │   │   │   │   ├── cdp_client.rs          Struct definition (fields pub(crate))
    │   │   │   │   └── cdp_client_builder.rs  Builder
    │   │   │   ├── types/rect.rs              Rect data type
    │   │   │   ├── traits/page_evaluator.rs   PageEvaluator trait + default impls
    │   │   │   ├── traits/validator.rs        Validator SPI trait
    │   │   │   ├── browser/browser_locator.rs BrowserLocator trait
    │   │   │   ├── spi/browser_session.rs     BrowserSession SPI trait
    │   │   │   └── js.rs                      deep_query_selector_js, js_string_literal
    │   │   ├── core/client.rs      CdpClient impl: launch, attach, navigate,
    │   │   │                       send, WebSocket helpers, PageEvaluator impl
    │   │   ├── core/browser/
    │   │   │   └── platform_browser_locator.rs  find(), get_ws_url(), wait_for_debugger()
    │   │   └── saf/mod.rs          Public constants: DEFAULT_DEBUG_PORT, viewport presets
    │   ├── examples/launch/src/
    │   │   └── main.rs             Minimal usage example ([[example]] name = "launch")
    │   └── tests/                  Auto-discovered by Cargo — no [[test]] entries in Cargo.toml
    │       ├── client_e2e_test.rs               CdpClient lifecycle
    │       ├── page_evaluator_e2e_test.rs        PageEvaluator methods
    │       ├── rect_e2e_test.rs                  Rect helpers (offline)
    │       ├── cdp_client_builder_e2e_test.rs    Builder
    │       ├── validator_e2e_test.rs             Validator trait contract
    │       ├── browser_locator_e2e_test.rs       Browser discovery
    │       ├── cdp_client_e2e_test.rs            CdpClient API surface
    │       ├── browser_session_e2e_test.rs       BrowserSession contract
    │       └── platform_browser_locator_e2e_test.rs  Platform discovery smoke tests
    │
    └── bin/                     Package "browsectl-bin", published — builds binary `browse`
        ├── Cargo.toml           Depends on browsectl (version-pinned path dep, required to publish)
        ├── src/
        │   ├── main.rs          browse binary: pure arg dispatch, no logic
        │   ├── api/
        │   │   ├── client/      Client trait + DTOs + colocated tests
        │   │   └── session/     SessionRepository trait + DTOs + colocated tests
        │   ├── core/
        │   │   ├── client/      dispatcher.rs, error_ops.rs
        │   │   ├── session/     store.rs, error_ops.rs
        │   │   ├── help.rs      Help: the top-level and per-command help text (stdout) and usage errors (stderr)
        │   │   └── os_process.rs  ProcessLocator: caller-liveness check (tasklist/PowerShell, ps)
        │   └── commands/
        │       ├── mod.rs       Only `pub mod`/`mod`/`pub use` — no logic
        │       ├── args.rs      expect_value, parse_value
        │       ├── connection.rs    attach
        │       └── {launch,eval,...}.rs   One module per subcommand
        └── tests/               Auto-discovered by Cargo — no [[test]] entries in Cargo.toml
            ├── cli_e2e_test.rs      Every browse subcommand, end to end
            ├── help_int_test.rs     Help: print_help/print_version output, via the real CLI
            ├── help_snapshot_test.rs  Golden tests for every --help text
            └── snapshots/           The expected help texts
```

## Commit style

```
type(scope): description

feat(client): add Page.printToPDF wrapper
fix(discovery): fall back to which on Linux when path check fails
test(page_evaluator): add e2e coverage for get_pseudo_style
```

Types: `feat`, `fix`, `test`, `refactor`, `docs`, `chore`.
