use super::SessionRecord;

/// Output of [`crate::api::session::SessionRepository::list`].
#[derive(Debug)]
pub struct ListSessionsResponse {
    pub records: Vec<SessionRecord>,
}
