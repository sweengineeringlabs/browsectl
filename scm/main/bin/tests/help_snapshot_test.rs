//! Golden tests for `browse --help` and each command's `--help`. A help text
//! can only change together with its snapshot, so a change to documented
//! behaviour is always a visible diff.
//!
//! To accept intended changes:
//! `BROWSE_UPDATE_SNAPSHOTS=1 cargo test -p browsectl-bin --test help_snapshot_test`
//! and review the diff in `tests/snapshots/`.
#![allow(clippy::unwrap_used, clippy::expect_used)]
use std::path::PathBuf;
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_browse");

const COMMANDS: &[&str] = &[
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
];

fn run(args: &[&str]) -> Output {
    Command::new(BIN).args(args).output().unwrap()
}

/// Stdout with the platform's line endings and `.exe` suffix normalised.
fn stdout_of(args: &[&str]) -> String {
    let output = run(args);
    assert!(output.status.success(), "browse {args:?} must exit 0");
    assert!(
        output.stderr.is_empty(),
        "explicit help must not write to stderr, got: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .unwrap()
        .replace("browse.exe", "browse")
        .replace("\r\n", "\n")
}

fn check_snapshot(name: &str, args: &[&str]) {
    let actual = stdout_of(args);
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/snapshots")
        .join(format!("{name}.txt"));
    if std::env::var_os("BROWSE_UPDATE_SNAPSHOTS").is_some() {
        std::fs::write(&path, &actual).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| {
            panic!(
                "missing snapshot {}; run with BROWSE_UPDATE_SNAPSHOTS=1",
                path.display()
            )
        })
        .replace("\r\n", "\n");
    assert_eq!(
        actual,
        expected,
        "`browse {}` help changed; if intended, regenerate with BROWSE_UPDATE_SNAPSHOTS=1",
        args.join(" ")
    );
}

#[test]
fn test_help_top_level_matches_snapshot() {
    check_snapshot("overview", &["--help"]);
}

#[test]
fn test_help_every_command_matches_its_snapshot() {
    for command in COMMANDS {
        check_snapshot(command, &[command, "--help"]);
    }
}

#[test]
fn test_help_spellings_print_the_same_overview() {
    let reference = stdout_of(&["--help"]);
    for args in [&["-h"][..], &["help"][..]] {
        assert_eq!(stdout_of(args), reference, "browse {args:?}");
    }
}

#[test]
fn test_help_command_forms_print_the_same_text() {
    for command in COMMANDS {
        let reference = stdout_of(&[command, "--help"]);
        assert_eq!(stdout_of(&[command, "-h"]), reference, "{command} -h");
        assert_eq!(stdout_of(&["help", command]), reference, "help {command}");
    }
}

#[test]
fn test_help_every_command_has_an_example_and_names_its_exit_codes() {
    for command in COMMANDS {
        let text = stdout_of(&[command, "--help"]);
        assert!(
            text.contains(&format!("browse {command}")),
            "{command}: no usage"
        );
        assert!(
            text.contains("EXAMPLES:\n    browse "),
            "{command}: no example"
        );
        assert!(text.contains("EXIT CODES:"), "{command}: no exit codes");
    }
}

#[test]
fn test_help_overview_documents_exit_codes_requirements_and_environment() {
    let text = stdout_of(&["--help"]);
    for needle in [
        "EXIT CODES:",
        "REQUIREMENTS:",
        "CHROME_PATH",
        "BROWSECTL_SESSION_DIR",
        "curl",
    ] {
        assert!(text.contains(needle), "overview must mention {needle}");
    }
}

#[test]
fn test_version_flag_and_subcommand_exit_zero() {
    for args in [&["--version"][..], &["-V"][..], &["version"][..]] {
        let output = run(args);
        assert_eq!(output.status.code(), Some(0), "{args:?}");
        assert!(String::from_utf8_lossy(&output.stdout)
            .starts_with(&format!("browse {} (browsectl)", env!("CARGO_PKG_VERSION"))));
    }
}

#[test]
fn test_unknown_command_exits_two_and_names_valid_commands_on_stderr() {
    let output = run(&["bogus"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unknown command: bogus"), "{stderr}");
    for command in COMMANDS {
        assert!(
            stderr.contains(command),
            "valid command {command} not named: {stderr}"
        );
    }
}

#[test]
fn test_help_for_unknown_command_exits_two_and_names_valid_commands() {
    let output = run(&["help", "bogus"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("Valid commands:"));
}

#[test]
fn test_unknown_option_exits_two_and_names_the_option() {
    let output = run(&["click", "--bogus"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown option: --bogus"));
}

#[test]
fn test_no_arguments_prints_usage_to_stderr_and_exits_two() {
    let output = run(&[]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("USAGE:"));
}
