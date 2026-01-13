use std::time::{SystemTime, UNIX_EPOCH};

use rand::{Rng, distr::Alphanumeric, rng};
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

pub fn generate_random_alphanumeric_id(length: usize) -> String {
    rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}
