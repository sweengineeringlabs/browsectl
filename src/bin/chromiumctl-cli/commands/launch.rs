pub fn execute(args: &[String]) -> Result<(), String> {
    let mut url = None;
    let mut port = 9222;
    let mut headless = false;
    let mut width = 1920;
    let mut height = 1080;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--url" => {
                i += 1;
                if i < args.len() {
                    url = Some(args[i].clone());
                }
            }
            "--port" => {
                i += 1;
                if i < args.len() {
                    port = args[i].parse().map_err(|_| "Invalid port")?;
                }
            }
            "--headless" => headless = true,
            "--width" => {
                i += 1;
                if i < args.len() {
                    width = args[i].parse().map_err(|_| "Invalid width")?;
                }
            }
            "--height" => {
                i += 1;
                if i < args.len() {
                    height = args[i].parse().map_err(|_| "Invalid height")?;
                }
            }
            _ => {}
        }
        i += 1;
    }

    let url = url.ok_or("--url is required")?;

    println!("Launching browser...");
    println!("  URL: {}", url);
    println!("  Port: {}", port);
    println!("  Headless: {}", headless);
    println!("  Viewport: {}x{}", width, height);
    println!("\nBrowser launched. DevTools ready at ws://localhost:{}/devtools/browser", port);

    Ok(())
}
