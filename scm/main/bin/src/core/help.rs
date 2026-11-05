/// Usage and version text for the `browse` CLI — a unit struct (matching
/// `ProcessLocator`/`SessionStore`'s convention elsewhere in this codebase)
/// since none of the functions carry or need any state.
///
/// Help requested explicitly (`browse help`, `--help`, `-h`, `browse <command>
/// --help`) is written to stdout and exits 0, so it can be piped to a pager.
/// Help shown because the invocation was wrong (no command, unknown command)
/// goes to stderr and exits 2.
pub(crate) struct Help;

/// Everything `browse <command> --help` prints for one command.
struct CommandHelp {
    name: &'static str,
    summary: &'static str,
    /// Arguments after `browse <name>`, e.g. `--url <URL> [--port <PORT>]`.
    usage: &'static str,
    description: &'static str,
    /// `(flag, explanation)` pairs.
    options: &'static [(&'static str, &'static str)],
    /// What a successful run prints.
    output: &'static str,
    examples: &'static [&'static str],
}

const COMMANDS: &[CommandHelp] = &[
    CommandHelp {
        name: "launch",
        summary: "Launch a headless browser and keep it running",
        usage: "--url <URL> [--port <PORT>] [--width <PX>] [--height <PX>] [--reap-stale]",
        description: "Starts a headless Chromium-based browser at --url with a remote-debugging port, then returns while the browser keeps running. Later commands attach to it with the same --port. Stop it with `browse stop`. The session is recorded so `browse reap` can clean it up if this caller dies.",
        options: &[
            ("--url <URL>", "Page to open (required)"),
            ("--port <PORT>", "Debug port (default: 9222)"),
            ("--width <PX>", "Viewport width (default: 1920)"),
            ("--height <PX>", "Viewport height (default: 1080)"),
            ("--reap-stale", "First reap other sessions whose launching process has died"),
            ("--headless", "Accepted for compatibility; the browser always runs headless"),
        ],
        output: "\"Browser launched.\" followed by the URL, port, viewport and DevTools WebSocket URL",
        examples: &[
            "browse launch --url https://example.com",
            "browse launch --url http://localhost:5173 --port 9300 --width 1280 --height 800",
        ],
    },
    CommandHelp {
        name: "eval",
        summary: "Evaluate JavaScript in the running session",
        usage: "--script <JS> [--port <PORT>] [--output text|json|yaml]",
        description: "Runs the expression in the current page and prints its result.",
        options: &[
            ("--script <JS>", "JavaScript to evaluate (required)"),
            ("--port <PORT>", "Debug port of the session (default: 9222)"),
            ("--output <FMT>", "text (default), json or yaml; any other value prints text"),
        ],
        output: "text: the raw result. json: {\"result\": <value>}. yaml: result: \"<value>\"",
        examples: &[
            "browse eval --script \"document.title\"",
            "browse eval --port 9300 --script \"document.querySelectorAll('a').length\" --output json",
        ],
    },
    CommandHelp {
        name: "screenshot",
        summary: "Capture a screenshot of the page",
        usage: "[--port <PORT>] [--output <PATH>] [--format png|jpeg|webp] [--full-page]",
        description: "Saves the current page as an image file.",
        options: &[
            ("--port <PORT>", "Debug port of the session (default: 9222)"),
            ("--output <PATH>", "File to write (default: screenshot.png)"),
            ("--format <FMT>", "Image format: png (default), jpeg or webp"),
            ("--full-page", "Capture beyond the viewport"),
        ],
        output: "\"Screenshot saved to <PATH>\"",
        examples: &[
            "browse screenshot --output page.png",
            "browse screenshot --port 9300 --format jpeg --full-page --output full.jpg",
        ],
    },
    CommandHelp {
        name: "navigate",
        summary: "Navigate the running session to a URL",
        usage: "--url <URL> [--port <PORT>]",
        description: "Loads the URL in the current page. Use `browse wait --navigation` to wait for it to finish loading.",
        options: &[
            ("--url <URL>", "Destination (required)"),
            ("--port <PORT>", "Debug port of the session (default: 9222)"),
        ],
        output: "\"Navigated to <URL>\"",
        examples: &["browse navigate --url https://example.com/login"],
    },
    CommandHelp {
        name: "wait",
        summary: "Wait for a condition on the page",
        usage: "(--selector <SEL> | --text <TEXT> | --navigation) [--timeout <SECS>] [--port <PORT>]",
        description: "Polls the page every 100 ms until the condition holds. Give one condition; if several are given, --selector wins, then --text. A selector also matches inside shadow roots.",
        options: &[
            ("--selector <SEL>", "Wait until an element matching this CSS selector exists"),
            ("--text <TEXT>", "Wait until the page text contains this string"),
            ("--navigation", "Wait until document.readyState is complete"),
            ("--timeout <SECS>", "Give up after this many whole seconds (default: 30)"),
            ("--port <PORT>", "Debug port of the session (default: 9222)"),
        ],
        output: "\"Condition met.\" Exits 3 if the condition is not met within the timeout",
        examples: &[
            "browse wait --selector \".loaded\" --timeout 10",
            "browse wait --text \"Welcome back\"",
            "browse wait --navigation",
        ],
    },
    CommandHelp {
        name: "click",
        summary: "Click an element",
        usage: "--selector <SEL> [--port <PORT>]",
        description: "Clicks the centre of the first element matching the CSS selector.",
        options: &[
            ("--selector <SEL>", "CSS selector of the element (required)"),
            ("--port <PORT>", "Debug port of the session (default: 9222)"),
        ],
        output: "\"Clicked <SEL> at (<x>, <y>)\"",
        examples: &["browse click --selector \"button.submit\""],
    },
    CommandHelp {
        name: "input",
        summary: "Type text into an input field",
        usage: "--selector <SEL> --text <TEXT> [--port <PORT>]",
        description: "Focuses the first element matching the selector (also inside shadow roots) and types the text.",
        options: &[
            ("--selector <SEL>", "CSS selector of the field (required)"),
            ("--text <TEXT>", "Text to type (required)"),
            ("--port <PORT>", "Debug port of the session (default: 9222)"),
        ],
        output: "\"Typed into <SEL>\"",
        examples: &["browse input --selector \"input#search\" --text \"hello\""],
    },
    CommandHelp {
        name: "set-files",
        summary: "Set files on an <input type=\"file\"> element",
        usage: "--selector <SEL> --files <PATHS> [--port <PORT>]",
        description: "Attaches local files to a file input without opening a file dialog.",
        options: &[
            ("--selector <SEL>", "CSS selector of the file input (required)"),
            ("--files <PATHS>", "Comma-separated file paths (required, at least one)"),
            ("--port <PORT>", "Debug port of the session (default: 9222)"),
        ],
        output: "\"Set <N> file(s) on <SEL>\"",
        examples: &["browse set-files --selector \"#file-input\" --files \"./a.png,./b.pdf\""],
    },
    CommandHelp {
        name: "get-dom",
        summary: "Export the current DOM as JSON",
        usage: "[--output <PATH>] [--port <PORT>]",
        description: "Prints the DOM of the current page as JSON, or writes it to a file.",
        options: &[
            ("--output <PATH>", "Write to this file instead of stdout"),
            ("--port <PORT>", "Debug port of the session (default: 9222)"),
        ],
        output: "The JSON document on stdout, or \"DOM exported to <PATH>\" with --output",
        examples: &["browse get-dom --output dom.json", "browse get-dom > dom.json"],
    },
    CommandHelp {
        name: "metrics",
        summary: "Get browser performance metrics",
        usage: "[--output <PATH>] [--port <PORT>]",
        description: "Prints the page's performance metrics as JSON, or writes them to a file.",
        options: &[
            ("--output <PATH>", "Write to this file instead of stdout"),
            ("--port <PORT>", "Debug port of the session (default: 9222)"),
        ],
        output: "The JSON document on stdout, or \"Metrics exported to <PATH>\" with --output",
        examples: &["browse metrics", "browse metrics --output metrics.json"],
    },
    CommandHelp {
        name: "stop",
        summary: "Terminate the browser session at --port",
        usage: "[--port <PORT>]",
        description: "Closes exactly the browser listening on --port (and its subprocesses) and forgets its recorded session.",
        options: &[("--port <PORT>", "Debug port of the session (default: 9222)")],
        output: "\"Stopped browser on port <PORT>.\"",
        examples: &["browse stop", "browse stop --port 9300"],
    },
    CommandHelp {
        name: "reap",
        summary: "Clean up sessions whose launcher has died or gone stale",
        usage: "[--dry-run] [--max-age <DURATION>]",
        description: "Closes recorded sessions whose launching process no longer exists. With --max-age it also closes sessions older than that even if their launcher is alive. Takes no --port: it looks at every recorded session.",
        options: &[
            ("--dry-run", "List what would be reaped without closing or deleting anything"),
            ("--max-age <DURATION>", "Also reap sessions older than this: seconds, or 30s, 5m, 1h"),
        ],
        output: "One line per session reaped (or \"Would reap\" with --dry-run) and a total, or \"No orphaned or stale sessions found.\"",
        examples: &["browse reap --dry-run", "browse reap --max-age 1h"],
    },
    CommandHelp {
        name: "mock",
        summary: "Answer matching requests with a fake response",
        usage: "--url-pattern <PAT> [--status <CODE>] [--body <TEXT>] [--port <PORT>]",
        description: "Intercepts requests whose URL matches the glob and answers them with the given status and body; all other requests are untouched. Blocks until you press Ctrl-C or one hour passes without a matching request.",
        options: &[
            ("--url-pattern <PAT>", "Glob of request URLs to intercept, e.g. \"*api.example.com*\" (required)"),
            ("--status <CODE>", "HTTP status to respond with (default: 200)"),
            ("--body <TEXT>", "Response body (default: empty)"),
            ("--port <PORT>", "Debug port of the session (default: 9222)"),
        ],
        output: "A summary line, then \"Mocked <URL> -> <STATUS>\" for each intercepted request",
        examples: &[
            "browse mock --url-pattern \"*api.example.com*\" --status 200 --body '{\"ok\":true}'",
        ],
    },
];

