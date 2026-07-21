// Colocated tests for the `Client` trait contract (api/traits/client.rs)
// and its `ClientError` (api/error/client_error.rs), exercised through a
// test-double implementation, per SEA's api/ test-organization rules. The
// real `BrowseClient` implementation is exercised end-to-end (spawning the
// actual compiled binary) by cli_e2e_test.rs and help_int_test.rs instead —
// this file verifies the *contract* itself, not `browse`'s specific
// subcommand behavior.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use crate::api::{Client, ClientError, DispatchRequest, DispatchResponse};

struct FakeClient<F: Fn(&DispatchRequest) -> Result<DispatchResponse, ClientError>>(F);

impl<F: Fn(&DispatchRequest) -> Result<DispatchResponse, ClientError>> Client for FakeClient<F> {
    fn dispatch(&self, request: DispatchRequest) -> Result<DispatchResponse, ClientError> {
        (self.0)(&request)
    }
}

/// @covers: dispatch
#[test]
fn test_dispatch_forwards_args_and_returns_the_exit_code_happy() {
    let client = FakeClient(|request| {
        assert_eq!(request.args, vec!["help".to_string()]);
        Ok(DispatchResponse { exit_code: 0 })
    });
    let request = DispatchRequest { args: vec!["help".to_string()] };
    assert_eq!(client.dispatch(request).unwrap().exit_code, 0);
}

/// @covers: dispatch
#[test]
fn test_dispatch_propagates_the_client_error_error() {
    let client = FakeClient(|_| Err(ClientError::InvalidArgs("--url is required".to_string())));
    let request = DispatchRequest { args: vec!["launch".to_string()] };
    let err = client.dispatch(request).unwrap_err();
    assert!(matches!(err, ClientError::InvalidArgs(_)));
}

/// @covers: dispatch
#[test]
fn test_dispatch_accepts_empty_args_edge() {
    let client = FakeClient(|request| {
        assert!(request.args.is_empty());
        Ok(DispatchResponse { exit_code: 2 })
    });
    let request = DispatchRequest { args: vec![] };
    assert_eq!(client.dispatch(request).unwrap().exit_code, 2);
}
