use serde::Deserialize;
use serde_json::Value;

use super::error::ProtocolError;

/// A SpiceAPI response message.
#[derive(Debug, Deserialize)]
pub struct Response {
    id: u64,
    errors: Vec<String>,
    data: Vec<Value>,
}

impl Response {
    /// Parse a response from JSON bytes, validating the expected request ID.
    ///
    /// Returns `ProtocolError::Api` if the server reported errors.
    /// Returns `ProtocolError::IdMismatch` if the response ID doesn't match.
    pub fn parse(json_bytes: &[u8], expected_id: u64) -> Result<Self, ProtocolError> {
        let resp: Self = serde_json::from_slice(json_bytes)?;

        if !resp.errors.is_empty() {
            return Err(ProtocolError::Api(resp.errors.join("\n")));
        }

        if resp.id != expected_id {
            return Err(ProtocolError::IdMismatch {
                expected: expected_id,
                actual: resp.id,
            });
        }

        Ok(resp)
    }

    pub fn data(&self) -> &[Value] {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_response() {
        let json = br#"{"id":1,"errors":[],"data":["hello",42]}"#;
        let resp = Response::parse(json, 1).unwrap();
        assert_eq!(resp.data().len(), 2);
        assert_eq!(resp.data()[0], "hello");
        assert_eq!(resp.data()[1], 42);
    }

    #[test]
    fn returns_api_error_on_nonempty_errors() {
        let json = br#"{"id":1,"errors":["not found","bad input"],"data":[]}"#;
        let err = Response::parse(json, 1).unwrap_err();
        match err {
            ProtocolError::Api(msg) => assert!(msg.contains("not found")),
            other => panic!("expected Api error, got: {other}"),
        }
    }

    #[test]
    fn returns_id_mismatch() {
        let json = br#"{"id":99,"errors":[],"data":[]}"#;
        let err = Response::parse(json, 1).unwrap_err();
        match err {
            ProtocolError::IdMismatch { expected, actual } => {
                assert_eq!(expected, 1);
                assert_eq!(actual, 99);
            }
            other => panic!("expected IdMismatch, got: {other}"),
        }
    }

    #[test]
    fn returns_json_error_on_invalid_json() {
        let err = Response::parse(b"not json", 1).unwrap_err();
        assert!(matches!(err, ProtocolError::Json(_)));
    }
}