impl Help {
    pub(crate) fn print_version() {
        println!("browse {} (browsectl)", env!("CARGO_PKG_VERSION"));
    }

    /// Top-level help, requested explicitly: stdout.
    pub(crate) fn print_help() {
        print!("{}", Self::overview());
    }

    /// Top-level help shown because the invocation was wrong: stderr.
    pub(crate) fn print_usage_error() {
        eprint!("{}", Self::overview());
    }

    /// Help for one command, requested explicitly: stdout. Returns false when
    /// `name` is not a command.
    pub(crate) fn print_command_help(name: &str) -> bool {
        match Self::command_text(name) {
            Some(text) => {
                print!("{}", text);
                true
            }
            None => false,
        }
    }

    pub(crate) fn command_names() -> Vec<&'static str> {
        COMMANDS.iter().map(|c| c.name).collect()
    }

    fn overview() -> String {
        let mut out = String::new();
        out.push_str("browse — Chromium DevTools Protocol CLI\n\n");
        out.push_str("USAGE:\n    browse <COMMAND> [OPTIONS]\n    browse <COMMAND> --help\n\n");
        out.push_str("COMMANDS:\n");
        for c in COMMANDS {
            out.push_str(&format!("    {:<12} {}\n", c.name, c.summary));
        }
        out.push_str(&format!(
            "    {:<12} {}\n",
            "version", "Print the browse/browsectl version"
        ));
        out.push_str(&format!(
            "    {:<12} {}\n\n",
            "help", "Print this message, or `browse help <command>`"
        ));
        out.push_str("A typical session:\n");
        out.push_str("    browse launch --url https://example.com\n");
        out.push_str("    browse wait --selector \"h1\"\n");
        out.push_str("    browse eval --script \"document.title\"\n");
        out.push_str("    browse screenshot --output page.png\n");
        out.push_str("    browse stop\n\n");
        out.push_str(
            "Commands other than launch attach to a running session by --port (default 9222).\n",
        );
        out.push_str(
            "Run `browse <command> --help` for a command's options, output and examples.\n\n",
        );
        out.push_str("OUTPUT:\n");
        out.push_str(
            "    Results go to stdout; errors (`Error: ...`) go to stderr. Only `eval` has\n",
        );
        out.push_str(
            "    selectable formats (--output text|json|yaml); get-dom and metrics print JSON.\n\n",
        );
        out.push_str("EXIT CODES:\n");
        out.push_str("    0   Success (also for --help and --version)\n");
        out.push_str("    1   The command ran but the browser-side action failed (JS exception, element not found)\n");
        out.push_str(
            "    2   Invalid usage: no command, unknown command or option, missing or bad value\n",
        );
        out.push_str("    3   Timed out (wait)\n");
        out.push_str("    4   Could not launch or connect to the browser\n\n");
        out.push_str("REQUIREMENTS:\n");
        out.push_str(
            "    A Chromium-based browser (Chrome, Chromium, Edge or Brave), found automatically\n",
        );
        out.push_str("    or via CHROME_PATH; `curl` on PATH to find the debugger URL.\n\n");
        out.push_str("ENVIRONMENT:\n");
        out.push_str("    CHROME_PATH            Browser executable to use instead of searching\n");
        out.push_str(
            "    BROWSECTL_SESSION_DIR  Where launched sessions are recorded for `reap`\n",
        );
        out.push_str("                           (default: <temp dir>/browsectl/sessions)\n");
        out
    }

    fn command_text(name: &str) -> Option<String> {
        let c = COMMANDS.iter().find(|c| c.name == name)?;
        let mut out = format!("browse {} — {}\n\n", c.name, c.summary);
        out.push_str(&format!("USAGE:\n    browse {} {}\n\n", c.name, c.usage));
        out.push_str(&format!("{}\n\nOPTIONS:\n", c.description));
        for (flag, text) in c.options {
            out.push_str(&format!("    {:<24} {}\n", flag, text));
        }
        out.push_str("    -h, --help               Print this help\n\n");
        out.push_str(&format!("OUTPUT:\n    {}\n\n", c.output));
        out.push_str("EXAMPLES:\n");
        for example in c.examples {
            out.push_str(&format!("    {}\n", example));
        }
        out.push_str("\nEXIT CODES: 0 success, 1 action failed, 2 invalid usage, 3 timeout, 4 browser unreachable\n");
        Some(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// @covers: command_names
    #[test]
    fn test_command_names_lists_every_dispatchable_command() {
        let names = Help::command_names();
        for expected in [
            "launch",
            "eval",
            "screenshot",
            "navigate",
            "wait",
            "click",
            "input",
            "set-files",
            "get-dom",
            "metrics",
            "stop",
            "reap",
            "mock",
        ] {
            assert!(names.contains(&expected), "missing {expected}");
        }
        assert_eq!(names.len(), 13);
    }

    /// @covers: command_text
    #[test]
    fn test_command_text_unknown_command_returns_none() {
        assert!(Help::command_text("bogus").is_none());
    }

    /// @covers: command_text
    #[test]
    fn test_command_text_every_command_has_an_example_and_usage() {
        for c in COMMANDS {
            let text = Help::command_text(c.name).unwrap();
            assert!(
                text.contains("EXAMPLES:\n    browse "),
                "{} has no example",
                c.name
            );
            assert!(text.contains(&format!("USAGE:\n    browse {}", c.name)));
        }
    }
}
