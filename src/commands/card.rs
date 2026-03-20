use anyhow::Result;
use serde_json::{json, Value};

use crate::cli::CardAction;
use crate::protocol::Connection;

pub fn execute(conn: &mut Connection, action: CardAction) -> Result<Value> {
    match action {
        CardAction::Insert { unit, card_id } => {
            conn.request("card", "insert", vec![json!(unit), json!(card_id)])?;
            Ok(json!(null))
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn insert_param_format() {
        let unit = 0u32;
        let card_id = "E004123456789ABC";
        let params = vec![json!(unit), json!(card_id)];
        assert_eq!(params[0], 0);
        assert_eq!(params[1], "E004123456789ABC");
    }
}
