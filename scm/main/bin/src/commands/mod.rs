pub mod click;
pub mod dom_snapshot;
pub mod eval;
pub mod file_selection;
pub mod input;
pub mod launch;
pub mod metrics;
pub mod mock;
pub mod navigate;
pub mod reap;
pub mod screenshot;
pub mod stop;
pub mod wait;

mod args;
mod connection;

pub use args::{expect_value, parse_value};
pub use connection::attach;
// `CliError` is a local alias for the real, public `api::ClientError` — kept
// so every subcommand module's existing `use super::CliError` still resolves
// without a crate-wide rename; the type itself lives in api/ since it's the
// `Client` trait's contract type, not a `commands`-internal detail.
pub(crate) use crate::api::ClientError as CliError;
