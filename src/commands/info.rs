use anyhow::{bail, Result};
use serde_json::Value;

use crate::cli::InfoAction;
use crate::protocol::Connection;

pub fn execute(conn: &mut Connection, action: InfoAction) -> Result<Value> {
    let function = match action {
        InfoAction::Avs => "avs",
        InfoAction::Launcher => "launcher",
        InfoAction::Memory => "memory",
    };
    let data = conn.request("info", function, vec![])?;
    match data.into_iter().next() {
        Some(value) => Ok(value),
        None => bail!("empty response from info {function}"),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn avs_response_shape() {
        // Server returns data[0] as object with these fields
        let data = vec![json!({
            "model": "LDJ",
            "dest": "J",
            "spec": "A",
            "rev": "L",
            "ext": "2024090200",
            "services": "http://example.com"
        })];
        let result = data.into_iter().next().unwrap();
        assert_eq!(result["model"], "LDJ");
        assert_eq!(result["dest"], "J");
    }

    #[test]
    fn launcher_response_shape() {
        let data = vec![json!({
            "version": "2.4.0",
            "compile_date": "Jan 01 2024",
            "compile_time": "12:00:00",
            "system_time": "2024-09-02T10:30:00Z",
            "args": ["-ea", "-io"]
        })];
        let result = data.into_iter().next().unwrap();
        assert_eq!(result["version"], "2.4.0");
        assert!(result["args"].is_array());
    }

    #[test]
    fn memory_response_shape() {
        let data = vec![json!({
            "mem_total": 16000000000_u64,
            "mem_total_used": 8000000000_u64,
            "mem_used": 500000000_u64,
            "vmem_total": 32000000000_u64,
            "vmem_total_used": 4000000000_u64,
            "vmem_used": 200000000_u64
        })];
        let result = data.into_iter().next().unwrap();
        assert!(result["mem_total"].is_number());
        assert!(result["vmem_used"].is_number());
    }
}
