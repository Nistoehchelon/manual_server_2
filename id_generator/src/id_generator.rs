use std::time::{SystemTime, UNIX_EPOCH};

pub fn generate_id() -> String {
    // Simple ID generator using timestamp
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros();
    format!("{:x}", timestamp)
}
