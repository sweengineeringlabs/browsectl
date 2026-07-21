pub mod dto;
pub mod errors;
pub mod traits;

pub use dto::{DispatchRequest, DispatchResponse};
pub use errors::ClientError;
pub use traits::Client;

#[cfg(test)]
mod tests;
