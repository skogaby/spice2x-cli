use thiserror::Error;

/// Errors that can occur during SpiceAPI protocol communication.
#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("connection failed: {0}")]
    Connection(#[from] std::io::Error),

    #[error("API error: {0}")]
    Api(String),

    #[error("malformed response: server returned empty payload")]
    MalformedResponse,

    #[error("response ID mismatch: expected {expected}, got {actual}")]
    IdMismatch { expected: u64, actual: u64 },

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("request failed after {attempts} attempts: {source}")]
    RetriesExhausted {
        attempts: u32,
        source: Box<ProtocolError>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_error_displays_message() {
        let err = ProtocolError::Api("not found".into());
        assert_eq!(err.to_string(), "API error: not found");
    }

    #[test]
    fn id_mismatch_displays_ids() {
        let err = ProtocolError::IdMismatch { expected: 1, actual: 2 };
        assert_eq!(err.to_string(), "response ID mismatch: expected 1, got 2");
    }

    #[test]
    fn retries_exhausted_displays_count() {
        let inner = ProtocolError::MalformedResponse;
        let err = ProtocolError::RetriesExhausted {
            attempts: 3,
            source: Box::new(inner),
        };
        assert!(err.to_string().contains("3 attempts"));
    }
}
