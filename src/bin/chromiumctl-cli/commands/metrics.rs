




    pub fn execute(args: &[String]) -> Result<(), String> {
        let mut port = 9222;

        let mut i = 0;
        while i < args.len() {
            if args[i] == "--port" {
                i += 1;
                if i < args.len() {
                    port = args[i].parse().map_err(|_| "Invalid port")?;
                }
            }
            i += 1;
        }

        println!("Fetching metrics from port {}...", port);
        Ok(())
    }
