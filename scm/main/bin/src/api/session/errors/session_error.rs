/// A failure from a `SessionRepository` operation — always a filesystem
/// error, since session records are plain JSON files on disk.
#[derive(Debug)]
pub struct SessionError(pub String);
