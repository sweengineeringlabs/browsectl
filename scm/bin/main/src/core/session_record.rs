use serde::{Deserialize, Serialize};

/// A record of one `launch`ed session, written so `reap` can find and clean
/// up the browser if its caller dies before calling `stop`.
///
/// `caller_pid` is the PID of `launch`'s *parent* process, not `launch`
/// itself — `launch` always exits immediately after writing this record, so
/// its own PID is never a usable liveness signal. `caller_start_time` is a
/// best-effort fingerprint (see
/// [`crate::core::os_process::ProcessLocator::start_time_fingerprint`]) used
/// to catch the case where `caller_pid` has since been reassigned to an
/// unrelated process by the OS — `None` when the fingerprint couldn't be
/// captured.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SessionRecord {
    pub(crate) port: u16,
    pub(crate) launched_at: u64,
    pub(crate) caller_pid: u32,
    pub(crate) caller_start_time: Option<String>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_session_record_round_trips_through_json_with_a_fingerprint() {
        let record = SessionRecord {
            port: 9222,
            launched_at: 1_752_700_000,
            caller_pid: 4242,
            caller_start_time: Some("20260716211523.123456+120".to_string()),
        };
        let json = serde_json::to_string(&record).unwrap();
        let round_tripped: SessionRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(round_tripped, record, "every field must survive a JSON round trip unchanged");
    }

    #[test]
    fn test_session_record_round_trips_without_a_fingerprint() {
        // caller_start_time is None on platforms/paths where the fingerprint
        // couldn't be captured — must serialize as JSON null, not be omitted
        // or cause an error, and must deserialize back to exactly None.
        let record = SessionRecord {
            port: 9223,
            launched_at: 1_752_700_001,
            caller_pid: 4243,
            caller_start_time: None,
        };
        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("\"caller_start_time\":null"), "None must serialize as JSON null, got: {}", json);
        let round_tripped: SessionRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(round_tripped, record);
    }
}
