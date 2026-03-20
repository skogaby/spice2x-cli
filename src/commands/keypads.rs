use anyhow::Result;
use serde_json::{json, Value};

use crate::cli::KeypadsAction;
use crate::protocol::Connection;

pub fn execute(conn: &mut Connection, action: KeypadsAction) -> Result<Value> {
    match action {
        KeypadsAction::Get { keypad } => {
            let data = conn.request("keypads", "get", vec![json!(keypad)])?;
            Ok(Value::Array(data))
        }
        KeypadsAction::Write { keypad, input } => {
            conn.request("keypads", "write", vec![json!(keypad), json!(input)])?;
            Ok(json!(null))
        }
        KeypadsAction::Set { keypad, keys } => {
            let mut params: Vec<Value> = vec![json!(keypad)];
            params.extend(keys.into_iter().map(|k| json!(k)));
            conn.request("keypads", "set", params)?;
            Ok(json!(null))
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn get_response_is_array_of_pressed_keys() {
        // Server returns one data element per pressed key (single-char strings)
        let data = vec![json!("1"), json!("3"), json!("A")];
        assert_eq!(data.len(), 3);
        assert_eq!(data[0], "1");
    }

    #[test]
    fn set_param_format() {
        let keypad = 0u32;
        let keys = vec!["1".to_string(), "2".to_string(), "A".to_string()];
        let mut params: Vec<serde_json::Value> = vec![json!(keypad)];
        params.extend(keys.into_iter().map(|k| json!(k)));
        assert_eq!(params, vec![json!(0), json!("1"), json!("2"), json!("A")]);
    }
}
