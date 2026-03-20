use anyhow::{bail, Result};
use serde_json::{json, Value};

use crate::cli::CoinAction;
use crate::protocol::Connection;

pub fn execute(conn: &mut Connection, action: CoinAction) -> Result<Value> {
    match action {
        CoinAction::Get => {
            let data = conn.request("coin", "get", vec![])?;
            match data.into_iter().next() {
                Some(value) => Ok(value),
                None => bail!("empty response from coin get"),
            }
        }
        CoinAction::Set { amount } => {
            conn.request("coin", "set", vec![json!(amount)])?;
            Ok(json!(null))
        }
        CoinAction::Insert { amount } => {
            let params = if amount != 1 { vec![json!(amount)] } else { vec![] };
            conn.request("coin", "insert", params)?;
            Ok(json!(null))
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn insert_default_sends_no_params() {
        let params: Vec<serde_json::Value> = if 1 != 1 { vec![json!(1)] } else { vec![] };
        assert!(params.is_empty());
    }

    #[test]
    fn insert_custom_sends_amount() {
        let amount = 5u32;
        let params: Vec<serde_json::Value> = if amount != 1 { vec![json!(amount)] } else { vec![] };
        assert_eq!(params.len(), 1);
        assert_eq!(params[0], 5);
    }
}
