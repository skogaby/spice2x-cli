use serde_json::Value;

/// Formats command output for display.
pub trait Formatter {
    fn format(&self, value: &Value) -> String;
}

/// Outputs raw JSON (pretty-printed).
pub struct JsonFormatter;

impl Formatter for JsonFormatter {
    fn format(&self, value: &Value) -> String {
        serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
    }
}

/// Outputs human-readable text.
pub struct TextFormatter;

impl Formatter for TextFormatter {
    fn format(&self, value: &Value) -> String {
        let mut buf = String::new();
        format_value(&mut buf, value, 0);
        // Trim trailing newline for clean output
        while buf.ends_with('\n') {
            buf.pop();
        }
        buf
    }
}

fn format_value(buf: &mut String, value: &Value, indent: usize) {
    let prefix = "  ".repeat(indent);
    match value {
        Value::Null => buf.push_str(&format!("{prefix}null\n")),
        Value::Bool(b) => buf.push_str(&format!("{prefix}{b}\n")),
        Value::Number(n) => buf.push_str(&format!("{prefix}{n}\n")),
        Value::String(s) => buf.push_str(&format!("{prefix}{s}\n")),
        Value::Array(arr) => {
            for (i, item) in arr.iter().enumerate() {
                if i > 0 && item.is_object() {
                    buf.push('\n');
                }
                format_value(buf, item, indent);
            }
        }
        Value::Object(map) => {
            for (key, val) in map {
                match val {
                    Value::Object(_) | Value::Array(_) => {
                        buf.push_str(&format!("{prefix}{key}:\n"));
                        format_value(buf, val, indent + 1);
                    }
                    _ => {
                        buf.push_str(&format!("{prefix}{key}: "));
                        let mut inline = String::new();
                        format_value(&mut inline, val, 0);
                        buf.push_str(inline.trim_end());
                        buf.push('\n');
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn json_formatter_pretty_prints() {
        let f = JsonFormatter;
        let val = json!({"name": "BT_A", "value": 0.0});
        let out = f.format(&val);
        assert!(out.contains("\"name\": \"BT_A\""));
        assert!(out.contains("\"value\": 0.0"));
    }

    #[test]
    fn text_formatter_object() {
        let f = TextFormatter;
        let val = json!({"model": "LDJ", "dest": "J"});
        let out = f.format(&val);
        assert!(out.contains("model: LDJ"));
        assert!(out.contains("dest: J"));
    }

    #[test]
    fn text_formatter_array_of_objects() {
        let f = TextFormatter;
        let val = json!([
            {"name": "BT_A", "value": 0.0},
            {"name": "BT_B", "value": 1.0}
        ]);
        let out = f.format(&val);
        assert!(out.contains("name: BT_A"));
        assert!(out.contains("value: 0"));
        assert!(out.contains("name: BT_B"));
        assert!(out.contains("value: 1"));
    }

    #[test]
    fn text_formatter_array_of_primitives() {
        let f = TextFormatter;
        let val = json!([0, 1, 2]);
        let out = f.format(&val);
        assert_eq!(out, "0\n1\n2");
    }

    #[test]
    fn text_formatter_single_number() {
        let f = TextFormatter;
        let val = json!(42);
        assert_eq!(f.format(&val), "42");
    }

    #[test]
    fn text_formatter_single_string() {
        let f = TextFormatter;
        let val = json!("hello");
        assert_eq!(f.format(&val), "hello");
    }

    #[test]
    fn text_formatter_null() {
        let f = TextFormatter;
        assert_eq!(f.format(&json!(null)), "null");
    }

    #[test]
    fn text_formatter_nested_object() {
        let f = TextFormatter;
        let val = json!({"outer": {"inner": "value"}});
        let out = f.format(&val);
        assert!(out.contains("outer:"));
        assert!(out.contains("  inner: value"));
    }
}
