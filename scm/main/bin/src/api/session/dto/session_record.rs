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
pub struct SessionRecord {
    pub port: u16,
    pub launched_at: u64,
    pub caller_pid: u32,
    pub caller_start_time: Option<String>,
}
