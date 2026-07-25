/// Errors surfaced by the `Client::dispatch` contract, mapped to RFC-0001's
/// exit codes. `api/` is a pure declaration layer — `exit_code()` and the
/// `Display` impl live in `core/client_error_ops.rs`, not here.
#[derive(Debug)]
pub enum ClientError {
    /// Exit code 1 — the command ran but the browser-side action failed
    /// (JS exception, element not found, bad CDP response).
    ExecutionFailed(String),
    /// Exit code 2 — invalid or missing command-line arguments.
    InvalidArgs(String),
    /// Exit code 3 — the operation did not complete within its timeout.
    Timeout(String),
    /// Exit code 4 — could not connect to (or launch) the browser's debugger.
    ConnectionFailed(String),
}
