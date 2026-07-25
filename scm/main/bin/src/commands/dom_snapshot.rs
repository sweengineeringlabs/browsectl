use super::{attach, expect_value, parse_value, CliError};

pub fn execute(args: &[String]) -> Result<(), CliError> {
    let mut port: Option<u16> = None;
    let mut output: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--port" => {
                i += 1;
                port = Some(parse_value(args, i, "--port")?);
            }
            "--output" => {
                i += 1;
                output = Some(expect_value(args, i, "--output")?);
            }
            other => return Err(CliError::InvalidArgs(format!("unknown option: {}", other))),
        }
        i += 1;
    }

    let client = attach(port)?;

    let dom = client
        .send("DOM.getDocument", serde_json::json!({ "depth": -1, "pierce": true }))
        .map_err(CliError::ExecutionFailed)?;

    let json = serde_json::to_string_pretty(&dom)
        .map_err(|e| CliError::ExecutionFailed(format!("failed to serialize DOM: {}", e)))?;

    match output {
        Some(path) => {
            std::fs::write(&path, &json)
                .map_err(|e| CliError::ExecutionFailed(format!("failed to write '{}': {}", path, e)))?;
            println!("DOM exported to {}", path);
        }
        None => println!("{}", json),
    }

    Ok(())
}
