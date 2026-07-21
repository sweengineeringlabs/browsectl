use super::SessionRecord;

/// Input for [`crate::api::session::SessionRepository::write`].
pub struct WriteSessionRequest {
    pub record: SessionRecord,
}
