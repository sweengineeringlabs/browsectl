




    pub fn execute(args: &[String]) -> Result<(), String> {
        let mut port = 9222;
        let mut output = "screenshot.png".to_string();

        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--port" => {
                    i += 1;
                    if i < args.len() {
                        port = args[i].parse().map_err(|_| "Invalid port")?;
                    }
                }
                "--output" => {
                    i += 1;
                    if i < args.len() {
                        output = args[i].clone();
                    }
                }
                _ => {}
            }
            i += 1;
        }

        println!("Capturing screenshot on port {}...", port);
        println!("Output: {}", output);
        Ok(())
    }
