use serde::Serialize;
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};

/// Global monotonic request ID counter.
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// A SpiceAPI request message.
#[derive(Debug, Serialize)]
pub struct Request {
    id: u64,
    module: String,
    function: String,
    params: Vec<Value>,
}

impl Request {
    /// Create a new request with an auto-assigned monotonic ID.
    pub fn new(module: &str, function: &str) -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            module: module.to_string(),
            function: function.to_string(),
            params: Vec::new(),
        }
    }

    /// Create a request with a specific ID (used for session_refresh).
    pub fn with_id(id: u64, module: &str, function: &str) -> Self {
        Self {
            id,
            module: module.to_string(),
            function: function.to_string(),
            params: Vec::new(),
        }
    }

    pub fn add_param(&mut self, param: Value) {
        self.params.push(param);
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    /// Serialize to compact JSON bytes (no whitespace, matching Python reference).
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serializes_to_compact_json() {
        let req = Request::with_id(42, "buttons", "read");
        let bytes = req.to_bytes().unwrap();
        let parsed: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(parsed["id"], 42);
        assert_eq!(parsed["module"], "buttons");
        assert_eq!(parsed["function"], "read");
        assert_eq!(parsed["params"], json!([]));
    }

    #[test]
    fn add_param_appends_to_params() {
        let mut req = Request::with_id(1, "coin", "set");
        req.add_param(json!(5));
        let bytes = req.to_bytes().unwrap();
        let parsed: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(parsed["params"], json!([5]));
    }

    #[test]
    fn monotonic_ids_increment() {
        let r1 = Request::new("a", "b");
        let r2 = Request::new("a", "b");
        assert!(r2.id() > r1.id());
    }
}
