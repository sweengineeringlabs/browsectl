use std::env;
use std::process;

mod commands;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        process::exit(2);
    }

    let command = &args[1];
    let cmd_args = &args[2..];

    let result = match command.as_str() {
        "launch" => commands::launch::execute(cmd_args),
        "eval" => commands::eval::execute(cmd_args),
        "screenshot" => commands::screenshot::execute(cmd_args),
        "navigate" => commands::navigate::execute(cmd_args),
        "wait" => commands::wait::execute(cmd_args),
        "click" => commands::click::execute(cmd_args),
        "input" => commands::input::execute(cmd_args),
        "get-dom" => commands::get_dom::execute(cmd_args),
        "metrics" => commands::metrics::execute(cmd_args),
        "help" | "-h" | "--help" => {
            print_help();
            Ok(())
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_help();
            process::exit(2);
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn print_help() {
    eprintln!("chromiumctl — Chromium DevTools Protocol CLI\n");
    eprintln!("USAGE:\n");
    eprintln!("    chromiumctl <COMMAND> [OPTIONS]\n");
    eprintln!("COMMANDS:\n");
    eprintln!("    launch       Launch headless browser and keep alive");
    eprintln!("    eval         Evaluate JavaScript in running session");
    eprintln!("    screenshot   Capture page screenshot");
    eprintln!("    navigate     Navigate to URL in running session");
    eprintln!("    wait         Wait for condition (selector, text, navigation)");
    eprintln!("    click        Click element on page");
    eprintln!("    input        Type text into input field");
    eprintln!("    get-dom      Export current DOM as JSON");
    eprintln!("    metrics      Get performance metrics");
    eprintln!("    help         Print this message\n");
    eprintln!("OPTIONS:\n");
    eprintln!("    --url <URL>           Target URL");
    eprintln!("    --port <PORT>         Debug port (default: 9222)");
    eprintln!("    --output <FILE>       Output file path");
    eprintln!("    --format <FORMAT>     Output format: json, yaml, text (default: text)");
    eprintln!("    --timeout <SECS>      Operation timeout (default: 30)");
    eprintln!("    --headless            Run in headless mode");
    eprintln!("    -v, --verbose         Verbose logging\n");
    eprintln!("EXAMPLES:\n");
    eprintln!("    chromiumctl launch --url https://example.com --headless\n");
    eprintln!("    chromiumctl eval --port 9222 --script \"document.title\"\n");
    eprintln!("    chromiumctl screenshot --port 9222 --output page.png\n");
}
