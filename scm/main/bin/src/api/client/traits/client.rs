use crate::api::client::dto::{DispatchRequest, DispatchResponse};
use crate::api::client::errors::ClientError;

/// The CLI's dispatch contract: takes the process's argument list (with the
/// program name already stripped) and executes the matching subcommand.
pub trait Client {
    /// Executes `request.args` and returns the process exit code on
    /// success, or a `ClientError` (itself mapped to an exit code) on
    /// failure.
    fn dispatch(&self, request: DispatchRequest) -> Result<DispatchResponse, ClientError>;
}
