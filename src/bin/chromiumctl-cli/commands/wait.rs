




    pub fn execute(args: &[String]) -> Result<(), String> {
        let mut port = 9222;
        let mut _selector = None;

        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--port" => {
                    i += 1;
                    if i < args.len() {
                        port = args[i].parse().map_err(|_| "Invalid port")?;
                    }
                }
                "--selector" => {
                    i += 1;
                    if i < args.len() {
                        _selector = Some(args[i].clone());
                    }
                }
                _ => {}
            }
            i += 1;
        }

        println!("Waiting on port {}...", port);
        Ok(())
    }
