use super::CliError;

/// Attach to a running session via `--port` (default 9222).
pub fn attach(port: Option<u16>) -> Result<browsectl::CdpClient, CliError> {
    browsectl::CdpClient::attach(port.unwrap_or(9222)).map_err(CliError::ConnectionFailed)
}
