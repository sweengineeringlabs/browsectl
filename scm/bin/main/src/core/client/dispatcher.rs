use crate::api::{Client, ClientError, DispatchRequest, DispatchResponse};
use crate::commands;
use crate::core::help::Help;

/// The real `Client`: routes a parsed subcommand to its `commands::` module.
pub(crate) struct BrowseClient;

impl Client for BrowseClient {
    fn dispatch(&self, request: DispatchRequest) -> Result<DispatchResponse, ClientError> {
        let args = &request.args;
        if args.is_empty() {
            Help::print_help();
            return Ok(DispatchResponse { exit_code: 2 });
        }

        let command = &args[0];
        let cmd_args = &args[1..];

        let result = match command.as_str() {
            "launch" => commands::launch::execute(cmd_args),
            "eval" => commands::eval::execute(cmd_args),
            "screenshot" => commands::screenshot::execute(cmd_args),
            "navigate" => commands::navigate::execute(cmd_args),
            "wait" => commands::wait::execute(cmd_args),
            "click" => commands::click::execute(cmd_args),
            "input" => commands::input::execute(cmd_args),
            "set-files" => commands::file_selection::execute(cmd_args),
            "get-dom" => commands::dom_snapshot::execute(cmd_args),
            "metrics" => commands::metrics::execute(cmd_args),
            "stop" => commands::stop::execute(cmd_args),
            "reap" => commands::reap::execute(cmd_args),
            "mock" => commands::mock::execute(cmd_args),
            "help" | "-h" | "--help" => {
                Help::print_help();
                return Ok(DispatchResponse { exit_code: 0 });
            }
            "version" | "-V" | "--version" => {
                Help::print_version();
                return Ok(DispatchResponse { exit_code: 0 });
            }
            _ => {
                eprintln!("Unknown command: {}", command);
                Help::print_help();
                return Ok(DispatchResponse { exit_code: 2 });
            }
        };

        result.map(|()| DispatchResponse { exit_code: 0 })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(args: &[&str]) -> DispatchRequest {
        DispatchRequest { args: args.iter().map(|s| s.to_string()).collect() }
    }

    /// @covers: dispatch
    #[test]
    fn test_dispatch_returns_2_for_empty_args() {
        assert_eq!(BrowseClient.dispatch(request(&[])).unwrap().exit_code, 2);
    }

    /// @covers: dispatch
    #[test]
    fn test_dispatch_returns_2_for_unknown_command() {
        assert_eq!(BrowseClient.dispatch(request(&["bogus"])).unwrap().exit_code, 2);
    }

    /// @covers: dispatch
    #[test]
    fn test_dispatch_returns_0_for_help() {
        assert_eq!(BrowseClient.dispatch(request(&["help"])).unwrap().exit_code, 0);
    }

    /// @covers: dispatch
    #[test]
    fn test_dispatch_returns_0_for_version() {
        assert_eq!(BrowseClient.dispatch(request(&["version"])).unwrap().exit_code, 0);
    }

    /// @covers: dispatch
    #[test]
    fn test_dispatch_returns_invalid_args_error_for_launch_without_url() {
        let err = BrowseClient.dispatch(request(&["launch"])).unwrap_err();
        assert_eq!(err.exit_code(), 2);
    }
}
