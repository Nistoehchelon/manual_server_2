use password_hashing::PasswordHasherConfig;
use std::{path::PathBuf, time::Duration};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {


    pub min_length_user_password: usize,
    pub max_length_user_password: usize,

    pub min_length_forum_thread_name: usize,
    pub max_length_forum_thread_name: usize,

    pub min_length_forum_message: usize,
    pub max_length_forum_message: usize,

    pub using_https: bool,

    pub rust_log: String,
    pub rust_log_directory_path: String,

    pub cloudflare_turnstile_enabled: bool,
    pub cloudflare_turnstile_site_key: String,
    pub cloudflare_turnstile_secret_key: String,

    pub port: u64,
    pub ip: String,
    pub database_url: String,

    pub server_global_request_token_bucket_capacity: f64,
    pub server_global_request_token_bucket_refill_rate_per_millis: f64,
    pub per_ip_token_bucket_capacity: f64,
    pub per_ip_token_bucket_refill_rate_per_millis: f64,

    pub keep_alive_enabled: bool,
    //    #[serde(with = "humantime_serde")] automatically converts values like 5000ms or 60s or 1h into Duration
    #[serde(with = "humantime_serde")]
    pub keep_alive_idle_timeout: Duration,
    pub keep_alive_max_requests_per_connection: u64,

    #[serde(with = "humantime_serde")]
    pub common_stream_timeout: Duration,

    pub max_total_headers_size_bytes: u64,
    pub max_headers_count: u64,
    pub max_header_name_length: u64,
    pub max_header_value_length: u64,
    pub max_total_body_size_bytes: u64,
    pub max_request_path_length: u64,
    pub max_cookies_count: u64,

    #[serde(with = "humantime_serde")]
    pub server_time_credits_refill_rate_per_millis: Duration,
    #[serde(with = "humantime_serde")]
    pub server_time_credits_capacity: Duration,
    #[serde(with = "humantime_serde")]
    pub per_ip_time_credits_refill_rate_per_millis: Duration,
    #[serde(with = "humantime_serde")]
    pub per_ip_time_credits_capacity: Duration,

    pub password_hasher_config: PasswordHasherConfig,
}

pub fn read_config(config_file_path: PathBuf) -> Config {
    dotenvy::from_path(config_file_path).ok();
    let c = config::Config::builder()
        .add_source(config::Environment::default()
    //.separator("__")  // default is ".". I guess that's fine?
    )
        .build()
        .expect("Failed to build config")
        .try_deserialize::<Config>()
        .expect("Invalid or missing config values");
    c
}
