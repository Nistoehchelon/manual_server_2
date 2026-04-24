use rand::{RngExt, distr::Alphanumeric, rng};
use uuid::Uuid;

pub fn generate_id() -> String {
    generate_random_alphanumeric_id(32)
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
