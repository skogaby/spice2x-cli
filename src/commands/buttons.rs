use anyhow::Result;
use serde_json::{json, Value};

use crate::cli::ButtonsAction;
use crate::protocol::Connection;

pub fn execute(conn: &mut Connection, action: ButtonsAction) -> Result<Value> {
    match action {
        ButtonsAction::Read => {
            let data = conn.request("buttons", "read", vec![])?;
            Ok(format_states(data))
        }
        ButtonsAction::Write { name, state } => {
            conn.request("buttons", "write", vec![json!([name, state])])?;
            Ok(json!(null))
        }
        ButtonsAction::WriteReset { names } => {
            let params: Vec<Value> = names.into_iter().map(|n| json!([n])).collect();
            conn.request("buttons", "write_reset", params)?;
            Ok(json!(null))
        }
    }
}

/// Convert raw `[name, state, active]` tuples into structured objects.
fn format_states(data: Vec<Value>) -> Value {
    Value::Array(
        data.into_iter()
            .filter_map(|item| {
                let arr = item.as_array()?;
                Some(json!({
                    "name": arr.get(0)?,
                    "state": arr.get(1)?,
                    "active": arr.get(2)?,
                }))
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn format_states_converts_tuples_to_objects() {
        let data = vec![
            json!(["BT_A", 0.0, false]),
            json!(["BT_B", 1.0, true]),
        ];
        let result = format_states(data);
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["name"], "BT_A");
        assert_eq!(arr[0]["state"], 0.0);
        assert_eq!(arr[0]["active"], false);
        assert_eq!(arr[1]["name"], "BT_B");
        assert_eq!(arr[1]["state"], 1.0);
        assert_eq!(arr[1]["active"], true);
    }

    #[test]
    fn format_states_skips_malformed_entries() {
        let data = vec![json!("not an array"), json!(["BT_A", 0.0, false])];
        let result = format_states(data);
        assert_eq!(result.as_array().unwrap().len(), 1);
    }

    #[test]
    fn write_param_format() {
        // Server expects params: [["name", state]]
        let param = json!(["BT_A", 1.0]);
        assert_eq!(param[0], "BT_A");
        assert_eq!(param[1], 1.0);
    }

    #[test]
    fn write_reset_param_format() {
        // Server expects params: [["name1"], ["name2"]] or [] for all
        let names = vec!["BT_A".to_string(), "BT_B".to_string()];
        let params: Vec<Value> = names.into_iter().map(|n| json!([n])).collect();
        assert_eq!(params.len(), 2);
        assert_eq!(params[0][0], "BT_A");
        assert_eq!(params[1][0], "BT_B");
    }

    #[test]
    fn write_reset_no_names_sends_empty_params() {
        let names: Vec<String> = vec![];
        let params: Vec<Value> = names.into_iter().map(|n| json!([n])).collect();
        assert!(params.is_empty());
    }
}
