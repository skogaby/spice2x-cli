use std::fs;
use std::io::Write;

use anyhow::Result;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use chrono::Local;
use serde_json::{json, Value};

use crate::cli::CaptureAction;
use crate::protocol::Connection;

pub fn execute(conn: &mut Connection, action: CaptureAction) -> Result<Value> {
    match action {
        CaptureAction::GetScreens => {
            let data = conn.request("capture", "get_screens", vec![])?;
            Ok(Value::Array(data))
        }
        CaptureAction::GetJpg { screen, quality, divide, output_path, output_folder, base64 } => {
            get_jpg(conn, screen, quality, divide, output_path, output_folder, base64)
        }
    }
}

fn get_jpg(
    conn: &mut Connection,
    screen: u32,
    quality: u32,
    divide: u32,
    output_path: Option<String>,
    output_folder: Option<String>,
    base64_mode: bool,
) -> Result<Value> {
    let data = conn.request(
        "capture",
        "get_jpg",
        vec![json!(screen), json!(quality), json!(divide)],
    )?;

    // Response: [timestamp, width, height, base64_string]
    let b64_str = data
        .get(3)
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("missing base64 data in capture response"))?;

    if base64_mode {
        return Ok(json!(b64_str));
    }

    // Decode and write to file
    let jpeg_bytes = BASE64.decode(b64_str)?;
    let auto_name = || Local::now().format("capture_%Y%m%d_%H%M%S.jpg").to_string();
    let folder = output_folder.unwrap_or_else(|| "capture".to_string());
    let path = if let Some(p) = output_path {
        p
    } else {
        fs::create_dir_all(&folder)?;
        format!("{}/{}", folder.trim_end_matches('/'), auto_name())
    };

    fs::File::create(&path)?.write_all(&jpeg_bytes)?;
    eprintln!("{path}");

    Ok(json!({
        "file": path,
        "width": data.get(1),
        "height": data.get(2),
    }))
}

#[cfg(test)]
mod tests {
    use base64::Engine;
    use base64::engine::general_purpose::STANDARD as BASE64;
    use chrono::Local;

    #[test]
    fn auto_filename_format() {
        let name = Local::now().format("capture_%Y%m%d_%H%M%S.jpg").to_string();
        assert!(name.starts_with("capture_"));
        assert!(name.ends_with(".jpg"));
    }

    #[test]
    fn base64_decode_roundtrip() {
        let original = b"fake jpeg data";
        let encoded = BASE64.encode(original);
        let decoded = BASE64.decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn response_data_shape() {
        // Server returns [timestamp, width, height, base64_string]
        let data = vec![
            serde_json::json!(1234567890_u64),
            serde_json::json!(1920),
            serde_json::json!(1080),
            serde_json::json!("dGVzdA=="),
        ];
        assert!(data[3].as_str().is_some());
        let decoded = BASE64.decode(data[3].as_str().unwrap()).unwrap();
        assert_eq!(decoded, b"test");
    }
}
