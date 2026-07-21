use crate::api::ClientError;

impl ClientError {
    pub fn exit_code(&self) -> i32 {
        match self {
            ClientError::ExecutionFailed(_) => 1,
            ClientError::InvalidArgs(_) => 2,
            ClientError::Timeout(_) => 3,
            ClientError::ConnectionFailed(_) => 4,
        }
    }
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            ClientError::ExecutionFailed(m) => m,
            ClientError::InvalidArgs(m) => m,
            ClientError::Timeout(m) => m,
            ClientError::ConnectionFailed(m) => m,
        };
        write!(f, "{}", msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// @covers: exit_code
    #[test]
    fn test_exit_code_maps_every_variant_to_its_rfc_0001_code() {
        assert_eq!(ClientError::ExecutionFailed("x".to_string()).exit_code(), 1);
        assert_eq!(ClientError::InvalidArgs("x".to_string()).exit_code(), 2);
        assert_eq!(ClientError::Timeout("x".to_string()).exit_code(), 3);
        assert_eq!(ClientError::ConnectionFailed("x".to_string()).exit_code(), 4);
    }

    /// @covers: fmt
    #[test]
    fn test_display_renders_the_inner_message() {
        let err = ClientError::ExecutionFailed("boom".to_string());
        assert_eq!(err.to_string(), "boom");
    }
}
