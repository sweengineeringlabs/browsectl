// Colocated tests for the `SessionRepository` trait contract
// (api/session/traits/session_repository.rs) and its DTOs/error type, per
// SEA's api/ test-organization rules. The real `SessionStore`
// implementation is exercised by its own inline tests in
// core/session/store.rs, which hit the real filesystem through the
// `*_in(dir, ..)` helpers — this file verifies the *contract* itself.
//
// Wrapped in `#[cfg(test)] mod tests { ... }` (rather than bare top-level
// items) so the whole body — including the test-double's `impl
// SessionRepository for FakeSessionRepository` and its `#[test] fn`s — is
// recognized as test code by tooling that strips `#[cfg(test)]` sections,
// the same convention already used by every other test module in this
// crate (e.g. core/session/store.rs). The test-double itself uses plain
// `fn` pointer fields (every closure here is non-capturing) rather than
// `Box<dyn Fn(..) + 'static>` — simpler, and it needs no generic builder
// methods to construct one per scenario.

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use crate::api::session::{
        DeleteSessionRequest, ListSessionsRequest, ListSessionsResponse, SessionError,
        SessionRecord, SessionRepository, WriteSessionRequest,
    };

    struct FakeSessionRepository {
        write: fn(WriteSessionRequest) -> Result<(), SessionError>,
        delete: fn(DeleteSessionRequest) -> Result<(), SessionError>,
        list: fn(ListSessionsRequest) -> Result<ListSessionsResponse, SessionError>,
    }

    impl SessionRepository for FakeSessionRepository {
        fn write(&self, request: WriteSessionRequest) -> Result<(), SessionError> {
            (self.write)(request)
        }
        fn delete(&self, request: DeleteSessionRequest) -> Result<(), SessionError> {
            (self.delete)(request)
        }
        fn list(&self, request: ListSessionsRequest) -> Result<ListSessionsResponse, SessionError> {
            (self.list)(request)
        }
    }

    fn unreachable_write(_: WriteSessionRequest) -> Result<(), SessionError> {
        unreachable!("write not exercised by this test")
    }

    fn unreachable_delete(_: DeleteSessionRequest) -> Result<(), SessionError> {
        unreachable!("delete not exercised by this test")
    }

    fn unreachable_list(_: ListSessionsRequest) -> Result<ListSessionsResponse, SessionError> {
        unreachable!("list not exercised by this test")
    }

    fn sample_record() -> SessionRecord {
        SessionRecord { port: 9222, launched_at: 1_752_700_000, caller_pid: 4242, caller_start_time: None }
    }

    /// @covers: write
    #[test]
    fn test_write_stores_the_given_record_happy() {
        let repo = FakeSessionRepository {
            write: |request| {
                assert_eq!(request.record, sample_record());
                Ok(())
            },
            delete: unreachable_delete,
            list: unreachable_list,
        };
        repo.write(WriteSessionRequest { record: sample_record() }).unwrap();
    }

    /// @covers: write
    #[test]
    fn test_write_propagates_a_storage_failure_error() {
        let repo = FakeSessionRepository {
            write: |_| Err(SessionError("disk full".to_string())),
            delete: unreachable_delete,
            list: unreachable_list,
        };
        let err = repo.write(WriteSessionRequest { record: sample_record() }).unwrap_err();
        assert_eq!(err.to_string(), "disk full");
    }

    /// @covers: write
    #[test]
    fn test_write_accepts_a_record_with_no_fingerprint_edge() {
        let repo = FakeSessionRepository {
            write: |request| {
                assert!(request.record.caller_start_time.is_none());
                Ok(())
            },
            delete: unreachable_delete,
            list: unreachable_list,
        };
        repo.write(WriteSessionRequest { record: sample_record() }).unwrap();
    }

    /// @covers: delete
    #[test]
    fn test_delete_removes_the_record_for_the_given_port_happy() {
        let repo = FakeSessionRepository {
            write: unreachable_write,
            delete: |request| {
                assert_eq!(request.port, 9222);
                Ok(())
            },
            list: unreachable_list,
        };
        repo.delete(DeleteSessionRequest { port: 9222 }).unwrap();
    }

    /// @covers: delete
    #[test]
    fn test_delete_propagates_a_storage_failure_error() {
        let repo = FakeSessionRepository {
            write: unreachable_write,
            delete: |_| Err(SessionError("permission denied".to_string())),
            list: unreachable_list,
        };
        let err = repo.delete(DeleteSessionRequest { port: 9222 }).unwrap_err();
        assert_eq!(err.to_string(), "permission denied");
    }

    /// @covers: delete
    #[test]
    fn test_delete_of_a_nonexistent_port_is_not_an_error_edge() {
        let repo = FakeSessionRepository { write: unreachable_write, delete: |_| Ok(()), list: unreachable_list };
        repo.delete(DeleteSessionRequest { port: 0 }).unwrap();
    }

    /// @covers: list
    #[test]
    fn test_list_returns_every_stored_record_happy() {
        let repo = FakeSessionRepository {
            write: unreachable_write,
            delete: unreachable_delete,
            list: |_| Ok(ListSessionsResponse { records: vec![sample_record()] }),
        };
        let response = repo.list(ListSessionsRequest).unwrap();
        assert_eq!(response.records, vec![sample_record()]);
    }

    /// @covers: list
    #[test]
    fn test_list_propagates_a_storage_failure_error() {
        let repo = FakeSessionRepository {
            write: unreachable_write,
            delete: unreachable_delete,
            list: |_| Err(SessionError("dir unreadable".to_string())),
        };
        let err = repo.list(ListSessionsRequest).unwrap_err();
        assert_eq!(err.to_string(), "dir unreadable");
    }

    /// @covers: list
    #[test]
    fn test_list_returns_an_empty_vec_when_nothing_is_stored_edge() {
        let repo = FakeSessionRepository {
            write: unreachable_write,
            delete: unreachable_delete,
            list: |_| Ok(ListSessionsResponse { records: vec![] }),
        };
        let response = repo.list(ListSessionsRequest).unwrap();
        assert!(response.records.is_empty());
    }
}
