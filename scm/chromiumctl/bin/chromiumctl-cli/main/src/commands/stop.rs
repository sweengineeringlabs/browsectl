use super::{attach, parse_value, expect_value, validate_connect_args, CliError};

pub fn execute(args: &[String]) -> Result<(), CliError> {
    let mut port: Option<u16> = None;
    let mut package: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--port" => {
                i += 1;
                port = Some(parse_value(args, i, "--port")?);
            }
            "--package" => {
                i += 1;
                package = Some(expect_value(args, i, "--package")?);
            }
            other => return Err(CliError::InvalidArgs(format!("unknown option: {}", other))),
        }
        i += 1;
    }
    validate_connect_args(port, &package)?;

    let client = attach(port, package.as_deref())?;
    let target_port = client.port();

    // `Browser.close` tells Chromium to terminate itself and its whole process
    // tree (including any renderer subprocesses), which is the only reliable
    // way to end a session on Windows: the OS PID `launch` spawns is a
    // launcher stub that re-execs and exits immediately, so the real browser
    // process is never something a caller could `taskkill`/`kill` directly.
    // Chromium may drop the WebSocket before acking the command, so a closed
    // connection here is expected success, not a failure to report.
    match client.send("Browser.close", serde_json::json!({})) {
        Ok(_) => {}
        Err(e) if e.contains("connection closed") => {}
        Err(e) => return Err(CliError::ExecutionFailed(e)),
    }

    println!("Stopped browser on port {}.", target_port);
    Ok(())
}
