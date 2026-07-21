// Integration tests for `core/help.rs` (the `Help` type's `print_help` /
// `print_version` output, exercised through the real CLI dispatch path).
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::process::Command;

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_browse"))
}

/// @covers: help
#[test]
fn test_help_lists_every_subcommand() {
    let output = cli().arg("help").output().unwrap();
    assert!(output.status.success(), "help must exit 0");
    let stderr = String::from_utf8_lossy(&output.stderr);
    for subcommand in [
        "launch", "eval", "screenshot", "navigate", "wait", "click", "input", "set-files",
        "get-dom", "metrics", "stop", "reap", "mock", "version", "help",
    ] {
        assert!(stderr.contains(subcommand), "help output must list '{}', got:\n{}", subcommand, stderr);
    }
    assert!(
        !stderr.to_lowercase().contains("chromiumctl"),
        "help output must not reference the old project name, got:\n{}",
        stderr
    );
}

/// @covers: help
#[test]
fn test_no_args_prints_help_and_exits_2() {
    let output = cli().output().unwrap();
    assert_eq!(output.status.code(), Some(2), "no subcommand at all must exit 2, same as an unknown command");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("USAGE"),
        "running with no arguments must print the same help text as `help`"
    );
}

/// @covers: version
#[test]
fn test_version_prints_the_crate_version() {
    let output = cli().arg("version").output().unwrap();
    assert!(output.status.success(), "version must exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(env!("CARGO_PKG_VERSION")),
        "version output must contain the actual crate version, got: {}",
        stdout
    );
    assert!(stdout.contains("browsectl"), "version output must name the browsectl crate, got: {}", stdout);
}

/// @covers: version
#[test]
fn test_version_flag_aliases_match_version_subcommand() {
    let subcommand = cli().arg("version").output().unwrap();
    for flag in ["-V", "--version"] {
        let via_flag = cli().arg(flag).output().unwrap();
        assert_eq!(
            via_flag.stdout, subcommand.stdout,
            "`browse {}` must print exactly what `browse version` prints",
            flag
        );
    }
}
