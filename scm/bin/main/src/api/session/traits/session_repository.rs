use crate::api::session::dto::{
    DeleteSessionRequest, ListSessionsRequest, ListSessionsResponse, WriteSessionRequest,
};
use crate::api::session::errors::SessionError;

/// Persists [`crate::api::session::dto::SessionRecord`]s to durable storage
/// so `reap`/`stop` can find a `launch`ed session across process boundaries.
pub trait SessionRepository {
    /// Write (or overwrite) `request.record`, keyed by its port.
    fn write(&self, request: WriteSessionRequest) -> Result<(), SessionError>;

    /// Delete the record for `request.port`, if one exists. A missing
    /// record is not an error.
    fn delete(&self, request: DeleteSessionRequest) -> Result<(), SessionError>;

    /// Read every session record currently in storage. Unreadable/corrupt
    /// entries are skipped rather than failing the whole scan.
    fn list(&self, request: ListSessionsRequest) -> Result<ListSessionsResponse, SessionError>;
}
