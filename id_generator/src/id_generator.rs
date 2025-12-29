use std::time::{SystemTime, UNIX_EPOCH};

use uuid::Uuid;

pub fn generate_id() -> String {
    // Simple ID generator using timestamp
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros();
    format!("{:x}", timestamp)
}

pub fn generate_unique_id() -> String {
    Uuid::new_v4().to_string()
}
