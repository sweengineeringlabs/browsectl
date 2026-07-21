use crate::api::session::SessionError;

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// @covers: fmt
    #[test]
    fn test_display_renders_the_inner_message() {
        let err = SessionError("disk full".to_string());
        assert_eq!(err.to_string(), "disk full");
    }
}
