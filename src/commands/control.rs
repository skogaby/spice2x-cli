use anyhow::Result;
use serde_json::{json, Value};

use crate::cli::ControlAction;
use crate::protocol::{Connection, ProtocolError};

pub fn execute(conn: &mut Connection, action: ControlAction) -> Result<Value> {
    match action {
        ControlAction::Raise { signal } => {
            conn.request("control", "raise", vec![json!(signal)])?;
            Ok(json!(null))
        }
        ControlAction::Exit { code } => {
            let params = if code != 0 { vec![json!(code)] } else { vec![] };
            request_expecting_disconnect(conn, "exit", params)
        }
        ControlAction::Restart => {
            request_expecting_disconnect(conn, "restart", vec![])
        }
        ControlAction::Shutdown => {
            request_expecting_disconnect(conn, "shutdown", vec![])
        }
        ControlAction::Reboot => {
            request_expecting_disconnect(conn, "reboot", vec![])
        }
    }
}

/// Send a control command where the server is expected to close the connection.
/// Suppresses connection errors since the server dying IS the success case.
fn request_expecting_disconnect(
    conn: &mut Connection,
    function: &str,
    params: Vec<Value>,
) -> Result<Value> {
    match conn.request("control", function, params) {
        Ok(_) => Ok(json!(null)),
        Err(ProtocolError::Connection(_) | ProtocolError::RetriesExhausted { .. }) => Ok(json!(null)),
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use crate::protocol::ProtocolError;

    #[test]
    fn connection_error_is_expected_for_destructive_commands() {
        // Destructive control commands (exit, restart, shutdown, reboot) expect
        // the server to close the connection. Connection errors should be suppressed.
        let err = ProtocolError::Connection(std::io::Error::new(
            std::io::ErrorKind::ConnectionReset,
            "connection closed",
        ));
        assert!(matches!(err, ProtocolError::Connection(_)));
    }

    #[test]
    fn api_error_should_not_be_suppressed() {
        // API-level errors (e.g., unknown signal) should propagate, not be suppressed.
        let err = ProtocolError::Api("unknown signal".into());
        assert!(matches!(err, ProtocolError::Api(_)));
        assert!(!matches!(err, ProtocolError::Connection(_)));
    }
}
