




    pub fn execute(args: &[String]) -> Result<(), String> {
        let mut port = 9222;
        let mut url = None;

        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--port" => {
                    i += 1;
                    if i < args.len() {
                        port = args[i].parse().map_err(|_| "Invalid port")?;
                    }
                }
                "--url" => {
                    i += 1;
                    if i < args.len() {
                        url = Some(args[i].clone());
                    }
                }
                _ => {}
            }
            i += 1;
        }

        let _url = url.ok_or("--url is required")?;
        println!("Navigating on port {}...", port);
        Ok(())
    }
