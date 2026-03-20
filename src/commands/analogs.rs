use anyhow::Result;
use serde_json::{json, Value};

use crate::cli::AnalogsAction;
use crate::protocol::Connection;

pub fn execute(conn: &mut Connection, action: AnalogsAction) -> Result<Value> {
    match action {
        AnalogsAction::Read => {
            let data = conn.request("analogs", "read", vec![])?;
            Ok(format_states(data))
        }
        AnalogsAction::Write { name, value } => {
            conn.request("analogs", "write", vec![json!([name, value])])?;
            Ok(json!(null))
        }
        AnalogsAction::WriteReset { names } => {
            let params: Vec<Value> = names.into_iter().map(|n| json!([n])).collect();
            conn.request("analogs", "write_reset", params)?;
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
            json!(["TURNTABLE", 0.5, false]),
            json!(["SLIDER", 0.75, true]),
        ];
        let result = format_states(data);
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["name"], "TURNTABLE");
        assert_eq!(arr[0]["state"], 0.5);
        assert_eq!(arr[1]["name"], "SLIDER");
        assert_eq!(arr[1]["active"], true);
    }

    #[test]
    fn write_param_format() {
        let param = json!(["TURNTABLE", 0.5]);
        assert_eq!(param[0], "TURNTABLE");
        assert_eq!(param[1], 0.5);
    }
}
