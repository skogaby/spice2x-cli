use anyhow::Result;
use serde_json::{json, Value};

use crate::cli::LightsAction;
use crate::protocol::Connection;

pub fn execute(conn: &mut Connection, action: LightsAction) -> Result<Value> {
    match action {
        LightsAction::Read { names } => {
            let params: Vec<Value> = names.into_iter().map(|n| json!(n)).collect();
            let data = conn.request("lights", "read", params)?;
            Ok(format_states(data))
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
    fn format_states_converts_tuples() {
        let data = vec![
            json!(["TOP_LED", 1.0, true]),
            json!(["SIDE_LED", 0.0, false]),
        ];
        let result = format_states(data);
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["name"], "TOP_LED");
        assert_eq!(arr[0]["state"], 1.0);
        assert_eq!(arr[1]["active"], false);
    }

    #[test]
    fn name_filter_param_format() {
        let names = vec!["TOP".to_string(), "SIDE".to_string()];
        let params: Vec<Value> = names.into_iter().map(|n| json!(n)).collect();
        assert_eq!(params, vec![json!("TOP"), json!("SIDE")]);
    }
}
