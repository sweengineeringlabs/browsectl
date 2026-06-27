pub fn execute(args: &[String]) -> Result<(), String> {
    let mut port = 9222;
    let mut script = None;
    let mut output_format = "text";

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--port" => {
                i += 1;
                if i < args.len() {
                    port = args[i].parse().map_err(|_| "Invalid port")?;
                }
            }
            "--script" => {
                i += 1;
                if i < args.len() {
                    script = Some(args[i].clone());
                }
            }
            "--output" => {
                i += 1;
                if i < args.len() {
                    output_format = &args[i];
                }
            }
            _ => {}
        }
        i += 1;
    }

    let _script = script.ok_or("--script is required")?;
    println!("Evaluating script on port {}...", port);
    println!("Output format: {}", output_format);
    Ok(())
}
